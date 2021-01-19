use std::{sync::{Arc, atomic::{AtomicU8, Ordering}, mpsc::{self, RecvTimeoutError, SyncSender}}, thread::{self, JoinHandle}, time::Duration};

use crate::definitions::TIMER_INTERVAL;

pub struct Timer {
    /// This is the main worker
    /// it is intended to be a part
    /// of the timer, but have no actuall
    /// implementation.
    _worker: Worker,
    /// will store the value of the timer
    value: Arc<AtomicU8>,
}

impl Timer {
    pub fn new(value: u8) -> Self {
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

        assert!(worker.is_alive(), "Something went wrong while initializing the worker thread!.");
        Self {
            _worker: worker,
            value: counter,
        }
    }

    pub fn set_value(&self, value: u8) {
        self.value.swap(value, Ordering::Release);
    }

    pub fn get_value(&self) -> u8 {
        self.value.load(Ordering::Relaxed)
    }
}

struct Worker {
    thread: Option<JoinHandle<()>>,
    shutdown: Option<SyncSender<()>>,
    alive: Arc<()>
}

impl Worker {
    fn new() -> Self {
        Self {
            thread: None,
            shutdown: None,
            alive: Arc::new(())
        }
    }

    fn start<T>(&mut self, mut callback: T, interval: Duration)
    where
        T: Send + FnMut() + 'static,
    {
        let (send, recv) = mpsc::sync_channel::<()>(1);
        let alive = self.alive.clone();
        let thread = thread::spawn(move || {
            // this is to count the references
            let _alive = alive;
            loop {
                match recv.recv_timeout(interval) {
                    Err(RecvTimeoutError::Timeout) => {
                        callback();
                    }
                    Ok(_) | Err(_) => break, // shutdown
                }
            }
        });

        self.thread = Some(thread);
        self.shutdown = Some(send)
    }

    fn stop(&mut self) {
        if let Some(sender) = self.shutdown.take() {
            sender
                .send(())
                .expect("This thread should be running here, but is not... Investigate.");
        }
        if let Some(thread) = self.thread.take() {
            if let Err(err) = thread.join() {
                panic!(err);
            }
        }
    }

    fn is_alive(&self) -> bool {
        Arc::strong_count(&self.alive) > 1
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use crate::definitions::TIMER_HERZ;
    use super::*;

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
