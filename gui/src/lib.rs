#[cfg(feature="wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod adapters;
mod definitions;
mod exported;
mod observer;
mod timer;
mod utils;

pub use exported::*;
