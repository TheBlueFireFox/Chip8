use {
    crate::definitions::TIMER_INTERVAL,
    std::{
        sync::{
            atomic::{AtomicU8, Ordering},
            mpsc::{self, RecvTimeoutError, SyncSender},
            Arc,
        },
        thread::{self, JoinHandle},
        time::Duration,
    },
};

pub trait Timed {
    /// Will create a new timer with the given value.
    fn new(value: u8) -> Self;

    /// Will set the value from which the timer shall count down from.
    fn set_value(&mut self, value: u8);

    /// Will get the value that the counter is currently at.
    fn get_value(&self) -> u8;
}

/// Represents a timer inside of the chip
/// infrastruture, it will count down to
/// zero from what ever number given in
/// the speck requireds 60Hz.
pub struct Timer {
    /// This is the main worker
    /// it is intended to be a part
    /// of the timer, but have no actuall
    /// implementation.
    _worker: Worker,
    /// will store the value of the timer
    value: Arc<AtomicU8>,
}

impl Timed for Timer {
    /// Will create a new timer with the given value.
    fn new(value: u8) -> Self {
        let counter = Arc::new(AtomicU8::new(value));
        // used to move into the callback
        let ccounter = counter.clone();
        let callback = move || {
            let val = ccounter.load(Ordering::Relaxed);
            if val > 0 {
                // make sure that there is no actuall
                // issue with the decrement
                // (this is acutally unneded as only this callback
                // will modify the counter, but there is not reason
                // not to use it)
                ccounter.compare_and_swap(val, val - 1, Ordering::SeqCst);
            }
        };

        let mut worker = Worker::new();
        worker.start(callback, Duration::from_millis(TIMER_INTERVAL));

        assert!(
            worker.is_alive(),
            "Something went wrong while initializing the worker thread!."
        );
        Self {
            _worker: worker,
            value: counter,
        }
    }

    /// Will set the value from which the timer shall count down from.
    fn set_value(&mut self, value: u8) {
        self.value.swap(value, Ordering::Release);
    }

    /// Will get the value that the counter is currently at.
    fn get_value(&self) -> u8 {
        self.value.load(Ordering::Relaxed)
    }
}

/// Is the internal worker, that exists on the
/// second thread.
struct Worker {
    /// Contains the actuall thread, that is running.
    thread: Option<JoinHandle<()>>,
    /// Contains the sync sender used to gracefull shutdown the thread.
    shutdown: Option<SyncSender<()>>,
    /// Counts the actuall threads used (this is never more then 2, but
    /// is simple to use.) It uses an ```()``` so that it doesn't use
    /// up too much memory.
    alive: Arc<()>,
}

pub trait Working {
    fn new() -> Self;
    fn start<T>(&mut self, callback: T, interval: Duration)
    where
        T: Send + FnMut() + 'static;
    fn stop(&mut self);
    fn is_alive(&self) -> bool;
}

impl Working for Worker {
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
    use crate::definitions::TIMER_HERZ;

    #[test]
    fn test_timer() {
        let mut timer = Timer::new(TIMER_HERZ);
        assert!(timer._worker.is_alive());

        std::thread::sleep(Duration::from_secs(1));
        assert_eq!(timer.get_value(), 0);

        timer._worker.stop();
        assert!(!timer._worker.is_alive());
    }
}
