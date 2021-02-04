use std::{sync::{Arc, RwLock}, time::Duration};

use chip::{
    definitions::TIMER_INTERVAL,
    timer::{Timed, Working},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn setInterval(closure: &Closure<dyn FnMut()>, time: u32) -> i32;
    fn clearInterval(id: i32);
}

pub(crate) struct Timer {
    value: Arc<RwLock<u8>>,
    _worker: Worker,
}

impl Timed for Timer {
    fn new(value: u8) -> Self {
        let mut worker = Worker::new();
        let value = Arc::new(RwLock::new(value));
        let rw_value = value.clone();

        let func = move || {
            let mut cvalue = rw_value
                .write()
                .expect("something went wrong while unlocking the RW-Value");
            if *cvalue > 0 {
                *cvalue -= 1;
            }
        };

        worker.start(func, Duration::from_millis(TIMER_INTERVAL));

        Self {
            value,
            _worker: worker,
        }
    }

    fn set_value(&mut self, value: u8) {
        let mut val = self
            .value
            .write()
            .expect("something went wrong with the read write lock, while setting the value");

        *val = value;
    }

    fn get_value(&self) -> u8 {
        *self
            .value
            .read()
            .expect("something went wrong, while returning from the RW-Lock.")
    }
}

/// see here https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen/closure/struct.Closure.html#using-the-setinterval-api
struct Worker {
    interval_id: Option<i32>,
    function: Option<Closure<dyn FnMut()>>,
}

impl Working for Worker {
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
