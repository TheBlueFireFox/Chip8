use crate::definitions::KEYBOARD_SIZE;

#[cfg_attr(test, mockall::automock)]
/// The traits responsible for the display based code
pub trait DisplayCommands {
    /// Will clear the display
    fn clear_display(&mut self);
    /// Will display all from the pixels
    fn display(&self, pixels: &[u8]);
}


/// Will represent the last set key with the previous 
/// value.
#[derive(Debug, Clone, Copy)]
pub(super) struct Key {
    index: usize,
    last: bool,
    current: bool
}

impl Key {
    fn new(index: usize, last: bool, current: bool) -> Self {
        Key {
            index,
            last,
            current
        }
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn get_last(&self) -> bool {
        self.last
    }

    pub fn get_current(&self) -> bool {
        self.current
    }
}
/// Will store the last change to the given keybord
/// and represent the internal keyboard as well
///
/// Input is done with a hex keyboard that has 16 keys ranging `0-F`. The `8`, `4`, `6`, and
/// `2` keys are typically used for directional input. Three opcodes are used to detect input.
/// One skips an instruction if a specific key is pressed, while another does the same if a
/// specific key is not pressed. The third waits for a key press, and then stores it in one of
/// the data registers.
#[derive(Default, Debug)]
pub(super) struct Keyboard {
    /// Input is done with a hex keyboard that has 16 keys ranging `0-F`. The `8`, `4`, `6`, and
    /// `2` keys are typically used for directional input. Three opcodes are used to detect input.
    /// One skips an instruction if a specific key is pressed, while another does the same if a
    /// specific key is not pressed. The third waits for a key press, and then stores it in one of
    /// the data registers.
    keyboard: Box<[bool; KEYBOARD_SIZE]>,
    last: Option<Key>
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard::default()
    }

    pub fn toggle_key(&mut self, key: usize) {
        self.set_key(key, !self.keyboard[key])
    }

    pub fn set_key(&mut self, key: usize, to: bool) {
        debug_assert!(key < KEYBOARD_SIZE);
        // setup last
        self.last = Some(Key::new(key, self.keyboard[key], to));
        
        // write back solution
        self.keyboard[key] = to;
    }

    pub fn set_mult(&mut self, keys: &[bool]) {
        assert!(keys.len() == self.keyboard.len());
        self.keyboard.copy_from_slice(keys);
        self.last = None;
    }

    pub fn get_keys(&self) -> &[bool] {
        &*self.keyboard
    }

    pub fn get_last(&self) -> Option<Key> {
        self.last
    }
}

#[cfg_attr(test, mockall::automock)]
/// The trait responsible for writing the keyboard data
pub trait KeyboardCommands {
    fn get_keyboard(&self) -> Box<[bool]>;
}
