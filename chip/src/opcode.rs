//! Opcode abstractions, functionality and constants.
use std::convert::{TryFrom, TryInto};

use crate::definitions::{cpu, memory};

/// the base mask used for generating all the other sub masks
pub(crate) const OPCODE_MASK_FFFF: u16 = u16::MAX;

/// the mask for the first twelve bytes
pub(crate) const OPCODE_MASK_FFF0: u16 = OPCODE_MASK_FFFF << 4;

/// the mask for the first eight bytes
pub(crate) const OPCODE_MASK_FF00: u16 = OPCODE_MASK_FFFF << 8;

/// the mask for the first four bytes
pub(crate) const OPCODE_MASK_F000: u16 = OPCODE_MASK_FFFF << 12;

/// the mask for the last four bytes
pub(crate) const OPCODE_MASK_000F: u16 = OPCODE_MASK_FFFF ^ OPCODE_MASK_FFF0;

/// the mask for the last eight bytes
pub(crate) const OPCODE_MASK_00FF: u16 = OPCODE_MASK_FFFF ^ OPCODE_MASK_FF00;

/// the mask for the last four bytes
pub(crate) const OPCODE_MASK_0FFF: u16 = OPCODE_MASK_FFFF ^ OPCODE_MASK_F000;

/// the size of a single byte
const BYTE_SIZE: u16 = 0x8;

/// a wrapper type for u16 to make it clear what is meant to be used
pub type Opcode = u16;

