use std::time::Duration;

use chip8::ChipSet;

use crate::{
    chip8,
    definitions::CPU_INTERVAL,
    devices::{DisplayCommands, KeyboardCommands},
    opcode::Operation,
    resources::RomArchives,
    timer::TimedWorker,
};

pub fn run<D, K, W>(mut display: D, mut keyboard: K, rom_name: &str) -> W
where
    D: DisplayCommands + 'static,
    K: KeyboardCommands + 'static,
    W: TimedWorker + 'static,
{
    let rom = RomArchives::new()
        .get_file_data(rom_name)
        .expect("Unexpected error during extraction of rom.");

    let mut chip: ChipSet<W> = chip8::ChipSet::new(rom);
    let mut last_op = Operation::None;

    let inner_run = move || {
        let work = if matches!(last_op, Operation::Wait) {
            /* wait for user input */
            keyboard.was_pressed()
        } else {
            true
        };

        if work {
            // run chip
            last_op = chip
                .next()
                .expect("An unexpected error occured during executrion.");

            if matches!(last_op, Operation::Draw) {
                /* draw the screen */
                display.display(&chip.get_display()[..]);
            }
        }
    };
    let mut worker = W::new();

    worker.start(inner_run, Duration::from_millis(CPU_INTERVAL));

    worker
}
