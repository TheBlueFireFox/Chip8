
/// the base mask used for generatring all the other sub masks
pub const OPCODE_MASK_FFFF: u16 = u16::MAX;
/// the mask for the first twelve bytes
pub const OPCODE_MASK_FFF0: u16 = OPCODE_MASK_FFFF << 4;
/// the mask for the first eight bytes
pub const OPCODE_MASK_FF00: u16 = OPCODE_MASK_FFFF << 8;
/// the mask for the first four bytes
pub const OPCODE_MASK_F000: u16 = OPCODE_MASK_FFFF << 12;
/// the mask for the last four bytes
pub const OPCODE_MASK_000F: u16 = OPCODE_MASK_FFFF ^ OPCODE_MASK_FFF0;
/// the mask for the last eight bytes
pub const OPCODE_MASK_00FF: u16 = OPCODE_MASK_FFFF ^ OPCODE_MASK_FF00;
/// the mask for the last four bytes
pub const OPCODE_MASK_0FFF: u16 = OPCODE_MASK_FFFF ^ OPCODE_MASK_F000;

/// a wrapper type for u16 to make it clear what is meant to be used
pub type Opcode = u16;

/// These are special traits used to filter out information
/// from opcodes
pub trait OpcodeTrait {

    /// this is an opcode extractor that will return the
    /// opcode number form any opcode
    /// - `T` is the opcode type
    fn t(&self) -> usize;

    /// this is an opcode extractor for the opcode type `TNNN`
    /// - `T` is the opcode type
    /// - `NNN` is a register index
    fn nnn(&self) -> usize;

    /// this is an opcode extractor for the opcode type `TXNN`
    /// - `T` is the opcode type
    /// - `X` is a register index
    /// - `NN` is a constant
    fn xnn(&self) -> (usize, u8);

    /// this is an opcode extractor for the opcode type `TXYN`
    /// - `T` is the opcode type
    /// - `X` is a register index
    /// - `Y` is a constant
    /// - `N` is a opcode subtype
    fn xyn(&self) -> (usize, usize, usize);

    /// this is an opcode extractor for the opcode type `TXYT`
    /// - `T` is the opcode type
    /// - `X` is a register index
    /// - `Y` is a constant
    fn xy(&self) -> (usize, usize);

    /// this is an opcode extractor for the opcode type `TXTT`
    /// - `T` is the opcode type
    /// - `X` is a register index
    fn x(&self) -> usize;
}

impl OpcodeTrait for Opcode {

    fn t(&self) -> usize {
        (self & OPCODE_MASK_F000) as usize
    }

    fn nnn(&self) -> usize {
        (self & OPCODE_MASK_0FFF) as usize
    }

    fn xnn(&self) -> (usize, u8) {
        let x = self.x();
        let nn = (self & OPCODE_MASK_00FF) as u8;
        (x, nn)
    }

    fn xyn(&self) -> (usize, usize, usize) {
        let (x,y) = self.xy();
        let n = (self & OPCODE_MASK_000F) as usize;
        (x,y,n)
    }

    fn xy(&self) -> (usize, usize) {
        let x = self.x();
        let y = (self & OPCODE_MASK_00FF & 0x00F0) as usize;
        (x, y)
    }

    fn x(&self) -> usize {
        (self & OPCODE_MASK_0FFF & OPCODE_MASK_FF00) as usize
    }
}

/// These are the traits that hava to be fullfilled for a working opcode
/// table
pub trait ChipOpcodes {

    /// will calculate the programs step by a single step
    fn calc(&mut self, opcode : Opcode) -> Result<(), String>{
        match opcode.t() {
            0x0000 => {
                self.zero(opcode)
            }
            0x1000 => {
                self.one(opcode)
            }
            0x2000 => {
                self.two(opcode)
            }
            0x3000 => {
                self.three(opcode)
            }
            0x4000 => {
                self.four(opcode)
            }
            0x5000 => {
                self.five(opcode)
            }
            0x6000 => {
                self.six(opcode)
            }
            0x7000 => {
                self.seven(opcode)
            }
            0x8000 => {
                self.eight(opcode)
            }
            0x9000 => {
                self.nine(opcode)
            }
            0xA000 => {
                self.a(opcode)
            }
            0xB000 => {
                self.b(opcode)
            }
            0xC000 => {
                self.c(opcode)
            }
            0xD000 => {
                self.d(opcode)
            }
            0xE000 => {
                self.e(opcode)
            }
            0xF000 => {
                self.f(opcode)
            }
            _ => {
                panic!(format!(
                    "An unsupported opcode was used {:#06X}",
                    opcode
                ));
            }
        }
    }
    
