use {
    crate::{
        definitions::{
            DISPLAY_HEIGHT, DISPLAY_RESOLUTION, FONTSET_LOCATION, MEMORY_SIZE, OPCODE_BYTE_SIZE,
            PROGRAM_COUNTER, REGISTER_SIZE, STACK_NESTING,
        },
        devices::Keyboard,
        fontset::FONSET,
        opcode::{self, ChipOpcodePreProcessHandler, Opcode, ProgramCounter, ProgramCounterStep},
        resources::Rom,
        timer::Timed,
        timer::{TimedWorker, Timer},
    },
    rand::RngCore,
};

/// The ChipSet struct represents the current state
/// of the system, it contains all the structures
/// needed for emulating an instant on the
/// Chip8 CPU.
pub struct ChipSet<W: TimedWorker> {
    /// name of the loaded rom
    pub(super) name: String,
    /// all two bytes long and stored big-endian
    pub(super) opcode: Opcode,
    /// - `0x000-0x1FF` - Chip 8 interpreter (contains font set in emu)
    /// - `0x050-0x0A0` - Used for the built in `4x5` pixel font set (`0-F`)
    /// - `0x200-0xFFF` - Program ROM and work RAM
    pub(super) memory: Box<[u8]>,
    /// `8-bit` data registers named `V0` to `VF`. The `VF` register doubles as a flag for some
    /// instructions; thus, it should be avoided. In an addition operation, `VF` is the carry flag,
    /// while in subtraction, it is the "no borrow" flag. In the draw instruction `VF` is set upon
    /// pixel collision.
    pub(super) registers: Box<[u8]>,
    /// The index for the register, this is a special register entry
    /// called index `I`
    pub(super) index_register: u16,
    /// The program counter is a CPU register in the computer processor which has the address of the
    /// next instruction to be executed from memory.
    pub(super) program_counter: usize,
    /// The stack is only used to store return addresses when subroutines are called. The original
    /// [RCA 1802](https://de.wikipedia.org/wiki/RCA1802) version allocated `48` bytes for up to
    /// `12` levels of nesting; modern implementations usually have more.
    /// (here we are using `16`)
    /// Addition: We are using the stack capability of the std::vec::Vec.
    pub(super) stack: Vec<usize>,
    /// Delay timer: This timer is intended to be used for timing the events of games. Its value
    /// can be set and read.
    /// Counts down at 60 hertz, until it reaches 0.
    pub(super) delay_timer: Timer<W>,
    /// Sound timer: This timer is used for sound effects. When its value is nonzero, a beeping
    /// sound is made.
    /// Counts down at 60 hertz, until it reaches 0.
    pub(super) sound_timer: Timer<W>,
    /// The graphics of the Chip 8 are black and white and the screen has a total of `2048` pixels
    /// `(64 x 32)`. This can easily be implemented using an array that hold the pixel state `(1 or 0)`:
    pub(super) display: Box<[Box<[bool]>]>,
    /// Input is done with a hex keyboard that has 16 keys ranging `0-F`. The `8`, `4`, `6`, and
    /// `2` keys are typically used for directional input. Three opcodes are used to detect input.
    /// One skips an instruction if a specific key is pressed, while another does the same if a
    /// specific key is not pressed. The third waits for a key press, and then stores it in one of
    /// the data registers.
    pub(super) keyboard: Keyboard,
    /// This stores the random number generator, used by the chipset.
    /// It is stored into the chipset, so as to enable simple mocking
    /// of the given type.
    pub(super) rng: Box<dyn RngCore + Send>,
    /// Will store the callbacks needed for certain tasks
    /// example, running special code after the main caller
    /// did his. (Do work after wait etc.)
    pub(super) preprocessor: Option<Box<dyn FnOnce(&mut Self) + Send>>,
}

