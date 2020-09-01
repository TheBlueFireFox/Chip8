use {
    crate::{
        definitions::{
            DISPLAY_RESOLUTION, MEMORY_SIZE, OPCODE_BYTE_SIZE, PROGRAM_COUNTER, REGISTER_LAST,
            REGISTER_SIZE, STACK_NESTING, TIMER_HERZ,
        },
        devices::{DisplayCommands, KeyboardCommands},
        fontset::FONSET,
        opcode::{self, ChipOpcodes, Opcode, OpcodeTrait},
        resources::Rom,
    },
    rand,
};

/// The ChipSet struct represents the current state
/// of the system, it contains all the structures
/// needed for emulating an instant on the
/// Chip8 cpu.
pub struct ChipSet<T: DisplayCommands, U: KeyboardCommands> {
    /// all two bytes long and stored big-endian
    opcode: Opcode,
    /// - `0x000-0x1FF` - Chip 8 interpreter (contains font set in emu)
    /// - `0x050-0x0A0` - Used for the built in `4x5` pixel font set (`0-F`)
    /// - `0x200-0xFFF` - Program ROM and work RAM
    memory: Box<[u8]>,
    /// `8-bit` data registers named `V0` to `VF`. The `VF` register doubles as a flag for some
    /// instructions; thus, it should be avoided. In an addition operation, `VF` is the carry flag,
    /// while in subtraction, it is the "no borrow" flag. In the draw instruction `VF` is set upon
    /// pixel collision.
    registers: Box<[u8]>,
    /// The index for the register, this is a special register entry
    /// called index `I`
    index_register: u16,
    /// The program counter is a CPU register in the computer processor which has the address of the
    /// next instruction to be executed from memory.
    program_counter: usize,
    /// The stack is only used to store return addresses when subroutines are called. The original
    /// [RCA 1802](https://de.wikipedia.org/wiki/RCA1802) version allocated `48` bytes for up to
    // 12 levels of nesting; modern implementations usually have more.
    /// (here we are using 16)
    stack: Box<[usize]>,
    /// The stack pointer stores the address of the last program request in a stack.
    /// it points to `+1` of the actuall entry, so `stack_pointer = 1` means the last requests is
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
    display: Box<[u8]>,
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
    pub fn new(rom: Rom, display_adapter: T, keyboard_adapter: U) -> Self {
        // initialize all the memory with 0

        let mut ram = Box::new([0; MEMORY_SIZE]);

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
            opcode: 0,
            memory: ram,
            registers: Box::new([0; REGISTER_SIZE]),
            index_register: 0,
            program_counter: PROGRAM_COUNTER,
            stack: Box::new([0; STACK_NESTING]),
            stack_pointer: 0,
            delay_timer: TIMER_HERZ,
            sound_timer: TIMER_HERZ,
            display: Box::new([0; DISPLAY_RESOLUTION]),
            keyboard: keyboard_adapter,
            adapter: display_adapter,
        }
    }

    /// will get the next opcode from memory
    fn set_opcode(&mut self) {
        self.opcode = self.opcode_builder(self.program_counter);
    }

    /// will build the opcode given from the pointer
    fn opcode_builder(&self, pointer: usize) -> Opcode {
        opcode::build_opcode(&self.memory, pointer)
    }

    /// will advance the program by a single step
    pub fn step(&mut self) -> Result<(), String> {
        // get next opcode
        self.set_opcode();

        self.calc(self.opcode)
    }

    /// will move the program counter forward be an offset
    fn program_counter_step(&mut self, offset: usize) {
        self.program_counter += offset * OPCODE_BYTE_SIZE;
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
    pub fn get_display(&self) -> Box<[u8]> {
        self.display.clone()
    }

    /// Will move the internal program counter to the given location
    /// assumes the given pointer is pointing to a 0 initialized memory
    fn move_program_counter(&self, pointer: usize) -> Result<usize, &'static str> {
        let pointer = pointer + PROGRAM_COUNTER;

        if pointer >= self.memory.len() {
            Err("Memory out of bounds error!")
        } else {
            Ok(pointer)
        }
    }

    /// Will push the current pointer to the stack
    /// stack_counter is alwas one bigger then the
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

