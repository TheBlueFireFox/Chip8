use parking_lot::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::Arc;

use chip::{
    devices::{DisplayCommands, Keyboard, KeyboardCommands},
    timer::TimerCallback,
};

#[derive(Debug, PartialEq, Default)]
pub(crate) struct DisplayState {
    state: Vec<Vec<bool>>,
    changes: Vec<Vec<bool>>,
}

impl DisplayState {
    fn new(state: Vec<Vec<bool>>) -> Self {
        let len_o = state.len();
        let len_i = state[0].len();
        Self {
            state,
            changes: vec![vec![false; len_i]; len_o],
        }
    }

    pub fn state(&self) -> &[Vec<bool>] {
        &self.state
    }
}

/// Translates the internal commands into the external ones.
#[derive(Debug, Clone)]
pub(crate) struct DisplayAdapter {
    display_state: Arc<Mutex<DisplayState>>,
    callback: yew::Callback<()>,
}

impl DisplayAdapter {
    pub fn new(
        state: Vec<Vec<bool>>,
        callback: yew::Callback<()>,
    ) -> (Self, Arc<Mutex<DisplayState>>) {
        let display_state = DisplayState::new(state);
        let display_state = Arc::new(Mutex::new(display_state));

        (
            Self {
                display_state: display_state.clone(),
                callback,
            },
            display_state,
        )
    }
}

impl DisplayCommands for DisplayAdapter {
    fn display<M, V>(&mut self, pixels: M)
    where
        M: AsRef<[V]>,
        V: AsRef<[bool]>,
    {
        log::debug!("Drawing the display");

        // TODO: update display cells and then callback to
        // update parent
        let mut display_state = self.display_state.lock();

        let DisplayState {
            state: elements,
            changes,
        } = &mut *display_state;

        let mut has_changes = false;

        for (back_row, front_row, changes_row) in itertools::izip!(
            pixels.as_ref().iter(),
            elements.iter_mut(),
            changes.iter_mut()
        ) {
            for (&back_cell, front_cell, changes_cell) in itertools::izip!(
                back_row.as_ref().iter(),
                front_row.iter_mut(),
                changes_row.iter_mut()
            ) {
                // if there is a difference then we know that
                // that given cell has updated
                let state = back_cell != *front_cell;

                // update the state if needed
                if state {
                    *front_cell = back_cell;
                    has_changes = true;
                }

                // make sure that we flag the needed cell
                *changes_cell = state;
            }
        }

        if has_changes {
            self.callback.emit(());
        }
    }
}

/// Abstracts away the awkward js keyboard interface
#[derive(Debug, Clone, Default)]
pub(crate) struct KeyboardAdapter {
    /// Stores the keyboard into to which the values are changed.
    keyboard: Arc<RwLock<Keyboard>>,
}

impl KeyboardAdapter {
    /// Generates a new keyboard interface.
    pub fn new() -> Self {
        Default::default()
    }

    fn get_keyboard_read(&self) -> RwLockReadGuard<'_, Keyboard> {
        self.keyboard.read()
    }

    fn get_keyboard_write(&self) -> RwLockWriteGuard<'_, Keyboard> {
        self.keyboard.write()
    }

    pub fn map_key(key: &str) -> Option<usize> {
        use std::collections::HashMap;
        /// maps the external keyboard layout to the internaly given.
        static LAYOUT_MAP: once_cell::sync::Lazy<HashMap<&str, usize>> =
            once_cell::sync::Lazy::new(|| {
                let mut map = HashMap::new();

                for (row_index, row) in crate::definitions::keyboard::BROWSER_LAYOUT
                    .iter()
                    .enumerate()
                {
                    for (cell_index, &cell) in row.iter().enumerate() {
                        // translate from the 2d matrix to the 1d
                        let key = row_index * row.len() + cell_index;
                        map.insert(cell, key);
                    }
                }

                map
            });

        LAYOUT_MAP.get(key).map(|a| *a)
    }
}

impl KeyboardCommands for KeyboardAdapter {
    fn was_pressed(&self) -> bool {
        self.get_keyboard_read().get_last().is_some()
    }

    fn get_keyboard(&mut self) -> Arc<RwLock<Keyboard>> {
        self.keyboard.clone()
    }

    fn set_key(&mut self, key: usize, to: bool) {
        self.get_keyboard_write().set_key(key, to);
    }
}

pub(crate) struct SoundCallback;

impl TimerCallback for SoundCallback {
    fn new() -> Self {
        Self {}
    }

    fn handle(&mut self) {
        // TODO: implement the sound callback
        todo!()
    }
}
