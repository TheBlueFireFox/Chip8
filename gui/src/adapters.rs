//! The adapters used to interface with the display, keyboard and sound system of the browser.
//! All of the given functionality is based on `wam_bindgen` abstractions.

use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{
    definitions,
    observer::{EventSystem, Observer},
    utils::BrowserWindow,
};
use chip::{
    devices::{DisplayCommands, Keyboard, KeyboardCommands},
    timer::TimerCallback,
};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{AudioContext, GainNode, OscillatorNode};

/// The Oscillator implementation, it contains structs that allow for live sound generation.
struct Oscillator {
    /// The AudioContext interface represents an audio-processing graph built from audio modules
    /// linked together.
    ctx: AudioContext,
    /// The OscillatorNode interface represents a periodic waveform, such as a sine wave.
    main: OscillatorNode,
    /// The GainNode interface represents a change in volume.
    gain: GainNode,
}

impl Oscillator {
    /// Initializes the oscillator node and sets it up.
    fn new() -> Result<Self, JsValue> {
        let ctx = AudioContext::new()?;
        let main = ctx.create_oscillator()?;
        let gain = ctx.create_gain()?;
        let me = Self { ctx, main, gain };
        me.setup()?;

        Ok(me)
    }

    /// The node setup
    fn setup(&self) -> Result<(), JsValue> {
        self.main.set_type(web_sys::OscillatorType::Sine);
        self.main.frequency().set_value(440.0); // A4 note
        self.gain.gain().set_value(0.5);

        // Connect the nodes up!

        // The primary oscillator is routed through the gain node, so that
        // it can control the overall output volume.
        self.main.connect_with_audio_node(&self.gain)?;

        // Then connect the gain node to the AudioContext destination (aka
        // your speakers).
        self.gain.connect_with_audio_node(&self.ctx.destination())?;

        Ok(())
    }

    /// Starts the sound production.
    fn start(&self) -> Result<(), JsValue> {
        self.main.start()
    }

    /// Stops the sound production.
    fn stop(&self) -> Result<(), JsValue> {
        self.main.stop()
    }
}

/// A struct that only contains the timeout id needed to stop sound execution.
pub(crate) struct SoundCallback {
    timeout_id: Arc<Mutex<Option<i32>>>,
}

impl SoundCallback {
    /// Starts to create the sound.
    fn start(&mut self, timeout: i32) -> Result<(), JsValue> {
        let mut timeout_id = self
            .timeout_id
            .lock()
            .or_else(|err| Err(JsValue::from(format!("{}", err))))?;

        if timeout_id.is_some() {
            return Err(JsValue::from("A soundcallback has already been send out"));
        }

        let ctimeout_id = self.timeout_id.clone();
        let osci = Oscillator::new()?;
        osci.start()?;

        // moving the osci into this closure keeps it alive
        let stop = move || {
            let mut timeout_id = ctimeout_id
                .lock()
                .or_else(|err| Err(JsValue::from(format!("{}", err))))?;

            osci.stop();
            *timeout_id = None;

            Ok(())
        };
        // SAFETY: As stopping the callback is rare to the point of never
        // being used, this might leak memory although only rarely and never
        // in large amounts.
        let callback = Closure::once_into_js(stop);

        let window = BrowserWindow::new()?;
        let id = window
            .window()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &callback.as_ref().unchecked_ref(),
                timeout,
            )?;

        *timeout_id = Some(id);

        Ok(())
    }

    /// Stops to create the sound if possible.
    fn stop(&mut self) -> Result<(), JsValue> {
        let mut timeout = self
            .timeout_id
            .lock()
            .or_else(|err| Err(JsValue::from(format!("{}", err))))?;

        // This is only ever be a problem when the sound callback get's dropped,
        // before the timeout function ran.
        if let Some(id) = timeout.take() {
            let window = BrowserWindow::new()?;
            window.window().clear_timeout_with_handle(id);
        }

        Ok(())
    }
}

impl TimerCallback for SoundCallback {
    fn new() -> Self {
        Self {
            timeout_id: Arc::new(Mutex::new(None)),
        }
    }

    fn handle(&mut self) {
        match self.start(chip::definitions::sound::DURRATION.as_millis() as i32) {
            Ok(_) => {}
            Err(err) => log::warn!("{:?}", err),
        }
    }
}

impl Drop for SoundCallback {
    fn drop(&mut self) {
        self.stop()
            .expect("Something went terribly wrong, while dropping the sound callback.")
    }
}

pub(crate) struct DisplayAdapter;

impl DisplayAdapter {
    pub fn new() -> Self {
        DisplayAdapter {}
    }

    fn draw_board<M, V>(pixels: M) -> Result<(), JsValue>
    where
        M: AsRef<[V]>,
        V: AsRef<[bool]>,
    {
        let html = BrowserWindow::new().or_else(|err| Err(JsValue::from(err)))?;
        let document = html.document();

        let table = document.create_element(definitions::field::TYPE)?;
        table.set_id(definitions::field::ID);

        for row in pixels.as_ref().iter() {
            let tr = document.create_element(definitions::field::TYPE_ROW)?;
            for value in row.as_ref().iter() {
                let td = document.create_element(definitions::field::TYPE_COLUMN)?;

                if !*value {
                    td.set_class_name(definitions::field::ACTIVE);
                }

                tr.append_child(&td)?;
            }
            table.append_child(&tr)?;
        }

        // check if already exists, if exists replace element
        if let Some(element) = document.get_element_by_id(definitions::field::ID) {
            let _ = html.body().replace_child(&table, &element)?;
        } else {
            html.body().append_child(&table)?;
        }

        Ok(())
    }
}

impl DisplayCommands for DisplayAdapter {
    fn display<M: AsRef<[V]>, V: AsRef<[bool]>>(&self, pixels: M) {
        log::debug!("Drawing the display");

        Self::draw_board(pixels).expect("something went wrong while working on the board");
    }
}

pub(crate) struct KeyboardAdapter {
    keyboard: Keyboard,
    event_system: EventSystem<usize>,
}

impl KeyboardAdapter {
    pub fn new() -> Self {
        Self {
            keyboard: Keyboard::new(),
            event_system: EventSystem::new(),
        }
    }

    pub fn register_callback<T>(&mut self, data: Rc<RefCell<T>>)
    where
        T: Observer<usize> + 'static,
    {
        self.event_system.register_observer(data);
    }

    /// Get a reference to the keyboard adapter's keyboard.
    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }
}

impl KeyboardCommands for KeyboardAdapter {
    fn was_pressed(&self) -> bool {
        self.keyboard.get_last().is_some()
    }

    fn get_keyboard(&self) -> &[bool] {
        todo!()
    }
}
