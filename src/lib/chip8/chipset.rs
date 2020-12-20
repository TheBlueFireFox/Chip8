// this is only used in read code and not in testing
// to remove unneded warnings it was captured like this.
use {
    crate::{
        definitions::{
            DISPLAY_RESOLUTION, MEMORY_SIZE, OPCODE_BYTE_SIZE, PROGRAM_COUNTER, REGISTER_LAST,
            REGISTER_SIZE, STACK_NESTING, TIMER_HERZ,
        },
        devices::{DisplayCommands, KeyboardCommands},
        fontset::FONSET,
        opcode::{
            self, ChipOpcodes, Opcode, OpcodeTrait, Operation, ProgramCounter, ProgramCounterStep,
        },
        resources::Rom,
    },
    rand::RngCore,
};

/// The ChipSet struct represents the current state
/// of the system, it contains all the structures
/// needed for emulating an instant on the
/// Chip8 CPU.
pub struct ChipSet<T: DisplayCommands, U: KeyboardCommands> {
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
    pub(super) stack: Box<[usize]>,
    /// The stack pointer stores the address of the last program request in a stack.
    /// it points to `+1` of the actual entry, so `stack_pointer = 1` means the last requests is
    /// in `stack[0]`.
    pub(super) stack_pointer: usize,
    /// Delay timer: This timer is intended to be used for timing the events of games. Its value
    /// can be set and read.
    /// Counts down at 60 hertz, until it reaches 0.
    pub(super) delay_timer: u8,
    /// Sound timer: This timer is used for sound effects. When its value is nonzero, a beeping
    /// sound is made.
    /// Counts down at 60 hertz, until it reaches 0.
    pub(super) sound_timer: u8,
    /// The graphics of the Chip 8 are black and white and the screen has a total of `2048` pixels
    /// `(64 x 32)`. This can easily be implemented using an array that hold the pixel state `(1 or 0)`:
    pub(super) display: Box<[u8]>,
    /// Input is done with a hex keyboard that has 16 keys ranging `0-F`. The `8`, `4`, `6`, and
    /// `2` keys are typically used for directional input. Three opcodes are used to detect input.
    /// One skips an instruction if a specific key is pressed, while another does the same if a
    /// specific key is not pressed. The third waits for a key press, and then stores it in one of
    /// the data registers.
    pub(super) keyboard: U,
    /// The display adapter used to comunicate with the print instructiions.
    /// It is currently implemented as a placeholder, until a final implementation
    /// is build.
    pub(super) adapter: T,
    /// This stores the random number generator, used by the chipset.
    /// It is stored into the chipset, so as to enable simple mocking
    /// of the given type.
    pub(super) rng: Box<dyn RngCore>,
}

impl<T: DisplayCommands, U: KeyboardCommands> ChipSet<T, U> {
    /// will create a new chipset object
    pub fn new(rom: Rom, display_adapter: T, keyboard_adapter: U) -> Self {
        // initialize all the memory with 0

        let mut ram = vec![0; MEMORY_SIZE];

        // load fonts
        let mut index = 0;
        for i in FONSET.iter() {
            ram[index] = *i;
            index += 1;
        }
        index = PROGRAM_COUNTER;
        // load rom data into memory
        for i in rom.get_data() {
            ram[index] = *i;
            index += 1;
        }

        ChipSet {
            name: rom.get_name().to_string(),
            opcode: 0,
            memory: ram.into_boxed_slice(),
            registers: vec![0; REGISTER_SIZE].into_boxed_slice(),
            index_register: 0,
            program_counter: PROGRAM_COUNTER,
            stack: vec![0; STACK_NESTING].into_boxed_slice(),
            stack_pointer: 0,
            delay_timer: TIMER_HERZ,
            sound_timer: TIMER_HERZ,
            display: vec![0; DISPLAY_RESOLUTION].into_boxed_slice(),
            keyboard: keyboard_adapter,
            adapter: display_adapter,
            rng: Box::new(rand::thread_rng()),
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
        let _ = self.set_opcode()?;

        self.calc(self.opcode)
    }

    /// will return the sound timer
    pub fn get_sound_timer(&self) -> u8 {
        self.sound_timer
    }

    /// will return the delay timer
    pub fn get_delay_timer(&self) -> u8 {
        self.delay_timer
    }

    /// will return a clone of the current display configuration
    pub fn get_display(&self) -> &[u8] {
        &self.display
    }

    /// Will push the current pointer to the stack
    /// stack_counter is always one bigger then the
    /// entry it points to
    pub(super) fn push_stack(&mut self, pointer: usize) -> Result<(), &'static str> {
        if self.stack.len() == self.stack_pointer {
            Err("Stack is full!")
        } else {
            // push to stack
            self.stack[self.stack_pointer] = pointer;
            // increment stack counter
            self.stack_pointer += 1;
            Ok(())
        }
    }

