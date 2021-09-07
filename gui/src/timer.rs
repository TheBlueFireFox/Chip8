//! All the workers for the WASM target.
//! The timers are based on the JS functions `setInterval` and `setTimeout`.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};
use wasm_bindgen::{prelude::*, JsCast};

use crate::utils::BrowserWindow;
use chip::timer::TimedWorker;

/// Wrapps the actuall implementation so that the TimedWorker thread condition,
/// for the Timer can be fullfilled correctly.
pub(crate) struct TimingWorker {
    worker: ProcessWorker,
}

impl TimedWorker for TimingWorker {
    fn new() -> Self {
        Self {
            worker: ProcessWorker::new().expect("Error during WasmWorker creation."),
        }
    }

    fn start<T>(&mut self, mut callback: T, interval: Duration)
    where
        T: Send + FnMut() + 'static,
    {
        let icallback = move || {
            callback();
            Ok(())
        };
        self.worker
            .start(icallback, interval)
            .expect("Unexpected error during start of timed worker");
    }

    fn stop(&mut self) {
        self.worker.stop();
    }

    fn is_alive(&self) -> bool {
        self.worker.is_alive()
    }
}

/// The states of the worker if it is running or not.
/// This is primarily for redability usage.
#[derive(Debug, Clone, Copy)]
enum WorkerState {
    CanRun,
    CannotRun,
}

/// All the states that the running thread can take
/// This is used so that possible crashed or expected
/// shutdowns can be logged.
#[derive(Debug, Clone, Copy)]
enum ProgrammState {
    Failure,
    Running,
    Shutdown,
    Stop,
}

/// Will take care that assuming, there was a
/// crash on the worker thread and the
/// function call get's called anyway
/// to stop any execution then.
pub struct ProcessWorker {
    /// The worker registration used for getting the chip running every few milliseconds.
    worker: WasmWorker,
    /// If the run method had run with out problems
    state: Rc<Cell<ProgrammState>>,
    /// A possible clean up function called once the worker
    /// exists processing.
    shutdown: Rc<RefCell<Option<Box<dyn FnOnce() + 'static>>>>,
}

impl ProcessWorker {
    /// Will init the struct.
    pub fn new() -> Result<Self, JsValue> {
        Ok(Self {
            worker: WasmWorker::new()?,
            state: Rc::new(Cell::new(ProgrammState::Stop)),
            shutdown: Rc::new(RefCell::new(None)),
        })
    }

    /// Will start the timed worker at every interval
    pub fn start_with_shutdown<M, S>(
        &mut self,
        callback: M,
        shutdown: S,
        interval: Duration,
    ) -> Result<(), JsValue>
    where
        M: FnMut() -> anyhow::Result<()> + 'static,
        S: FnOnce() + 'static,
    {
        {
            let state = self.state.get();
            if let ProgrammState::Running = state {
                // Worker is already running
                return Err(JsValue::from("There is alread a worker running"));
            }
        }
        {
            let mut s = self.shutdown.borrow_mut();
            *s = Some(Box::new(shutdown));
        }
        self.start(callback, interval)
    }

    /// Will start the timed worker every the interval
    pub fn start<T>(&mut self, mut callback: T, interval: Duration) -> Result<(), JsValue>
    where
        T: FnMut() -> anyhow::Result<()> + 'static,
    {
        let istate = self.state.clone();
        let ishutdown = self.shutdown.clone();

        // set up the state state

        match self.set_start_state() {
            WorkerState::CanRun => {}
            WorkerState::CannotRun => {
                return Err(JsValue::from("Cannot start the worker."));
            }
        }

        let internal_callback = move || {
            // check if state was poisoned => there was a crash
            let state = istate.get();

            // check sucess state so that the system will not be overwhelem
            // and crash by error messages or that the thread continues to
            // run after crash.

            let shutdown = match state {
                ProgrammState::Running => false,
                ProgrammState::Failure => {
                    log::error!("Shuting down due to error"); // print error message
                    true
                }
                ProgrammState::Shutdown => {
                    log::info!("Shutting down the processing");
                    true
                }
                ProgrammState::Stop => {
                    log::error!("Something unexpected paniced");
                    true
                }
            };

            if shutdown {
                // try to call the shutdown process
                if let Some(shutdown_callback) = ishutdown.borrow_mut().take() {
                    shutdown_callback();
                }

                istate.set(ProgrammState::Shutdown);
                return;
            }

            if let Err(err) = callback() {
                log::error!("{}", err);
                istate.set(ProgrammState::Failure);
            }
        };

        self.worker.start(internal_callback, interval)
    }

    /// Will stop the timed worker
    pub fn stop(&mut self) {
        self.state.set(ProgrammState::Stop);
        self.worker.stop();
    }

    /// If the worker is alive.
    pub fn is_alive(&self) -> bool {
        self.worker.is_alive()
    }

    /// Will setup the struct so that the worker can run, in case that there is already a worker
    /// running a `WorkerState::CannotRun` is returned.
    fn set_start_state(&mut self) -> WorkerState {
        let state = self.state.get();
        if let ProgrammState::Running = state {
            // Worker is already running
            WorkerState::CannotRun
        } else {
            self.state.set(ProgrammState::Running);
            WorkerState::CanRun
        }
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
    /// Will create the wasm worker
    pub(crate) fn new() -> Result<Self, JsValue> {
        Ok(Self {
            interval_id: None,
            function: None,
            browser: BrowserWindow::new().or_else(|err| Err(JsValue::from(err)))?,
        })
    }

    /// Will start to run the process.
    /// Will return an error, if there is already a running worker.
    pub(crate) fn start<T>(&mut self, callback: T, interval: Duration) -> Result<(), JsValue>
    where
        T: FnMut() + 'static,
    {
        // stop any action around
        if self.is_alive() {
            return Err(JsValue::from(
                "Unable to start worker, as worker is already running.",
            ));
        }

        let function = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);

        let interval_id = self.browser.set_interval(
            function.as_ref().unchecked_ref(),
            interval.as_millis() as i32,
        )?;
        self.interval_id = Some(interval_id);
        self.function = Some(function);
        Ok(())
    }

    /// Will stop the worker
    pub(crate) fn stop(&mut self) {
        // stop the interval call
        if let Some(id) = self.interval_id.take() {
            self.browser.clear_interval(id);
        }
        // remove the closure struct to return the memory
        if let Some(function) = self.function.take() {
            drop(function);
        }
    }

    /// Checks if the worker is alive
    pub(crate) fn is_alive(&self) -> bool {
        self.interval_id.is_some() && self.function.is_some()
    }
}

impl Drop for WasmWorker {
    fn drop(&mut self) {
        self.stop();
    }
}
