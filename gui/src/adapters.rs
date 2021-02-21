use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::prelude::*;
use crate::{
    definitions,
    observer::{EventSystem, Observer},
    utils::BrowserWindow,
};
use chip::devices::{DisplayCommands, Keyboard, KeyboardCommands};

pub(crate) struct DisplayAdapter;

impl DisplayAdapter {
    pub fn new() -> Self {
        DisplayAdapter {}
    }

    fn draw_board<'a>(pixels: &'a [&'a [bool]]) -> Result<(), JsValue> {
        let html = BrowserWindow::new();
        let document = html.document();

        let table = document.create_element(definitions::field::TYPE)?;
        table.set_id(definitions::field::ID);
        for row in pixels.iter() {
            let tr = document.create_element(definitions::field::TYPE_ROW)?;
            for value in row.iter() {
                let td = document.create_element(definitions::field::TYPE_COLUMN)?;

                if *value {
                    td.set_class_name(definitions::field::ACTIVE);
                }

                tr.append_child(&td)?;
            }
            table.append_child(&tr)?;
        }

        // check if already exists, if exists replace element
        if let Some(element) = document.get_element_by_id(definitions::field::ID) {
            let _ = document.replace_child(&table, &element)?;
        } else {
            html.body().append_child(&table)?;
        }

        Ok(())
    }
}

impl DisplayCommands for DisplayAdapter {
    fn display<'a>(&'a self, pixels: &'a [&'a [bool]]) {
        Self::draw_board(pixels).expect("something went wrong while working on the board");
    }
}

pub(crate) struct KeyboardAdapter {
    keyboard: Keyboard,
    event_system: EventSystem<usize>,
}

impl KeyboardAdapter {
    pub fn new() -> Self {
        Self {
            keyboard: Keyboard::new(),
            event_system: EventSystem::new(),
        }
    }

    pub fn register_callback<T>(&mut self, data: Rc<RefCell<T>>)
    where
        T: Observer<usize> + 'static,
    {
        self.event_system.register_observer(data);
    }

    /// Get a reference to the keyboard adapter's keyboard.
    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }
}

impl KeyboardCommands for KeyboardAdapter {
    fn was_pressed(&self) -> bool {
        self.keyboard.get_last().is_some()
    }

    fn get_keyboard(&self) -> &[bool] {
        todo!()
    }
}
