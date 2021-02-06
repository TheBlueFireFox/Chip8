use chip::{chip8::ChipSet, devices::{DisplayCommands, KeyboardCommands}};

use crate::timer::Worker;

pub struct Controller<T: DisplayCommands, U: KeyboardCommands> {
    display: T,
    keyboard: U,
    chipset: Option<ChipSet<Worker>>
}

impl<T: DisplayCommands, U: KeyboardCommands> Controller<T, U> {
    pub fn new(dis: T, key: U) -> Self {
        Controller {
            display: dis,
            keyboard: key,
            chipset: None,
        }
    }
}
