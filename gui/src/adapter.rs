use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{cell::RefCell, rc::Rc, sync::Arc};

use chip::devices::{DisplayCommands, Keyboard, KeyboardCommands};

pub(crate) struct DisplayState {
    state: Vec<Vec<bool>>,
    changes: Vec<Vec<bool>>,
}

/// Translates the internal commands into the external ones.
pub(crate) struct DisplayAdapter {
    display_state: Rc<RefCell<DisplayState>>,
    callback: yew::Callback<()>,
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
        let mut display_state = self.display_state.borrow_mut();

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
pub(crate) struct KeyboardAdapter {
    /// Stores the keyboard into to which the values are changed.
    keyboard: Arc<RwLock<Keyboard>>,
}

impl KeyboardAdapter {
    /// Generates a new keyboard interface.
    pub fn new() -> Self {
        Self {
            keyboard: Arc::new(RwLock::new(Keyboard::new())),
        }
    }

    fn get_keyboard_read(&self) -> RwLockReadGuard<'_, Keyboard> {
        self.keyboard.read()
    }

    fn get_keyboard_write(&self) -> RwLockWriteGuard<'_, Keyboard> {
        self.keyboard.write()
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
