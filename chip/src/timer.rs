//! The countdown timers required by the Chip8 specification.
use std::{
    sync::{
        mpsc::{self, RecvTimeoutError, SyncSender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use num_traits as num;
use parking_lot::{Mutex, RwLock};

/// Handles the callback onces the timer reaches zero.
pub trait TimerCallback: Send + 'static {
    /// Creates a new callback instance.
    fn new() -> Self;
    /// Handles the callback.
    fn handle(&mut self);
}

/// An abstraction over the internal timer, so that
/// different worker implementations, can be used.
pub trait TimedWorker {
    /// Will create the respective timer
    /// The reason that this is a required method
    /// is so that the implementing types can
    /// instantiate it them selves.
    fn new() -> Self;
    /// Will start the timed worker every the interval
    fn start<T>(&mut self, callback: T, interval: Duration)
    where
        T: Send + FnMut() + 'static;
    /// Will stop the timed worker
    fn stop(&mut self);
    /// Will check if the worker is currently working
    fn is_alive(&self) -> bool;
}

/// Empty implementation (default where there is no callback)
pub struct NoCallback;

impl TimerCallback for NoCallback {
    fn new() -> Self {
        Self {}
    }
    fn handle(&mut self) {}
}

/// The clonable value holder of the timer.
#[derive(Clone)]
pub struct TimerValue<V> {
    /// will store the value of the timer.
    value: Arc<RwLock<V>>,
}

impl<V: num::Unsigned + Copy> TimerValue<V> {
    /// This create the TimerValue instance.
    /// Attention is is set to private, so that there can not be an instance created execept from
    /// [`Timer::new`](Timer::new).
    fn new(value: Arc<RwLock<V>>) -> Self {
        Self { value }
    }

    /// Setter for the internal value.
    pub fn set_value(&mut self, value: V) {
        let mut val = self.value.write();

        *val = value;
    }

    /// Getter for the internal value.
    pub fn get_value(&self) -> V {
        *self.value.read()
    }
}

/// A timer that will count down to 0, from any type that does support it
pub struct Timer<W, V, S>
where
    W: TimedWorker,
    V: num::Unsigned,
    S: TimerCallback,
{
    /// will store the value of the timer
    value: Arc<RwLock<V>>,
    /// Represents a timer inside of the chip
    /// infrastruture, it will count down to
    /// zero from what ever number given in
    /// the speck requireds 60Hz.
    _worker: W,
    /// Is the optional function that might get called once the timer
    /// reaches zero.
    callback: Arc<Mutex<Option<S>>>,
}
impl<W, V> Timer<W, V, NoCallback>
where
    W: TimedWorker,
    V: num::Unsigned + std::cmp::PartialOrd<V> + Send + Sync + Copy + 'static,
{
    /// generates the default timer.
    pub fn new(value: V, interval: Duration) -> (Self, TimerValue<V>) {
        Self::internal_new(value, interval)
    }
}

impl<W, V, S> Timer<W, V, S>
where
    W: TimedWorker,
    V: num::Unsigned + std::cmp::PartialOrd<V> + Send + Sync + Copy + 'static,
    S: TimerCallback,
{
    /// Will actually generate the timer.
    /// This function has been abstracted out for simplicity.
    fn internal_new(value: V, interval: Duration) -> (Self, TimerValue<V>) {
        let cb: Arc<Mutex<Option<S>>> = Arc::new(Mutex::new(None));
        let mut worker = W::new();

        let value = Arc::new(RwLock::new(value));
        let rw_value = value.clone();
        let ccb = cb.clone();

        let func = move || {
            let mut cvalue = rw_value.write();

            let value = *cvalue;

            // basically the last moment before the timer stops working
            if value == V::one() {
                // This is safe as this block will only ever once be called from a single
                // other thread.
                let mut lock = ccb.lock();

                if let Some(callback_handler) = lock.as_mut() {
                    callback_handler.handle();
                }
            }
            if value > V::zero() {
                *cvalue = value - V::one();
            }
        };

        worker.start(func, interval);

        (
            Self {
                value: value.clone(),
                _worker: worker,
                callback: cb,
            },
            TimerValue::new(value),
        )
    }

    /// Will create a new timer that has an internal callback.
    pub fn with_callback(value: V, interval: Duration, sound_handler: S) -> (Self, TimerValue<V>) {
        let (timer, value) = Self::internal_new(value, interval);
        // using internal scope to remove uneeded borrow and to return value from
        // function
        {
            let mut lock = timer.callback.lock();
            *lock = Some(sound_handler);
        }
        (timer, value)
    }

    /// The setter for the timer value.
    pub fn set_value(&mut self, value: V) {
        let mut val = self.value.write();

        *val = value;
    }

    /// The getter fo the timer value at this current moment.
    pub fn get_value(&self) -> V {
        *self.value.read()
    }
}

/// Is the internal worker, that exists on the
/// second thread.
pub struct Worker {
    /// Contains the actuall thread, that is running.
    thread: Option<JoinHandle<()>>,
    /// Contains the sync sender used to gracefull shutdown the thread.
    shutdown: Option<SyncSender<()>>,
    /// Counts the actuall threads used (this is never more then 2, but
    /// is simple to use.) It uses an ```()``` so that it doesn't use
    /// up too much memory.
    alive: Arc<()>,
}

impl TimedWorker for Worker {
    /// Will initialize the new worker.
    fn new() -> Self {
        Self {
            thread: None,
            shutdown: None,
            alive: Arc::new(()),
        }
    }

    /// Will start the worker that will run the callback function
    /// all duration.
    /// Attention the timer assumes the callback will finish
    /// calculation faster then the callback.
    fn start<T>(&mut self, mut callback: T, interval: Duration)
    where
        T: Send + FnMut() + 'static,
    {
        // stop any action around
        self.stop();

        let (send, recv) = mpsc::sync_channel::<()>(1);
        let alive = self.alive.clone();
        let thread = thread::spawn(move || {
            // this is to count the references, as it will not actually
            // be used ```_``` is used in front of the name.
            let _alive = alive;
            let mut timeout = interval;

            while let Err(RecvTimeoutError::Timeout) = recv.recv_timeout(timeout) {
                // set the duration to the correct interval
                let start = std::time::SystemTime::now();

                // run the callback function
                callback();

                // make sure there the system will at most wait the interval
                let duration = start
                    .elapsed()
                    .expect("For unknown reasons time moved back in time...");

                timeout = if interval <= duration {
                    Duration::from_secs(0)
                } else {
                    interval - duration
                };
            }
        });

        self.thread = Some(thread);
        self.shutdown = Some(send);
    }

    /// Will stop the worker.
    fn stop(&mut self) {
        // Will stop the worker, in two steps one by sending an empty message
        // and second by droping the only sender for the given receiver.
        if let Some(sender) = self.shutdown.take() {
            sender
                .send(())
                .expect("This thread should be running here, but is not... Investigate.");
        }
        if let Some(thread) = self.thread.take() {
            thread
                .join()
                .expect("Something went wrong with joining the worker thread.")
        }
    }

    /// Checks if the thread is alive.
    fn is_alive(&self) -> bool {
        // This is okay as there can ever only be a single second thread around, so
        // the problem that there might be a reference count change right during
        // function execution is given the implementation rare.
        Arc::strong_count(&self.alive) > 1
    }
}

impl Drop for Worker {
    /// Will drop the worker
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definitions::timer;

    #[test]
    fn test_timer() {
        let (mut timer, _): (Timer<Worker, u8, NoCallback>, _) =
            Timer::new(timer::HERZ, Duration::from_millis(timer::INTERVAL));
        assert!(timer._worker.is_alive());

        std::thread::sleep(Duration::from_secs(1));
        assert_eq!(timer.get_value(), 0);

        timer._worker.stop();
        assert!(!timer._worker.is_alive());
    }
}
