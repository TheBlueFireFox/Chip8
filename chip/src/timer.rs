use std::{
    sync::{
        mpsc::{self, RecvTimeoutError, SyncSender},
        Arc, RwLock,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub trait Timed<V>
where
    V: num::Unsigned + std::cmp::PartialOrd<V> + Send + Sync + Copy + 'static,
{
    /// Will create a new timer with the given value.
    fn new(value: V, interval: Duration) -> Self;

    /// Will set the value from which the timer shall count down from.
    fn set_value(&mut self, value: V);

    /// Will get the value that the counter is currently at.
    fn get_value(&self) -> V;
}

/// A timer that will count down to 0, from any type that does support it
pub(crate) struct Timer<W, V>
where
    W: TimedWorker,
    V: num::Unsigned,
{
    /// will store the value of the timer
    value: Arc<RwLock<V>>,
    /// Represents a timer inside of the chip
    /// infrastruture, it will count down to
    /// zero from what ever number given in
    /// the speck requireds 60Hz.
    _worker: W,
}

impl<W, V> Timed<V> for Timer<W, V>
where
    W: TimedWorker,
    V: num::Unsigned + std::cmp::PartialOrd<V> + Send + Sync + Copy + 'static,
{
    fn new(value: V, interval: Duration) -> Self {
        let mut worker = W::new();
        let value = Arc::new(RwLock::new(value));
        let rw_value = value.clone();

        let func = move || {
            let mut cvalue = rw_value
                .write()
                .expect("something went wrong while unlocking the RW-Value");

            let value = *cvalue;
            if value > V::zero() {
                *cvalue = value - V::one();
            }
        };

        worker.start(func, interval);

        Self {
            value,
            _worker: worker,
        }
    }

    fn set_value(&mut self, value: V) {
        let mut val = self
            .value
            .write()
            .expect("something went wrong with the read write lock, while setting the value");

        *val = value;
    }

    fn get_value(&self) -> V {
        *self
            .value
            .read()
            .expect("something went wrong, while returning from the RW-Lock.")
    }
}

/// Is the internal worker, that exists on the
/// second thread.
pub(super) struct Worker {
    /// Contains the actuall thread, that is running.
    thread: Option<JoinHandle<()>>,
    /// Contains the sync sender used to gracefull shutdown the thread.
    shutdown: Option<SyncSender<()>>,
    /// Counts the actuall threads used (this is never more then 2, but
    /// is simple to use.) It uses an ```()``` so that it doesn't use
    /// up too much memory.
    alive: Arc<()>,
}

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
    /// Will check if the worker is currntly working
    fn is_alive(&self) -> bool;
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
            loop {
                match recv.recv_timeout(timeout) {
                    Err(RecvTimeoutError::Timeout) => {
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
                    Ok(_) | Err(_) => break, // shutdown
                }
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
        let mut timer: Timer<Worker, u8> =
            Timer::new(timer::HERZ, Duration::from_millis(timer::INTERVAL));
        assert!(timer._worker.is_alive());

        std::thread::sleep(Duration::from_secs(1));
        assert_eq!(timer.get_value(), 0);

        timer._worker.stop();
        assert!(!timer._worker.is_alive());
    }
}
