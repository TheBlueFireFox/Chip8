use std::u16;

use crate::definitions::FONTSET_LOCATION;

use {
    crate::{
        definitions::{
            DISPLAY_RESOLUTION, MEMORY_SIZE, OPCODE_BYTE_SIZE, PROGRAM_COUNTER, REGISTER_LAST,
            REGISTER_SIZE, STACK_NESTING,
        },
        devices::Keyboard,
        fontset::FONSET,
        opcode::{
            self, ChipOpcodePreProcessHandler, ChipOpcodes, Opcode, OpcodeTrait, Operation,
            ProgramCounter, ProgramCounterStep,
        },
        resources::Rom,
        timer::Timer,
    },
    rand::RngCore,
};

/// The ChipSet struct represents the current state
/// of the system, it contains all the structures
/// needed for emulating an instant on the
/// Chip8 CPU.
pub struct ChipSet {
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
    pub(super) delay_timer: Timer,
    /// Sound timer: This timer is used for sound effects. When its value is nonzero, a beeping
    /// sound is made.
    /// Counts down at 60 hertz, until it reaches 0.
    pub(super) sound_timer: Timer,
    /// The graphics of the Chip 8 are black and white and the screen has a total of `2048` pixels
    /// `(64 x 32)`. This can easily be implemented using an array that hold the pixel state `(1 or 0)`:
    pub(super) display: Box<[u8]>,
    /// Input is done with a hex keyboard that has 16 keys ranging `0-F`. The `8`, `4`, `6`, and
    /// `2` keys are typically used for directional input. Three opcodes are used to detect input.
    /// One skips an instruction if a specific key is pressed, while another does the same if a
    /// specific key is not pressed. The third waits for a key press, and then stores it in one of
    /// the data registers.
    pub(super) keyboard: Keyboard,
    /// This stores the random number generator, used by the chipset.
    /// It is stored into the chipset, so as to enable simple mocking
    /// of the given type.
    pub(super) rng: Box<dyn RngCore>,
    /// Will store the callbacks needed for certain tasks
    /// example, running special code after the main caller
    /// did his. (Do work after wait etc.)
    pub(super) preprocessor: Option<Box<dyn FnOnce(&mut Self)>>,
}

