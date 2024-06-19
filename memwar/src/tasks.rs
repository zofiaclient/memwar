use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SendError, Sender, TryRecvError};
use std::sync::Arc;

pub struct Task<T, E> {
    data_sender: Sender<T>,
    is_enabled: Arc<AtomicBool>,
    err_receiver: Receiver<E>,
}
impl<T, E> Task<T, E> {
    pub fn read_error(&self) -> Result<E, TryRecvError> {
        self.err_receiver.try_recv()
    }

    pub fn set_enabled(&self, is_enabled: bool) {
        self.is_enabled.store(is_enabled, Ordering::Relaxed);
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled.load(Ordering::Relaxed)
    }

    pub fn toggle_enabled(&self) {
        self.set_enabled(!self.is_enabled());
    }

    pub fn send_data(&self, data: T) -> Result<(), SendError<T>> {
        self.data_sender.send(data)
    }

    pub const fn new(
        data_sender: Sender<T>,
        is_enabled: Arc<AtomicBool>,
        err_receiver: Receiver<E>,
    ) -> Self {
        Self {
            data_sender,
            is_enabled,
            err_receiver,
        }
    }
}