/// will build an opcode from data and the given point
/// # Arguments
///
/// - `data` - A slice of u8 data entries used to generate the opcodes
/// - `pointer` - Where in the data the opcode shall be extracted, so `pointer` and `pointer + 1` make
/// the opcode up
///
/// # Example
/// ```rust
/// # use chip::opcode::*;
///  const OPCODES: [Opcode; 2] = [0x00EE, 0x1EDA];
///  const SPLIT_OPCODE: [u8; 4] = [0x00, 0xEE, 0x1E, 0xDA];
///  for (i, val) in OPCODES.iter().enumerate() {
///      let opcode = build_opcode(&SPLIT_OPCODE, i * 2).expect("This will work.");
///      assert_eq!(opcode, *val);
///  }
/// # // comment this test out for the visible part, as it doesn't help showing the function usage.
/// # let pointer = 3;
/// # assert_eq!(
/// #    Err("Pointer location invalid there can not be an opcode at 3, if data len is 4".to_string()),
/// #    build_opcode(&SPLIT_OPCODE, pointer)
/// # );
/// ```
pub fn build_opcode(data: &[u8], pointer: usize) -> Result<Opcode, String> {
    // controlling that there is no illegal access here
    if pointer + 1 < data.len() {
        Ok(Opcode::from_be_bytes([data[pointer], data[pointer + 1]]))
    } else {
        Err(format!(
            "Pointer location invalid there can not be an opcode at {}, if data len is {}",
            pointer,
            data.len()
        ))
    }
}

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
    /// this is an opcode extractor that will return the
    /// opcode number form any opcode
    /// - `T` is the opcode type
    ///
    /// # Example
    /// ```rust
    /// # use chip::opcode::*;
    /// const BASE_OPCODE: Opcode = 0x1EDA;
    /// assert_eq!(BASE_OPCODE.t(), 0x1000);
    /// ```
    fn t(&self) -> usize {
        (self & OPCODE_MASK_F000) as usize
    }

    /// this is an opcode extractor for the opcode type `TNNN`
    /// - `T` is the opcode type
    /// - `NNN` is a register index
    /// this is an opcode extractor for the opcode type `TNNN`
    /// - `T` is the opcode type
    /// - `NNN` is a register index
    ///
    /// # Example
    /// ```rust
    /// # use chip::opcode::*;
    ///  const BASE_OPCODE: Opcode = 0x1EDA;
    ///  assert_eq!(BASE_OPCODE.nnn(), 0xEDA)
    /// ```
    fn nnn(&self) -> usize {
        (self & OPCODE_MASK_0FFF) as usize
    }

    /// this is an opcode extractor for the opcode type `TXNN`
    /// - `T` is the opcode type
    /// - `X` is a register index
    /// - `NN` is a constant
    ///
    /// # Example
    /// ```rust
    /// # use chip::opcode::*;
    /// const BASE_OPCODE: Opcode = 0x1EDA;
    /// assert_eq!(BASE_OPCODE.xnn(), (0xE, 0xDA));
    /// ```
    fn xnn(&self) -> (usize, u8) {
        let x = self.x();
        let nn = (self & OPCODE_MASK_00FF) as u8;
        (x, nn)
    }

    /// this is an opcode extractor for the opcode type `TXYN`
    /// - `T` is the opcode type
    /// - `X` is a register index
    /// - `Y` is a constant
    /// - `N` is a opcode subtype
    /// ```rust
    /// # use chip::opcode::*;
    ///  const BASE_OPCODE: Opcode = 0x1EDA;
    ///  assert_eq!(BASE_OPCODE.xyn(), (0xE, 0xD, 0xA));
    /// ```
    fn xyn(&self) -> (usize, usize, usize) {
        let (x, y) = self.xy();
        let n = (self & OPCODE_MASK_000F) as usize;
        (x, y, n)
    }

    /// this is an opcode extractor for the opcode type `TXYT`
    /// - `T` is the opcode type
    /// - `X` is a register index
    /// - `Y` is a constant
    /// ```rust
    /// # use chip::opcode::*;
    ///  const BASE_OPCODE: Opcode = 0x1EDA;
    ///  assert_eq!(BASE_OPCODE.xy(), (0xE, 0xD));
    /// ```
    fn xy(&self) -> (usize, usize) {
        let x = self.x();
        let y = ((self & (OPCODE_MASK_00FF ^ OPCODE_MASK_000F)) >> BYTE_SIZE / 2) as usize;
        (x, y)
    }

    /// this is an opcode extractor for the opcode type `TXTT`
    /// - `T` is the opcode type
    /// - `X` is a register index
    /// # Example
    /// ```rust
    /// # use chip::opcode::*;
    ///  const BASE_OPCODE: Opcode = 0x1EDA;
    ///  assert_eq!(BASE_OPCODE.x(), 0xE);
    /// ```
    fn x(&self) -> usize {
        ((self & OPCODE_MASK_0FFF & OPCODE_MASK_FF00) >> BYTE_SIZE) as usize
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
/// Represents the program steps that the chip
/// can take.
pub enum ProgramCounterStep {
    /// Will not change the program counter
    None,
    /// Will increment the program counter by 1
    Next,
    /// Will increment the program counter by 2
    Skip,
    /// Will simply move the program counter to the given location.
    ///
    /// Attention this can __panic__ if there is an out of bound
    /// situation.
    Jump(usize),
}

impl ProgramCounterStep {
    /// Will return a Skip if the condition is true.
    ///
    /// # Example
    /// ```rust
    /// # use chip::opcode::ProgramCounterStep;
    /// assert_eq!(ProgramCounterStep::Next, ProgramCounterStep::cond(false));
    /// assert_eq!(ProgramCounterStep::Skip, ProgramCounterStep::cond(true));
    /// ```
    pub fn cond(cond: bool) -> Self {
        if cond {
            ProgramCounterStep::Skip
        } else {
            ProgramCounterStep::Next
        }
    }

    /// Maps the [`ProgramCounterStep`](ProgramCounterStep) to the corresponding movement distanz.
    pub fn step(&self) -> usize {
        match self {
            ProgramCounterStep::Next => memory::opcodes::SIZE,
            ProgramCounterStep::Skip => 2 * memory::opcodes::SIZE,
            ProgramCounterStep::None => 0,
            ProgramCounterStep::Jump(pointer) => {
                if cpu::PROGRAM_COUNTER <= *pointer && *pointer < memory::SIZE {
                    *pointer
                } else {
                    panic!("Memory out of bounds error!")
                }
            }
        }
    }
}

pub enum Zero {
    /// Clears the display
    Clear,
    /// Returns from the subroutine
    Return,
}

impl TryFrom<usize> for Zero {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0x00E0 => Ok(Zero::Clear),
            0x00EE => Ok(Zero::Return),
            _ => Err(()),
        }
    }
}

