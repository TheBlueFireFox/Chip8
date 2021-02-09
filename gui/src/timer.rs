use chip::timer::TimedWorker;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::Window;

fn window() -> Window {
    web_sys::window().expect("No `global` window found")
}

/// see here https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen/closure/struct.Closure.html#using-the-setinterval-api
pub(super) struct Worker {
    interval_id: Option<i32>,
}

impl TimedWorker for Worker {
    fn new() -> Self {
        Self { interval_id: None }
    }

    fn start<T>(&mut self, callback: T, interval: std::time::Duration)
    where
        T: Send + FnMut() + 'static,
    {
        // stop any action around
        self.stop();

        let function = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);

        // SAFETY: unwrap is safe here, as it is set a line above.

        let interval_id = window()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                function.as_ref().unchecked_ref(),
                interval.as_millis() as i32,
            )
            .expect("something went wrong");
        self.interval_id = Some(interval_id);
        // SAFETY: Attention leaks memory, but as the system shall support both
        // threaded and set intervall and a Closure is not Send no other option
        // is available.
        // Once WEAK_REF is supported this problem will be solved.
        function.forget();
    }

    fn stop(&mut self) {
        if let Some(id) = self.interval_id.take() {
            window().clear_interval_with_handle(id);
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
