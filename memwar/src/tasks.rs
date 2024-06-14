use std::sync::mpsc::{RecvTimeoutError, SendError, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::mem::{Allocation, CVoidPtr, SendAlloc};

/// Settings related to the execution of update logic.
#[derive(Clone, Debug)]
pub struct UpdateSettings {
    unit_length_ms: usize,
    updates_per_unit: usize,
}

impl UpdateSettings {
    /// Returns the unit length divided by the amount of updates per unit.
    pub fn to_duration(&self) -> Duration {
        Duration::from_millis((self.unit_length_ms / self.updates_per_unit) as u64)
    }

    /// Returns the length of each unit in milliseconds.
    pub const fn unit_length_ms(&self) -> usize {
        self.unit_length_ms
    }

    /// Returns how many times the update logic shall be run per unit.
    pub const fn updates_per_unit(&self) -> usize {
        self.updates_per_unit
    }

    pub const fn new(unit_length_ms: usize, updates_per_unit: usize) -> Self {
        Self {
            unit_length_ms,
            updates_per_unit,
        }
    }
}

impl Default for UpdateSettings {
    /// Constructs an [UpdateSettings] that specifies to update 60 times every second.
    fn default() -> Self {
        Self::new(1000, 60)
    }
}

pub struct WriteTask {
    send: Sender<bool>,
    is_enabled: bool,
}

impl WriteTask {
    pub fn set_enabled(&mut self, enabled: bool) -> Result<(), SendError<bool>> {
        self.is_enabled = enabled;
        self.send.send(enabled)
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn spawn(
        alloc: SendAlloc,
        dest_ptr: Arc<Mutex<CVoidPtr>>,
        data: Arc<Mutex<CVoidPtr>>,
        data_size: usize,
        update_settings: UpdateSettings,
    ) -> Result<Self, u32>
    {
        let (send, recv) = mpsc::channel();

        thread::spawn(move || {
            let mut is_enabled = false;
            let alloc: Allocation = alloc.into();

            loop {
                match recv.recv_timeout(update_settings.to_duration()) {
                    Ok(should_enable) => is_enabled = should_enable,
                    Err(RecvTimeoutError::Timeout) => (),
                    _ => return,
                }

                if is_enabled
                    && alloc
                        .write(
                            dest_ptr.lock().unwrap().0,
                            data.lock().unwrap().0,
                            data_size,
                        )
                        .is_err()
                {
                    return;
                }
            }
        });
        Ok(Self {
            send,
            is_enabled: false,
        })
    }
}
