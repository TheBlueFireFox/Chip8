use chip::{
    chip8::ChipSet,
    definitions::KEYBOARD_SIZE,
    devices::{DisplayCommands, KeyboardCommands},
    resources::RomArchives,
};

fn main() {
    let mut rom = RomArchives::new();
    let mut files = rom.file_names();
    files.sort();
    #[derive(Debug)]
    struct DC {
        keyboard: Box<[bool]>,
    }

    impl DC {
        fn new() -> Self {
            DC {
                keyboard: Box::new([false; KEYBOARD_SIZE]),
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

    let rom_name = files[0].to_string();

    let c = ChipSet::new(
        rom_name.to_string(),
        rom.get_file_data(&rom_name).unwrap(),
        t,
        t2,
    );
    println!("{}", c);
}
