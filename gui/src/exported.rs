use wasm_bindgen::prelude::*;
use web_sys::Element;

use crate::{definitions, helpers::BrowserWindow, wrappers::*};
use chip::{definitions::{DISPLAY_HEIGHT, DISPLAY_WIDTH}, devices::DisplayCommands, resources::RomArchives};

fn create_board(window: &BrowserWindow) -> Result<Element, JsValue> {
    let table = window
        .document()
        .create_element(definitions::field::TYPE)?;

    for _ in 0..DISPLAY_HEIGHT {
        
        for _ in 0..DISPLAY_WIDTH {
            
        }
    }



    Ok(table)
}

fn crate_dropdown(window: &BrowserWindow, files: &[&str]) -> Result<Element, JsValue> {
    let dropdown = window.document().create_element(definitions::selector::TYPE)?;
    dropdown.set_id(definitions::selector::ID);
    for file in files.into_iter() {
        let option = window.document().create_element("option")?;
        option.set_attribute("value", *file)?;
        option.set_text_content(Some(*file));
    }
    Ok(dropdown)
}

#[wasm_bindgen]
pub fn setup() -> Result<(), JsValue> {
    let browser_window = BrowserWindow::new();
    // create elements
    let val = browser_window.document().create_element("p")?;
    val.set_inner_html("Hello from Rust");
    browser_window.body().append_child(&val)?;

    // get rom names
    let ra = RomArchives::new();
    let mut files = ra.file_names();
    files.sort();

    let select = crate_dropdown(&browser_window, &files)?;
    browser_window.body().append_child(&select)?;

    let board = create_board(&browser_window)?;
    browser_window.body().append_child(&board)?;

    Ok(())
}

#[wasm_bindgen]
pub fn main(rom_name: String) -> Result<(), JsValue> {
    let mut ra = RomArchives::new();

    let rom = ra
        .get_file_data(&rom_name)
        .expect("Some unknown rom name was used to get the file data.");

    let run_wrapper = Data::new(rom);
    let chip = &mut *run_wrapper.chipset.borrow_mut();

    run_wrapper.display.borrow().display(&chip.get_display());

    // let val = document.create_element("pre")?;
    // val.set_inner_html(&format!("{}", chip));
    // body.append_child(&val)?;

    Ok(())
}
