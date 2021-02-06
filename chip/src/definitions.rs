/// The size of the chipset ram
pub(super) const MEMORY_SIZE: usize = 0x1000; // 4096
/// The starting point for the program
pub(super) const PROGRAM_COUNTER: usize = 0x0200;
/// The step used for calculating the program counter increments
pub(super) const OPCODE_BYTE_SIZE: usize = 2;
/// The size of the chip set registers
pub(super) const REGISTER_SIZE: usize = 16;
/// The last entry of the registers
pub(super) const REGISTER_LAST: usize = REGISTER_SIZE - 1;
/// The count of nesting entries
pub(super) const STACK_NESTING: usize = 16;
/// The amound of hertz the emulation shall run at.
pub const CPU_HERTZ: u64 = 500;
/// The amount of times the cpu shall run per second
pub const CPU_INTERVAL: u64 = 1000 / CPU_HERTZ;
/// The amount of hertz the clocks run at in milliseconds
pub const TIMER_HERZ: u8 = 60;
/// The amount of hertz the clocks run at in milliseconds
pub const TIMER_INTERVAL: u64 = 1000 / TIMER_HERZ as u64;
/// The amount of pixels height
pub const DISPLAY_HEIGHT: usize = 64;
/// The amount of pixels width
pub const DISPLAY_WIDTH: usize = 23;
/// The amount of pixels the display has
pub const DISPLAY_RESOLUTION: usize = DISPLAY_HEIGHT * DISPLAY_WIDTH;
/// all the different keyboards
pub const KEYBOARD_SIZE: usize = 16;
/// Is the location of the beginning to the font in memory
pub(super) const FONTSET_LOCATION: usize = 0;
