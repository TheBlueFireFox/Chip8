//! All the workers for the WASM target.
//! The timers are based on the JS functions `setInterval` and `setTimeout`.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use chip::timer::TimedWorker;
use gloo::timers::callback::Interval;

use crate::error;

/// Wrapps the actuall implementation so that the TimedWorker thread condition,
/// for the Timer can be fullfilled correctly.
pub(crate) struct TimingWorker {
    worker: ProcessWorker,
}

impl TimedWorker for TimingWorker {
    fn new() -> Self {
        Self {
            worker: ProcessWorker::new(),
        }
    }

    fn start<T>(&mut self, mut callback: T, interval: Duration)
    where
        T: FnMut() + 'static,
    {
        // ignore the &str here it is needed for some trait bound
        let icallback = move || -> Result<(), &'static str> {
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

// CallBack is a type abstraction used to simplify reading the ProcessWorker
type CallBack = Box<dyn FnOnce() + 'static>;

/// Will take care that, assuming there was a
/// crash on the worker thread and that the
/// function call get's called anyway,
/// any execution will be stopped.
pub struct ProcessWorker {
    /// The worker registration used for getting the chip running every few milliseconds.
    worker: WasmWorker,
    /// If the run method had run with out problems
    state: Rc<Cell<ProgrammState>>,
    /// A possible clean up function called once the worker
    /// exists processing.
    shutdown: Rc<RefCell<Option<CallBack>>>,
}

impl ProcessWorker {
    /// Will init the struct.
    pub fn new() -> Self {
        Self {
            worker: WasmWorker::new(),
            state: Rc::new(Cell::new(ProgrammState::Stop)),
            shutdown: Rc::new(RefCell::new(None)),
        }
    }

    /// Will start the timed worker every the interval
    pub fn start<T, E>(
        &mut self,
        mut callback: T,
        interval: Duration,
    ) -> Result<(), error::WasmWorkerError>
    where
        T: FnMut() -> Result<(), E> + 'static,
        E: std::fmt::Display,
    {
        let istate = self.state.clone();
        let ishutdown = self.shutdown.clone();

        // set up the state state

        if let WorkerState::CannotRun = self.set_start_state() {
            return Err(error::WasmWorkerError::DoesNotStart);
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

        self.worker.start(internal_callback, interval)?;
        Ok(())
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
#[derive(Debug, Default)]
pub(crate) struct WasmWorker {
    /// The Closure object that has to be held
    /// or the function will stop executing
    /// and crash after the drop is called.
    function: Option<Interval>,
}

impl WasmWorker {
    /// Will create the wasm worker
    pub(crate) fn new() -> Self {
        Default::default()
    }

    /// Will start to run the process.
    /// Will return an error, if there is already a running worker.
    pub(crate) fn start<T>(
        &mut self,
        callback: T,
        interval: Duration,
    ) -> Result<(), error::WasmWorkerError>
    where
        T: FnMut() + 'static,
    {
        // stop any action around
        if self.is_alive() {
            return Err(error::WasmWorkerError::AlreadyActive);
        }

        self.function = Some(Interval::new(
            interval
                .as_millis()
                .try_into()
                .expect("interval duration might only be max 2^32-1ms long"),
            callback,
        ));
        Ok(())
    }

    /// Will stop the worker
    pub(crate) fn stop(&mut self) {
        // remove the closure struct to return the memory
        if let Some(function) = self.function.take() {
            drop(function);
        }
    }

    /// Checks if the worker is alive
    pub(crate) fn is_alive(&self) -> bool {
        self.function.is_some()
    }
}

impl Drop for WasmWorker {
    fn drop(&mut self) {
        self.stop();
    }
}
