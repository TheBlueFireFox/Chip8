use chip8_lib::{
    chip8::{ChipSet, DisplayCommands, KeybordCommands},
    resources::RomArchives,
};

fn main() {
    let mut rom = RomArchives::new();
    let files = rom.file_names();

    #[derive(Debug)]
    struct DC {
        keyboard: Vec<bool>,
    }

    impl DC {
        fn new() -> Self {
            DC {
                keyboard: vec![false; 4],
            }
        }
    }

    impl DisplayCommands for DC {
        fn clear_display(&self) {}
        fn display(&self, _: &[u8]) {}
    }

    impl KeybordCommands for DC {
        fn get_keybord(&self) -> &[bool] {
            &self.keyboard
        }
    }
    let t = DC::new();
    let t2 = DC::new();

    let c = ChipSet::new(rom.get_file_data(&files[0]).unwrap(), t, t2);
    println!("{}", c);
}
