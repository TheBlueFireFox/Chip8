//! Opcode abstractions, functionality and constants.
use std::convert::{TryFrom, TryInto};

use crate::{
    definitions::{cpu, memory},
    OpcodeError, ProcessError,
};

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
/// # use chip::OpcodeError;
///  const OPCODES: [Opcode; 2] = [0x00EE, 0x1EDA];
///  const SPLIT_OPCODE: [u8; 4] = [0x00, 0xEE, 0x1E, 0xDA];
///  for (i, val) in OPCODES.iter().enumerate() {
///      let opcode = build_opcode(&SPLIT_OPCODE, i * 2).expect("This will work.");
///      assert_eq!(opcode, *val);
///  }
/// # // comment this test out for the visible part, as it doesn't help showing the function usage.
/// # let pointer = 3;
/// # let err = OpcodeError::MemoryInvalid {pointer, len: SPLIT_OPCODE.len() };
/// # assert_eq!(
/// #    Err(err),
/// #    build_opcode(&SPLIT_OPCODE, pointer)
/// # );
/// # assert_eq!(
/// #   "Pointer location invalid there can not be an opcode at 3, if data len is 4".to_string(),
/// #   format!("{}", err),
/// # );
/// ```
pub fn build_opcode(data: &[u8], pointer: usize) -> Result<Opcode, OpcodeError> {
    // controlling that there is no illegal access here
    if pointer + 1 < data.len() {
        Ok(Opcode::from_be_bytes([data[pointer], data[pointer + 1]]))
    } else {
        Err(OpcodeError::MemoryInvalid {
            pointer,
            len: data.len(),
        })
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
        const MASK: u16 = OPCODE_MASK_00FF ^ OPCODE_MASK_000F;
        const NIBBLE: u16 = BYTE_SIZE / 2;
        let y = ((self & MASK) >> NIBBLE) as usize;
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
    #[inline]
    pub fn cond(cond: bool) -> Self {
        if cond {
            ProgramCounterStep::Skip
        } else {
            ProgramCounterStep::Next
        }
    }

    /// Maps the [`ProgramCounterStep`](ProgramCounterStep) to the corresponding movement distanz.
    #[inline]
    pub fn step(&self) -> usize {
        match *self {
            ProgramCounterStep::Next => memory::opcodes::SIZE,
            ProgramCounterStep::Skip => 2 * memory::opcodes::SIZE,
            ProgramCounterStep::None => 0,
            ProgramCounterStep::Jump(pointer) => {
                assert!(
                    cpu::PROGRAM_COUNTER <= pointer && pointer < memory::SIZE,
                    "Memory pointer '{:#06X}' is out of bounds error!",
                    pointer
                );

                pointer
            }
        }
    }
}

/// Inner is an internally used wrapper used for the implTryInto
/// macro. It is primarly used for converting to the correct type, without
/// disturbing its namespace.
#[repr(transparent)]
struct TryIntoHandler<T>(T);

#[inline]
fn err<T>(value: Opcode) -> Result<T, OpcodeError> {
    Err(OpcodeError::InvalidOpcode(value))
}

#[inline]
fn try_into<To, From>(val: From, value: Opcode) -> Result<To, OpcodeError>
where
    From: TryInto<TryIntoHandler<To>>,
{
    let inner: TryIntoHandler<To>;
    inner = val.try_into().or_else(|_| err(value))?;
    Ok(inner.0)
}

/// implTryInto is a macro responsible for creating the boilerplate code
/// needed for the opcode convertions.
macro_rules! implTryIntoInner {
    ( $type_name:ty : $type_from:ty : $inner:expr) => {
        impl TryFrom<$type_from> for TryIntoHandler<$type_name> {
            type Error = ();

            fn try_from(value: $type_from) -> Result<Self, Self::Error> {
                let inner = $inner(value)?;
                Ok(Self(inner))
            }
        }
    };
}

macro_rules! implTryIntoEnum {
    ($type_name:ty : $type_from:ty : $( $key:literal => $val:expr ),+ $(,)? ) => {
        implTryIntoInner!(
            $type_name : $type_from :
            |value: $type_from| {
                match value {
                    $(
                        $key => Ok($val),
                    )+
                    _ => Err(()),
                }
            }
        );
    };
}

macro_rules! implTryIntoXNN {
    ($type_name:ident) => {
        implTryIntoInner!(
            $type_name : Opcode :
            |value: Opcode| {
                let (x, nn) = value.xnn();
                Ok($type_name { x, nn })
            }
        );
    };
}

macro_rules! implTryIntoNNN {
    ($type_name:ident) => {
        implTryIntoInner! {
            $type_name: Opcode :
            |value: Opcode| {
                let nnn = value.nnn();
                Ok($type_name { nnn })
            }
        }
    };
}

macro_rules! implTryIntoXY0 {
    ($type_name:ident) => {
        implTryIntoInner! {
            $type_name: Opcode :
            |value: Opcode| {
                match value.xyn() {
                    (x, y, 0) => Ok($type_name { x, y }),
                    _ => Err(()),
                }
            }
        }
    };
}

macro_rules! implTryIntoXNNE {
    ($type_name:ident) => {
        implTryIntoInner! {
            $type_name: Opcode :
            |value: Opcode| {
                let (x, nn) = value.xnn();
                let ops = try_into(nn, value).map_err(|_| ())?;
                Ok($type_name { ops, x })
            }
        }
    };
}

macro_rules! implTryIntoXYN {
    ($type_name:ident) => {
        implTryIntoInner! {
            $type_name: Opcode :
            |value: Opcode| {
                let (x, y, n) = value.xyn();
                Ok($type_name { x, y, n })
            }
        }
    };
}

macro_rules! implTryIntoXYNE {
    ($type_name:ident) => {
        implTryIntoInner! {
            $type_name: Opcode :
            |value: Opcode| {
                let (x, y, n) = value.xyn();
                let ops = try_into(n, value).map_err(|_| ())?;
                Ok($type_name { ops, x, y })
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zero {
    /// Clears the display
    Clear,
    /// Returns from the subroutine
    Return,
}

implTryIntoEnum!(Zero : Opcode :
    // 00E0
    // clear display
    0x00E0 => Zero::Clear,
    // 00EE
    // Return from sub routine => pop from stack
    0x00EE => Zero::Return,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct One {
    pub nnn: usize,
}

implTryIntoNNN!(One);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Two {
    pub nnn: usize,
}

implTryIntoNNN!(Two);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Three {
    pub x: usize,
    pub nn: u8,
}

implTryIntoXNN!(Three);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Four {
    pub x: usize,
    pub nn: u8,
}

implTryIntoXNN!(Four);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Five {
    pub x: usize,
    pub y: usize,
}

implTryIntoXY0!(Five);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Six {
    pub x: usize,
    pub nn: u8,
}

implTryIntoXNN!(Six);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seven {
    pub x: usize,
    pub nn: u8,
}

implTryIntoXNN!(Seven);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EightOpcode {
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

implTryIntoEnum!(EightOpcode : usize :
    // 8XY0
    // Sets VX to the value of VY.
    0x0 => EightOpcode::Zero,
    // 8XY1
    // Sets VX to VX or VY. (Bitwise OR operation)
    0x1 => EightOpcode::One,
    // 8XY2
    // Sets VX to VX and VY. (Bitwise AND operation)
    0x2 => EightOpcode::Two,
    // 8XY3
    // Sets VX to VX xor VY.
    0x3 => EightOpcode::Three,
    // 8XY4
    // Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
    0x4 => EightOpcode::Four,
    // 8XY5
    // VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there
    // isn't.
    0x5 => EightOpcode::Five,
    // 8XY6
    // Stores the least significant bit of VX in VF and then shifts VX to the right
    // by 1.
    0x6 => EightOpcode::Six,
    // 8XY7
    // Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there
    // isn't.
    0x7 => EightOpcode::Seven,
    // 8XYE
    // Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
    0xE => EightOpcode::E,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eight {
    pub ops: EightOpcode,
    pub x: usize,
    pub y: usize,
}

implTryIntoXYNE!(Eight);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nine {
    pub x: usize,
    pub y: usize,
}

implTryIntoXY0!(Nine);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ten {
    pub nnn: usize,
}

implTryIntoNNN!(Ten);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eleven {
    pub nnn: usize,
}

implTryIntoNNN!(Eleven);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Twelve {
    pub x: usize,
    pub nn: u8,
}

implTryIntoXNN!(Twelve);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Thirteen {
    pub x: usize,
    pub y: usize,
    pub n: usize,
}

implTryIntoXYN!(Thirteen);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FourteenOpcode {
    Pressed,
    NotPressed,
}

implTryIntoEnum!(FourteenOpcode : u8 :
    // EX9E
    // Skips the next instruction if the key stored in VX is pressed. (Usually the next
    // instruction is a jump to skip a code block)
    0x9E => FourteenOpcode::Pressed,
    // EXA1
    // Skips the next instruction if the key stored in VX isn't pressed. (Usually the
    // next instruction is a jump to skip a code block)
    0xA1 => FourteenOpcode::NotPressed,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fourteen {
    pub ops: FourteenOpcode,
    pub x: usize,
}

implTryIntoXNNE!(Fourteen);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FifteenOpcode {
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

implTryIntoEnum!(FifteenOpcode : u8 :
    // FX07
    // Sets VX to the value of the delay timer.
    0x07 => FifteenOpcode::GetDelayTimer,
    // FX0A
    // A key press is awaited, and then stored in VX. (Blocking Operation. All
    // instruction halted until next key event)
    0x0A =>FifteenOpcode::AwaitKeyPress,
   // FX15
   // Sets the delay timer to VX.
    0x15 => FifteenOpcode::SetDelayTimer,
    // FX18
    // Sets the sound timer to VX.
    0x18 => FifteenOpcode::SetSoundTimer,
    // FX1E
    // Adds VX to I. VF is set to 1 when there is a range overflow (I+VX>0xFFF), and to
    // 0 when there isn't. (not used in this system)
    //
    // Adds VX to I. VF is not affected.[c]
    0x1E => FifteenOpcode::AddVxToI,
    // FX29
    // Sets I to the location of the sprite for the character in VX. Characters 0-F (in
    // hexadecimal) are represented by a 4x5 font.
    0x29 => FifteenOpcode::SetIToSprite,
    // FX33
    // Stores the binary-coded decimal representation of VX, with the most significant
    // of three digits at the address in I, the middle digit at I plus 1, and the least
    // significant digit at I plus 2. (In other words, take the decimal representation
    // of VX, place the hundreds digit in memory at location in I, the tens digit at
    // location I+1, and the ones digit at location I+2.)
    0x33 => FifteenOpcode::StoreBCD,
    // FX55
    // Stores V0 to VX (including VX) in memory starting at address I. The offset from I
    // is increased by 1 for each value written, but I itself is left unmodified.
    0x55 => FifteenOpcode::StoreV0ToVx,
    // FX65
    // Fills V0 to VX (including VX) with values from memory starting at address I. The
    // offset from I is increased by 1 for each value written, but I itself is left
    // unmodified.
    0x65 => FifteenOpcode::FillV0ToVx,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fifteen {
    pub ops: FifteenOpcode,
    pub x: usize,
}

implTryIntoXNNE!(Fifteen);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    A(Ten),
    B(Eleven),
    C(Twelve),
    D(Thirteen),
    E(Fourteen),
    F(Fifteen),
}

impl TryFrom<Opcode> for Opcodes {
    type Error = OpcodeError;

    fn try_from(value: Opcode) -> Result<Self, Self::Error> {
        // Outer convert
        // Shiffing t here so that match can use a loopuptable instead of a 'if else' - blocks
        const SHIFT: usize = 4 * 3;
        let t = value.t() >> SHIFT;
        let res = match t {
            0x0 => Opcodes::Zero(try_into(value, value)?),
            0x1 => Opcodes::One(try_into(value, value)?),
            0x2 => Opcodes::Two(try_into(value, value)?),
            0x3 => Opcodes::Three(try_into(value, value)?),
            0x4 => Opcodes::Four(try_into(value, value)?),
            0x5 => Opcodes::Five(try_into(value, value)?),
            0x6 => Opcodes::Six(try_into(value, value)?),
            0x7 => Opcodes::Seven(try_into(value, value)?),
            0x8 => Opcodes::Eight(try_into(value, value)?),
            0x9 => Opcodes::Nine(try_into(value, value)?),
            0xA => Opcodes::A(try_into(value, value)?),
            0xB => Opcodes::B(try_into(value, value)?),
            0xC => Opcodes::C(try_into(value, value)?),
            0xD => Opcodes::D(try_into(value, value)?),
            0xE => Opcodes::E(try_into(value, value)?),
            0xF => Opcodes::F(try_into(value, value)?),
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
    fn calc(&mut self, opcode: &Opcodes) -> Result<Operation, ProcessError> {
        // preprocess
        self.preprocess();

        let mut operation = Operation::None;
        let step_op = |(step, op)| {
            operation = op;
            step
        };

        let step = match opcode {
            Opcodes::Zero(opcode) => self.zero(opcode).map(step_op),
            Opcodes::One(opcode) => self.one(opcode),
            Opcodes::Two(opcode) => self.two(opcode),
            Opcodes::Three(opcode) => self.three(opcode),
            Opcodes::Four(opcode) => self.four(opcode),
            Opcodes::Five(opcode) => self.five(opcode),
            Opcodes::Six(opcode) => self.six(opcode),
            Opcodes::Seven(opcode) => self.seven(opcode),
            Opcodes::Eight(opcode) => self.eight(opcode),
            Opcodes::Nine(opcode) => self.nine(opcode),
            Opcodes::A(opcode) => self.a(opcode),
            Opcodes::B(opcode) => self.b(opcode),
            Opcodes::C(opcode) => self.c(opcode),
            Opcodes::D(opcode) => self.d(opcode).map(step_op),
            Opcodes::E(opcode) => self.e(opcode),
            Opcodes::F(opcode) => self.f(opcode).map(step_op),
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
    fn zero(&mut self, opcode: &Zero) -> Result<(ProgramCounterStep, Operation), ProcessError>;

    /// - `1NNN` - Flow     - `goto NNN;`           - Jumps to address `NNN`.
    ///
    /// Returns any possible error
    fn one(&self, opcode: &One) -> Result<ProgramCounterStep, ProcessError>;

    /// - `2NNN` - Flow     - `*(0xNNN)()`          - Calls subroutine at `NNN`.
    ///
    /// Returns any possible error
    fn two(&mut self, opcode: &Two) -> Result<ProgramCounterStep, ProcessError>;

    /// - `3XNN` - Cond    - `if(Vx==NN)`          - Skips the next instruction if `VX` equals `NN`. (Usually the next instruction is a jump to skip a code block)
    ///
    /// Returns any possible error
    fn three(&self, opcode: &Three) -> Result<ProgramCounterStep, ProcessError>;

    /// - `4XNN` - Cond     - `if(Vx!=NN)`          - Skips the next instruction if `VX` doesn' t equal `NN`. (Usually the next instruction is a jump to skip a code block)
    ///
    /// Returns any possible error
    fn four(&self, opcode: &Four) -> Result<ProgramCounterStep, ProcessError>;

    /// - `5XY0` - Cond     - `if(Vx==Vy)`          - Skips the next instruction if `VX` equals `VY`. (Usually the next instruction is a jump to skip a code block)
    ///
    /// Returns any possible error
    fn five(&self, opcode: &Five) -> Result<ProgramCounterStep, ProcessError>;

    /// - `6XNN` - Const    - `Vx = NN`             - Sets `VX` to `NN`.
    ///
    /// Returns any possible error
    fn six(&mut self, opcode: &Six) -> Result<ProgramCounterStep, ProcessError>;

    /// - `7XNN` - Const    - `Vx += NN`            - Adds `NN` to `VX`. (Carry flag is not changed)
    ///
    /// Returns any possible error
    fn seven(&mut self, opcode: &Seven) -> Result<ProgramCounterStep, ProcessError>;

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
    fn eight(&mut self, opcode: &Eight) -> Result<ProgramCounterStep, ProcessError>;

    /// - `9XY0` - Cond     - `if(Vx!=Vy)`          - Skips the next instruction if `VX` doesn't equal `VY`. (Usually the next instruction is a jump to skip a code block)
    ///
    /// Returns any possible error
    fn nine(&self, opcode: &Nine) -> Result<ProgramCounterStep, ProcessError>;

    /// - `ANNN` - MEM    - `I = NNN`             - Sets `I` to the address `NNN`.
    ///
    /// Returns any possible error
    fn a(&mut self, opcode: &Ten) -> Result<ProgramCounterStep, ProcessError>;

    /// - `BNNN` - Flow    - `PC=V0+NNN`           - Jumps to the address `NNN` plus `V0`.
    ///
    /// Returns any possible error
    fn b(&self, opcode: &Eleven) -> Result<ProgramCounterStep, ProcessError>;

    /// - `CXNN` - Rand     - `Vx=rand()&NN`        - Sets `VX` to the result of a bitwise and operation on a random number (Typically: `0 to 255`) and `NN`.
    ///
    /// Returns any possible error
    fn c(&mut self, opcode: &Twelve) -> Result<ProgramCounterStep, ProcessError>;

    /// - `DXYN` - Disp     - `draw(Vx,Vy,N)`       - Draws a sprite at coordinate `(VX, VY)` that has a width of `8` pixels and a height of `N` pixels. Each row of `8` pixels is read as bit-coded starting from memory location `I`; `I` value doesn’t change after the execution of this instruction. As described above, `VF` is set to `1` if any screen pixels are flipped from set to unset when the sprite is drawn, and to `0` if that doesn’t happen
    ///
    /// Returns any possible error
    fn d(&mut self, opcode: &Thirteen) -> Result<(ProgramCounterStep, Operation), ProcessError>;

    /// A multiuse opcode base for type `EXTT` (T is a sub opcode)
    ///
    /// - `EX9E` - KeyOp    - `if(key()==Vx)`       - Skips the next instruction if the key stored in `VX` is pressed. (Usually the next instruction is a jump to skip a code block)
    /// - `EXA1` - KeyOp    - `if(key()!=Vx)`       - Skips the next instruction if the key stored in `VX` isn't pressed. (Usually the next instruction is a jump to skip a code block)
    ///
    /// Returns any possible error
    fn e(&self, opcode: &Fourteen) -> Result<ProgramCounterStep, ProcessError>;

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
    fn f(&mut self, opcode: &Fifteen) -> Result<(ProgramCounterStep, Operation), ProcessError>;
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;

    #[test]
    fn test_tryfrom_opcode_simple() {
        let value = 0x00E0;
        let res = Ok(Opcodes::Zero(Zero::Clear));
        let conv = value.try_into();
        assert_eq!(conv, res);
    }

    #[test]
    fn test_tryfrom_opcode_simple_fail() {
        let value: Opcode = 0x00E1;
        let conv: Result<Opcodes, _> = value.try_into();
        assert!(conv.is_err());
    }

    #[test]
    fn test_tryfrom_opcode_multiple() {
        let tests = [
            // Zero
            (0x00E0, Ok(Opcodes::Zero(Zero::Clear))),
            (0x00EE, Ok(Opcodes::Zero(Zero::Return))),
            (0x00E1, Err("")),
            // One
            (0x1919, Ok(Opcodes::One(One { nnn: 0x919 }))),
            // Two
            (0x2222, Ok(Opcodes::Two(Two { nnn: 0x222 }))),
            // Three
            (0x3123, Ok(Opcodes::Three(Three { x: 0x1, nn: 0x23 }))),
            // Four
            (0x4123, Ok(Opcodes::Four(Four { x: 0x1, nn: 0x23 }))),
            // Five
            (0x5120, Ok(Opcodes::Five(Five { x: 0x1, y: 0x2 }))),
            (0x5121, Err("")),
            // Six
            (0x6123, Ok(Opcodes::Six(Six { x: 0x1, nn: 0x23 }))),
            // Seven
            (0x7123, Ok(Opcodes::Seven(Seven { x: 0x1, nn: 0x23 }))),
            // Eight
            (
                0x8121,
                Ok(Opcodes::Eight(Eight {
                    ops: EightOpcode::One,
                    x: 0x1,
                    y: 0x2,
                })),
            ),
            (
                0x8122,
                Ok(Opcodes::Eight(Eight {
                    ops: EightOpcode::Two,
                    x: 0x1,
                    y: 0x2,
                })),
            ),
            (
                0x8123,
                Ok(Opcodes::Eight(Eight {
                    ops: EightOpcode::Three,
                    x: 0x1,
                    y: 0x2,
                })),
            ),
            (
                0x8124,
                Ok(Opcodes::Eight(Eight {
                    ops: EightOpcode::Four,
                    x: 0x1,
                    y: 0x2,
                })),
            ),
            (
                0x8125,
                Ok(Opcodes::Eight(Eight {
                    ops: EightOpcode::Five,
                    x: 0x1,
                    y: 0x2,
                })),
            ),
            (
                0x8126,
                Ok(Opcodes::Eight(Eight {
                    ops: EightOpcode::Six,
                    x: 0x1,
                    y: 0x2,
                })),
            ),
            (
                0x8127,
                Ok(Opcodes::Eight(Eight {
                    ops: EightOpcode::Seven,
                    x: 0x1,
                    y: 0x2,
                })),
            ),
            (
                0x812E,
                Ok(Opcodes::Eight(Eight {
                    ops: EightOpcode::E,
                    x: 0x1,
                    y: 0x2,
                })),
            ),
            (0x8128, Err("")),
            // Nine
            (0x9120, Ok(Opcodes::Nine(Nine { x: 0x1, y: 0x2 }))),
            (0x9121, Err("")),
            // A
            (0xA222, Ok(Opcodes::A(Ten { nnn: 0x222 }))),
            // B
            (0xB222, Ok(Opcodes::B(Eleven { nnn: 0x222 }))),
            // C
            (0xC123, Ok(Opcodes::C(Twelve { x: 0x1, nn: 0x23 }))),
            // D
            (
                0xD123,
                Ok(Opcodes::D(Thirteen {
                    x: 0x1,
                    y: 0x2,
                    n: 0x3,
                })),
            ),
            // E
            (
                0xE19E,
                Ok(Opcodes::E(Fourteen {
                    x: 0x1,
                    ops: FourteenOpcode::Pressed,
                })),
            ),
            (
                0xE1A1,
                Ok(Opcodes::E(Fourteen {
                    x: 0x1,
                    ops: FourteenOpcode::NotPressed,
                })),
            ),
            (0xE111, Err("")),
            // F
            (
                0xF007,
                Ok(Opcodes::F(Fifteen {
                    x: 0x0,
                    ops: FifteenOpcode::GetDelayTimer,
                })),
            ),
            (
                0xF00A,
                Ok(Opcodes::F(Fifteen {
                    x: 0x0,
                    ops: FifteenOpcode::AwaitKeyPress,
                })),
            ),
            (
                0xF015,
                Ok(Opcodes::F(Fifteen {
                    x: 0x0,
                    ops: FifteenOpcode::SetDelayTimer,
                })),
            ),
            (
                0xF018,
                Ok(Opcodes::F(Fifteen {
                    x: 0x0,
                    ops: FifteenOpcode::SetSoundTimer,
                })),
            ),
            (
                0xF01E,
                Ok(Opcodes::F(Fifteen {
                    x: 0x0,
                    ops: FifteenOpcode::AddVxToI,
                })),
            ),
            (
                0xF029,
                Ok(Opcodes::F(Fifteen {
                    x: 0x0,
                    ops: FifteenOpcode::SetIToSprite,
                })),
            ),
            (
                0xF033,
                Ok(Opcodes::F(Fifteen {
                    x: 0x0,
                    ops: FifteenOpcode::StoreBCD,
                })),
            ),
            (
                0xF055,
                Ok(Opcodes::F(Fifteen {
                    x: 0x0,
                    ops: FifteenOpcode::StoreV0ToVx,
                })),
            ),
            (
                0xF065,
                Ok(Opcodes::F(Fifteen {
                    x: 0x0,
                    ops: FifteenOpcode::FillV0ToVx,
                })),
            ),
            (0xF0AA, Err("")),
        ];
        for (value, res) in tests {
            let conv: Result<Opcodes, _> = value.try_into();
            assert_eq!(conv, res.map_err(|_| OpcodeError::InvalidOpcode(value)));
        }
    }
}
