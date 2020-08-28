/// The size of the chipset ram
pub const MEMORY_SIZE: usize = 0x1000; // 4096
/// The starting point for the program
pub const PROGRAM_COUNTER: usize = 0x200;
/// The step used for calculating the program counter increments
pub const PROGRAM_COUNTER_STEP: usize = 2;
/// The size of the chipset registers
pub const REGISTER_SIZE: usize = 16;
/// The last entry of the registers
pub const REGISTER_LAST: usize = REGISTER_SIZE - 1;
/// The count of nesting entries
pub const STACK_NESTING: usize = 16;
/// The amount of herz the clocks run at in millisec
pub const TIMER_HERZ: u8 = 60;
/// The amount of herz the clocks run at in millisec
pub const TIMER_INTERVAL: u32 = 1000 / TIMER_HERZ as u32;
/// The amount of pixels the display has
pub const DISPLAY_RESOLUTION: usize = 64 * 23;
/// all the different keybords
pub const KEYBOARD_SIZE: usize = 16;