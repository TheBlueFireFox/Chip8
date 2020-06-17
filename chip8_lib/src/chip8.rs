use {
    crate::{
        fontset::FONSET, 
        opcode::*, 
        resources::Rom
    },
    rand,
};

/// The size of the chipset ram
const MEMORY_SIZE: usize = 4096;
/// The starting point for the program
pub const PROGRAM_COUNTER: usize = 0x200;
/// The step used for calculating the program counter increments
const PROGRAM_COUNTER_STEP: usize = 2;
/// The size of the chipset registers
const REGISTER_SIZE: usize = 0xF;
/// The last entry of the registers
const REGISTER_LAST: usize = REGISTER_SIZE - 1;
/// The count of nesting entries
const STACK_NESTING: usize = 16;
/// The amount of herz the clocks run at in millisec
const TIMER_HERZ: u8 = 60;
/// The amount of herz the clocks run at in millisec
pub const TIMER_INTERVAL: u32 = 1000 / TIMER_HERZ as u32;
/// The amount of pixels the display has
const DISPLAY_RESOLUTION: usize = 64 * 23;
/// all the different keybords
const KEYBOARD_SIZE: usize = 0xF;

/// The ChipSet struct represents the current state
/// of the system, it contains all the structures
/// needed for emulating an instant on the
/// Chip8 cpu.
pub struct ChipSet {
    /// all two bytes long and stored big-endian
    opcode: Opcode,
    /// 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
    /// 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
    /// 0x200-0xFFF - Program ROM and work RAM
    memory: Vec<u8>,
    /// 8-bit data registers named V0 to VF. The VF register doubles as a flag for some
    /// instructions; thus, it should be avoided. In an addition operation, VF is the carry flag,
    /// while in subtraction, it is the "no borrow" flag. In the draw instruction VF is set upon
    /// pixel collision.
    registers: Vec<u8>,
    /// The index for the register, this is a special register entrie
    /// called index I
    index_register: u16,
    // The program counter => where in the program we are
    program_counter: usize,
    /// The stack is only used to store return addresses when subroutines are called. The original
    /// RCA 1802 version allocated 48 bytes for up to 12 levels of nesting; modern
    /// implementations usually have more.
    /// (here we are using 16)
    stack: Vec<usize>,
    // The stack counter => where in the stack we are
    stack_counter: usize,
    /// Delay timer: This timer is intended to be used for timing the events of games. Its value
    /// can be set and read.
    /// Counts down at 60 hertz, until it reaches 0.
    pub delay_timer: u8,
    /// Sound timer: This timer is used for sound effects. When its value is nonzero, a beeping
    /// sound is made.
    /// Counts down at 60 hertz, until it reaches 0.
    pub sound_timer: u8,
    /// The graphics of the Chip 8 are black and white and the screen has a total of 2048 pixels
    /// (64 x 32). This can easily be implemented using an array that hold the pixel state (1 or 0):
    pub display: Vec<u8>,
    /// Input is done with a hex keyboard that has 16 keys ranging 0 to F. The '8', '4', '6', and
    /// '2' keys are typically used for directional input. Three opcodes are used to detect input.
    /// One skips an instruction if a specific key is pressed, while another does the same if a
    /// specific key is not pressed. The third waits for a key press, and then stores it in one of
    /// the data registers.
    pub keyboard: Vec<bool>,
}
/// These are the traits that hava to be fullfilled for a working opcode
/// table
pub trait ChipOpcodes {
    /// A mutiuse opcode base for type `0NNN`
    /// 
    /// - `0NNN` - Call     -                       - Calls machine code routine ([RCA 1802](https://en.wikipedia.org/wiki/RCA_1802) for COSMAC VIP) at address `NNN`. Not necessary for most ROMs. 
    /// - `00E0` - Display  - `disp_clear()`        - Clears the screen. 
    /// - `00EE` - Flow     - `return;`             - Returns from a subroutine. 
    fn zero(&mut self);
    /// - `1NNN` - Flow     - `goto NNN;`           - Jumps to address `NNN`. 
    fn one(&mut self);
    /// - `2NNN` - Flow     - `*(0xNNN)()`          - Calls subroutine at `NNN`. 
    fn two(&mut self);
    /// - `3XNN` - Cond 	- `if(Vx==NN)`          - Skips the next instruction if `VX` equals `NN`. (Usually the next instruction is a jump to skip a code block) 
    fn three(&mut self);
    /// - `4XNN` - Cond     - `if(Vx!=NN)`          - Skips the next instruction if `VX` doesn't equal `NN`. (Usually the next instruction is a jump to skip a code block) 
    fn four(&mut self);
    /// - `5XY0` - Cond     - `if(Vx==Vy)`          - Skips the next instruction if `VX` equals `VY`. (Usually the next instruction is a jump to skip a code block) 
    fn five(&mut self);
    /// - `6XNN` - Const    - `Vx = NN`             - Sets `VX` to `NN`. 
    fn six(&mut self);
    /// - `7XNN` - Const    - `Vx += NN`            - Adds `NN` to `VX`. (Carry flag is not changed) 
    fn seven(&mut self);
    /// A mutiuse opcode base for type `8NNT` (T is a sub obcode)
    /// 
    /// - `8XY0` - Assign   - `Vx=Vy`               - Sets `VX` to the value of `VY`.
    /// - `8XY1` - BitOp    - `Vx=Vx|Vy`            - Sets `VX` to `VX` or `VY`. (Bitwise OR operation) 
    /// - `8XY2` - BitOp    - `Vx=Vx&Vy`            - Sets `VX` to `VX` and `VY`. (Bitwise AND operation) 
    /// - `8XY3` - BitOp    - `Vx=Vx^Vy`            - Sets `VX` to `VX` xor `VY`. (Bitwise XOR operation) 
    /// - `8XY4` - Math     - `Vx += Vy`            - Adds `VY` to `VX`. `VF` is set to `1` when there's a carry, and to `0` when there isn't. 
    /// - `8XY5` - Math     - `Vx -= Vy`            - `VY` is subtracted from VX. `VX` is set to `0` when there's a borrow, and `1` when there isn't. 
    /// - `8XY6` - BitOp    - `Vx>>=1`              - Stores the least significant bit of `VX` in `VF` and then shifts VX to the right by `1`.
    /// - `8XY7` - Math     - `Vx=Vy-Vx`            - Sets `VX` to `VY` minus `VX`. `VF` is set to `0` when there's a borrow, and `1` when there isn't.
    /// - `8XYE` - BitOp    - `Vx<<=1`              - Stores the most significant bit of `VX` in `VF` and then shifts `VX` to the left by `1`.
    fn eight(&mut self);
    /// - `9XY0` - Cond     - `if(Vx!=Vy)`          - Skips the next instruction if `VX` doesn't equal `VY`. (Usually the next instruction is a jump to skip a code block) 
    fn nine(&mut self);
    /// - `ANNN` - MEM      - `I = NNN`             - Sets `I` to the address `NNN`.
    fn a(&mut self);
    /// - `BNNN` - Flow 	- `PC=V0+NNN`           - Jumps to the address `NNN` plus `V0`. 
    fn b(&mut self);
    /// - `CXNN` - Rand     - `Vx=rand()&NN`        - Sets `VX` to the result of a bitwise and operation on a random number (Typically: `0 to 255`) and `NN`. 
    fn c(&mut self);
    /// - `DXYN` - Disp     - `draw(Vx,Vy,N)`       - Draws a sprite at coordinate `(VX, VY)` that has a width of `8` pixels and a height of `N` pixels. Each row of `8` pixels is read as bit-coded starting from memory location `I`; `I` value doesn’t change after the execution of this instruction. As described above, `VF` is set to `1` if any screen pixels are flipped from set to unset when the sprite is drawn, and to `0` if that doesn’t happen 
    fn d(&mut self);
    /// A mutiuse opcode base for type `EXTT` (T is a sub obcode)
    ///
    /// - `EX9E` - KeyOp    - `if(key()==Vx)`       - Skips the next instruction if the key stored in `VX` is pressed. (Usually the next instruction is a jump to skip a code block) 
    /// - `EXA1` - KeyOp    - `if(key()!=Vx)`       - Skips the next instruction if the key stored in `VX` isn't pressed. (Usually the next instruction is a jump to skip a code block)
    fn e(&mut self);
    /// A mutiuse opcode base for type `FXTT` (T is a sub obcode)
    ///
    /// - `FX07` - Timer    - `Vx = get_delay()`    - Sets `VX` to the value of the delay timer. 
    /// - `FX0A` - KeyOp    - `Vx = get_key()`      - A key press is awaited, and then stored in `VX`. (Blocking Operation. All instruction halted until next key event)
    /// - `FX15` - Timer    - `delay_timer(Vx)`     - Sets the delay timer to `VX`.
    /// - `FX18` - Sound    - `sound_timer(Vx)`     - Sets the sound timer to `VX`. 
    /// - `FX1E` - MEM      - `I +=Vx`              - Adds `VX` to `I`. `VF` is not affected.
    /// - `FX29` - MEM      - `I=sprite_addr[Vx]`   - Sets `I` to the location of the sprite for the character in `VX`. Characters `0-F` (in hexadecimal) are represented by a `4x5` font. 
    /// - `FX33` - BCD      - `246 / 100 => 2` `246 / 10 => 24 % 10 => 4` `246 % 10 => 6` - Stores the [binary-coded decimal](https://en.wikipedia.org/wiki/Binary-coded_decimal) representation of `VX`, with the most significant of three digits at the address in `I`, the middle digit at `I` plus `1`, and the least significant digit at `I` plus `2`. (In other words, take the decimal representation of `VX`, place the hundreds digit in memory at location in `I`, the tens digit at location `I+1`, and the ones digit at location `I+2`.) 
    /// - `FX55` - MEM      - `reg_dump(Vx,&I)`     - Stores `V0` to `VX`  (including `VX`) in memory starting at address `I`. The offset from `I` is increased by `1` for each value written, but `I` itself is left unmodified.
    /// - `FX65` - MEM      - `reg_load(Vx,&I)`     - Fills `V0` to `VX` (including `VX`) with values from memory starting at address `I`. The offset from `I` is increased by `1` for each value written, but `I` itself is left unmodified.
    fn f(&mut self);
}

