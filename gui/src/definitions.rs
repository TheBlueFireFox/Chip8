/// The keyboard constants.
pub mod keyboard {

    pub const OUTER_TEXT: &str = "⚙";

    pub const ID: &str = "keyboard-layout";

    pub const HEADER_CHIP: &str = "Chip8 Keypad";
    pub const HEADER_EMULATOR: &str = "Emulator Keyboard Mapping";

    /// represents the external layout and how it translates
    /// to the internal
    pub const LAYOUT: [[char; 4]; 4] = [
        ['1', '2', '3', '4'],
        ['Q', 'W', 'E', 'R'],
        ['A', 'S', 'D', 'F'],
        ['Y', 'X', 'C', 'V'],
    ];

    pub const BROWSER_LAYOUT: [[&str; 4]; 4] = [
        ["Digit1", "Digit2", "Digit3", "Digit4"],
        ["KeyQ", "KeyW", "KeyE", "KeyR"],
        ["KeyA", "KeyS", "KeyD", "KeyF"],
        ["KeyY", "KeyX", "KeyC", "KeyV"],
    ];

    pub const CHIP_LAYOUT: [[char; 4]; 4] = [
        ['1', '2', '3', 'C'],
        ['4', '5', '6', 'D'],
        ['7', '8', '9', 'E'],
        ['A', '0', 'B', 'F'],
    ];
}

/// The board in which the chip implementation runs.
pub mod field {
    /// The upper most id.
    pub const ID: &str = "board";

    /// The state of which the values exist on.
    /// Attention the implemtnation is in reverse, so a not `active` cell is per this definition
    /// `alive`.
    pub const ACTIVE: &str = "alive";
}
