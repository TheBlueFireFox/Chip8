use crate::definitions::keyboard;

/// The traits responsible for the display based code
pub trait DisplayCommands {
    /// Will display all from the pixels
    fn display<'a>(&'a self, pixels: &'a [&'a [bool]]);
}

/// The trait responsible for writing the keyboard data
pub trait KeyboardCommands {
    fn was_pressed(&self) -> bool;
    fn get_keyboard(&self) -> &[bool];
}

/// Will represent the last set key with the previous
/// value.
#[derive(Debug, Clone, Copy)]
pub struct Key {
    index: usize,
    last: bool,
    current: bool,
}

impl Key {
    /// Will instantiate a new key.
    pub fn new(index: usize, last: bool, current: bool) -> Self {
        Self {
            index,
            last,
            current,
        }
    }

    /// Will get the given index.
    pub fn get_index(&self) -> usize {
        self.index
    }

    /// Will get the last value of the given key.
    pub fn get_last(&self) -> bool {
        self.last
    }

    /// Will get current value set
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
pub struct Keyboard {
    /// Input is done with a hex keyboard that has 16 keys ranging `0-F`. The `8`, `4`, `6`, and
    /// `2` keys are typically used for directional input. Three opcodes are used to detect input.
    /// One skips an instruction if a specific key is pressed, while another does the same if a
    /// specific key is not pressed. The third waits for a key press, and then stores it in one of
    /// the data registers.
    keys: [bool; keyboard::SIZE],
    last: Option<Key>,
}

impl Keyboard {
    /// Will initiate a new keyboard
    pub fn new() -> Self {
        Keyboard::default()
    }

    /// Will reset the keyboard to it's false state.
    fn reset(&mut self) {
        self.keys.fill(false);
    }

    /// Will toggle a given key on or off.
    pub fn toggle_key(&mut self, key: usize) {
        self.set_key(key, !self.keys[key])
    }

    /// Will set the given key to a state
    pub fn set_key(&mut self, key: usize, to: bool) {
        self.reset();

        // setup last
        self.last = Some(Key::new(key, self.keys[key], to));

        // write back solution
        self.keys[key] = to;
    }

    /// Will set multiple keys
    pub fn set_mult(&mut self, keys: &[bool]) {
        self.keys.copy_from_slice(keys);
        self.last = None;
    }

    /// Will get all the keys
    pub fn get_keys(&self) -> &[bool] {
        &self.keys
    }

    /// Will get the last changes key 
    pub fn get_last(&self) -> Option<Key> {
        self.last
    }
}
