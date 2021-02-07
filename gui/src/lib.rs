mod controller;
mod timer;
mod wrappers;

use wasm_bindgen::prelude::*;

use chip::{devices::DisplayCommands, resources::RomArchives};
use wrappers::{body, document, window, DisplayWrapper};
pub use wrappers::*;


#[wasm_bindgen]
pub fn main() -> Result<(), JsValue> {
    let mut ra = RomArchives::new();
    let mut files = ra.file_names();
    files.sort();

    let rom_name = &files[0].to_string();
    let rom = ra.get_file_data(rom_name).unwrap();

    let run_wrapper = RunWrapper::new(rom);
    let chip = &mut *run_wrapper.chipset.borrow_mut();
    for i in 0..chip.get_keyboard().len() {
        chip.set_key(i, i % 2 == 1);
    }

    let document = document(&window());
    let body = body(&document);

    // create elements
    let val = document.create_element("p")?;
    val.set_inner_html("Hello from Rust");
    body.append_child(&val)?;
    DisplayWrapper {}.display(&chip.get_display());

    let val = document.create_element("pre")?;
    val.set_inner_html(&format!("{}", chip));
    body.append_child(&val)?;

    Ok(())
}