    /// Will pop from the counter
    /// stack_counter is always one bigger then the entry
    /// it points to
    pub(super) fn pop_stack(&mut self) -> Result<usize, &'static str> {
        if self.stack_pointer == 0 {
            Err("Stack is empty!")
        } else {
            self.stack_pointer -= 1;
            let pointer = self.stack[self.stack_pointer];
            Ok(pointer)
        }
    }
}

impl<T: DisplayCommands, U: KeyboardCommands> ProgramCounter for ChipSet<T, U> {
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

impl<T: DisplayCommands, U: KeyboardCommands> ChipOpcodes for ChipSet<T, U> {
    fn zero(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        match opcode {
            0x00E0 => {
                // 00E0
                // clear display
                self.adapter.clear_display();
                Ok(ProgramCounterStep::Next)
            }
            0x00EE => {
                // 00EE
                // Return from sub routine => pop from stack
                self.program_counter = self.pop_stack()?;
                Ok(ProgramCounterStep::None)
            }
            _ => Err(format!(
                "An unsupported opcode was used {:#06X?}",
                self.opcode
            )),
        }
    }

    fn one(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
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

    fn three(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 3XNN
        // Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to
        // skip a code block)
        let (x, nn) = opcode.xnn();
        Ok(ProgramCounterStep::cond(self.registers[x] == nn))
    }

    fn four(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 4XNN
        // Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a
        // jump to skip a code block)
        let (x, nn) = opcode.xnn();
        Ok(ProgramCounterStep::cond(self.registers[x] != nn))
    }

    fn five(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
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
                let (res, overflow) = self.registers[x].overflowing_add(self.registers[y]);
                self.registers[x] = res;
                self.registers[REGISTER_LAST] = if overflow { 1 } else { 0 };
            }
            0x5 => {
                // 8XY5
                // VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there
                // isn't.
                let (res, overflow) = self.registers[x].overflowing_sub(self.registers[y]);
                self.registers[x] = res;
                self.registers[REGISTER_LAST] = if overflow { 1 } else { 0 };
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
                let (res, overflow) = self.registers[y].overflowing_sub(self.registers[x]);
                self.registers[x] = res;
                self.registers[REGISTER_LAST] = if overflow { 1 } else { 0 };
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

    fn nine(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
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

    fn b(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
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

    fn d(&mut self, opcode: Opcode) -> Result<(ProgramCounterStep, Operation), String> {
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

    fn e(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        let (x, nn) = opcode.xnn();
        let keyboard = self.keyboard.get_keyboard();
        let step = match nn {
            0x9E => {
                // EX9E
                // Skips the next instruction if the key stored in VX is pressed. (Usually the next
                // instruction is a jump to skip a code block)
                ProgramCounterStep::cond(keyboard[self.registers[x] as usize])
            }
            0xA1 => {
                // EXA1
                // Skips the next instruction if the key stored in VX isn't pressed. (Usually the
                // next instruction is a jump to skip a code block)
                ProgramCounterStep::cond(!keyboard[self.registers[x] as usize])
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

    fn f(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        let (x, nn) = opcode.xnn();
        match nn {
            0x7 => {
                // FX07
                // Sets VX to the value of the delay timer.
                self.registers[x] = self.delay_timer;
            }
            0xA => {
                // FX0A
                // A key press is awaited, and then stored in VX. (Blocking Operation. All
                // instruction halted until next key event)
            }
            0x15 => {
                // FX15
                // Sets the delay timer to VX.
                self.delay_timer = self.registers[x];
            }
            0x18 => {
                // FX18
                // Sets the sound timer to VX.
                self.sound_timer = self.registers[x];
            }
            0x1E => {
                // FX1E
                // Adds VX to I. VF is set to 1 when there is a range overflow (I+VX>0xFFF), and to
                // 0 when there isn't.
                // Adds VX to I. VF is not affected.[c]
                let xi = self.registers[x] as u16;
                let (res, _) = self.index_register.overflowing_add(xi);
                self.index_register = res;
            }
            0x29 => {
                // FX29
                // Sets I to the location of the sprite for the character in VX. Characters 0-F (in
                // hexadecimal) are represented by a 4x5 font.
                // TODO: this needs more work, as the front end is not yet implemented
                todo!();
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
                for i in 0..=x {
                    self.memory[index + i] = self.registers[i];
                }
            }
            0x65 => {
                // FX65
                // Fills V0 to VX (including VX) with values from memory starting at address I. The
                // offset from I is increased by 1 for each value written, but I itself is left
                // unmodified.
                let index = self.index_register as usize;
                for i in 0..=x {
                    self.registers[i] = self.memory[index + i];
                }
            }
            _ => {
                return Err(format!(
                    "An unsupported opcode was used {:#06X?}",
                    self.opcode
                ))
            }
        }
        Ok(ProgramCounterStep::Next)
    }
}
