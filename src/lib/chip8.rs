
/// The size of the chipset ram
const MEMORY_SIZE : usize = 4096;
/// The size of the chipset registers
const REGISTER_SIZE : usize = 0xF;
/// The count of nesting entries
const STACK_NESTING : usize = 16;
/// The amount of herz the clocks run at in millisec
const TIMER_HERZ : u8 = 60;
/// The amount of herz the clocks run at in millisec
const TIMER_INTERVAL : u32 = 1000 / TIMER_HERZ as u32;
/// The amount of pixels the display has
const DISPLAY_RESOLUTION : usize = 64 * 23;
/// all the different keybords
const KEYBOARD_SIZE : usize = 0xF;
/// the mask for the first four bytes
const OPCODE_MASK_BASE : u16 = 0xF000;
/// the mask for the last four bytes
const OPCODE_MASK_FOUR : u16 = 0x000F;
/// the mask for the last four bytes
const OPCODE_MASK_EIGHT : u16 = 0x00FF;
/// The starting point for the program
pub const PROGRAM_COUNTER_BASE : usize = 0x200;

/// The ChipSet struct represents the current state
/// of the system, it contains all the structures 
/// needed for emulating an instant on the
/// Chip8 cpu.
pub struct ChipSet <'a>{
    /// all two bytes long and stored big-endian
    opcode : u16,
    /// 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
    /// 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
    /// 0x200-0xFFF - Program ROM and work RAM
    memory : &'a [u8; MEMORY_SIZE],
    /// 8-bit data registers named V0 to VF. The VF register doubles as a flag for some 
    /// instructions; thus, it should be avoided. In an addition operation, VF is the carry flag, 
    /// while in subtraction, it is the "no borrow" flag. In the draw instruction VF is set upon 
    /// pixel collision. 
    registers : &'a [u8; REGISTER_SIZE],
    /// The index for the register, this is a special register entrie
    /// called index I
    index_register : usize,
    // The program counter => where in the program we are
    program_counter : usize,
    /// The stack is only used to store return addresses when subroutines are called. The original 
    /// RCA 1802 version allocated 48 bytes for up to 12 levels of nesting; modern 
    /// implementations usually have more. 
    /// (here we are using 16)
    stack : &'a [u8; STACK_NESTING],
    // The stack counter => where in the stack we are
    stack_counter : usize,
    /// Delay timer: This timer is intended to be used for timing the events of games. Its value
    /// can be set and read.
    /// Counts down at 60 hertz, until it reaches 0.
    pub delay_timer : u8,
    /// Sound timer: This timer is used for sound effects. When its value is nonzero, a beeping 
    /// sound is made.
    /// Counts down at 60 hertz, until it reaches 0.
    pub sound_timer : u8,
    /// The graphics of the Chip 8 are black and white and the screen has a total of 2048 pixels 
    /// (64 x 32). This can easily be implemented using an array that hold the pixel state (1 or 0):
    pub display : &'a [u8; DISPLAY_RESOLUTION],
    /// Input is done with a hex keyboard that has 16 keys ranging 0 to F. The '8', '4', '6', and 
    /// '2' keys are typically used for directional input. Three opcodes are used to detect input.
    ///  One skips an instruction if a specific key is pressed, while another does the same if a 
    /// specific key is not pressed. The third waits for a key press, and then stores it in one of 
    /// the data registers. 
    pub keyboard : &'a [u8; KEYBOARD_SIZE]
}

