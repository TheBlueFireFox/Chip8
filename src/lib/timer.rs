use std::{ sync::{
        atomic::{AtomicU8,Ordering},
        mpsc::{self, RecvTimeoutError, SyncSender},
        Arc,
    }, thread::{self, JoinHandle}, time::Duration};

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
        let ccounter= counter.clone();
        let callback = move || {
            let val = ccounter.load(Ordering::Relaxed);
            if val > 0 {
                // make sure that there is no actuall 
                // issue with the decrement 
                // (this is acutally unneded as only this callback
                // will modify the counter, but there is not reason
                // not to use it)
                ccounter.compare_and_swap(val, val-1, Ordering::SeqCst);
            }
        };
        let mut worker = Worker::new();
        worker.start(callback, Duration::from_millis(TIMER_INTERVAL as u64));
        Self {
            _worker : worker,
            value: counter
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
}

impl Worker {
    fn new() -> Self {
        Self {
            thread: None, 
            shutdown: None
        }
    }

    fn start<T>(&mut self, mut callback: T, interval: Duration)
    where
        T: Send + FnMut() + 'static,
    {
        let (send, recv) = mpsc::sync_channel::<()>(1);

        let thread = thread::spawn(move || {
            let recv = recv;
            match recv.recv_timeout(interval) {
                Err(RecvTimeoutError::Timeout) => {
                    callback();
                }
                Ok(_) | Err(_) => {
                    // shutdown
                    return;
                }
            }
        });
        self.thread = Some(thread);
        self.shutdown = Some(send)
    }

    fn stop(&mut self) {
        if let Some(sender) = self.shutdown.take() {
            sender.send(()).expect("This thread should be running here, but is not... Investigate.");
        }
        if let Some(thread) = self.thread.take() {
            if let Err(err) = thread.join() {
                panic!(err);
            }
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {

}