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
/// The amount of hertz the clocks run at in milliseconds
pub(super) const TIMER_HERZ: u8 = 60;
/// The amount of hertz the clocks run at in milliseconds
pub(super) const TIMER_INTERVAL: u64 = 1000 / TIMER_HERZ as u64;
/// The amount of pixels the display has
pub(super) const DISPLAY_RESOLUTION: usize = 64 * 23;
/// all the different keyboards
pub(super) const KEYBOARD_SIZE: usize = 16;
/// Is the location of the beginning to the font in memory
pub(super) const FONTSET_LOCATION : usize = 0;