pub struct One {
    pub nnn: usize,
}

pub struct Two {
    pub nnn: usize,
}

pub struct Three {
    pub x: usize,
    pub nn: u8,
}

pub struct Four {
    pub x: usize,
    pub nn: u8,
}

pub struct Five {
    pub x: usize,
    pub y: usize,
}

pub struct Six {
    pub x: usize,
    pub nn: u8,
}

pub struct Seven {
    pub x: usize,
    pub nn: u8,
}

pub enum EightType {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    E,
}

impl TryFrom<usize> for EightType {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let res = match value {
            0x0 => {
                // 8XY0
                // Sets VX to the value of VY.
                EightType::Zero
            }
            0x1 => {
                // 8XY1
                // Sets VX to VX or VY. (Bitwise OR operation)
                EightType::One
            }
            0x2 => {
                // 8XY2
                // Sets VX to VX and VY. (Bitwise AND operation)
                EightType::Two
            }
            0x3 => {
                // 8XY3
                // Sets VX to VX xor VY.
                EightType::Three
            }
            0x4 => {
                // 8XY4
                // Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
                EightType::Four
            }
            0x5 => {
                // 8XY5
                // VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there
                // isn't.
                EightType::Five
            }
            0x6 => {
                // 8XY6
                // Stores the least significant bit of VX in VF and then shifts VX to the right
                // by 1.
                EightType::Six
            }
            0x7 => {
                // 8XY7
                // Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there
                // isn't.
                EightType::Seven
            }
            0xE => {
                // 8XYE
                // Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
                EightType::E
            }
            _ => return Err(()),
        };
        Ok(res)
    }
}

pub struct Eight {
    pub ops: EightType,
    pub x: usize,
    pub y: usize,
}

pub struct Nine {
    pub x: usize,
    pub y: usize,
}

pub struct A {
    pub nnn: usize,
}

pub struct B {
    pub nnn: usize,
}

pub struct C {
    pub x: usize,
    pub nn: u8,
}

pub struct D {
    pub x: usize,
    pub y: usize,
    pub n: usize,
}

pub enum EType {
    Pressed,
    NotPressed,
}

pub struct E {
    pub ops: EType,
    pub x: usize,
}

pub enum FType {
    SetDelayTimer,
    SetSoundTimer,
    GetDelayTimer,
    AwaitKeyPress,
    AddVxToI,
    SetIToSprite,
    StoreBCD,
    StoreV0ToVx,
    FillV0ToVx,
}

pub struct F {
    pub ops: FType,
    pub x: usize,
}

pub enum Opcodes {
    Zero(Zero),
    One(One),
    Two(Two),
    Three(Three),
    Four(Four),
    Five(Five),
    Six(Six),
    Seven(Seven),
    Eight(Eight),
    Nine(Nine),
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
}

impl TryFrom<Opcode> for Opcodes {
    type Error = String;

