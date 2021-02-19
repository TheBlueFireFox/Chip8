use wasm_bindgen::prelude::*;
use web_sys::Element;

use crate::{definitions, helpers::BrowserWindow, Data};
use chip::{
    definitions::{DISPLAY_HEIGHT, DISPLAY_WIDTH},
    resources::RomArchives,
};

fn create_board(window: &BrowserWindow) -> Result<Element, JsValue> {
    let table = window.document().create_element(definitions::field::TYPE)?;

    for i in 0..DISPLAY_HEIGHT {
        let tr = window
            .document()
            .create_element(definitions::field::TYPE_ROW)?;
        for j in 0..DISPLAY_WIDTH {
            let td = window
                .document()
                .create_element(definitions::field::TYPE_COLUMN)?;
            if (i + j) % 2 == 0 {
                td.set_class_name(definitions::field::ACTIVE);
            }

            tr.append_child(&td)?;
        }
        table.append_child(&tr)?;
    }

    Ok(table)
}

fn crate_dropdown(window: &BrowserWindow, files: &[&str]) -> Result<Element, JsValue> {
    let dropdown = window
        .document()
        .create_element(definitions::selector::TYPE)?;
    dropdown.set_id(definitions::selector::ID);
    for file in files.into_iter() {
        let option = window.document().create_element("option")?;
        option.set_attribute("value", *file)?;
        option.set_text_content(Some(*file));
        dropdown.append_child(&option)?;
    }
    Ok(dropdown)
}

#[wasm_bindgen]
pub fn setup() -> Result<Data, JsValue> {
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

    let data = Data::new();

    Ok(data)
}

