mod gui;

use chip::{chip8::ChipSet, definitions::KEYBOARD_SIZE, resources::RomArchives};

fn main() {
    let mut ra = RomArchives::new();
    let mut files = ra.file_names();
    files.sort();

    let keyboard: Vec<bool> = (0..KEYBOARD_SIZE).map(|i| i % 2 == 1).collect();

    let rom_name = &files[0].to_string();
    let rom = ra.get_file_data(rom_name).unwrap();

    let mut chip = ChipSet::new(rom);
    chip.set_keyboard(&keyboard);

    println!("{}", chip);
}