impl<T: DisplayCommands, U: KeyboardCommands> ChipOpcodes for ChipSet<T, U> {
    fn zero(&mut self, opcode: Opcode) -> Result<(), String> {
        match opcode {
            0x00E0 => {
                // 00E0
                // clear display
                self.adapter.clear_display();
            }
            0x00EE => {
                // 00EE
                // Return from sub routine => pop from stack

                self.program_counter = self.pop_stack()?;
            }
            _ => {
                // not needed so empty
            }
        }
        Ok(())
    }

    fn one(&mut self, opcode: Opcode) -> Result<(), String> {
        // 1NNN
        // Jumps to address NNN.
        self.program_counter = match self.move_program_counter(opcode.nnn()) {
            Ok(res) => res,
            Err(err) => return Err(String::from(err))
        };
        Ok(())
    }

    fn two(&mut self, opcode: Opcode) -> Result<(), String> {
        // 2NNN
        // Calls subroutine at NNN
        self.push_stack(self.program_counter)?;
        self.program_counter = match self.move_program_counter(opcode.nnn()) {
            Ok(res) => res,
            Err(err) => return Err(String::from(err))
        };
        Ok(())
    }

    fn three(&mut self, opcode: Opcode) -> Result<(), String> {
        // 3XNN
        // Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to
        // skip a code block)
        let (x, nn) = opcode.xnn();
        if self.registers[x] == nn {
            self.program_counter_step(2);
        } else {
            self.program_counter_step(1);
        }
        Ok(())
    }

    fn four(&mut self, opcode: Opcode) -> Result<(), String> {
        // 4XNN
        // Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a
        // jump to skip a code block)
        let (x, nn) = opcode.xnn();
        if self.registers[x] != nn {
            self.program_counter_step(2);
        } else {
            self.program_counter_step(1);
        }
        Ok(())
    }

    fn five(&mut self, opcode: Opcode) -> Result<(), String> {
        // 5XY0
        // Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to
        // skip a code block)
        let (x, y) = opcode.xy();
        if self.registers[x] == self.registers[y] {
            self.program_counter_step(2);
        } else {
            self.program_counter_step(1);
        }
        Ok(())
    }

    fn six(&mut self, opcode: Opcode) -> Result<(), String> {
        // 6XNN
        // Sets VX to NN.
        let (x, nn) = opcode.xnn();
        self.registers[x] = nn;
        self.program_counter_step(1);
        Ok(())
    }

    fn seven(&mut self, opcode: Opcode) -> Result<(), String> {
        // 7XNN
        // Adds NN to VX. (Carry flag is not changed)
        let (x, nn) = opcode.xnn();
        self.registers[x] += nn;
        self.program_counter_step(1);
        Ok(())
    }

