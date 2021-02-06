use chip::{
    timer::{TimedWorker},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn setInterval(closure: &Closure<dyn FnMut()>, time: u32) -> i32;
    fn clearInterval(id: i32);
}

/// see here https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen/closure/struct.Closure.html#using-the-setinterval-api
pub(super) struct Worker {
    interval_id: Option<i32>,
    function: Option<Closure<dyn FnMut()>>,
}

impl TimedWorker for Worker {
    fn new() -> Self {
        Self {
            interval_id: None,
            function: None,
        }
    }

    fn start<T>(&mut self, callback: T, interval: std::time::Duration)
    where
        T: Send + FnMut() + 'static,
    {
        // stop any action around
        self.stop();

        let function = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);

        // SAFETY: unwrap is safe here, as it is set a line above.
        let interval_id = setInterval(&function, interval.as_millis() as u32);

        self.function = Some(function);
        self.interval_id = Some(interval_id);
    }

    fn stop(&mut self) {
        if let Some(id) = self.interval_id.take() {
            clearInterval(id);
        }

        let _ = self.function.take();
    }

    fn is_alive(&self) -> bool {
        self.interval_id.is_some() && self.function.is_some()
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop();
    }
}
