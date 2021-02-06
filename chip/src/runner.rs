use crate::{
    chip8::ChipSet,
    devices::{DisplayCommands, KeyboardCommands},
    opcode::Operation,
    resources::RomArchives,
    timer::TimedWorker,
};

pub fn run<'a, D, K, W>(mut display: D, keyboard: K, rom_name: &str) -> Box<dyn FnMut() + 'a>
where
    D: DisplayCommands + 'a,
    K: KeyboardCommands + 'a,
    W: TimedWorker + 'a,
{
    let rom = RomArchives::new()
        .get_file_data(rom_name)
        .expect("Unexpected error during extraction of rom.");
    let inner_run = {
        let mut chip: ChipSet<W> = ChipSet::new(rom);
        let mut last_op = Operation::None;

        let func = move || {
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
        func
    };
    Box::new(inner_run)
}
