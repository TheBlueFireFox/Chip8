//! The full implementation of the chip8 enumalator, from the opcodes to an option to pretty
//! print them. 
mod chipset;
mod opcodes;
mod print;

/// reexport chipset structs and data for simpler usage
pub use chipset::*;

/// split up tests into an other file for simpler implementation
#[cfg(test)]
mod tests;
