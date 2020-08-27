
#[cfg(test)]
use mockall::automock;
#[cfg_attr(test, automock)]
/// The traits responsible for the display based code
pub trait DisplayCommands {
    /// Will clear the display
    fn clear_display(&mut self);
    /// Will display all from the pixels
    fn display(&self, pixels: &[u8]);
}

#[cfg_attr(test, automock)]
/// The trait responsible for writing the keybord data
pub trait KeybordCommands {
    fn get_keybord(&self) -> Vec<bool>;
}
