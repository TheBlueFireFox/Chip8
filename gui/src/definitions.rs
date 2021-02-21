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
    pub const LAYOUT: [[char; 4]; 4] = [
        ['1', '2', '3', '4'],
        ['Q', 'W', 'E', 'R'],
        ['A', 'S', 'D', 'F'],
        ['Y', 'X', 'C', 'V'],
    ];
}
