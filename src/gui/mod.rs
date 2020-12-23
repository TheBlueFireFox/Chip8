mod observer;

use chip::{
    chip8,
    devices::{DisplayCommands, KeyboardCommands},
};
pub struct Controller<T: DisplayCommands, U: KeyboardCommands> {
    pub display: T,
    pub keyboard: U,
    pub chipset: Option<chip8::ChipSet>,
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