impl<W: TimedWorker> ChipSet<W> {
    /// will create a new chipset object
    pub fn new(rom: Rom) -> Self {
        // initialize all the memory with 0

        let mut ram = vec![0; MEMORY_SIZE];

        // load fonts
        ram[FONTSET_LOCATION..(FONTSET_LOCATION + FONSET.len())].copy_from_slice(&FONSET);

        // write the rom data into memory
        ram[PROGRAM_COUNTER..(PROGRAM_COUNTER + rom.get_data().len())]
            .copy_from_slice(&rom.get_data());

        Self {
            name: rom.get_name().to_string(),
            opcode: 0,
            memory: ram.into_boxed_slice(),
            registers: vec![0; REGISTER_SIZE].into_boxed_slice(),
            index_register: 0,
            program_counter: PROGRAM_COUNTER,
            stack: Vec::with_capacity(STACK_NESTING),
            delay_timer: Timer::new(0),
            sound_timer: Timer::new(0),
            display: vec![vec![false; DISPLAY_HEIGHT].into_boxed_slice(); DISPLAY_RESOLUTION]
                .into_boxed_slice(),
            keyboard: Keyboard::new(),
            rng: Box::new(rand::rngs::OsRng {}),
            preprocessor: None,
        }
    }

    /// will get the next opcode from memory
    pub(super) fn set_opcode(&mut self) -> Result<(), String> {
        // will build the opcode given from the pointer
        self.opcode = opcode::build_opcode(&self.memory, self.program_counter)?;
        Ok(())
    }

    /// will advance the program by a single step
    pub fn next(&mut self) -> Result<opcode::Operation, String> {
        // import here as to not bloat the namespace
        use crate::opcode::ChipOpcodes;
        // get next opcode
        self.set_opcode()?;
        // run the opcode
        self.calc(self.opcode)
    }

    /// Will write keyboard data into interncal keyboard representation.
    pub fn set_keyboard(&mut self, keys: &[bool]) {
        // copy_from_slice checks the keys lenght during copy
        self.keyboard.set_mult(keys);
    }

    /// Will set the value of the given key
    pub fn set_key(&mut self, key: usize, to: bool) {
        self.keyboard.set_key(key, to)
    }

    /// Will toggle the given key
    pub fn toggle_key(&mut self, key: usize) {
        self.keyboard.toggle_key(key)
    }

    /// Will get the current state of the keyboard
    pub fn get_keyboard(&self) -> &[bool] {
        self.keyboard.get_keys()
    }

    /// will return the sound timer
    pub fn get_sound_timer(&self) -> u8 {
        self.sound_timer.get_value()
    }

    /// will return the delay timer
    pub fn get_delay_timer(&self) -> u8 {
        self.delay_timer.get_value()
    }

    /// will return a clone of the current display configuration
    pub fn get_display(&self) -> Vec<&[bool]> {
        self.display.iter().map(|row| &row[..]).collect()
    }

    /// Will push the current pointer to the stack
    /// stack_counter is always one bigger then the
    /// entry it points to
    pub(super) fn push_stack(&mut self, pointer: usize) -> Result<(), &'static str> {
        if self.stack.len() == self.stack.capacity() {
            Err("Stack is full!")
        } else {
            // push to stack
            self.stack.push(pointer);
            Ok(())
        }
    }

    /// Will pop from the counter
    /// stack_counter is always one bigger then the entry
    /// it points to
    pub(super) fn pop_stack(&mut self) -> Result<usize, &'static str> {
        if self.stack.is_empty() {
            Err("Stack is empty!")
        } else {
            let pointer = self
                .stack
                .pop()
                .expect("During poping of the stack an unusual error occured.");
            Ok(pointer)
        }
    }
}

impl<W: TimedWorker> ProgramCounter for ChipSet<W> {
    fn step(&mut self, step: ProgramCounterStep) {
        match step {
            ProgramCounterStep::Next => self.program_counter += OPCODE_BYTE_SIZE,
            ProgramCounterStep::Skip => self.program_counter += 2 * OPCODE_BYTE_SIZE,
            ProgramCounterStep::None => {}
            ProgramCounterStep::Jump(pointer) => {
                if PROGRAM_COUNTER <= pointer && pointer < self.memory.len() {
                    self.program_counter = pointer;
                } else {
                    panic!("Memory out of bounds error!")
                }
            }
        }
    }
}

impl<W: TimedWorker> ChipOpcodePreProcessHandler for ChipSet<W> {
    fn preprocess(&mut self) {
        if let Some(func) = self.preprocessor.take() {
            func(self);
        }
    }
}