    fn try_from(value: Opcode) -> Result<Self, Self::Error> {
        fn err<T>(value: Opcode) -> Result<T, String> {
            Err(format!("An unsupported opcode was used {:#06X}", value))
        }

        // Outer convert
        let t = value.t();
        let res = match t {
            0x0000 => Opcodes::Zero(t.try_into().or_else(|_| err(value))?),
            0x1000 => Opcodes::One(One { nnn: value.nnn() }),
            0x2000 => Opcodes::Two(Two { nnn: value.nnn() }),
            0x3000 => {
                let (x, nn) = value.xnn();
                Opcodes::Three(Three { x, nn })
            }
            0x4000 => {
                let (x, nn) = value.xnn();
                Opcodes::Four(Four { x, nn })
            }
            0x5000 => match value.xyn() {
                (x, y, 0) => Opcodes::Five(Five { x, y }),
                _ => return err(value),
            },
            0x6000 => {
                let (x, nn) = value.xnn();
                Opcodes::Six(Six { x, nn })
            }
            0x7000 => {
                let (x, nn) = value.xnn();
                Opcodes::Seven(Seven { x, nn })
            }
            0x8000 => {
                let (x, y, n) = value.xyn();
                let inner = n.try_into().or_else(|_| err(value))?;
                Opcodes::Eight(Eight { ops: inner, x, y })
            }
            0x9000 => match value.xyn() {
                (x, y, 0) => Opcodes::Nine(Nine { x, y }),
                _ => return err(value),
            },
            0xA000 => Opcodes::A(A { nnn: value.nnn() }),
            0xB000 => Opcodes::B(B { nnn: value.nnn() }),
            0xC000 => {
                let (x, nn) = value.xnn();
                Opcodes::C(C { x, nn })
            }
            0xD000 => {
                let (x, y, n) = value.xyn();
                Opcodes::D(D { x, y, n })
            }
            0xE000 => {
                let (x, nn) = value.xnn();
                let inner = match nn {
                    0x9E => {
                        // EX9E
                        // Skips the next instruction if the key stored in VX is pressed. (Usually the next
                        // instruction is a jump to skip a code block)
                        EType::Pressed
                    }
                    0xA1 => {
                        // EXA1
                        // Skips the next instruction if the key stored in VX isn't pressed. (Usually the
                        // next instruction is a jump to skip a code block)
                        EType::NotPressed
                    }
                    _ => return err(value),
                };
                Opcodes::E(E { ops: inner, x })
            }
            0xF000 => {
                let (x, nn) = value.xnn();
                let inner = match nn {
                    0x07 => {
                        // FX07
                        // Sets VX to the value of the delay timer.
                        FType::GetDelayTimer
                    }
                    0x0A => {
                        // FX0A
                        // A key press is awaited, and then stored in VX. (Blocking Operation. All
                        // instruction halted until next key event)
                        FType::AwaitKeyPress
                    }
                    0x15 => {
                        // FX15
                        // Sets the delay timer to VX.
                        FType::SetDelayTimer
                    }
                    0x18 => {
                        // FX18
                        // Sets the sound timer to VX.
                        FType::SetSoundTimer
                    }
                    0x1E => {
                        // FX1E
                        // Adds VX to I. VF is set to 1 when there is a range overflow (I+VX>0xFFF), and to
                        // 0 when there isn't. (not used in this system)
                        //
                        // Adds VX to I. VF is not affected.[c]
                        FType::AddVxToI
                    }
                    0x29 => {
                        // FX29
                        // Sets I to the location of the sprite for the character in VX. Characters 0-F (in
                        // hexadecimal) are represented by a 4x5 font.
                        FType::SetIToSprite
                    }
                    0x33 => {
                        // FX33
                        // Stores the binary-coded decimal representation of VX, with the most significant
                        // of three digits at the address in I, the middle digit at I plus 1, and the least
                        // significant digit at I plus 2. (In other words, take the decimal representation
                        // of VX, place the hundreds digit in memory at location in I, the tens digit at
                        // location I+1, and the ones digit at location I+2.)
                        FType::StoreBCD
                    }
                    0x55 => {
                        // FX55
                        // Stores V0 to VX (including VX) in memory starting at address I. The offset from I
                        // is increased by 1 for each value written, but I itself is left unmodified.
                        FType::StoreV0ToVx
                    }
                    0x65 => {
                        // FX65
                        // Fills V0 to VX (including VX) with values from memory starting at address I. The
                        // offset from I is increased by 1 for each value written, but I itself is left
                        // unmodified.
                        FType::FillV0ToVx
                    }
                    _ => return err(value),
                };
                Opcodes::F(F { ops: inner, x })
            }
            _ => return err(value),
        };
        Ok(res)
    }
}

