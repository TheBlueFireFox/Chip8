mod gui;

// use chip::{chip8::ChipSet, resources::RomArchives};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // let mut ra = RomArchives::new();
    // let mut files = ra.file_names();
    // files.sort();

    // let chip = chip::ChipSet::new();

    // let rom_name = &files[0].to_string();
    // let rom = ra.get_file_data(rom_name).unwrap();

    // let mut chip = ChipSet::new(rom);

    // for i in 0..chip.get_keyboard().len() {
    //     chip.set_key(i, i % 2 == 1);
    // }

    // println!("{}", chip);

    Ok(())
}
