
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

pub type Opcode = u16;

pub trait OpcodeTrait {
    /// this is a opcode extractor for the opcode type `TXNN`
    /// - `T` is the opcode type
    /// - `X` is a register index
    /// - `NN` is a constant
    fn xnn(&self) -> (usize, u8);

    /// this is a opcode extractor for the opcode type `TXYT`
    /// - `T` is the opcode type
    /// - `X` is a register index
    /// - `Y` is a constant
    fn xy(&self) -> (usize, usize);

    /// this is a opcode extractor for the opcode type `TXTT`
    /// - `T` is the opcode type
    /// - `X` is a register index
    fn x(&self) -> usize;
}

impl OpcodeTrait for Opcode {
    fn xnn(&self) -> (usize, u8) {
        let x = self.x();
        let nn = (self & OPCODE_MASK_00FF) as u8;
        (x, nn)
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