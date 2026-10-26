use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, Sender};

/// Blocking multi-producer multi-consumer queue.
pub struct Queue<T> {
    send: Sender<T>,
    recv: Mutex<Receiver<T>>,
}

impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        let (send, recv) = mpsc::channel();

        Queue { send: send, recv: Mutex::new(recv) }
    }

    pub fn push(&self, item: T) {
        // Receiver lives as long as self, send cannot fail
        self.send.send(item).unwrap();
    }

    /// Pop an item, blocking until one is available.
    pub fn pop(&self) -> T {
        self.recv.lock().unwrap().recv().unwrap()
    }
}
