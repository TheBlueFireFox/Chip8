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
    let last_op = operation;
    let chip = chipset
        .as_mut()
        .ok_or_else(|| "There is no valid chipset initialized.".to_string())?;

    let work = match last_op {
        Operation::Wait => keyboard.was_pressed(),
        _ => false,
    };

    if work {
        // run chip
        *last_op = chip.next()?;

        match last_op {
            Operation::Draw => {
                /* draw the screen */
                display.display(&chip.get_display()[..]);
            }
            _ => {}
        }
    }
    Ok(())
}
