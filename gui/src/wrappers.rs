use std::fmt::Display;

use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, Window};

use crate::timer::Worker;
use chip::{
    chip8::ChipSet,
    devices::{DisplayCommands, Keyboard, KeyboardCommands},
    opcode::Operation,
    resources::Rom,
};

pub struct ChipSetWrapper {
    pub(crate) chipset: ChipSet<Worker>,
}

impl ChipSetWrapper {
    pub(crate) fn new(rom: Rom) -> Self {
        Self {
            chipset: ChipSet::new(rom),
        }
    }
}

impl Display for ChipSetWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.chipset)
    }
}

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
pub struct RunWrapper {
    chipset: ChipSet<Worker>,
    display: DisplayWrapper,
    keyboard: KeyboardWrapper,
    operation: OperationWrapper,
}

impl RunWrapper {
    fn new(rom: Rom) -> Self {
        Self {
            chipset: ChipSet::new(rom),
            display: DisplayWrapper::new(),
            keyboard: KeyboardWrapper::new(),
            operation: OperationWrapper::None,
        }
    }
}

pub(crate) fn run_wrapper(run_wrapper: &mut RunWrapper) {
    let display = &run_wrapper.display;
    let keyboard = &run_wrapper.keyboard;
    let last_op = &mut run_wrapper.operation;
    let chip = &mut run_wrapper.chipset;

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
