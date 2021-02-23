use crate::{
    chip8::ChipSet,
    devices::{DisplayCommands, KeyboardCommands},
    opcode::Operation,
    resources::Rom,
    timer::TimedWorker,
};

pub struct Controller<D, K, W>
where
    D: DisplayCommands,
    K: KeyboardCommands,
    W: TimedWorker,
{
    display: D,
    keyboard: K,
    chipset: Option<ChipSet<W>>,
    operation: Operation,
}

impl<D, K, W> Controller<D, K, W>
where
    D: DisplayCommands,
    K: KeyboardCommands,
    W: TimedWorker,
{
    pub fn new(dis: D, key: K) -> Self {
        Controller {
            display: dis,
            keyboard: key,
            chipset: None,
            operation: Operation::None,
        }
    }

    /// Get a reference to the controller's chipset.
    pub fn chipset(&self) -> &Option<ChipSet<W>> {
        &self.chipset
    }

    pub fn chipset_mut(&mut self) -> Option<&mut ChipSet<W>> {
        self.chipset.as_mut()
    }

    /// Set the controller's chipset.
    pub fn set_rom(&mut self, rom: Rom) {
        let chipset = ChipSet::new(rom);
        self.chipset = Some(chipset);
    }

    pub fn remove_rom(&mut self) {
        self.chipset = None;
    }

    /// Get a reference to the controller's keyboard.
    pub fn keyboard(&self) -> &K {
        &self.keyboard
    }

    /// Get a reference to the controller's display.
    pub fn display(&self) -> &D {
        &self.display
    }

    /// Get a reference to the controller's operation.
    pub fn operation(&self) -> Operation {
        self.operation
    }

    /// Set the controller's operation.
    pub fn set_operation(&mut self, operation: Operation) {
        self.operation = operation;
    }
}

pub fn run<D, K, W>(
    Controller {
        display,
        keyboard,
        chipset,
        operation,
    }: &mut Controller<D, K, W>,
) -> Result<(), String>
where
    D: DisplayCommands,
    K: KeyboardCommands,
    W: TimedWorker,
{
    // Checks if the last operation was a wait and if 
    // processing can continue.
    if *operation == Operation::Wait && !keyboard.was_pressed() {
        return Ok(());
    }

    // Extract the chip from the chipset option
    let chip = chipset
        .as_mut()
        .ok_or_else(|| "There is no valid chipset initialized.".to_string())?;

    // run chip
    *operation = chip.next()?;

    // Checks if we can redraw the screen after this or not.
    if *operation == Operation::Draw {
        /* draw the screen */
        display.display(chip.get_display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::timer::Worker;
    use mockall::predicate::*;

    #[mockall::automock]
    trait InternalDCommands {
        fn display(&self);
    }

    struct DisplayAdapter<M>
    where
        M: InternalDCommands,
    {
        da: M,
    }

    impl<MD> DisplayCommands for DisplayAdapter<MD>
    where
        MD: InternalDCommands,
    {
        fn display<M: AsRef<[V]>, V: AsRef<[bool]>>(&self, _pixels: M) {
            self.da.display()
        }
    }

    #[mockall::automock]
    trait InternalKCommands {
        fn was_pressed(&self) -> bool;
        fn get_keyboard(&self) -> &[bool];
    }

    struct KeyboardAdapter<M>
    where
        M: InternalKCommands,
    {
        ka: M,
    }

    impl<M: InternalKCommands> KeyboardCommands for KeyboardAdapter<M> {
        fn was_pressed(&self) -> bool {
            self.ka.was_pressed()
        }

        fn get_keyboard(&self) -> &[bool] {
            self.ka.get_keyboard()
        }
    }

    #[test]
    fn test_runner() {
        const ROM_NAME: &str = "IBMLOGO";

        let mut mock_display = MockInternalDCommands::new();

        mock_display.expect_display().times(1).return_const(());

        let da = DisplayAdapter { da: mock_display };

        let mock_keyboard = MockInternalKCommands::new();

        let ka = KeyboardAdapter { ka: mock_keyboard };

        let mut controller: Controller<_, _, Worker> = Controller::new(da, ka);

        let rom = crate::resources::RomArchives::new()
            .get_file_data(ROM_NAME)
            .expect("Something went wrong while extracting the rom");

        controller.set_rom(rom);

        assert_eq!(Ok(()), run(&mut controller));
        assert_eq!(Operation::Draw, controller.operation());
        assert_eq!(Ok(()), run(&mut controller));
    }
}