impl ChipSet {
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
            display: vec![0; DISPLAY_RESOLUTION].into_boxed_slice(),
            keyboard: Keyboard::new(),
            rng: Box::new(rand::thread_rng()),
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
        // get next opcode
        // We don't need the `Ok(())` output here.
        self.set_opcode()?;
        self.calc(self.opcode)
    }

    /// Will write keyboard data into interncal keyboard representation.
    pub fn set_keyboard(&mut self, keys: &[bool]) {
        // copy_from_slice checks the keys lenght during copy
        self.keyboard.set_mult(keys);
    }

    pub fn set_key(&mut self, key: usize, to: bool) {
        self.keyboard.set_key(key, to)
    }

    pub fn toggle_key(&mut self, key: usize) {
        self.keyboard.toggle_key(key)
    }

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
    pub fn get_display(&self) -> &[u8] {
        &self.display
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

impl ProgramCounter for ChipSet {
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

impl ChipOpcodePreProcessHandler for ChipSet {
    fn preprocess(&mut self) {
        if let Some(func) = self.preprocessor.take() {
            func(self);
        }
    }
}

impl ChipOpcodes for ChipSet {
    fn zero(&mut self, opcode: Opcode) -> Result<(ProgramCounterStep, Operation), String> {
        match opcode {
            0x00E0 => {
                // 00E0
                // clear display
                Ok((ProgramCounterStep::Next, Operation::Clear))
            }
            0x00EE => {
                // 00EE
                // Return from sub routine => pop from stack
                let pc = self.pop_stack()?;
                Ok((ProgramCounterStep::Jump(pc), Operation::None))
            }
            _ => Err(format!(
                "An unsupported opcode was used {:#06X?}",
                self.opcode
            )),
        }
    }

    fn one(&self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 1NNN
        // Jumps to address NNN.
        Ok(ProgramCounterStep::Jump(opcode.nnn()))
    }

    fn two(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 2NNN
        // Calls subroutine at NNN
        match self.push_stack(self.program_counter) {
            Ok(_) => Ok(ProgramCounterStep::Jump(opcode.nnn())),
            Err(err) => Err(err.to_string()),
        }
    }

    fn three(&self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 3XNN
        // Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to
        // skip a code block)
        let (x, nn) = opcode.xnn();
        Ok(ProgramCounterStep::cond(self.registers[x] == nn))
    }

    fn four(&self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 4XNN
        // Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a
        // jump to skip a code block)
        let (x, nn) = opcode.xnn();
        Ok(ProgramCounterStep::cond(self.registers[x] != nn))
    }

    fn five(&self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 5XY0
        // Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to
        // skip a code block)
        match opcode.xyn() {
            (x, y, 0) => Ok(ProgramCounterStep::cond(
                self.registers[x] == self.registers[y],
            )),
            _ => Err(format!("An unsupported opcode was used {:#06X?}", opcode)),
        }
    }

    fn six(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 6XNN
        // Sets VX to NN.
        let (x, nn) = opcode.xnn();
        self.registers[x] = nn;
        Ok(ProgramCounterStep::Next)
    }

    fn seven(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 7XNN
        // Adds NN to VX. (Carry flag is not changed)
        let (x, nn) = opcode.xnn();
        // let VX overflow, but ignore carry
        let res = self.registers[x].wrapping_add(nn);
        self.registers[x] = res;
        Ok(ProgramCounterStep::Next)
    }

    fn eight(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // remove the middle 8 bits for calculations
        let (x, y, n) = opcode.xyn();
        match n {
            0x0 => {
                // 8XY0
                // Sets VX to the value of VY.
                self.registers[x] = self.registers[y];
            }
            0x1 => {
                // 8XY1
                // Sets VX to VX or VY. (Bitwise OR operation)
                self.registers[x] = self.registers[x] | self.registers[y];
            }
            0x2 => {
                // 8XY2
                // Sets VX to VX and VY. (Bitwise AND operation)
                self.registers[x] = self.registers[x] & self.registers[y];
            }
            0x3 => {
                // 8XY3
                // Sets VX to VX xor VY.
                self.registers[x] = self.registers[x] ^ self.registers[y];
            }
            0x4 => {
                // 8XY4
                // Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
                let left = self.registers[x] as u16;
                let right = self.registers[y] as u16;
                let res = left + right;
                let carry = res & 0x0100 == 0x0100;
                self.registers[x] = res as u8;
                self.registers[REGISTER_LAST] = if carry { 1 } else { 0 };
            }
            0x5 => {
                // 8XY5
                // VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there
                // isn't.
                let left = self.registers[x] as u16;
                let right = ((!self.registers[y]).wrapping_add(1)) as u16;
                let res = left + right;
                let carry = (res & 0x0100) == 0x0100;
                self.registers[x] = res as u8;
                self.registers[REGISTER_LAST] = if carry { 1 } else { 0 };
            }
            0x6 => {
                // 8XY6
                // Stores the least significant bit of VX in VF and then shifts VX to the right
                // by 1.
                self.registers[REGISTER_LAST] = self.registers[x] & 1;
                self.registers[x] = self.registers[x] >> 1;
            }
            0x7 => {
                // 8XY7
                // Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there
                // isn't.
                let left = self.registers[y] as u16;
                let right = ((!self.registers[x]).wrapping_add(1)) as u16;
                let res = left + right;
                let carry = (res & 0x0100) == 0x0100;
                self.registers[x] = res as u8;
                self.registers[REGISTER_LAST] = if carry { 1 } else { 0 };
            }
            0xE => {
                // 8XYE
                // Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
                const SHIFT_SIGNIFICANT: u8 = 7;
                const AND_SIGNIFICANT: u8 = 1 << SHIFT_SIGNIFICANT;
                self.registers[REGISTER_LAST] =
                    (self.registers[x] & AND_SIGNIFICANT) >> SHIFT_SIGNIFICANT;
                self.registers[x] = self.registers[x] << 1;
            }
            _ => {
                return Err(format!(
                    "An unsupported opcode was used {:#06X?}",
                    self.opcode
                ));
            }
        }
        // increment the program counter by one
        Ok(ProgramCounterStep::Next)
    }

    fn nine(&self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 9XY0
        // Skips the next instruction if VX doesn't equal VY. (Usually the next instruction is
        // a jump to skip a code block)
        match opcode.xyn() {
            (x, y, 0) => Ok(ProgramCounterStep::cond(
                self.registers[x] != self.registers[y],
            )),
            _ => Err(format!("An unsupported opcode was used {:#06X?}", opcode)),
        }
    }

    fn a(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // ANNN
        // Sets I to the address NNN.
        self.index_register = opcode.nnn() as u16;
        Ok(ProgramCounterStep::Next)
    }

    fn b(&self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // BNNN
        // Jumps to the address NNN plus V0.
        let nnn = opcode.nnn();
        let v0 = self.registers[0] as usize;
        Ok(ProgramCounterStep::Jump(v0 + nnn))
    }

    fn c(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // CXNN
        // Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255)
        // and NN.

        let (x, nn) = opcode.xnn();
        // using a fill bytes call here, as the trait RngCore does not
        // support random u8.
        let mut rand: [u8; 1] = [0];
        self.rng.fill_bytes(&mut rand);
        self.registers[x] = nn & rand[0];
        Ok(ProgramCounterStep::Next)
    }

    fn d(&self, opcode: Opcode) -> Result<(ProgramCounterStep, Operation), String> {
        // DXYN
        // Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N
        // pixels. Each row of 8 pixels is read as bit-coded starting from memory location I; I
        // value doesn’t change after the execution of this instruction. As described above, VF is
        // set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and
        // to 0 if that doesn’t happen

        // TODO: finish implementation
        let (x, y, n) = opcode.xyn();
        let i = self.index_register as usize;
        Ok((
            ProgramCounterStep::Next,
            opcode::Operation::Draw {
                x,
                y,
                height: n,
                width: 8, // default width this is not doing to change, but is kept in for simplicity
                location: i,
            },
        ))
    }

    fn e(&self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        let (x, nn) = opcode.xnn();
        let step = match nn {
            0x9E => {
                // EX9E
                // Skips the next instruction if the key stored in VX is pressed. (Usually the next
                // instruction is a jump to skip a code block)
                ProgramCounterStep::cond(self.keyboard.get_keys()[self.registers[x] as usize])
            }
            0xA1 => {
                // EXA1
                // Skips the next instruction if the key stored in VX isn't pressed. (Usually the
                // next instruction is a jump to skip a code block)
                ProgramCounterStep::cond(!self.keyboard.get_keys()[self.registers[x] as usize])
            }
            _ => {
                // directly return with the given error
                return Err(format!(
                    "An unsupported opcode was used {:#06X?}",
                    self.opcode
                ));
            }
        };
        Ok(step)
    }

    fn f(&mut self, opcode: Opcode) -> Result<(ProgramCounterStep, Operation), String> {
        let (x, nn) = opcode.xnn();
        let mut op = Operation::None;
        let mut pcs = ProgramCounterStep::Next;
        match nn {
            0x07 => {
                // FX07
                // Sets VX to the value of the delay timer.
                self.registers[x] = self.get_delay_timer();
            }
            0x0A => {
                // FX0A
                // A key press is awaited, and then stored in VX. (Blocking Operation. All
                // instruction halted until next key event)
                let callback_after_keypress = move |chip: &mut Self| {
                    let last = chip.keyboard.get_last().expect(
                        "The contract that states a last key has to be set was not fullfilled.",
                    );
                    chip.registers[x] = last.get_index() as u8;
                    // move the counter to the next instruction
                    chip.step(ProgramCounterStep::Next);
                };

                op = Operation::Wait;
                // don't change the counter until the rest of the function is called.
                pcs = ProgramCounterStep::None;

                self.preprocessor = Some(Box::new(callback_after_keypress));
            }
            0x15 => {
                // FX15
                // Sets the delay timer to VX.
                self.delay_timer.set_value(self.registers[x]);
            }
            0x18 => {
                // FX18
                // Sets the sound timer to VX.
                self.sound_timer.set_value(self.registers[x]);
            }
            0x1E => {
                // FX1E
                // Adds VX to I. VF is set to 1 when there is a range overflow (I+VX>0xFFF), and to
                // 0 when there isn't. (not used in this system)
                //
                // Adds VX to I. VF is not affected.[c]
                let xi = self.registers[x] as u16;
                let res = self.index_register.wrapping_add(xi);
                self.index_register = res;
            }
            0x29 => {
                // FX29
                // Sets I to the location of the sprite for the character in VX. Characters 0-F (in
                // hexadecimal) are represented by a 4x5 font.
                // TODO: implement sprite offset
                let val = self.registers[x] as u16;
                if val > 0xF {
                    return Err(format!(
                        "The value {} has no hexadecimal representation",
                        val
                    ));
                }
                self.index_register = (FONTSET_LOCATION + 5 * (self.registers[x] as usize)) as u16;
            }
            0x33 => {
                // FX33
                // Stores the binary-coded decimal representation of VX, with the most significant
                // of three digits at the address in I, the middle digit at I plus 1, and the least
                // significant digit at I plus 2. (In other words, take the decimal representation
                // of VX, place the hundreds digit in memory at location in I, the tens digit at
                // location I+1, and the ones digit at location I+2.)
                let i = self.index_register as usize;
                let r = self.registers[x];

                self.memory[i] = r / 100; // 246u8 / 100 => 2
                self.memory[i + 1] = r / 10 % 10; // 246u8 / 10 => 24 % 10 => 4
                self.memory[i + 2] = r % 10; // 246u8 % 10 => 6
            }
            0x55 => {
                // FX55
                // Stores V0 to VX (including VX) in memory starting at address I. The offset from I
                // is increased by 1 for each value written, but I itself is left unmodified.
                let index = self.index_register as usize;
                self.memory[index..=(index + x)].copy_from_slice(&self.registers[..=x]);
            }
            0x65 => {
                // FX65
                // Fills V0 to VX (including VX) with values from memory starting at address I. The
                // offset from I is increased by 1 for each value written, but I itself is left
                // unmodified.
                let index = self.index_register as usize;
                self.registers[..=x].copy_from_slice(&self.memory[index..=(index + x)]);
            }
            _ => {
                return Err(format!(
                    "An unsupported opcode was used {:#06X?}",
                    self.opcode
                ))
            }
        }
        Ok((pcs, op))
    }
}
