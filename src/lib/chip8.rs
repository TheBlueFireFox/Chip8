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
    rand,
};
/// The ChipSet struct represents the current state
/// of the system, it contains all the structures
/// needed for emulating an instant on the
/// Chip8 CPU.
pub struct ChipSet<T: DisplayCommands, U: KeyboardCommands> {
    name: String,
    /// all two bytes long and stored big-endian
    opcode: Opcode,
    /// - `0x000-0x1FF` - Chip 8 interpreter (contains font set in emu)
    /// - `0x050-0x0A0` - Used for the built in `4x5` pixel font set (`0-F`)
    /// - `0x200-0xFFF` - Program ROM and work RAM
    memory: Vec<u8>,
    /// `8-bit` data registers named `V0` to `VF`. The `VF` register doubles as a flag for some
    /// instructions; thus, it should be avoided. In an addition operation, `VF` is the carry flag,
    /// while in subtraction, it is the "no borrow" flag. In the draw instruction `VF` is set upon
    /// pixel collision.
    registers: Vec<u8>,
    /// The index for the register, this is a special register entry
    /// called index `I`
    index_register: u16,
    /// The program counter is a CPU register in the computer processor which has the address of the
    /// next instruction to be executed from memory.
    program_counter: usize,
    /// The stack is only used to store return addresses when subroutines are called. The original
    /// [RCA 1802](https://de.wikipedia.org/wiki/RCA1802) version allocated `48` bytes for up to
    /// `12` levels of nesting; modern implementations usually have more.
    /// (here we are using `16`)
    stack: Vec<usize>,
    /// The stack pointer stores the address of the last program request in a stack.
    /// it points to `+1` of the actual entry, so `stack_pointer = 1` means the last requests is
    /// in `stack[0]`.
    stack_pointer: usize,
    /// Delay timer: This timer is intended to be used for timing the events of games. Its value
    /// can be set and read.
    /// Counts down at 60 hertz, until it reaches 0.
    delay_timer: u8,
    /// Sound timer: This timer is used for sound effects. When its value is nonzero, a beeping
    /// sound is made.
    /// Counts down at 60 hertz, until it reaches 0.
    sound_timer: u8,
    /// The graphics of the Chip 8 are black and white and the screen has a total of `2048` pixels
    /// `(64 x 32)`. This can easily be implemented using an array that hold the pixel state `(1 or 0)`:
    display: Vec<u8>,
    /// Input is done with a hex keyboard that has 16 keys ranging `0-F`. The `8`, `4`, `6`, and
    /// `2` keys are typically used for directional input. Three opcodes are used to detect input.
    /// One skips an instruction if a specific key is pressed, while another does the same if a
    /// specific key is not pressed. The third waits for a key press, and then stores it in one of
    /// the data registers.
    keyboard: U,

    adapter: T,
}

