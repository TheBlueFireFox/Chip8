pub mod chip8;
pub mod definitions;
pub mod devices;
pub mod opcode;
pub mod resources;
pub mod timer;
mod error;

// reexporting for convinience
mod runner;
pub use runner::*;
pub use error::*;
