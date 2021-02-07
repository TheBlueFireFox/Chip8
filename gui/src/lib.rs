mod controller;
mod timer;
mod wrappers;

use wasm_bindgen::prelude::*;

use chip::{devices::DisplayCommands, resources::RomArchives};
pub use wrappers::*;
use wrappers::{body, document, window};

#[wasm_bindgen]
pub fn roms() -> js_sys::Array {
    let ra = RomArchives::new();
    let mut files = ra.file_names();
    files.sort();

    let arr = js_sys::Array::new_with_length(files.len() as u32);
    for file in files {
        arr.push(&JsValue::from_str(file));
    }
    arr
}

pub fn setup() -> Result<(), JsValue> {
    let document = document(&window());
    let body = body(&document);

    // create elements
    let val = document.create_element("p")?;
    val.set_inner_html("Hello from Rust");
    body.append_child(&val)?;

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
