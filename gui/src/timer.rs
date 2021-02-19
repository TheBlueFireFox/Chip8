use std::time::Duration;
use wasm_bindgen::{prelude::*, JsCast};

use crate::helpers::BrowserWindow;
use chip::timer::TimedWorker;

/// see here https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen/closure/struct.Closure.html#using-the-setinterval-api
pub(super) struct Worker {
    interval_id: Option<i32>,
    function: Option<Closure<dyn FnMut()>>,
    browser: BrowserWindow,
}

impl Worker {
    /// Get a reference to the worker's interval id.
    pub(crate) fn interval_id(&self) -> Option<i32> {
        self.interval_id
    }
}

impl TimedWorker for Worker {
    fn new() -> Self {
        Self {
            interval_id: None,
            function: None,
            browser: BrowserWindow::new(),
        }
    }

    fn start<T>(&mut self, callback: T, interval: Duration)
    where
        T: Send + FnMut() + 'static,
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
            )
            .expect("something went wrong");
        self.interval_id = Some(interval_id);
        self.function = Some(function);
    }

    fn stop(&mut self) {
        if let Some(id) = self.interval_id.take() {
            self.browser.window().clear_interval_with_handle(id);
        }
    }

    fn is_alive(&self) -> bool {
        self.interval_id.is_some()
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop();
    }
}
