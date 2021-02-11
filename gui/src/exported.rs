
use alloc::string::String;
use wasm_bindgen::prelude::*;

use crate::wrappers::*;
use chip::{resources::RomArchives, devices::DisplayCommands};


#[wasm_bindgen]
pub fn setup() -> Result<(), JsValue> {
    let document = document(&window());
    let body = body(&document);

    // create elements
    let val = document.create_element("p")?;
    val.set_inner_html("Hello from Rust");
    body.append_child(&val)?;

    // get rom names
    let ra = RomArchives::new();
    let mut files = ra.file_names();
    files.sort();

    

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