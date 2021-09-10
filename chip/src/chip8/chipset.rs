//! The main chip8 implementation module.
//! The given implementation is based primatily on the [wikipedia
//! page](https://en.wikipedia.org/wiki/CHIP-8) definitions.

use crate::{
    definitions::{cpu, display, keyboard, memory, timer},
    devices::Keyboard,
    opcode::{self, ChipOpcodePreProcessHandler, Opcodes, ProgramCounter, ProgramCounterStep},
    resources::Rom,
    timer::{NoCallback, TimerCallback},
    timer::{TimedWorker, Timer, TimerValue},
    OpcodeError, ProcessError, StackError,
};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use rand::RngCore;
use std::{convert::TryInto, sync::Arc, time::Duration};
use tinyvec::ArrayVec;

use hashbrown::HashMap;

/// The chipset struct containing the internal implementation of the chipset
/// and the timers.
/// The struct has been split up into two instances to simplyfiy the implementation.
pub struct ChipSet<W, S>
where
    W: TimedWorker,
    S: TimerCallback,
{
    /// The actuall chipset implementation.
    chipset: InternalChipSet,
    /// Holds the delaytimer struct, so that the internal closures do not go out of scope and
    /// then drop.
    _delay_timer: Timer<W, u8, NoCallback>,
    /// Holds the sound timer struct, so that the internally used closures will not be dropped.
    _sound_timer: Timer<W, u8, S>,
}

impl<W, S> ChipSet<W, S>
where
    W: TimedWorker,
    S: TimerCallback + 'static,
{
    /// Creates a new chip set from a given rom.
    pub fn new(rom: Rom) -> Self {
        Self::with_keyboard(rom, Arc::new(RwLock::new(Keyboard::new())))
    }

    /// Crates a new chip with an external keyboard.
    pub fn with_keyboard(rom: Rom, keyboard: Arc<RwLock<Keyboard>>) -> Self {
        let (delay_timer, delay_value) = Timer::new(0, Duration::from_millis(timer::INTERVAL));
        let (sound_timer, sound_value) =
            Timer::with_callback(0, Duration::from_millis(timer::INTERVAL), S::new());
        let chipset = InternalChipSet::new(rom, delay_value, sound_value, keyboard);

        Self {
            chipset,
            _delay_timer: delay_timer,
            _sound_timer: sound_timer,
        }
    }

    /// Will return a slice of displays.
    pub fn get_display(&self) -> &[Vec<bool>] {
        self.chipset.get_display()
    }

    /// Will execute the next operation.
    /// Returns the operation that has to be run by the caller.
    pub fn step(&mut self) -> Result<opcode::Operation, ProcessError> {
        self.chipset.next()
    }

    /// Will set the given key into the keyboard.
    pub fn set_key(&mut self, key: usize, to: bool) {
        self.chipset.set_key(key, to);
    }

    /// Get a reference to the chip set's chipset.
    pub(super) fn chipset(&self) -> &InternalChipSet {
        &self.chipset
    }

    /// Get a mutable reference to the chip set's chipset.
    /// This function is only used in the context of tests
    /// as there never is a need to expose the internal
    /// chipset otherwise.
    #[cfg(test)]
    pub(super) fn chipset_mut(&mut self) -> &mut InternalChipSet {
        &mut self.chipset
    }

    /// Will write keyboard data into interncal keyboard representation.
    pub fn set_keyboard(&mut self, keys: &[bool; keyboard::SIZE]) {
        self.chipset.set_keyboard(keys);
    }

    /// will return the sound timer
    pub fn get_sound_timer(&self) -> u8 {
        self.chipset.get_sound_timer()
    }
}

/// The ChipSet struct represents the current state
/// of the system, it contains all the structures
/// needed for emulating an instant on the
/// Chip8 CPU.
pub(super) struct InternalChipSet {
    /// name of the loaded rom
    pub(super) name: String,
    /// - `0x000-0x1FF` - Chip 8 interpreter (contains font set in emu)
    /// - `0x050-0x0A0` - Used for the built in `4x5` pixel font set (`0-F`)
    /// - `0x200-0xFFF` - Program ROM and work RAM
    pub(super) memory: Vec<u8>,
    /// Contains the precalculated opcode data, this vector is significatly smaller then the
    /// actuall memory portion, as it will ever only use as much memory as required
    /// for the emulation.
    pub(super) opcode_memory: HashMap<usize, Opcodes>,
    /// `8-bit` data registers named `V0` to `VF`. The `VF` register doubles as a flag for some
    /// instructions; thus, it should be avoided. In an addition operation, `VF` is the carry flag,
    /// while in subtraction, it is the "no borrow" flag. In the draw instruction `VF` is set upon
    /// pixel collision.
    pub(super) registers: [u8; cpu::register::SIZE],
    /// The index for the register, this is a special register entry
    /// called index `I`
    pub(super) index_register: usize,
    /// The program counter is a CPU register in the computer processor which has the address of the
    /// next instruction to be executed from memory.
    pub(super) program_counter: usize,
    /// The stack is only used to store return addresses when subroutines are called. The original
    /// [RCA 1802](https://de.wikipedia.org/wiki/RCA1802) version allocated `48` bytes for up to
    /// `12` levels of nesting; modern implementations usually have more.
    /// (here we are using `16`)
    /// Addition: We are using the stack capability of the std::vec::Vec.
    pub(super) stack: ArrayVec<[usize; cpu::stack::SIZE]>,
    /// Delay timer: This timer is intended to be used for timing the events of games. Its value
    /// can be set and read.
    /// Counts down at 60 hertz, until it reaches 0.
    pub(super) delay_timer: TimerValue<u8>,
    /// Sound timer: This timer is used for sound effects. When its value is nonzero, a beeping
    /// sound is made.
    /// Counts down at 60 hertz, until it reaches 0.
    pub(super) sound_timer: TimerValue<u8>,
    /// The graphics of the Chip 8 are black and white and the screen has a total of `2048` pixels
    /// `(64 x 32)`. This can easily be implemented using an array that hold the pixel state `(1 or 0)`:
    pub(super) display: Vec<Vec<bool>>,
    /// Input is done with a hex keyboard that has 16 keys ranging `0-F`. The `8`, `4`, `6`, and
    /// `2` keys are typically used for directional input. Three opcodes are used to detect input.
    /// One skips an instruction if a specific key is pressed, while another does the same if a
    /// specific key is not pressed. The third waits for a key press, and then stores it in one of
    /// the data registers.
    pub(super) keyboard: Arc<RwLock<Keyboard>>,
    /// This stores the random number generator, used by the chipset.
    /// It is stored into the chipset, so as to enable simple mocking
    /// of the given type.
    pub(super) rng: Box<dyn RngCore + Send>,
    /// Will store the callbacks needed for certain tasks
    /// example, running special code after the main caller
    /// did his. (Do work after wait etc.)
    pub(super) preprocessor: Option<Box<dyn FnOnce(&mut Self) + Send>>,
}