/// Represents a step of the program counter
/// this requires the enum ProgramCounterStep
/// to work.
pub trait ProgramCounter {
    /// will move the program counter forward by a step.
    fn step(&mut self, step: ProgramCounterStep);
}

#[derive(Debug, PartialEq, Clone, Copy)]
/// Represents a command from the interpreter up to the gui.
pub enum Operation {
    /// If no action has to be taken.
    None,
    /// If the gui shall, wait
    /// for the next key press
    Wait,
    /// A redraw command with the individual parameters
    Draw,
}

/// Handles the preprocessing before opcode execution.
///
/// As there are opcodes, where the execution is midway stoped, until a given event happens. There is a need to restart execution from the that position, so this trait handles those cases.
pub trait ChipOpcodePreProcessHandler {
    /// Runs the preprocessed functionality.
    fn preprocess(&mut self);
}

/// These are the traits that have to be full filled for a working opcode
/// table.
///
/// This trait requires the implementation of the  [`ProgramCounter`](ProgramCounter) trait for the step
/// functionality has to be implemented as well.
/// Additionally the
/// [`ChipOpcodePreProcessHandler`](ChipOpcodePreProcessHandler) is needed as to handle a different aspect of opcode handling.
///
/// Attention: These three traits have been split up into three, so to simplify the respective
/// implementations.
pub trait ChipOpcodes: ProgramCounter + ChipOpcodePreProcessHandler {
    /// will calculate the programs step by a single step
    fn calc(&mut self, opcode: Opcode) -> Result<Operation, String> {
        // preprocess
        self.preprocess();

        let mut operation = Operation::None;
        let step_op = |(step, op)| {
            operation = op;
            step
        };

        let t = opcode.t();

        let step = match t {
            0x0000 => self.zero(opcode).map(step_op),
            0x1000 => self.one(opcode),
            0x2000 => self.two(opcode),
            0x3000 => self.three(opcode),
            0x4000 => self.four(opcode),
            0x5000 => self.five(opcode),
            0x6000 => self.six(opcode),
            0x7000 => self.seven(opcode),
            0x8000 => self.eight(opcode),
            0x9000 => self.nine(opcode),
            0xA000 => self.a(opcode),
            0xB000 => self.b(opcode),
            0xC000 => self.c(opcode),
            0xD000 => self.d(opcode).map(step_op),
            0xE000 => self.e(opcode),
            0xF000 => self.f(opcode).map(step_op),
            _ => Err(format!("An unsupported opcode was used {:#06X}", opcode)),
        }?;

        self.step(step);
        Ok(operation)
    }

    /// A multiuse opcode base for type `0NNN`
    ///
    /// - `0NNN` - Call     -                       - Calls machine code routine ([RCA 1802](https://en.wikipedia.org/wiki/RCA_1802) for COSMAC VIP) at address `NNN`. Not necessary for most ROMs.
    /// - `00E0` - Display  - `disp_clear()`        - Clears the screen.
    /// - `00EE` - Flow     - `return;`             - Returns from a subroutine.
    ///
    /// Returns any possible error
    fn zero(&mut self, opcode: Opcode) -> Result<(ProgramCounterStep, Operation), String>;

    /// - `1NNN` - Flow     - `goto NNN;`           - Jumps to address `NNN`.
    ///
    /// Returns any possible error
    fn one(&self, opcode: Opcode) -> Result<ProgramCounterStep, String>;

