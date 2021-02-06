use std::fmt::Display;

use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, Window};

use crate::timer::Worker;
use chip::{chip8::ChipSet, devices::DisplayCommands, opcode::Operation, resources::Rom};

#[wasm_bindgen]
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

#[wasm_bindgen]
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

#[wasm_bindgen]
pub struct DisplayWrapper;

impl DisplayWrapper {
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

fn run_wrapper(
    chip_wrapper: &mut ChipSetWrapper,
    last_op: &mut OperationWrapper,
    display: &DisplayWrapper,
) {
    let mut last_inner_op: Operation = OperationWrapper::into(*last_op);
    // chip::run(&mut chip_wrapper.chipset, &mut last_inner_op, &display, );
    *last_op = OperationWrapper::from(last_inner_op);
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
