use std::{
    cell::{Cell, Ref, RefCell},
    rc::Rc,
    time::Duration,
};
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
            worker: WasmWorker::new().expect("Error during WasmWorker creation."),
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

#[derive(Debug, Clone, Copy)]
enum WorkerState {
    CanRun,
    CannotRun
}

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
pub struct WorkerWrapper<W: TimedWorker> {
    worker: W,
    /// If the run method had run with out problems
    state: Rc<Cell<ProgrammState>>,
    shutdown: Rc<RefCell<Option<Box<dyn FnOnce() + Send + 'static>>>>,
}

impl<W: TimedWorker> WorkerWrapper<W> {
    pub fn new() -> Self {
        Self {
            worker: W::new(),
            state: Rc::new(Cell::new(ProgrammState::Stop)),
            shutdown: Rc::new(RefCell::new(None)),
        }
    }

    /// Will start the timed worker every the interval
    pub fn start_with_shutdown<M, S>(&mut self, callback: M, shutdown: S, interval: Duration)
    where
        M: Send + FnMut() -> Result<(), String> + 'static,
        S: Send + FnOnce() + 'static,
    {
        {
            let state = self.state.get();
            if let ProgrammState::Running = state {
                // Worker is already running
                return;
            }
        }
        {
            let mut s = self.shutdown.borrow_mut();
            *s = Some(Box::new(shutdown));
        }
        self.start(callback, interval);
    }

    /// Will start the timed worker every the interval
    pub fn start<T>(&mut self, mut callback: T, interval: Duration)
    where
        T: Send + FnMut() -> Result<(), String> + 'static,
    {
        let istate = self.state.clone();
        let ishutdown = self.shutdown.clone();

        // set up the state state
        
        match self.set_start_state() {
            WorkerState::CanRun => {}
            WorkerState::CannotRun => {return;}
        }
        
        let internal_callback = move || {
            // check if state was poisoned => there was a crash
            let state = match istate.read() {
                Ok(s) => *s,
                Err(_) => ProgrammState::Failure,
            };

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
                let mut success = false;
                if let Ok(mut shutdown_lock) = ishutdown.lock() {
                    if let Some(shutdown_callback) = shutdown_lock.take() {
                        shutdown_callback();
                    }
                    success = true;
                }
                if let Ok(mut state) = istate.write() {
                    *state = if success {
                        ProgrammState::Shutdown
                    } else {
                        ProgrammState::Failure
                    };
                }
                return;
            }

            if let Err(err) = callback() {
                log::error!("{}", err);
                if let Ok(mut state) = istate.write() {
                    *state = ProgrammState::Failure;
                }
            }
        };

        self.worker.start(internal_callback, interval);
    }

    /// Will stop the timed worker
    pub fn stop(&mut self) {
        let mut state = self.state.write().expect("Unlocking went wrong.");
        *state = ProgrammState::Stop;

        self.worker.stop();
    }

    fn set_start_state(&mut self) -> WorkerState {
        let mut state = self.state.get();
        if let ProgrammState::Running = state {
            // Worker is already running
            WorkerState::CanRun
        } else {
            self.state.set(ProgrammState::Running);
            WorkerState::CannotRun
        }
    }

}

impl<W: TimedWorker> Drop for WorkerWrapper<W> {
    fn drop(&mut self) {
        self.stop()
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
    pub(crate) fn new() -> Result<Self, JsValue> {
        Ok(Self {
            interval_id: None,
            function: None,
            browser: BrowserWindow::new().or_else(|err| Err(JsValue::from(err)))?,
        })
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
}

impl Drop for WasmWorker {
    fn drop(&mut self) {
        self.stop();
    }
}