impl ChipOpcodes for ChipSet {
    fn zero(&mut self) {
        match self.opcode {
            0x0E0 => {
                // 00E0
                // clear display
            }
            0x0EE => {
                // 00EE
                // Return from sub routine => pop from stack
                self.program_counter = self.stack[self.stack_counter];
                self.stack_counter -= 1;
            }
            _ => {
                // not needed so empty
            }
        }
    }

    fn one(&mut self) {
        // 1NNN
        // Jumps to address NNN.
        self.program_counter = (self.opcode & OPCODE_MASK_0FFF) as usize;
    }

    fn two(&mut self) {
        // 2NNN
        // Calls subroutine at NNN
        self.stack[self.stack_counter] = self.program_counter;
        self.stack_counter += 1;
        self.program_counter = (self.opcode & OPCODE_MASK_0FFF) as usize;
    }

    fn three(&mut self) {
        // 3XNN
        // Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to
        // skip a code block)
        let (x, nn) = self.opcode.xnn();
        if self.registers[x] == nn {
            self.program_counter_step(2);
        } else {
            self.program_counter_step(1);
        }
    }

    fn four(&mut self) {
        // 4XNN
        // Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a
        // jump to skip a code block)
        let (x, nn) = self.opcode.xnn();
        if self.registers[x] != nn {
            self.program_counter_step(2);
        } else {
            self.program_counter_step(1);
        }
    }

