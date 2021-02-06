use crate::{
    chip8::ChipSet,
    devices::{DisplayCommands, KeyboardCommands},
    opcode::Operation,
    timer::TimedWorker,
};

pub fn run<'a, D, K, W>(chip: &mut ChipSet<W>, last_op: &mut Operation, display: D, keyboard: K)
where
    D: DisplayCommands + 'a,
    K: KeyboardCommands + 'a,
    W: TimedWorker + 'a,
{
    let work = if matches!(last_op, Operation::Wait) {
        /* wait for user input */
        keyboard.was_pressed()
    } else {
        true
    };

    if work {
        // run chip
        *last_op = chip
            .next()
            .expect("An unexpected error occured during executrion.");

        if matches!(last_op, Operation::Draw) {
            /* draw the screen */
            display.display(&chip.get_display()[..]);
        }
    }
}
