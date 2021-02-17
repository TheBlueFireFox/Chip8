use core::todo;


use alloc::string::String;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, Window};

use crate::{definitions, helpers::BrowerWindow, wrappers::*};
use chip::{devices::DisplayCommands, resources::RomArchives};


fn create_board(document: &BrowerWindow) {
    todo!()
}

fn crate_dropdown(window: &BrowerWindow, files: &[&str]) -> Result<(), JsValue> {
    let dropdown : Element = window.document().create_element("select")?;
    dropdown.set_id(definitions::selector::ID);

    Ok(())
}

#[wasm_bindgen]
pub fn setup() -> Result<(), JsValue> {
    let brower_window = BrowerWindow::new();
    // create elements
    let val = brower_window.document().create_element("p")?;
    val.set_inner_html("Hello from Rust");
    brower_window.body().append_child(&val)?;

    // get rom names
    let ra = RomArchives::new();
    let mut files = ra.file_names();
    files.sort();

    crate_dropdown(&brower_window, &files)?;

    Ok(())
}

#[wasm_bindgen]
pub fn main(rom_name: String) -> Result<(), JsValue> {
    let mut ra = RomArchives::new();

    let rom = ra
        .get_file_data(&rom_name)
        .expect("Some unknown rom name was used to get the file data.");

    let run_wrapper = RunWrapper::new(rom);
    let chip = &mut *run_wrapper.chipset.borrow_mut();

    run_wrapper.display.borrow().display(&chip.get_display());

    // let val = document.create_element("pre")?;
    // val.set_inner_html(&format!("{}", chip));
    // body.append_child(&val)?;

    Ok(())
}
