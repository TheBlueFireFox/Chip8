//! The main interface out of the crate.
//!
//! Handles part of the execution and interaction with the display, keyboard and sound system.
use crate::{
    chip8::ChipSet,
    devices::{DisplayCommands, KeyboardCommands},
    opcode::Operation,
    resources::Rom,
    timer::{TimedWorker, TimerCallback},
};

/// A collection of all the important interfaces.
/// Is primarily used to simplify the crate api.
pub struct Controller<D, K, W, S>
where
    D: DisplayCommands,
    K: KeyboardCommands,
    S: TimerCallback,
    W: TimedWorker,
{
    /// The display adapter, so that is can be controlled during execution.
    display: D,
    /// The keyboard adapter, so that the keypresses can be registred and red.
    keyboard: K,
    /// The all important chipset implementation.
    chipset: Option<ChipSet<W, S>>,
    /// The next run operation.
    operation: Operation,
}

impl<D, K, W, S> Controller<D, K, W, S>
where
    D: DisplayCommands,
    K: KeyboardCommands,
    W: TimedWorker,
    S: TimerCallback,
{
    /// Creates a new constroller.
    pub fn new(dis: D, key: K) -> Self {
        Controller {
            display: dis,
            keyboard: key,
            chipset: None,
            operation: Operation::None,
        }
    }

    /// Get a reference to the controller's chipset.
    pub fn chipset(&self) -> &Option<ChipSet<W, S>> {
        &self.chipset
    }

    /// Get a mutable reference to the controller's chipset.
    pub fn chipset_mut(&mut self) -> Option<&mut ChipSet<W, S>> {
        self.chipset.as_mut()
    }

    /// Set the controller's chipset.
    pub fn set_rom(&mut self, rom: Rom) {
        let chipset = ChipSet::with_keyboard(rom, self.keyboard.get_keyboard());
        self.chipset = Some(chipset);
    }

    /// Remove the rom and resets the internal state of the chip to the new state.
    pub fn remove_rom(&mut self) {
        self.chipset = None;
        self.operation = Operation::None;
    }

    /// Get a reference to the controller's keyboard.
    pub fn keyboard(&mut self) -> &mut K {
        &mut self.keyboard
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

/// The main function that has to be called every
/// [`interval`](super::definitions::cpu::INTERVAL).
///
/// This function handles all of the heavy lifing required by the operations and
/// interact with the different adapters.
pub fn run<D, K, W, S>(
    Controller {
        display,
        keyboard,
        chipset,
        operation,
    }: &mut Controller<D, K, W, S>,
) -> Result<(), String>
where
    D: DisplayCommands,
    K: KeyboardCommands,
    S: TimerCallback,
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

    use std::sync::{Arc, RwLock};

    use super::*;
    use crate::{
        devices::Keyboard,
        timer::{NoCallback, Worker},
    };
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
        fn set_key(&mut self, key: usize, to: bool);
        fn was_pressed(&self) -> bool;
        fn get_keyboard(&mut self) -> Arc<RwLock<Keyboard>>;
    }

    struct KeyboardAdapter<M>
    where
        M: InternalKCommands,
    {
        ka: M,
    }

    impl<M: InternalKCommands> KeyboardCommands for KeyboardAdapter<M> {
        fn set_key(&mut self, key: usize, to: bool) {
            self.ka.set_key(key, to);
        }

        fn was_pressed(&self) -> bool {
            self.ka.was_pressed()
        }

        fn get_keyboard(&mut self) -> Arc<RwLock<Keyboard>> {
            self.ka.get_keyboard()
        }
    }

    #[test]
    fn test_runner() {
        const ROM_NAME: &str = "IBMLOGO";

        let mut mock_display = MockInternalDCommands::new();

        mock_display.expect_display().times(1).return_const(());

        let da = DisplayAdapter { da: mock_display };

        let mut mock_keyboard = MockInternalKCommands::new();
        mock_keyboard
            .expect_get_keyboard()
            .returning(|| Arc::new(RwLock::new(Keyboard::new())));

        let ka = KeyboardAdapter { ka: mock_keyboard };

        let mut controller: Controller<_, _, Worker, NoCallback> = Controller::new(da, ka);

        assert_eq!(
            Err("There is no valid chipset initialized.".to_string()),
            run(&mut controller)
        );

        let rom = crate::resources::RomArchives::new()
            .get_file_data(ROM_NAME)
            .expect("Something went wrong while extracting the rom");

        controller.set_rom(rom);

        assert_eq!(Ok(()), run(&mut controller));
        assert_eq!(Operation::Draw, controller.operation());

        assert_eq!(Ok(()), run(&mut controller));
    }
}
