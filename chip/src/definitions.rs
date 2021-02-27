/// The definitions

pub mod memory {
    /// The size of the chipset ram
    pub const SIZE: usize = 0x1000; // 4096

    /// opcode information
    pub mod opcodes {
        /// The step used for calculating the program counter increments
        pub const SIZE: usize = 2;
    }
}

/// The definitions for the cpu
pub mod cpu {
    /// The starting point for the program
    pub(in super::super) const PROGRAM_COUNTER: usize = 0x0200;
    /// The amound of hertz the emulation shall run at.
    pub const HERTZ: u64 = 500;
    /// The amount of times the cpu shall run per second
    pub const INTERVAL: u64 = 1000 / HERTZ;

    /// The definitions needed for the register
    pub(crate) mod register {
        /// The size of the chip set registers
        pub const SIZE: usize = 16;
        /// The last entry of the registers
        pub const LAST: usize = SIZE - 1;
    }

    /// The stack definitions
    pub(crate) mod stack {
        /// The count of nesting entries
        pub const SIZE: usize = 16;
    }
}

/// The timer definitions
pub mod timer {
    /// The amount of hertz the clocks run at in milliseconds
    pub const HERZ: u8 = 60;
    /// The amount of hertz the clocks run at in milliseconds
    pub const INTERVAL: u64 = 1000 / HERZ as u64;
}

pub mod sound {
    use std::time::Duration;

    pub const DURRATION: Duration = Duration::from_millis(250);
}

/// The display definitions
pub mod display {
    /// The amount of pixels height
    pub const HEIGHT: usize = 64;
    /// The amount of pixels width
    pub const WIDTH: usize = 32;
    /// The amount of pixels the display has
    pub const RESOLUTION: usize = HEIGHT * WIDTH;

    /// The fontset information
    pub mod fontset {
        /// Is the location of the beginning to the font in memory
        pub const LOCATION: usize = 0x50;
        /// The font set character to be rendered on the screen
        pub const FONTSET: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
    }
}

/// The definitions needed for correct keyboard definitions.
pub mod keyboard {
    /// all the different keyboard entries
    pub const SIZE: usize = 16;
    /// The keyboard layout requested by the chipset
    pub const LAYOUT: [[usize; 4]; 4] = [
        [0x1, 0x2, 0x3, 0xC],
        [0x4, 0x5, 0x6, 0xD],
        [0x7, 0x8, 0x9, 0xE],
        [0xA, 0x0, 0xB, 0xF],
    ];
}
