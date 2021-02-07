use std::{cell::RefCell, fmt::Display, rc::Rc};

use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, Window};

use crate::timer::Worker;
use chip::{
    chip8::ChipSet,
    devices::{DisplayCommands, Keyboard, KeyboardCommands},
    opcode::Operation,
    resources::Rom,
};

#[derive(Clone, Copy)]
pub enum OperationWrapper {
    None,
    Wait,
    Draw,
}

impl From<Operation> for OperationWrapper {
    fn from(op: Operation) -> Self {
        match op {
            Operation::None => Self::None,
            Operation::Wait => Self::Wait,
            Operation::Draw => Self::Draw,
        }
    }
}

impl Into<Operation> for OperationWrapper {
    fn into(self) -> Operation {
        match self {
            OperationWrapper::None => Operation::None,
            OperationWrapper::Wait => Operation::Wait,
            OperationWrapper::Draw => Operation::Draw,
        }
    }
}

pub struct DisplayWrapper;

impl DisplayWrapper {
    fn new() -> Self {
        DisplayWrapper {}
    }

    fn draw_board<'a>(pixels: &'a [&'a [bool]]) -> Result<Element, JsValue> {
        let window = window();
        let document = document(&window);
        let table = document.create_element("table")?;
        for row in pixels.iter() {
            let tr = document.create_element("tr")?;
            for value in row.iter() {
                let td = document.create_element("td")?;

                if *value {
                    td.set_class_name("alive");
                }

                tr.append_child(&td)?;
            }
            table.append_child(&tr)?;
        }

        Ok(table)
    }
}

impl DisplayCommands for DisplayWrapper {
    fn display<'a>(&'a self, pixels: &'a [&'a [bool]]) {
        Self::draw_board(pixels).expect("something went wrong while working on the board");
    }
}

#[derive(Default)]
pub struct KeyboardWrapper {
    keyboard: Keyboard,
}

impl KeyboardWrapper {
    fn new() -> Self {
        Self::default()
    }
}

impl KeyboardCommands for KeyboardWrapper {
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
pub struct RunWrapper {
   pub(crate) chipset: Rc<RefCell<ChipSet<Worker>>>,
   pub(crate) display: Rc<RefCell<DisplayWrapper>>,
   pub(crate) keyboard: Rc<RefCell<KeyboardWrapper>>,
   pub(crate) operation: Rc<RefCell<OperationWrapper>>,
}

impl RunWrapper {
    pub(crate) fn new(rom: Rom) -> Self {
        Self {
            chipset: Rc::new(RefCell::new(ChipSet::new(rom))),
            display:  Rc::new(RefCell::new(DisplayWrapper::new())),
            keyboard:  Rc::new(RefCell::new(KeyboardWrapper::new())),
            operation:  Rc::new(RefCell::new(OperationWrapper::None)),
        }
    }
}

/// This is a wrapper function designed to split the `RunWrapper`
/// into it's internal parts to be used by the chip run function.
/// It also translates from the external 
pub(crate) fn run_wrapper(run_wrapper: &mut RunWrapper) {
    let display = &(*run_wrapper.display.borrow());
    let keyboard = &(*run_wrapper.keyboard.borrow());
    let last_op = &mut (*run_wrapper.operation.borrow_mut());
    let chip = &mut (*run_wrapper.chipset.borrow_mut());

    let mut last_inner_op: Operation = OperationWrapper::into(*last_op);
    let last_inner_op = &mut last_inner_op;

    chip::run(chip, last_inner_op, display, keyboard)
        .expect("Something went wrong while stepping to the next step.");

    *last_op = OperationWrapper::from(*last_inner_op);
}

pub(crate) fn window() -> Window {
    web_sys::window().expect("no global `window` exists.")
}

pub(crate) fn document(window: &Window) -> Document {
    window.document().expect("no document available")
}

pub(crate) fn body(document: &Document) -> HtmlElement {
    document.body().expect("document should have a valid body")
}