    fn eight(&mut self, opcode: Opcode) -> Result<(), String> {
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
                        // addition worked as intendet
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
                        // addition worked as intendet
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
                        // addition worked as intendet
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
                panic!(format!(
                    "An unsupported opcode was used {:#06X?}",
                    self.opcode
                ));
            }
        }
        // increment the program counter by one
        self.program_counter_step(1);
        Ok(())
    }

    fn nine(&mut self, opcode: Opcode) -> Result<(), String> {
        // 9XY0
        // Skips the next instruction if VX doesn't equal VY. (Usually the next instruction is
        // a jump to skip a code block)
        let (x, y) = opcode.xy();
        if self.registers[x] != self.registers[y] {
            self.program_counter_step(2);
        } else {
            self.program_counter_step(1);
        }
        Ok(())
    }

    fn a(&mut self, opcode: Opcode) -> Result<(), String> {
        // ANNN
        // Sets I to the address NNN.
        self.index_register = opcode.nnn() as u16;
        self.program_counter_step(1);
        Ok(())
    }

    fn b(&mut self, opcode: Opcode) -> Result<(), String> {
        // BNNN
        // Jumps to the address NNN plus V0.
        let nnn = opcode.nnn();
        let v0 = self.registers[0] as usize;
        self.program_counter = v0 + nnn;
        self.program_counter_step(1);
        Ok(())
    }

    fn c(&mut self, opcode: Opcode) -> Result<(), String> {
        // CXNN
        // Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255)
        // and NN.
        let (x, nn) = opcode.xnn();
        let rand = rand::random::<u8>();
        self.registers[x] = nn & rand;
        self.program_counter_step(1);
        Ok(())
    }

    fn d(&mut self, opcode: Opcode) -> Result<(), String> {
        // DXYN
        // Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N
        // pixels. Each row of 8 pixels is read as bit-coded starting from memory location I; I
        // value doesn’t change after the execution of this instruction. As described above, VF is
        // set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and
        // to 0 if that doesn’t happen
        let (x, y, n) = opcode.xyn();
        panic!("Not implemented!")
    }

    fn e(&mut self, opcode: Opcode) -> Result<(), String> {
        let (x, nn) = opcode.xnn();
        let keyboard = self.keyboard.get_keyboard();
        let inc = match nn {
            0x9E => {
                // EX9E
                // Skips the next instruction if the key stored in VX is pressed. (Usually the next
                // instruction is a jump to skip a code block)
                if keyboard[self.registers[x] as usize] {
                    2
                } else {
                    1
                }
            }
            0xA1 => {
                // EXA1
                // Skips the next instruction if the key stored in VX isn't pressed. (Usually the
                // next instruction is a jump to skip a code block)
                if !keyboard[self.registers[x] as usize] {
                    2
                } else {
                    1
                }
            }
            _ => {
                return Err(format!(
                    "An unsupported opcode was used {:#06X?}",
                    self.opcode
                ));
            }
        };

        self.program_counter_step(inc);
        Ok(())
    }

    fn f(&mut self, opcode: Opcode) -> Result<(), String> {
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
                let xi = self.registers[x] as u16;
                let res = self.index_register.checked_add(xi);

                self.index_register = match res {
                    Some(res) => {
                        // addition without issues
                        self.registers[REGISTER_LAST] = 1;
                        res
                    }
                    None => {
                        self.registers[REGISTER_LAST] = 0;
                        self.index_register.wrapping_add(xi)
                    }
                }
            }
            0x29 => {
                // FX29
                // Sets I to the location of the sprite for the character in VX. Characters 0-F (in
                // hexadecimal) are represented by a 4x5 font.
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
        self.program_counter_step(1);
        Ok(())
    }
}

mod print {
    use {
        super::{ChipSet, DisplayCommands, KeyboardCommands},
        std::fmt,
    };

    /// The lenght of the pretty print data
    /// as a single instruction is u16 the ocata
    /// size will show how often the block shall
    /// be repeated has to be bigger then 0
    const HEX_PRINT_STEP: usize = 8;

