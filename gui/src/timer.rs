use std::time::Duration;
use wasm_bindgen::{prelude::*, JsCast};

use crate::utils::BrowserWindow;
use chip::timer::TimedWorker;

pub(crate) struct TimingWorker {
    /// Wrapps the actuall implementation so that the TimedWorker thread condition,
    /// for the Timer can be fullfilled correctly.
    worker: WasmWorker,
}

impl TimedWorker for TimingWorker {
    fn new() -> Self {
        Self {
            worker: WasmWorker::new(),
        }
    }

    fn start<T>(&mut self, callback: T, interval: Duration)
    where
        T: Send + FnMut() + 'static,
    {
        self.worker
            .start(callback, interval)
            .expect("Something went terribly wrong while initializing the worker thread.");
    }

    fn stop(&mut self) {
        self.worker.stop();
    }

    fn is_alive(&self) -> bool {
        self.worker.is_alive()
    }
}

/// The actuall worker for the peudo-wasm-thread.
/// The start function in this version does not
/// need the Send bound, as well as to send the
/// Controller over a !Send is requiered.
pub(crate) struct WasmWorker {
    /// The by JS given interval id.
    interval_id: Option<i32>,
    /// The Closure object that has to be held
    /// or the function will stop executing
    /// and crash after the drop is called.
    function: Option<Closure<dyn FnMut()>>,
    /// The browser window object wrapper
    /// is held for convinience.
    browser: BrowserWindow,
}

impl WasmWorker {
    pub(crate) fn new() -> Self {
        Self {
            interval_id: None,
            function: None,
            browser: BrowserWindow::new(),
        }
    }

    pub(crate) fn start<T>(&mut self, callback: T, interval: Duration) -> Result<(), JsValue>
    where
        T: FnMut() + 'static,
    {
        // stop any action around
        self.stop();

        let function = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);

        let interval_id = self
            .browser
            .window()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                function.as_ref().unchecked_ref(),
                interval.as_millis() as i32,
            )?;
        self.interval_id = Some(interval_id);
        self.function = Some(function);
        Ok(())
    }

    pub(crate) fn stop(&mut self) {
        if let Some(id) = self.interval_id.take() {
            self.browser.window().clear_interval_with_handle(id);
        }
    }

    pub(crate) fn is_alive(&self) -> bool {
        self.interval_id.is_some()
    }

    pub(crate) fn interval_id(&self) -> Option<i32> {
        self.interval_id
    }
}

impl Drop for WasmWorker {
    fn drop(&mut self) {
        self.stop();
    }
}
