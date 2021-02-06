use std::time::Duration;

use chip8::ChipSet;

use crate::{
    chip8,
    definitions::CPU_INTERVAL,
    devices::{DisplayCommands, KeyboardCommands},
    resources::RomArchives,
    timer::TimedWorker,
};

pub fn run<D, K, W>(display: D, keyboard: K, rom_name: &str) -> W
where
    D: DisplayCommands,
    K: KeyboardCommands,
    W: TimedWorker + 'static,
{
    let rom = RomArchives::new()
        .get_file_data(rom_name)
        .expect("Unexpected error during extraction of rom.");

    let mut chip: ChipSet<W> = chip8::ChipSet::new(rom);
    let inner_run = move || {
        // run chip
        let op = chip
            .next()
            .expect("An unexpected error occured during executrion.");

    };
    let mut worker = W::new();

    worker.start(inner_run, Duration::from_millis(CPU_INTERVAL));

    worker
}
