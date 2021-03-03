//! The constant definitions needed to run the application.

/// The selector with the overview of the supported roms.
pub mod selector {
    /// Represents the id used inside of the html, so that the selector element can be found by id.
    pub const ID: &str = "name-selector";
    /// The type of the selector.
    pub const TYPE: &str = "select";
}

/// The board in which the chip implementation runs.
pub mod field {
    /// The upper most id.
    pub const ID: &str = "board";
    /// The type of the board is a standard html table
    pub const TYPE: &str = "table";
    /// A html row type.
    pub const TYPE_ROW: &str = "tr";
    /// A html cell.
    pub const TYPE_COLUMN: &str = "td";
    /// The state of which the values exist on.
    /// Attention the implemtnation is in reverse, so a not `active` cell is per this definition
    /// `alive`.
    pub const ACTIVE: &str = "alive";
}

/// The keyboard constants.
pub mod keyboard {
    use std::collections::HashMap;

    /// represents the external layout and how it translates
    /// to the internal
    pub const LAYOUT: [[char; 4]; 4] = [
        ['1', '2', '3', '4'],
        ['Q', 'W', 'E', 'R'],
        ['A', 'S', 'D', 'F'],
        ['Y', 'X', 'C', 'V'],
    ];

    lazy_static::lazy_static! {
        /// maps the external keyboard layout to the internaly given.
        pub static ref LAYOUT_MAP : HashMap<char, usize> = {
            let mut map = HashMap::new();

            for (external_row, internal_row) in LAYOUT.iter().zip(chip::definitions::keyboard::LAYOUT.iter()) {
                for (external_value, internal_value) in external_row.iter().zip(internal_row) {
                    map.insert(*external_value, *internal_value);
                }
            }

            map
        };
    }
}