    /// will add an indent post processing
    fn indent_helper(data: &str, indent: usize) -> String {
        let indent = "\t".repeat(indent);
        data.split("\n")
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

        /// The internal lenght of the given data
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

        /// using the fmt::Display for simple printing of the data later on
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
        /// as the offset is calculated from the beggining of the
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

        /// a function to keep the correct format lenght
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
            let mem = opcode_print::printer(&self.memory, 0);
            let reg = integer_print::printer(&self.registers, 0);
            let sta = integer_print::printer(&self.stack, 0);
            let key = bool_print::printer(&self.keyboard.get_keyboard(), 0);

            let mem = indent_helper(&mem, 2);
            let reg = indent_helper(&reg, 2);
            let key = indent_helper(&key, 2);
            let sta = indent_helper(&sta, 2);

            let opc = integer_print::formatter(self.opcode);
            let prc = integer_print::formatter(self.program_counter);
            let stc = integer_print::formatter(self.stack_pointer);

            write!(
                f,
                "Chipset {{ \n\
                  \tOpcode : {}\n\
                  \tProgram Pointer : {}\n\
                  \tMemory :\n{}\n\
                  \tKeybord :\n{}\n\
                  \tStack Pointer : {}\n\
                  \tStack :\n{}\n\
                  \tRegister :\n{}\n\
                }}",
                opc, prc, mem, key, stc, sta, reg
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::{ChipOpcodes, ChipSet},
        crate::{
            definitions::{MEMORY_SIZE, PROGRAM_COUNTER, STACK_NESTING},
            devices,
            resources::{Rom, RomArchives},
            opcode::Opcode
        },
        lazy_static::lazy_static,
    };

    lazy_static! {
        /// pre calculating this as it get's called multiple times per unit
        static ref BASE_ROM : Rom = {
            let mut ra = RomArchives::new();
            ra.get_file_data(&ra.file_names()[0]).unwrap()
        };
    }

    fn get_base() -> (
        Rom,
        devices::MockDisplayCommands,
        devices::MockKeyboardCommands,
    ) {
        (
            BASE_ROM.clone(),
            devices::MockDisplayCommands::new(),
            devices::MockKeyboardCommands::new(),
        )
    }

    fn set_up_default_chip() -> ChipSet<devices::MockDisplayCommands, devices::MockKeyboardCommands>
    {
        let (rom, dis, key) = get_base();
        ChipSet::new(rom, dis, key)
    }

    #[test]
    /// test clear display opcode
    fn test_clear_display_opcode() {
        let (rom, mut dis, key) = get_base();

        // setup mock
        dis.expect_clear_display().times(1).return_const(());

        let mut chip = ChipSet::new(rom, dis, key);

        // set opcode
        let opcode = 0x00E0;
        // setup chip state
        chip.opcode = opcode;
        // run - if there was no panic it worked as intened
        assert_eq!(chip.calc(opcode), Ok(()));
    }

    #[test]
    fn test_push_pop_stack() {
        let mut chip = set_up_default_chip();

        // check empty initial stack
        assert_eq!(0, chip.stack_pointer);

        let next_counter = 0x0133;
        let res = next_counter + PROGRAM_COUNTER;
        //// test move pc instructions
        // positiv test
        assert_eq!(Ok(res), chip.move_program_counter(next_counter));
        // negative test
        assert_eq!(
            Err("Memory out of bounds error!"),
            chip.move_program_counter(MEMORY_SIZE)
        );

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
    fn test_jump_address() {
        let mut chip = set_up_default_chip();
        let base  = 0x234;
        let opcode = 0x1000 ^ base as Opcode;
        let res = chip.move_program_counter(base).unwrap();
        chip.opcode = opcode;

        assert_eq!(Ok(()), chip.calc(opcode));

        assert_eq!(res, chip.program_counter);
    }

    #[test]
    /// test inserting a location into the stack
    fn test_call_subrutine() {
        let mut chip = set_up_default_chip();
        let base = 0x234;
        let res = chip.move_program_counter(base as usize).unwrap();
        let opcode = 0x2000 ^ base;
        let curr_pc = chip.program_counter;

        chip.opcode = opcode;
        
        assert_eq!(Ok(()), chip.calc(opcode));

        assert_eq!(res, chip.program_counter);

        assert_eq!(curr_pc, chip.stack[0]);
    }

    #[test]
    /// test return from subrutine
    fn test_return_subrutine() {
        let mut chip = set_up_default_chip();
        // set opcode
        let opcode = 0x00EE;
        chip.opcode = opcode;
    }
}