impl<T: DisplayCommands, U: KeyboardCommands> ChipSet<T, U> {
    /// will create a new chipset object
    pub fn new(name: &str, rom: Rom, display_adapter: T, keyboard_adapter: U) -> Self {
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
            name: name.to_string(),
            opcode: 0,
            memory: ram,
            registers: vec![0; REGISTER_SIZE],
            index_register: 0,
            program_counter: PROGRAM_COUNTER,
            stack: vec![0; STACK_NESTING],
            stack_pointer: 0,
            delay_timer: TIMER_HERZ,
            sound_timer: TIMER_HERZ,
            display: vec![0; DISPLAY_RESOLUTION],
            keyboard: keyboard_adapter,
            adapter: display_adapter,
        }
    }

    /// will get the next opcode from memory
    fn set_opcode(&mut self) {
        // will build the opcode given from the pointer
        self.opcode = opcode::build_opcode(&self.memory, self.program_counter);
    }

    /// will advance the program by a single step
    pub fn next(&mut self) -> Result<opcode::Operation, String> {
        // get next opcode
        self.set_opcode();

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
    fn push_stack(&mut self, pointer: usize) -> Result<(), &'static str> {
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
    fn pop_stack(&mut self) -> Result<usize, &'static str> {
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
        if let Err(err) = self.push_stack(self.program_counter) {
            Err(err.to_string())
        } else {
            Ok(ProgramCounterStep::Jump(opcode.nnn()))
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
        let (x, y) = opcode.xy();
        Ok(ProgramCounterStep::cond(
            self.registers[x] == self.registers[y],
        ))
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
        let (res, _) = self.registers[x].overflowing_add(nn);
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
                let res = self.registers[x].checked_add(self.registers[y]);

                self.registers[x] = match res {
                    Some(res) => {
                        // addition worked as intended
                        // no carry
                        self.registers[REGISTER_LAST] = 0;
                        res
                    }
                    None => {
                        // addition needs carry
                        self.registers[REGISTER_LAST] = 0;
                        self.registers[x].wrapping_add(self.registers[y])
                    }
                };
            }
            0x5 => {
                // 8XY5
                // VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there
                // isn't.
                let res = self.registers[x].checked_sub(self.registers[y]);

                self.registers[x] = match res {
                    Some(res) => {
                        // addition worked as intended
                        // no carry
                        self.registers[REGISTER_LAST] = 1;
                        res
                    }
                    None => {
                        // addition needs carry
                        self.registers[REGISTER_LAST] = 0;
                        self.registers[x].wrapping_sub(self.registers[y])
                    }
                };
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
                let res = self.registers[y].checked_sub(self.registers[x]);

                self.registers[x] = match res {
                    Some(res) => {
                        // addition worked as intended
                        // no carry
                        self.registers[REGISTER_LAST] = 0;
                        res
                    }
                    None => {
                        // addition needs carry
                        self.registers[REGISTER_LAST] = 1;
                        self.registers[y].wrapping_sub(self.registers[x])
                    }
                };
            }
            0xE => {
                // 8XYE
                // Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
                self.registers[REGISTER_LAST] = self.registers[x] & (1 << 7);
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
        let (x, y) = opcode.xy();
        Ok(ProgramCounterStep::cond(
            self.registers[x] != self.registers[y],
        ))
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
        self.program_counter = v0 + nnn;
        Ok(ProgramCounterStep::Next)
    }

    fn c(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // CXNN
        // Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255)
        // and NN.
        let (x, nn) = opcode.xnn();
        let rand = rand::random::<u8>();
        self.registers[x] = nn & rand;
        Ok(ProgramCounterStep::Next)
    }

    fn d(&mut self, opcode: Opcode) -> Result<(ProgramCounterStep, Operation), String> {
        // DXYN
        // Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N
        // pixels. Each row of 8 pixels is read as bit-coded starting from memory location I; I
        // value doesn’t change after the execution of this instruction. As described above, VF is
        // set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and
        // to 0 if that doesn’t happen

        let (x, y, n) = opcode.xyn();
        let i = self.index_register as usize;
        Ok((
            ProgramCounterStep::Next,
            opcode::Operation::Draw(x, y, n, i),
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
                // TODO:
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
            _ => {}
        }
        Ok(ProgramCounterStep::Next)
    }
}

mod print {
    use {
        super::{ChipSet, DisplayCommands, KeyboardCommands},
        std::fmt,
    };

    /// The length of the pretty print data
    /// as a single instruction is u16 the octa
    /// size will show how often the block shall
    /// be repeated has to be bigger then 0
    const HEX_PRINT_STEP: usize = 8;

    /// will add an indent post processing
    ///
    /// Example
    pub fn indent_helper(text: &str, indent: usize) -> String {
        let indent = "\t".repeat(indent);
        text.split("\n")
            .map(|x| format!("{}{}\n", indent, x))
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    mod pointer_print {
        use super::integer_print;
        /// will formatt the pointers according to definition
        pub fn formatter(from: usize, to: usize) -> String {
            format!(
                "{} - {} :",
                integer_print::formatter(from),
                integer_print::formatter(to)
            )
        }
    }

    mod opcode_print {
        use {
            super::{integer_print, pointer_print, HEX_PRINT_STEP},
            crate::{
                definitions::OPCODE_BYTE_SIZE,
                opcode::{self, Opcode},
            },
            lazy_static,
            std::fmt,
        };

        /// The internal length of the given data
        /// as the data is stored as u8 and an opcode
        /// is u16 long
        const POINTER_INCREMENT: usize = HEX_PRINT_STEP * OPCODE_BYTE_SIZE;

        lazy_static::lazy_static! {
            // preparing for the 0 block fillers
            static ref ZERO_FILLER : String = {
                let formatted = integer_print::formatter(0u16);
                match HEX_PRINT_STEP {
                    1 => formatted,
                    2 => vec![formatted; 2].join(" "),
                    _ => {
                        let filler_base = "...";
                        let lenght = formatted.len() * (HEX_PRINT_STEP - 2) + (HEX_PRINT_STEP - 1)
                             - filler_base.len();
                        let filler = " ".repeat(lenght / 2);
                        format!("{}{}{}{}{}",
                            formatted.clone(),
                            filler.clone(),
                            filler_base,
                            filler,
                            formatted
                        )
                    }
                }
           };
        }

        /// this struct will simulate a single row of opcodes (only in this context)
        struct Row {
            from: usize,
            to: usize,
            data: [Opcode; HEX_PRINT_STEP],
            only_null: bool,
        }

        /// using the fmt::Display` for simple printing of the data later on
        impl fmt::Display for Row {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut res = Vec::with_capacity(HEX_PRINT_STEP + 1);
                res.push(pointer_print::formatter(self.from, self.to));

                if !self.only_null {
                    for entry in self.data.iter() {
                        res.push(integer_print::formatter(*entry));
                    }
                } else {
                    res.push(ZERO_FILLER.clone());
                }
                write!(f, "{}", res.join(" "))
            }
        }

        /// will pretty print the content of the raw memory
        /// this functions assumes the full data to be passed
        /// as the offset is calculated from the beginning of the
        /// memory block
        pub fn printer(memory: &[u8], offset: usize) -> String {
            // using the offset
            let data_last_index = memory.len() - 1;
            let mut rows: Vec<Row> = Vec::with_capacity((memory.len() - offset) / HEX_PRINT_STEP);

            for from in (offset..memory.len()).step_by(POINTER_INCREMENT) {
                // precalculate the end location
                let to = (from + POINTER_INCREMENT - 1).min(data_last_index);

                let mut data = [0; HEX_PRINT_STEP];
                let mut data_index = 0;
                let mut only_null = true;

                // loop over all the opcodes u8 pairs
                for index in (from..=to).step_by(OPCODE_BYTE_SIZE) {
                    // set the opcode
                    data[data_index] = opcode::build_opcode(memory, index);

                    // check if opcode is above 0, if so toggle the is null flag
                    if data[data_index] > 0 {
                        only_null = false;
                    }
                    data_index += 1;
                }

                // create the row that shall be used later on
                let mut row = Row {
                    from,
                    to,
                    data,
                    only_null,
                };

                if only_null {
                    if let Some(last_row) = rows.last() {
                        if last_row.only_null {
                            row.from = last_row.from;
                            rows.pop();
                        }
                    }
                }
                rows.push(row)
            }
            // create the end structure to be used for calculations
            rows.iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }

    mod integer_print {
        use {
            super::{pointer_print, HEX_PRINT_STEP},
            num,
            std::fmt,
        };
        /// will format all integer types
        pub fn formatter<T: fmt::Display + fmt::UpperHex + num::Unsigned + Copy>(
            data: T,
        ) -> String {
            format!("{:#06X}", data)
        }

        /// will pretty print all the integer data given
        pub fn printer<T: fmt::Display + fmt::UpperHex + num::Unsigned + Copy>(
            data: &[T],
            offset: usize,
        ) -> String {
            let mut res = Vec::new();
            for i in (offset..data.len()).step_by(HEX_PRINT_STEP) {
                let n = (i + HEX_PRINT_STEP - 1).min(data.len() - 1);
                let mut row = vec![pointer_print::formatter(i, n)];

                for j in i..=n {
                    row.push(formatter(data[j]));
                }
                res.push(row.join(" "));
            }
            res.join("\n")
        }
    }

    mod bool_print {
        use {
            super::{integer_print, pointer_print, HEX_PRINT_STEP},
            lazy_static,
        };

        lazy_static::lazy_static! {
            static ref TRUE : String = formatter("true");
            static ref FALSE: String = formatter("false");
        }

        /// a function to keep the correct format length
        fn formatter(string: &str) -> String {
            let mut string = string.to_string();
            let formatted = integer_print::formatter(0u16);
            while string.len() < formatted.len() {
                string.push(' ');
            }
            string
        }

        /// will pretty print all the boolean data given
        /// the offset will be calculated automatically from
        /// the data block
        pub fn printer(data: &[bool], offset: usize) -> String {
            let mut res = Vec::new();

            for i in (offset..data.len()).step_by(HEX_PRINT_STEP) {
                let n = (i + HEX_PRINT_STEP - 1).min(data.len() - 1);
                let mut row = vec![pointer_print::formatter(i, n)];

                for j in i..=n {
                    row.push(if data[j] { TRUE.clone() } else { FALSE.clone() });
                }
                res.push(row.join(" "));
            }
            res.join("\n")
        }
    }

    impl<T: DisplayCommands, U: KeyboardCommands> fmt::Display for ChipSet<T, U> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            // keeping the strings mutable so that they can be indented later on
            let mut mem = opcode_print::printer(&self.memory, 0);
            let mut reg = integer_print::printer(&self.registers, 0);
            let mut sta = integer_print::printer(&self.stack, 0);
            let mut key = bool_print::printer(&self.keyboard.get_keyboard(), 0);

            let mut opc = integer_print::formatter(self.opcode);
            let mut prc = integer_print::formatter(self.program_counter);
            let mut stc = integer_print::formatter(self.stack_pointer);

            // using a mutable slice here for convenient iterating
            let mut data = [
                &mut mem, &mut reg, &mut key, &mut sta, &mut opc, &mut prc, &mut stc,
            ];

            for string in data.iter_mut() {
                **string = indent_helper(string, 2);
            }

            write!(
                f,
                "Chipset {{\n\
                \tProgram Name: {}\n\
                \tOpcode : \n{}\n\
                \tProgram Counter: \n{}\n\
                \tMemory :\n{}\n\
                \tKeybord :\n{}\n\
                \tStack Pointer : \n{}\n\
                \tStack :\n{}\n\
                \tRegister :\n{}\n\
                }}",
                self.name, opc, prc, mem, key, stc, sta, reg
            )
        }
    }

    #[cfg(test)]
    mod tests {

        use super::*;
        #[test]
        fn test_indent_helper() {
            let text = "some relevant text\nsome more";
            let text_expected = "\t\tsome relevant text\n\t\tsome more";
            let indent = 2;
            let result = indent_helper(text, indent);
            assert_eq!(&result, text_expected);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::definitions::{KEYBOARD_SIZE, REGISTER_SIZE};

    use {
        super::{ChipOpcodes, ChipSet},
        crate::{
            definitions::{OPCODE_BYTE_SIZE, PROGRAM_COUNTER, STACK_NESTING},
            devices,
            opcode::{Opcode, Operation, ProgramCounter, ProgramCounterStep},
            resources::{Rom, RomArchives},
        },
        lazy_static::lazy_static,
        rand::prelude::*,
        std::panic,
    };

    const ROM_NAME: &'static str = "15PUZZLE";

    lazy_static! {
        /// preloading this as it get's called multiple times per unit
        static ref BASE_ROM : Rom = {
            let mut ra = RomArchives::new();
            // unwrap is safe here as this never even should be able to crash
            // and in the unlikely case that it does a panic is correct.
            ra.get_file_data(ROM_NAME).unwrap()
        };
    }

    fn get_base() -> (
        Rom,
        devices::MockDisplayCommands,
        devices::MockKeyboardCommands,
        &'static str,
    ) {
        (
            BASE_ROM.clone(),
            devices::MockDisplayCommands::new(),
            devices::MockKeyboardCommands::new(),
            ROM_NAME,
        )
    }

    /// will setup the default configured chip
    fn get_default_chip() -> ChipSet<devices::MockDisplayCommands, devices::MockKeyboardCommands> {
        let (rom, dis, key, name) = get_base();
        setup_chip(rom, dis, key, name)
    }

    fn setup_chip(
        rom: Rom,
        dis: devices::MockDisplayCommands,
        key: devices::MockKeyboardCommands,
        name: &str,
    ) -> ChipSet<devices::MockDisplayCommands, devices::MockKeyboardCommands> {
        let mut chip = ChipSet::new(name, rom, dis, key);
        // fill up register with random values
        let mut rng = rand::thread_rng();
        assert_eq!(chip.registers.len(), 16);
        chip.registers = (0..REGISTER_SIZE)
            .map(|_| {
                // 1 (inclusive) to 21 (exclusive)
                rng.gen_range(u8::MIN, u8::MAX)
            })
            .collect();

        assert_eq!(chip.registers.len(), 16);
        chip
    }

    /// Will write the opcode to the memory location specified
    fn write_opcode_to_memory(memory: &mut [u8], from: usize, opcode: Opcode) {
        write_slice_to_memory(memory, from, &opcode.to_be_bytes());
    }

    /// Will write the slice to the memory location specified
    fn write_slice_to_memory(memory: &mut [u8], from: usize, data: &[u8]) {
        for i in 0..data.len() {
            memory[from + i] = data[i];
        }
    }

    #[test]
    /// tests if the pretty print output is as expected
    /// this test is mainly for coverage purposes, as
    /// the given module takes up a multitude of lines.
    fn test_full_print() {
        const OUTPUT: &str = r#"Chipset {
	Program Name: 15PUZZLE
	Opcode : 
		0x0000
	Program Counter: 
		0x0200
	Memory :
		0x0000 - 0x000F : 0xF090 0x9090 0xF020 0x6020 0x2070 0xF010 0xF080 0xF0F0
		0x0010 - 0x001F : 0x10F0 0x10F0 0x9090 0xF010 0x10F0 0x80F0 0x10F0 0xF080
		0x0020 - 0x002F : 0xF090 0xF0F0 0x1020 0x4040 0xF090 0xF090 0xF0F0 0x90F0
		0x0030 - 0x003F : 0x10F0 0xF090 0xF090 0x90E0 0x90E0 0x90E0 0xF080 0x8080
		0x0040 - 0x004F : 0xF0E0 0x9090 0x90E0 0xF080 0xF080 0xF0F0 0x80F0 0x8080
		0x0050 - 0x01FF : 0x0000                    ...                    0x0000
		0x0200 - 0x020F : 0x00E0 0x6C00 0x4C00 0x6E0F 0xA203 0x6020 0xF055 0x00E0
		0x0210 - 0x021F : 0x22BE 0x2276 0x228E 0x225E 0x2246 0x1210 0x6100 0x6217
		0x0220 - 0x022F : 0x6304 0x4110 0x00EE 0xA2E8 0xF11E 0xF065 0x4000 0x1234
		0x0230 - 0x023F : 0xF029 0xD235 0x7101 0x7205 0x6403 0x8412 0x3400 0x1222
		0x0240 - 0x024F : 0x6217 0x7306 0x1222 0x6403 0x84E2 0x6503 0x85D2 0x9450
		0x0250 - 0x025F : 0x00EE 0x4403 0x00EE 0x6401 0x84E4 0x22A6 0x1246 0x6403
		0x0260 - 0x026F : 0x84E2 0x6503 0x85D2 0x9450 0x00EE 0x4400 0x00EE 0x64FF
		0x0270 - 0x027F : 0x84E4 0x22A6 0x125E 0x640C 0x84E2 0x650C 0x85D2 0x9450
		0x0280 - 0x028F : 0x00EE 0x4400 0x00EE 0x64FC 0x84E4 0x22A6 0x1276 0x640C
		0x0290 - 0x029F : 0x84E2 0x650C 0x85D2 0x9450 0x00EE 0x440C 0x00EE 0x6404
		0x02A0 - 0x02AF : 0x84E4 0x22A6 0x128E 0xA2E8 0xF41E 0xF065 0xA2E8 0xFE1E
		0x02B0 - 0x02BF : 0xF055 0x6000 0xA2E8 0xF41E 0xF055 0x8E40 0x00EE 0x3C00
		0x02C0 - 0x02CF : 0x12D2 0x221C 0x22D8 0x221C 0xA2F8 0xFD1E 0xF065 0x8D00
		0x02D0 - 0x02DF : 0x00EE 0x7CFF 0xCD0F 0x00EE 0x7D01 0x600F 0x8D02 0xED9E
		0x02E0 - 0x02EF : 0x12D8 0xEDA1 0x12E2 0x00EE 0x0102 0x0304 0x0506 0x0708
		0x02F0 - 0x02FF : 0x090A 0x0B0C 0x0D0E 0x0F00 0x0D00 0x0102 0x0405 0x0608
		0x0300 - 0x030F : 0x090A 0x0C0E 0x0307 0x0B0F 0x84E4 0x22A6 0x1276 0x640C
		0x0310 - 0x031F : 0x84E2 0x650C 0x85D2 0x9450 0x00EE 0x440C 0x00EE 0x6404
		0x0320 - 0x032F : 0x84E4 0x22A6 0x128E 0xA2E8 0xF41E 0xF065 0xA2E8 0xFE1E
		0x0330 - 0x033F : 0xF055 0x6000 0xA2E8 0xF41E 0xF055 0x8E40 0x00EE 0x3C00
		0x0340 - 0x034F : 0x12D2 0x221C 0x22D8 0x221C 0xA2F8 0xFD1E 0xF065 0x8D00
		0x0350 - 0x035F : 0x00EE 0x7CFF 0xCD0F 0x00EE 0x7D01 0x600F 0x8D02 0xED9E
		0x0360 - 0x036F : 0x12D8 0xEDA1 0x12E2 0x00EE 0x0102 0x0304 0x0506 0x0708
		0x0370 - 0x037F : 0x090A 0x0B0C 0x0D0E 0x0F00 0x0D00 0x0102 0x0405 0x0608
		0x0380 - 0x0FFF : 0x0000                    ...                    0x0000
	Keybord :
		0x0000 - 0x0007 : false  true   false  true   false  true   false  true  
		0x0008 - 0x000F : false  true   false  true   false  true   false  true
	Stack Pointer : 
		0x0000
	Stack :
		0x0000 - 0x0007 : 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000
		0x0008 - 0x000F : 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000
	Register :
		0x0000 - 0x0007 : 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000
		0x0008 - 0x000F : 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000
}"#;
        let (rom, dis, mut key, name) = get_base();
        let keys = (0..KEYBOARD_SIZE)
            .map(|i| i % 2 != 0)
            .collect::<Vec<bool>>()
            .into_boxed_slice();
        key.expect_get_keyboard().returning(move || keys.clone());
        let mut chip = setup_chip(rom, dis, key, name);

        // override the chip register as they are generated randomly

        chip.registers = (0..REGISTER_SIZE).map(|_| 0 as u8).collect();
        assert_eq!(format!("{}", chip), OUTPUT);
    }

    #[test]
    /// test reading of the first opcode
    fn test_set_opcode() {
        let mut chip = get_default_chip();
        chip.set_opcode();
        let opcode = chip.opcode;
        assert_eq!(0x00E0, opcode);
    }

    #[test]
    /// testing internal functionality of popping and pushing into the stack
    fn test_push_pop_stack() {
        let mut chip = get_default_chip();

        // check empty initial stack
        assert_eq!(0, chip.stack_pointer);

        let next_counter = 0x0133 + PROGRAM_COUNTER;

        for i in 0..STACK_NESTING {
            // as the stack is empty just accept the result
            assert_eq!(Ok(()), chip.push_stack(next_counter + i * 8));
        }
        // check for the correct error message
        assert_eq!(Err("Stack is full!"), chip.push_stack(next_counter));

        // check if the stack counter moved as expected
        assert_eq!(STACK_NESTING, chip.stack_pointer);
        // pop the stack
        for i in (0..STACK_NESTING).rev() {
            assert_eq!(Ok(next_counter + i * 8), chip.pop_stack());
        }
        assert_eq!(0, chip.stack_pointer);
        // test if stack is now empty
        assert_eq!(Err("Stack is empty!"), chip.pop_stack());
    }

    #[test]
    fn test_step() {
        let mut chip = get_default_chip();
        let mut pc = chip.program_counter;

        pc = pc + OPCODE_BYTE_SIZE;
        chip.step(ProgramCounterStep::Next);
        assert_eq!(chip.program_counter, pc);

        pc = pc + 2 * OPCODE_BYTE_SIZE;
        chip.step(ProgramCounterStep::Skip);
        assert_eq!(chip.program_counter, pc);

        pc = pc + 8 * OPCODE_BYTE_SIZE;
        chip.step(ProgramCounterStep::Jump(pc));
        assert_eq!(chip.program_counter, pc);

        chip.step(ProgramCounterStep::None);
        assert_eq!(chip.program_counter, pc);
    }

    #[test]
    #[should_panic(expected = "Memory out of bounds error!")]
    fn test_step_panic_lower_bound() {
        let mut chip = get_default_chip();
        let pc = PROGRAM_COUNTER - 1;
        chip.step(ProgramCounterStep::Jump(pc));
    }

    #[test]
    #[should_panic(expected = "Memory out of bounds error!")]
    fn test_step_panic_upper_bound() {
        let mut chip = get_default_chip();
        let pc = chip.memory.len();
        chip.step(ProgramCounterStep::Jump(pc));
    }

    #[test]
    /// test clear display opcode and next (for coverage)
    /// `0x00E0`
    fn test_clear_display_opcode() {
        let (rom, mut dis, key, name) = get_base();

        // setup mock
        // will assert to __false__ if condition is not
        // met
        dis.expect_clear_display().times(1).return_const(());

        let mut chip = setup_chip(rom, dis, key, name);

        // as the first opcode used is already clear screen no
        // modifications are needed.

        // run - if there was no panic it worked as intended
        assert_eq!(chip.next(), Ok(Operation::None));
    }

    #[test]
    /// test return from subroutine
    /// `0x00EE`
    fn test_return_subrutine() {
        let mut chip = get_default_chip();
        let curr_pc = chip.program_counter;
        // set up test
        let base = 0x234;
        let opcode: Opcode = 0x2000 ^ base;

        // write the to subroutine to memory
        chip.opcode = opcode;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));
        // set opcode
        let opcode = 0x00EE;

        // write bytes to chip memory
        write_opcode_to_memory(&mut chip.memory, chip.program_counter, opcode);
        chip.opcode = opcode;

        assert_eq!(Ok(Operation::None), chip.next());

        assert_eq!(curr_pc, chip.program_counter)
    }

    #[test]
    fn test_illigal_zero_opcode() {
        let mut chip = get_default_chip();
        let opcode = 0x00EA;
        write_opcode_to_memory(&mut chip.memory, chip.program_counter, opcode);
        assert_eq!(Err("An unsupported opcode was used 0x00EA"), chip.next());
    }

    #[test]
    /// test a simple jump to the next address
    /// `1NNN`
    fn test_jump_address() {
        let mut chip = get_default_chip();
        let base = 0x0234;
        let opcode = 0x1000 ^ base as Opcode;
        // let _ = chip.move_program_counter(base);
        chip.step(ProgramCounterStep::Jump(base));
        chip.opcode = opcode;

        assert_eq!(chip.calc(opcode), Ok(Operation::None));

        assert_eq!(base, chip.program_counter);
    }

    #[test]
    /// test inserting a location into the stack
    /// "2NNN"
    fn test_call_subrutine() {
        let mut chip = get_default_chip();
        let base = 0x234;
        let opcode = 0x2000 ^ base;
        let curr_pc = chip.program_counter;

        chip.opcode = opcode;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(base as usize, chip.program_counter);

        assert_eq!(curr_pc, chip.stack[0]);
    }

    #[test]
    /// test the skip instruction if equal method
    /// `3XNN`
    fn test_skip_instruction_if_const_equals() {
        let mut chip = get_default_chip();
        let register = 0x1;
        let solution = 0x3;
        // skip register 1 if it is equal to 03
        let opcode = 0x3 << (3 * 4) ^ (register << (2 * 4)) ^ solution;

        let curr_pc = chip.program_counter;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);

        let curr_pc = chip.program_counter;
        chip.registers[register as usize] = solution as u8;
        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.program_counter, curr_pc + 2 * OPCODE_BYTE_SIZE);
    }

    #[test]
    /// `4XNN`
    /// Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a
    /// jump to skip a code block)
    fn test_skip_instruction_if_const_not_equals() {
        let mut chip = get_default_chip();
        let register = 0x1;
        let solution = 0x3;
        // skip register 1 if it is not equal to 03
        let opcode = 0x4 << (3 * 4) ^ (register << (2 * 4)) ^ solution;

        // will not skip next instruction
        let curr_pc = chip.program_counter;
        chip.registers[register as usize] = solution as u8;
        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);

        // skip next block because it's not equal
        let curr_pc = chip.program_counter;
        chip.registers[register as usize] = 0x66;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.program_counter, curr_pc + 2 * OPCODE_BYTE_SIZE);
    }

    #[test]
    /// 5XY0
    /// Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to
    /// skip a code block)
    fn test_skip_instruction_if_register_equals() {
        let mut chip = get_default_chip();
        let registery = 0x1;
        let registerx = 0x2;
        // skip register 1 if VY is not equals to VX
        let opcode = 0x5 << (3 * 4) ^ (registerx << (2 * 4)) ^ (registery << (1 * 4));

        // setup register for a none skip
        chip.registers[registerx as usize] = 0x6;
        chip.registers[registery as usize] = 0x66;
        // will not skip next instruction
        let curr_pc = chip.program_counter;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);

        // skip next block because it's not equal
        // setup register
        chip.registers[registerx as usize] = 0x66;
        chip.registers[registery as usize] = 0x66;
        // copy current state of program counter
        let curr_pc = chip.program_counter;
        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.program_counter, curr_pc + 2 * OPCODE_BYTE_SIZE);
    }

    #[test]
    /// 6XNN
    /// Sets VX to NN.
    fn test_set_vx_to_nn() {
        let mut chip = get_default_chip();
        let register = 0x1;
        let value = 0x66 & chip.registers[register];
        let curr_pc = chip.program_counter;
        chip.registers[register] = value;
        // skip register 1 if VY is not equals to VX
        let opcode: Opcode = 0x6 << (3 * 4) ^ ((register as u16) << (2 * 4)) ^ (value as u16);

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(value, chip.registers[register]);

        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);
    }

    #[test]
    /// 7XNN
    /// Adds NN to VX. (Carry flag is not changed)
    fn test_add_nn_to_vx() {
        let mut chip = get_default_chip();
        let register = 0x1;
        let value = 0x66 & chip.registers[register];
        let curr_pc = chip.program_counter;
        chip.registers[register] = value;
        // skip register 1 if VY is not equals to VX
        let opcode: Opcode = 0x7 << (3 * 4) ^ ((register as u16) << (2 * 4)) ^ (value as u16);

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        let (res, _) = value.overflowing_add(value);
        assert_eq!(res, chip.registers[register]);

        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);
    }
}