impl ChipSet<'_> {
    /// will create a new chipset object
    pub fn new() -> Self {
        ChipSet {
            opcode : 0,
            memory : &[0; MEMORY_SIZE],
            registers : &[0; REGISTER_SIZE],
            index_register : 0,
            program_counter : PROGRAM_COUNTER_BASE,
            stack : &[0; STACK_NESTING],
            stack_counter : 0,
            delay_timer : TIMER_HERZ,
            sound_timer : TIMER_HERZ,
            display : &[0; DISPLAY_RESOLUTION],
            keyboard : &[0; KEYBOARD_SIZE]
        }
    }

    /// will advance the program by a single step
    pub fn step(&mut self) {
        // get next opcode
        self.opcode = u16::from_be_bytes(
            [self.memory[self.program_counter], self.memory[self.program_counter + 1]]
        );
        
        match self.opcode & OPCODE_MASK_BASE {
            0x0000 => {
                self.zero();
            },
            0x1000 => {
                self.one();
            },
            0x2000 => {
                self.two();
            },
            0x3000 => {
                self.three();
            },
            0x4000 => {
                self.four();
            },
            0x5000 => {
                self.five();
            },
            0x6000 => {
                self.six();
            },
            0x7000 => {
                self.seven();
            },
            0x8000 => {
                self.eight();
            },
            0x9000 => {
                self.nine();
            },
            0xA000 => {
                self.a();
            },
            0xB000 => {
                self.b();
            },
            0xC000 => {
                self.c();
            },
            0xD000 => {
                self.d();
            },
            0xE000 => {
                self.e();
            },
            0xF000 => {
                self.f();
            },
            _ => {
                panic!(format!("An unsupported opcode was used {:#X?}", self.opcode));
            }
        }

    }

    fn zero(&mut self) {
        match self.opcode {
            0x0E0 => {
                // 00E0
                // clear display
                
            },
            0x0EE => {
                // 00EE
                // Return from sub routine => pop from stack

            },
            _ => {
                // not needed so empty
            }
        }
    }

    fn one(&mut self) {
        // 1NNN
        // Jumps to address NNN. 

    }

    fn two(&mut self) {
        // 2NNN
        // Calls subroutine at NNN

    }

    fn three(&mut self) {
        // 3XNN
        // Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to 
        // skip a code block) 

    }

    fn four(&mut self) {
        // 4XNN
        // Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a 
        // jump to skip a code block) 

    }

    fn five(&mut self) {
        // 5XY0
        // Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to
        // skip a code block) 

    }

    fn six(&mut self) {
        // 6XNN
        // Sets VX to NN. 

    }

    fn seven(&mut self) {
        // 7XNN
        // Adds NN to VX. (Carry flag is not changed) 

    }

    fn eight(&mut self) {
        match self.opcode & OPCODE_MASK_FOUR{
            0x0000 => {
                // 8XY0
                // Sets VX to the value of VY. 

            },
            0x0001 => {
                // 8XY1
                // Sets VX to VX or VY. (Bitwise OR operation) 

            },
            0x0002 => {
                // 8XY2
                // Sets VX to VX and VY. (Bitwise AND operation) 

            },
            0x0003 => {
                // 8XY3
                // Sets VX to VX xor VY.
                
            },
            0x0004 => {
                // 8XY4
                // Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't. 

            },
            0x0005 => {
                // 8XY5
                // VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there 
                // isn't. 
            }
            0x0006 => {
                // 8XY6
                // Stores the least significant bit of VX in VF and then shifts VX to the right
                // by 1.
            }
            0x0007 => {
                // 8XY7
                // Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there 
                // isn't. 

            }
            0x000E => {
                // 8XYE
                // Stores the most significant bit of VX in VF and then shifts VX to the left by 1.

            }
            _ => {
                panic!(format!("An unsupported opcode was used {:#X?}", self.opcode));
            }
        }
    }

    fn nine(&mut self) {
        // 9XY0
        // Skips the next instruction if VX doesn't equal VY. (Usually the next instruction is
        // a jump to skip a code block) 

    }

    fn a(&mut self) {
        // ANNN
        // Sets I to the address NNN. 

    }

    fn b(&mut self) {
        // BNNN
        // Jumps to the address NNN plus V0. 

    }

    fn c(&mut self) {
        // CXNN
        // Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255)
        // and NN. 

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
        match self.opcode & OPCODE_MASK_FOUR {
            0x000E => {
                // EX9E
                // Skips the next instruction if the key stored in VX is pressed. (Usually the next 
                // instruction is a jump to skip a code block) 

            },
            0x0001 => {
                // EXA1
                // Skips the next instruction if the key stored in VX isn't pressed. (Usually the 
                // next instruction is a jump to skip a code block) 

            }
            _ => {
                panic!(format!("An unsupported opcode was used {:#X?}", self.opcode));
            }
        }
    }

    fn f(&mut self) {
        match self.opcode & OPCODE_MASK_EIGHT {
            0x007 => {
                // FX07
                // Sets VX to the value of the delay timer. 
            },
            0x00A => {
                // FX0A
                // A key press is awaited, and then stored in VX. (Blocking Operation. All 
                // instruction halted until next key event) 

            },
            0x0015 => {
                // FX15
                // Sets the delay timer to VX. 

            },
            0x0018 => {
                // FX18
                // Sets the sound timer to VX. 
            },
            0x001E => {
                // FX1E
                // Adds VX to I. VF is set to 1 when there is a range overflow (I+VX>0xFFF), and to 
                // 0 when there isn't.

            },
            0x0029 => {
                // FX29
                // Sets I to the location of the sprite for the character in VX. Characters 0-F (in
                // hexadecimal) are represented by a 4x5 font.

            },
            0x0033 => {
                // FX33
                // Stores the binary-coded decimal representation of VX, with the most significant 
                // of three digits at the address in I, the middle digit at I plus 1, and the least
                // significant digit at I plus 2. (In other words, take the decimal representation 
                // of VX, place the hundreds digit in memory at location in I, the tens digit at 
                // location I+1, and the ones digit at location I+2.) 
                
            },
            0x0055 => {
                // FX55
                // Stores V0 to VX (including VX) in memory starting at address I. The offset from I
                // is increased by 1 for each value written, but I itself is left unmodified.

            },
            0x0065 => {
                // FX65
                // Fills V0 to VX (including VX) with values from memory starting at address I. The 
                // offset from I is increased by 1 for each value written, but I itself is left 
                // unmodified.

            },
            _ => {

            }
        }
    }
}