    fn five(&mut self) {
        // 5XY0
        // Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to
        // skip a code block)
        let (x, y) = self.opcode.xy();
        if self.registers[x] == self.registers[y] {
            self.program_counter_step(2);
        } else {
            self.program_counter_step(1);
        }
    }

    fn six(&mut self) {
        // 6XNN
        // Sets VX to NN.
        let (x, nn) = self.opcode.xnn();
        self.registers[x] = nn;
        self.program_counter_step(1);
    }

    fn seven(&mut self) {
        // 7XNN
        // Adds NN to VX. (Carry flag is not changed)
        let (x, nn) = self.opcode.xnn();
        self.registers[x] += nn;
        self.program_counter_step(1);
    }

    fn eight(&mut self) {
        // remove the middle 8 bits for calculations
        let (x, y) = self.opcode.xy();
        match self.opcode & OPCODE_MASK_000F {
            0x0000 => {
                // 8XY0
                // Sets VX to the value of VY.
                self.registers[x] = self.registers[y];
            }
            0x0001 => {
                // 8XY1
                // Sets VX to VX or VY. (Bitwise OR operation)
                self.registers[x] = self.registers[x] | self.registers[y];
            }
            0x0002 => {
                // 8XY2
                // Sets VX to VX and VY. (Bitwise AND operation)
                self.registers[x] = self.registers[x] & self.registers[y];
            }
            0x0003 => {
                // 8XY3
                // Sets VX to VX xor VY.
                self.registers[x] = self.registers[x] ^ self.registers[y];
            }
            0x0004 => {
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
            0x0005 => {
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
            0x0006 => {
                // 8XY6
                // Stores the least significant bit of VX in VF and then shifts VX to the right
                // by 1.
                self.registers[REGISTER_LAST] = self.registers[x] & 1;
                self.registers[x] = self.registers[x] >> 1;
            }
            0x0007 => {
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
            0x000E => {
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
    }

    fn nine(&mut self) {
        // 9XY0
        // Skips the next instruction if VX doesn't equal VY. (Usually the next instruction is
        // a jump to skip a code block)
        let (x, y) = self.opcode.xy();
        if self.registers[x] != self.registers[y] {
            self.program_counter_step(2);
        } else {
            self.program_counter_step(1);
        }
    }

    fn a(&mut self) {
        // ANNN
        // Sets I to the address NNN.
        self.index_register = self.opcode & OPCODE_MASK_0FFF;
        self.program_counter_step(1);
    }

    fn b(&mut self) {
        // BNNN
        // Jumps to the address NNN plus V0.
        let v0 = self.registers[0] as usize;
        self.program_counter = v0 + (self.opcode & OPCODE_MASK_0FFF) as usize;
        self.program_counter_step(1);
    }

    fn c(&mut self) {
        // CXNN
        // Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255)
        // and NN.
        let (x, nn) = self.opcode.xnn();
        let rand = rand::random::<u8>();
        self.registers[x] = nn & rand;
        self.program_counter_step(1);
    }

    fn d(&mut self) {
        // DXYN
        // Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N
        // pixels. Each row of 8 pixels is read as bit-coded starting from memory location I; I
        // value doesn’t change after the execution of this instruction. As described above, VF is
        // set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and
        // to 0 if that doesn’t happen
    }

    fn e(&mut self) {
        let x = self.opcode.x();
        let inc = match self.opcode & OPCODE_MASK_00FF {
            0x009E => {
                // EX9E
                // Skips the next instruction if the key stored in VX is pressed. (Usually the next
                // instruction is a jump to skip a code block)
                if self.keyboard[self.registers[x] as usize] {
                    2
                } else {
                    1
                }
            }
            0x00A1 => {
                // EXA1
                // Skips the next instruction if the key stored in VX isn't pressed. (Usually the
                // next instruction is a jump to skip a code block)
                if !self.keyboard[self.registers[x] as usize] {
                    2
                } else {
                    1
                }
            }
            _ => {
                panic!(format!(
                    "An unsupported opcode was used {:#06X?}",
                    self.opcode
                ));
            }
        };

        self.program_counter_step(inc);
    }

    fn f(&mut self) {
        let x = self.opcode.x();
        match self.opcode & OPCODE_MASK_FF00 {
            0x007 => {
                // FX07
                // Sets VX to the value of the delay timer.
                self.registers[x] = self.delay_timer;
            }
            0x00A => {
                // FX0A
                // A key press is awaited, and then stored in VX. (Blocking Operation. All
                // instruction halted until next key event)
            }
            0x0015 => {
                // FX15
                // Sets the delay timer to VX.
                self.delay_timer = self.registers[x];
            }
            0x0018 => {
                // FX18
                // Sets the sound timer to VX.
                self.sound_timer = self.registers[x];
            }
            0x001E => {
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
            0x0029 => {
                // FX29
                // Sets I to the location of the sprite for the character in VX. Characters 0-F (in
                // hexadecimal) are represented by a 4x5 font.
            }
            0x0033 => {
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
            0x0055 => {
                // FX55
                // Stores V0 to VX (including VX) in memory starting at address I. The offset from I
                // is increased by 1 for each value written, but I itself is left unmodified.
                let index = self.index_register as usize;
                for i in 0..=x {
                    self.memory[index + i] = self.registers[i];
                }
            }
            0x0065 => {
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
    }
}

impl ChipSet {
    /// will create a new chipset object
    pub fn new(rom: Rom) -> Self {
        // initialize all the memory with 0
        let mut ram = Vec::with_capacity(MEMORY_SIZE);

        // load font set
        for data in FONSET.iter() {
            ram.push(*data);
        }

        // write all the data from the rom to memory
        for data in rom.get_data() {
            ram.push(data);
        }

        // fill up the rest of memory as some roms use memory
        // space for saving information
        for _ in ram.len()..MEMORY_SIZE {
            ram.push(0);
        }

        ChipSet {
            opcode: 0,
            memory: ram,
            registers: vec![0; REGISTER_SIZE],
            index_register: 0,
            program_counter: PROGRAM_COUNTER,
            stack: vec![0; STACK_NESTING],
            stack_counter: 0,
            delay_timer: TIMER_HERZ,
            sound_timer: TIMER_HERZ,
            display: vec![0; DISPLAY_RESOLUTION],
            keyboard: vec![false; KEYBOARD_SIZE],
        }
    }

    /// will advance the program by a single step
    pub fn step(&mut self) {
        // get next opcode
        self.opcode = u16::from_be_bytes([
            self.memory[self.program_counter],
            self.memory[self.program_counter + 1],
        ]);
        match self.opcode & OPCODE_MASK_F000 {
            0x0000 => {
                self.zero();
            }
            0x1000 => {
                self.one();
            }
            0x2000 => {
                self.two();
            }
            0x3000 => {
                self.three();
            }
            0x4000 => {
                self.four();
            }
            0x5000 => {
                self.five();
            }
            0x6000 => {
                self.six();
            }
            0x7000 => {
                self.seven();
            }
            0x8000 => {
                self.eight();
            }
            0x9000 => {
                self.nine();
            }
            0xA000 => {
                self.a();
            }
            0xB000 => {
                self.b();
            }
            0xC000 => {
                self.c();
            }
            0xD000 => {
                self.d();
            }
            0xE000 => {
                self.e();
            }
            0xF000 => {
                self.f();
            }
            _ => {
                panic!(format!(
                    "An unsupported opcode was used {:#06X}",
                    self.opcode
                ));
            }
        }
    }

    fn program_counter_step(&mut self, by: usize) {
        self.program_counter += by * PROGRAM_COUNTER_STEP;
    }

}