    /// - `2NNN` - Flow     - `*(0xNNN)()`          - Calls subroutine at `NNN`.
    ///
    /// Returns any possible error
    fn two(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String>;

    /// - `3XNN` - Cond 	- `if(Vx==NN)`          - Skips the next instruction if `VX` equals `NN`. (Usually the next instruction is a jump to skip a code block)
    ///
    /// Returns any possible error
    fn three(&self, opcode: Opcode) -> Result<ProgramCounterStep, String>;

    /// - `4XNN` - Cond     - `if(Vx!=NN)`          - Skips the next instruction if `VX` doesn' t equal `NN`. (Usually the next instruction is a jump to skip a code block)
    ///
    /// Returns any possible error
    fn four(&self, opcode: Opcode) -> Result<ProgramCounterStep, String>;

    /// - `5XY0` - Cond     - `if(Vx==Vy)`          - Skips the next instruction if `VX` equals `VY`. (Usually the next instruction is a jump to skip a code block)
    ///
    /// Returns any possible error
    fn five(&self, opcode: Opcode) -> Result<ProgramCounterStep, String>;

    /// - `6XNN` - Const    - `Vx = NN`             - Sets `VX` to `NN`.
    ///
    /// Returns any possible error
    fn six(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String>;

    /// - `7XNN` - Const    - `Vx += NN`            - Adds `NN` to `VX`. (Carry flag is not changed)
    ///
    /// Returns any possible error
    fn seven(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String>;

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
    fn eight(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String>;

    /// - `9XY0` - Cond     - `if(Vx!=Vy)`          - Skips the next instruction if `VX` doesn't equal `VY`. (Usually the next instruction is a jump to skip a code block)
    ///
    /// Returns any possible error
    fn nine(&self, opcode: Opcode) -> Result<ProgramCounterStep, String>;

    /// - `ANNN` - MEM      - `I = NNN`             - Sets `I` to the address `NNN`.
    ///
    /// Returns any possible error
    fn a(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String>;

    /// - `BNNN` - Flow 	- `PC=V0+NNN`           - Jumps to the address `NNN` plus `V0`.
    ///
    /// Returns any possible error
    fn b(&self, opcode: Opcode) -> Result<ProgramCounterStep, String>;

    /// - `CXNN` - Rand     - `Vx=rand()&NN`        - Sets `VX` to the result of a bitwise and operation on a random number (Typically: `0 to 255`) and `NN`.
    ///
    /// Returns any possible error
    fn c(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String>;

    /// - `DXYN` - Disp     - `draw(Vx,Vy,N)`       - Draws a sprite at coordinate `(VX, VY)` that has a width of `8` pixels and a height of `N` pixels. Each row of `8` pixels is read as bit-coded starting from memory location `I`; `I` value doesn’t change after the execution of this instruction. As described above, `VF` is set to `1` if any screen pixels are flipped from set to unset when the sprite is drawn, and to `0` if that doesn’t happen
    ///
    /// Returns any possible error
    fn d(&mut self, opcode: Opcode) -> Result<(ProgramCounterStep, Operation), String>;

    /// A multiuse opcode base for type `EXTT` (T is a sub opcode)
    ///
    /// - `EX9E` - KeyOp    - `if(key()==Vx)`       - Skips the next instruction if the key stored in `VX` is pressed. (Usually the next instruction is a jump to skip a code block)
    /// - `EXA1` - KeyOp    - `if(key()!=Vx)`       - Skips the next instruction if the key stored in `VX` isn't pressed. (Usually the next instruction is a jump to skip a code block)
    ///
    /// Returns any possible error
    fn e(&self, opcode: Opcode) -> Result<ProgramCounterStep, String>;

    /// A multiuse opcode base for type `FXTT` (T is a sub opcode)
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
    fn f(&mut self, opcode: Opcode) -> Result<(ProgramCounterStep, Operation), String>;
}
