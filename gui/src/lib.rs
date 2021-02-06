mod controller;
mod timer;

use std::fmt::Display;

use chip::{chip8::ChipSet, definitions::{DISPLAY_HEIGHT, DISPLAY_WIDTH}, resources::{Rom, RomArchives}};
use timer::Worker;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, Window};

#[wasm_bindgen]
pub struct ChipSetWrapper {
    chipset : ChipSet<Worker>
}

impl ChipSetWrapper {
    fn new(rom: Rom) -> Self {
        Self {
            chipset: ChipSet::new(rom)
        }
    }
}

impl Display for ChipSetWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.chipset)
    }
}

#[wasm_bindgen]
pub fn main() -> Result<(), JsValue> {
    let mut ra = RomArchives::new();
    let mut files = ra.file_names();
    files.sort();

    let rom_name = &files[0].to_string();
    let rom = ra.get_file_data(rom_name).unwrap();

    let mut chip = ChipSetWrapper::new(rom);

    for i in 0..chip.chipset.get_keyboard().len() {
        chip.chipset.set_key(i, i % 2 == 1);
    }

    let window: Window = web_sys::window().expect("no global `window` exists.");
    let document: Document = window.document().expect("no document available");
    let body: HtmlElement = document.body().expect("document should have a valid body");

    // create elements
    let val = document.create_element("p")?;
    val.set_inner_html("Hello from Rust");
    body.append_child(&val)?;
    let board = init_board(&document)?;
    body.append_child(&board)?;

    let val = document.create_element("pre")?;
    val.set_inner_html(&format!("{}", chip));
    body.append_child(&val)?;

    Ok(())
}

fn init_board(document: &Document) -> Result<Element, JsValue> {
    let table = document.create_element("table")?;
    for i in 0..DISPLAY_HEIGHT {
        let tr = document.create_element("tr")?;
        for j in 0..DISPLAY_WIDTH {
            let td = document.create_element("td")?;

            if (i + j) % 2 == 0 {
                td.set_class_name("alive");
            }

            tr.append_child(&td)?;
        }
        table.append_child(&tr)?;
    }

    Ok(table)
}
