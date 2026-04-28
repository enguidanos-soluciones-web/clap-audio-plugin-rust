use crossbeam_queue::ArrayQueue;
use std::sync::Arc;

pub struct Sender<T>(Arc<ArrayQueue<T>>);
pub struct Receiver<T>(Arc<ArrayQueue<T>>);

pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let q = Arc::new(ArrayQueue::new(capacity));
    (Sender(Arc::clone(&q)), Receiver(q))
}

impl<T> Sender<T> {
    pub fn push(&self, val: T) -> Result<(), T> {
        self.0.push(val)
    }

    pub fn new_receiver(&self) -> Receiver<T> {
        Receiver(Arc::clone(&self.0))
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender(Arc::clone(&self.0))
    }
}

impl<T> Receiver<T> {
    pub fn pop(&self) -> Option<T> {
        self.0.pop()
    }

    pub fn new_sender(&self) -> Sender<T> {
        Sender(Arc::clone(&self.0))
    }
}
