use thiserror::Error;

use crate::opcode::Opcode;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum ProcessError {
    #[error("Invalid opcode state '{0}'.")]
    Opcode(#[from] OpcodeError),
    #[error("Invalid calculation '{0}'")]
    Calculation(String),
    #[error("Invalid stack state '{0}'.")]
    Stack(#[from] StackError),
    #[error("There is no valid chipset initialized.")]
    UninitializedChipset,
}

#[derive(Error, Debug, PartialEq, Clone, Copy)]
pub enum OpcodeError {
    #[error("An unsupported opcode was used {0:#06X?}.")]
    InvalidOpcode(Opcode),
    #[error("Pointer location invalid there can not be an opcode at {pointer}, if data len is {len}")]
    MemoryInvalid{
        pointer: usize, 
        len: usize
    }
}

#[derive(Error, Debug, PartialEq, Clone, Copy)]
pub enum StackError {
    #[error("Stack is full!")]
    Full,
    #[error("Stack is empty!")]
    Empty,
}