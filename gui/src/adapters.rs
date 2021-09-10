//! The adapters used to interface with the display, keyboard and sound system of the browser.
//! All of the given functionality is based on `wam_bindgen` abstractions.

use std::sync::Arc;

use parking_lot::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{definitions, utils::BrowserWindow};
use chip::{
    definitions::display,
    devices::{DisplayCommands, Keyboard, KeyboardCommands},
    timer::TimerCallback,
};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{AudioContext, Element, GainNode, OscillatorNode};

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
        let mut timeout_id = self.timeout_id.lock();

        if timeout_id.is_some() {
            return Err(JsValue::from("A soundcallback has already been send out"));
        }

        let ctimeout_id = self.timeout_id.clone();
        let osci = Oscillator::new()?;
        osci.start()?;

        // moving the osci into this closure keeps it alive
        let stop = move || {
            let mut timeout_id = ctimeout_id.lock();

            osci.stop()?;
            *timeout_id = None;

            Ok(())
        };
        // SAFETY: As stopping the callback is rare to the point of never
        // being used, this might leak memory although only rarely and never
        // in large amounts.
        let callback = Closure::once_into_js(stop);

        let window = BrowserWindow::new()?;
        let id = window.set_timeout(callback.as_ref().unchecked_ref(), timeout)?;

        *timeout_id = Some(id);

        Ok(())
    }

    /// Stops to create the sound if possible.
    fn stop(&mut self) -> Result<(), JsValue> {
        let mut timeout = self.timeout_id.lock();

        // This is only ever be a problem when the sound callback get's dropped,
        // before the timeout function ran.
        if let Some(id) = timeout.take() {
            BrowserWindow::new()?.clear_timeout(id);
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

/// Translates the internal commands into the external ones.
pub(crate) struct DisplayAdapter {
    elements: Option<Vec<Vec<(bool, Element)>>>,
    bw: BrowserWindow,
}

impl DisplayAdapter {
    /// Creates a new DisplayAdapter
    pub fn new(bw: &BrowserWindow) -> Self {
        Self {
            elements: None,
            bw: bw.clone(),
        }
    }

    /// Will draw the empty initial board. For visual confirmation, that the process started
    /// the board will be drawn in a chess like pattern.
    /// Additionally only this method will finish initializing the board.
    pub(crate) fn create_board(&mut self) -> Result<(), JsValue> {
        let table = self.bw.create_element(definitions::field::TYPE)?;
        table.set_id(definitions::field::ID);

        let mut rows = Vec::with_capacity(display::WIDTH);

        for i in 0..display::WIDTH {
            let tr = self.bw.create_element(definitions::field::TYPE_ROW)?;
            let mut row = Vec::with_capacity(display::HEIGHT);
            for j in 0..display::HEIGHT {
                let td = self.bw.create_element(definitions::field::TYPE_COLUMN)?;
                let state = (i + j) % 2 == 0;
                if state {
                    td.set_class_name(definitions::field::ACTIVE);
                }

                tr.append_child(&td)?;
                row.push((state, td));
            }
            rows.push(row);
            table.append_child(&tr)?;
        }

        // if there is already a board drawn.
        if self.elements.is_some() {
            let old = self
                .bw
                .get_element_by_id(definitions::field::ID)
                .ok_or_else(|| JsValue::from("Element not found"))?;

            self.bw.replace_child(&old, &table)?;
        } else {
            self.bw.append_child(&table)?;
        }

        self.elements = Some(rows);
        Ok(())
    }

    /// Will draw the actuall board this function is generic
    /// over all the parameters that deref first into an array / slice of array/slices of bool,
    /// then secondly into a pointer to a boolean.
    fn draw_board<M, V>(&mut self, pixels: M) -> Result<(), JsValue>
    where
        M: AsRef<[V]>,
        V: AsRef<[bool]>,
    {
        assert!(
            self.elements.is_some(),
            "Display adapter has not called create_board."
        );
        let elements = self.elements.as_mut().unwrap();

        for (state_row, elements_row) in pixels.as_ref().iter().zip(elements) {
            for (pstate, (old_state, element)) in state_row.as_ref().iter().zip(elements_row) {
                let state = !*pstate;

                if state == *old_state {
                    continue;
                }

                *old_state = state;

                let class_list = element.class_list();

                if state {
                    class_list.add_1(definitions::field::ACTIVE)?;
                } else {
                    class_list.remove_1(definitions::field::ACTIVE)?;
                }
            }
        }

        Ok(())
    }
}

impl DisplayCommands for DisplayAdapter {
    fn display<M: AsRef<[V]>, V: AsRef<[bool]>>(&mut self, pixels: M) {
        log::debug!("Drawing the display");

        self.draw_board(pixels)
            .expect("something went wrong while working on the board");
    }
}

/// Abstracts away the awkward js keyboard interface
pub(crate) struct KeyboardAdapter {
    /// Stores the keyboard into to which the values are changed.
    keyboard: Arc<RwLock<Keyboard>>,
}

impl KeyboardAdapter {
    /// Generates a new keyboard interface.
    pub fn new() -> Self {
        Self {
            keyboard: Arc::new(RwLock::new(Keyboard::new())),
        }
    }

    fn get_keyboard_read(&self) -> RwLockReadGuard<Keyboard> {
        self.keyboard.read()
    }

    fn get_keyboard_write(&self) -> RwLockWriteGuard<Keyboard> {
        self.keyboard.write()
    }
}

impl KeyboardCommands for KeyboardAdapter {
    fn was_pressed(&self) -> bool {
        self.get_keyboard_read().get_last().is_some()
    }

    fn get_keyboard(&mut self) -> Arc<RwLock<Keyboard>> {
        self.keyboard.clone()
    }

    fn set_key(&mut self, key: usize, to: bool) {
        self.get_keyboard_write().set_key(key, to);
    }
}
