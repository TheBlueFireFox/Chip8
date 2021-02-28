pub mod selector {
    pub const ID: &str = "name-selector";
    pub const TYPE: &str = "select";
}

pub mod field {
    pub const ID: &str = "board";
    pub const TYPE: &str = "table";
    pub const TYPE_ROW: &str = "tr";
    pub const TYPE_COLUMN: &str = "td";
    pub const ACTIVE: &str = "alive";
}

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

    lazy_static::lazy_static!{
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