impl InternalChipSet {
    /// will create a new chipset object
    pub fn new(
        rom: Rom,
        delay_timer: TimerValue<u8>,
        sound_timer: TimerValue<u8>,
        keyboard: Arc<RwLock<Keyboard>>,
    ) -> Self {
        // initialize all the memory with 0

        let mut ram = vec![0; memory::SIZE];

        // load fonts
        ram[display::fontset::LOCATION
            ..(display::fontset::LOCATION + display::fontset::FONTSET.len())]
            .copy_from_slice(&display::fontset::FONTSET);

        // write the rom data into memory
        let data = rom.get_data();
        ram[cpu::PROGRAM_COUNTER..(cpu::PROGRAM_COUNTER + rom.get_data().len())]
            .copy_from_slice(data);

        Self {
            name: rom.get_name().to_string(),
            memory: ram,
            opcode_memory: HashMap::new(),
            registers: [0; cpu::register::SIZE],
            index_register: 0,
            program_counter: cpu::PROGRAM_COUNTER,
            stack: ArrayVec::new(),
            delay_timer,
            sound_timer,
            display: vec![vec![false; display::HEIGHT]; display::WIDTH],
            keyboard,
            rng: Box::new(rand::rngs::OsRng {}),
            preprocessor: None,
        }
    }

    /// Will get the next opcode from memory
    pub fn get_opcode(&mut self) -> Result<Opcodes, OpcodeError> {
        // Sadly we have to use copy here, given the borrow mut later on
        let iops = match self.opcode_memory.get(&self.program_counter) {
            None => {
                let iops = opcode::build_opcode(&self.memory, self.program_counter)?.try_into()?;
                self.opcode_memory.insert(self.program_counter, iops);
                iops
            }
            Some(value) => *value,
        };

        Ok(iops)
    }

    /// will advance the program by a single step
    pub fn next(&mut self) -> Result<opcode::Operation, ProcessError> {
        // import here as to not bloat the namespace
        use crate::opcode::ChipOpcodes;
        // get next opcode
        let opcode = self.get_opcode()?;
        // run the opcode
        self.calc(&opcode)
    }

    pub(super) fn get_keyboard_write(&mut self) -> RwLockWriteGuard<Keyboard> {
        self.keyboard.write()
    }

    pub(super) fn get_keyboard_read(&self) -> RwLockReadGuard<Keyboard> {
        self.keyboard.read()
    }

    /// Will write keyboard data into interncal keyboard representation.
    pub fn set_keyboard(&mut self, keys: &[bool; keyboard::SIZE]) {
        // copy_from_slice checks the keys lenght during copy
        self.get_keyboard_write().set_mult(keys);
    }

    /// Will set the value of the given key
    pub fn set_key(&mut self, key: usize, to: bool) {
        self.get_keyboard_write().set_key(key, to)
    }

    /// will return the sound timer
    pub fn get_sound_timer(&self) -> u8 {
        self.sound_timer.get_value()
    }

    /// will return the delay timer
    pub fn get_delay_timer(&self) -> u8 {
        self.delay_timer.get_value()
    }

    /// Will return a immutable slice of the current display configuration
    pub fn get_display(&self) -> &[Vec<bool>] {
        &self.display[..]
    }

    /// Will push the current pointer to the stack
    /// stack_counter is always one bigger then the
    /// entry it points to
    pub fn push_stack(&mut self, pointer: usize) -> Result<(), StackError> {
        if self.stack.len() == self.stack.capacity() {
            Err(StackError::Full)
        } else {
            // push to stack
            self.stack.push(pointer);
            Ok(())
        }
    }

    /// Will pop from the counter
    /// stack_counter is always one bigger then the entry
    /// it points to
    pub fn pop_stack(&mut self) -> Result<usize, StackError> {
        if self.stack.is_empty() {
            Err(StackError::Empty)
        } else {
            let pointer = self.stack.pop().ok_or(StackError::Unexpected)?;
            Ok(pointer)
        }
    }
}

impl ProgramCounter for InternalChipSet {
    fn step(&mut self, step: ProgramCounterStep) {
        self.program_counter = if let ProgramCounterStep::Jump(_) = step {
            step.step()
        } else {
            self.program_counter + step.step()
        }
    }
}

impl ChipOpcodePreProcessHandler for InternalChipSet {
    fn preprocess(&mut self) {
        if let Some(func) = self.preprocessor.take() {
            func(self);
        }
    }
}