    /// A mutiuse opcode base for type `0NNN`
    ///
    /// - `0NNN` - Call     -                       - Calls machine code routine ([RCA 1802](https://en.wikipedia.org/wiki/RCA_1802) for COSMAC VIP) at address `NNN`. Not necessary for most ROMs.
    /// - `00E0` - Display  - `disp_clear()`        - Clears the screen.
    /// - `00EE` - Flow     - `return;`             - Returns from a subroutine.
    /// 
    /// Returns any possible error 
    fn zero(&mut self, opcode : Opcode) -> Result<(), String>;
    /// - `1NNN` - Flow     - `goto NNN;`           - Jumps to address `NNN`.
    /// 
    /// Returns any possible error 
    fn one(&mut self, opcode : Opcode) -> Result<(), String>;
    /// - `2NNN` - Flow     - `*(0xNNN)()`          - Calls subroutine at `NNN`.
    /// 
    /// Returns any possible error 
    fn two(&mut self, opcode : Opcode) -> Result<(), String>;
    /// - `3XNN` - Cond 	- `if(Vx==NN)`          - Skips the next instruction if `VX` equals `NN`. (Usually the next instruction is a jump to skip a code block)
    /// 
    /// Returns any possible error 
    fn three(&mut self, opcode : Opcode) -> Result<(), String>;
    /// - `4XNN` - Cond     - `if(Vx!=NN)`          - Skips the next instruction if `VX` doesn' t equal `NN`. (Usually the next instruction is a jump to skip a code block)
    /// 
    /// Returns any possible error 
    fn four(&mut self, opcode : Opcode) -> Result<(), String>;
    /// - `5XY0` - Cond     - `if(Vx==Vy)`          - Skips the next instruction if `VX` equals `VY`. (Usually the next instruction is a jump to skip a code block)
    /// 
    /// Returns any possible error 
    fn five(&mut self, opcode : Opcode) -> Result<(), String>;
    /// - `6XNN` - Const    - `Vx = NN`             - Sets `VX` to `NN`.
    /// 
    /// Returns any possible error 
    fn six(&mut self, opcode : Opcode) -> Result<(), String>;
    /// - `7XNN` - Const    - `Vx += NN`            - Adds `NN` to `VX`. (Carry flag is not changed)
    /// 
    /// Returns any possible error 
    fn seven(&mut self, opcode : Opcode) -> Result<(), String>;
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
    /// 
    /// Returns any possible error 
    fn eight(&mut self, opcode : Opcode) -> Result<(), String>;
    /// - `9XY0` - Cond     - `if(Vx!=Vy)`          - Skips the next instruction if `VX` doesn't equal `VY`. (Usually the next instruction is a jump to skip a code block)
    /// 
    /// Returns any possible error 
    fn nine(&mut self, opcode : Opcode) -> Result<(), String>;
    /// - `ANNN` - MEM      - `I = NNN`             - Sets `I` to the address `NNN`.
    /// 
    /// Returns any possible error 
    fn a(&mut self, opcode : Opcode) -> Result<(), String>;
    /// - `BNNN` - Flow 	- `PC=V0+NNN`           - Jumps to the address `NNN` plus `V0`.
    /// 
    /// Returns any possible error 
    fn b(&mut self, opcode : Opcode) -> Result<(), String>;
    /// - `CXNN` - Rand     - `Vx=rand()&NN`        - Sets `VX` to the result of a bitwise and operation on a random number (Typically: `0 to 255`) and `NN`.
    /// 
    /// Returns any possible error 
    fn c(&mut self, opcode : Opcode) -> Result<(), String>;
    /// - `DXYN` - Disp     - `draw(Vx,Vy,N)`       - Draws a sprite at coordinate `(VX, VY)` that has a width of `8` pixels and a height of `N` pixels. Each row of `8` pixels is read as bit-coded starting from memory location `I`; `I` value doesn’t change after the execution of this instruction. As described above, `VF` is set to `1` if any screen pixels are flipped from set to unset when the sprite is drawn, and to `0` if that doesn’t happen
    /// 
    /// Returns any possible error 
    fn d(&mut self, opcode : Opcode) -> Result<(), String>;
    /// A mutiuse opcode base for type `EXTT` (T is a sub obcode)
    ///
    /// - `EX9E` - KeyOp    - `if(key()==Vx)`       - Skips the next instruction if the key stored in `VX` is pressed. (Usually the next instruction is a jump to skip a code block)
    /// - `EXA1` - KeyOp    - `if(key()!=Vx)`       - Skips the next instruction if the key stored in `VX` isn't pressed. (Usually the next instruction is a jump to skip a code block)
    /// 
    /// Returns any possible error 
    fn e(&mut self, opcode : Opcode) -> Result<(), String>;
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
    /// 
    /// Returns any possible error 
    fn f(&mut self, opcode : Opcode) -> Result<(), String>;
}