//! The constant definitions needed to run the application.

pub mod styling {
    use std::borrow::Cow;

    use lazy_static;
    use urlencoding;

    /// Is the tag with which the styling can be applied to
    pub const TYPE: &str = "link";
    
    pub const OUTER_TEXT : &str= "⚙";

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

        #keyboard-layout {
            width: min-content;
        }

        #keyboard-layout div {
            display: none;
            width: min-content;
        }

        #keyboard-layout:hover div {
            display: inherit;
            width: min-content;
        }

        #keyboard-layout:hover {
            display: block;
            border: black solid 1px;
            border-radius: 25px;
            padding: 5px;
        }
    ";

    lazy_static::lazy_static! {
        static ref CSS_URI_COMPRESSED : String = {
            let mut res = String::with_capacity(CSS.len());
            let mut iter = CSS.split_whitespace().filter(|v| !v.is_empty());

            // SAFETY: As we know the string this unwrap here is perfectly safe.
            let word = iter.next().unwrap();
            res.push_str(word);

            for word in iter {
                res.push(' ');
                res.push_str(word);
            }

            // remove unneded bytes
            res.shrink_to_fit();
            res
        };

        static ref CSS_URI_ENCODED: Cow<'static, str> = urlencoding::encode(&*CSS_URI_COMPRESSED);
        static ref CSS_HREF_ATTRIBUTE : String = format!("data:text/css;charset=UTF-8,{}", CSS_URI_ENCODED.as_ref());

        // we can use the &* here to create a static str, as the CSS_HREF_ATTRIBUTE also is 'static
        pub static ref CSS_ATTRIBUTES : [(&'static str, &'static str); 3] = [
            ("rel", "stylesheet"),
            ("type", "text/css"),
            ("href", &*CSS_HREF_ATTRIBUTE)
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

    pub const TYPE: &str = "div";
    pub const ID: &str = "keyboard-layout";

    pub const TYPE_HEADER: &str = "h2";

    pub const TYPE_TABLE: &str = "table";
    pub const TYPE_ROW: &str = "tr";
    pub const TYPE_CELL: &str = "th";

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
