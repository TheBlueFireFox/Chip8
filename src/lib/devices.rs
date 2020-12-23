#[cfg_attr(test, mockall::automock)]
/// The traits responsible for the display based code
pub trait DisplayCommands {
    /// Will clear the display
    fn clear_display(&mut self);
    /// Will display all from the pixels
    fn display(&self, pixels: &[u8]);
}

#[cfg_attr(test, mockall::automock)]
/// The trait responsible for writing the keyboard data
pub trait KeyboardCommands {
    fn get_keyboard(&self) -> Box<[bool]>;
}
