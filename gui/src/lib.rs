
/// Is the global allocator used for, when the chip is used
/// inside of wasm.
/// This is locked behind a feature gate for the ability to
/// change to the std alloc if needed.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod adapters;
mod definitions;
mod exported;
mod setup;
mod timer;
mod utils;

pub use exported::*;
