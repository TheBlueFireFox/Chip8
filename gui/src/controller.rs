use chip::{
    chip8,
    devices::{DisplayCommands, KeyboardCommands},
};

use crate::timer::Timer;
pub struct Controller<T: DisplayCommands, U: KeyboardCommands> {
    display: T,
    keyboard: U,
    chipset: Option<chip8::ChipSet<Timer>>,
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
