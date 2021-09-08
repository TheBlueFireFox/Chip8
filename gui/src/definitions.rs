//! The constant definitions needed to run the application.

pub mod styling {
    use std::borrow::Cow;

    use lazy_static;
    use urlencoding;

    /// Is the tag with which the styling can be applied to
    pub const TYPE: &str = "link";
    /// Is the styling that shall be applied to the website
    pub const CSS: &str = "
        table {
          margin: auto;
          border-collapse: collapse;
        }
        
        .alive {
          background: black;
        }
        
        td, th {
          border: black solid 1px;
          padding: 0px;
          height: 19px;
          width: 19px;
        }
        
        pre {
          margin: auto;
          width: 50%;
          padding: 50px;
        }
    ";

    lazy_static::lazy_static! {
        static ref CSS_URI_ENCODED: Cow<'static, str> = urlencoding::encode(CSS);
        pub static ref CSS_ATTRIBUTES : [(&'static str, String); 3] = [
            ("rel", "stylesheet".into()),
            ("type", "text/css".into()),
            ("href", format!("data:text/css;charset=UTF-8,{}", CSS_URI_ENCODED.to_string()))
        ];
    }
}

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

/// The pre in which the notifications are written into
pub mod info {
    /// Represents the id used for logging chip information to.
    pub const ID: &str = "info";
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

    pub const BROWSER_LAYOUT: [[&str; 4]; 4] = [
        ["Digit1", "Digit2", "Digit3", "Digit4"],
        ["KeyQ", "KeyW", "KeyE", "KeyR"],
        ["KeyA", "KeyS", "KeyD", "KeyF"],
        ["KeyY", "KeyX", "KeyC", "KeyV"],
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
