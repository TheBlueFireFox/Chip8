use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};
use wasm_bindgen::prelude::*;

use crate::{definitions, helpers::BrowserWindow, timer::Worker};
use chip::{
    devices::{DisplayCommands, Keyboard, KeyboardCommands},
    Controller,
};

pub struct DisplayAdapter;

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

#[derive(Default)]
pub struct KeyboardAdapter {
    keyboard: Keyboard,
}

impl KeyboardAdapter {
    fn new() -> Self {
        Self::default()
    }
}

impl KeyboardCommands for KeyboardAdapter {
    fn was_pressed(&self) -> bool {
        todo!()
    }

    fn get_keyboard(&self) -> &[bool] {
        todo!()
    }
}

/// This struct is the one that will be passed back and forth between
/// JS and WASM, as WASM API only allow for `&T` or `T` and not `&mut T`  
/// see [here](https://rustwasm.github.io/docs/wasm-bindgen/reference/types/jsvalue.html?highlight=JSV#jsvalue)
/// a compromise had to be chosen, so here is `Rc<RefCell<>>` used.
#[wasm_bindgen]
pub struct Data {
    controller: Rc<RefCell<Controller<DisplayAdapter, KeyboardAdapter, Worker>>>,
}

#[wasm_bindgen]
impl Data {
    pub(crate) fn new() -> Self {
        let controller = Controller::new(DisplayAdapter::new(), KeyboardAdapter::new());

        Self {
            controller: Rc::new(RefCell::new(controller)),
        }
    }

    /// Get a mutable reference to the data's controller.
    pub(crate) fn controller_mut(
        &self,
    ) -> RefMut<'_, Controller<DisplayAdapter, KeyboardAdapter, Worker>> {
        self.controller.borrow_mut()
    }

    /// Get a reference to the data's controller.
    pub(crate) fn controller(
        &self,
    ) -> Ref<'_, Controller<DisplayAdapter, KeyboardAdapter, Worker>> {
        self.controller.borrow()
    }

    /// Will convert the Data type into a mutable controller, so that
    /// it can be used by the chip
    #[wasm_bindgen]
    pub fn run(&mut self) -> Result<(), JsValue> {
        chip::run(&mut *self.controller_mut()).map_err(|err| {
            let line = format!(
                "Something went wrong while stepping to the next step.\n{}",
                err
            );
            JsValue::from(line)
        })
    }
}
