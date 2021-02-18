use definitions::field;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

use crate::{definitions, helpers::BrowserWindow, timer::Worker};
use chip::{
    chip8::ChipSet,
    devices::{DisplayCommands, Keyboard, KeyboardCommands},
    opcode::Operation,
    resources::Rom,
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

#[wasm_bindgen]
/// This struct is the one that will be passed back and forth between
/// JS and WASM, as WASM API only allow for `&T` or `T` and not `&mut T`  
/// see [here](https://rustwasm.github.io/docs/wasm-bindgen/reference/types/jsvalue.html?highlight=JSV#jsvalue)
/// a compromise had to be chosen, so here is `Rc<RefCell<>>` used.
/// In addition to not have multiple borrows at the same time instead of
/// a single wrapper multiple are used.
pub struct Data {
    pub(crate) chipset: Rc<RefCell<ChipSet<Worker>>>,
    pub(crate) display: Rc<RefCell<DisplayAdapter>>,
    pub(crate) keyboard: Rc<RefCell<KeyboardAdapter>>,
    pub(crate) operation: Rc<RefCell<Operation>>,
}

impl Data {
    pub(crate) fn new(rom: Rom) -> Self {
        Self {
            chipset: Rc::new(RefCell::new(ChipSet::new(rom))),
            display: Rc::new(RefCell::new(DisplayAdapter::new())),
            keyboard: Rc::new(RefCell::new(KeyboardAdapter::new())),
            operation: Rc::new(RefCell::new(Operation::None)),
        }
    }
}

/// This is a wrapper function designed to split the `RunWrapper`
/// into it's internal parts to be used by the chip run function.
/// It also translates from the external
pub(crate) fn run(run_wrapper: &mut Data) {
    let display = &(*run_wrapper.display.borrow());
    let keyboard = &(*run_wrapper.keyboard.borrow());
    let last_op = &mut (*run_wrapper.operation.borrow_mut());
    let chip = &mut (*run_wrapper.chipset.borrow_mut());

    chip::run(chip, last_op, display, keyboard)
        .expect("Something went wrong while stepping to the next step.");
}
