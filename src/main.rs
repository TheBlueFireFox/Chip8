mod gui;

use chip::{
    chip8::ChipSet,
    definitions::KEYBOARD_SIZE,
    devices::{DisplayCommands, KeyboardCommands},
    resources::RomArchives,
};

fn main() {
    let mut ra = RomArchives::new();
    let mut files = ra.file_names();
    files.sort();
    #[derive(Debug)]
    struct DC {
        keyboard: Box<[bool]>,
    }

    impl DC {
        fn new() -> Self {
            DC {
                keyboard: vec![false; KEYBOARD_SIZE].into_boxed_slice(),
            }
        }
    }

    impl DisplayCommands for DC {
        fn clear_display(&mut self) {}
        fn display(&self, _: &[u8]) {}
    }

    impl KeyboardCommands for DC {
        fn get_keyboard(&self) -> Box<[bool]> {
            self.keyboard.clone()
        }
    }
    let t = DC::new();
    let mut t2 = DC::new();

    for entry in t2.keyboard.iter_mut().skip(1).step_by(2) {
        *entry = true;
    }

    let rom_name = &files[0].to_string();
    let rom = ra.get_file_data(rom_name).unwrap();

    let c = ChipSet::new(rom, t, t2);
    println!("{}", c);
}
