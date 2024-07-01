use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use memwar::mem::{Allocation, SendAlloc};
use memwar::tasks::SenderTask;

use crate::pointers;

unsafe fn new_health_task(alloc: SendAlloc) -> SenderTask<i32, u32> {
    let (health_sender, health_receiver) = mpsc::channel();
    let (error_sender, error_receiver) = mpsc::channel();

    let is_enabled = Arc::<AtomicBool>::default();
    let is_enabled_sent = is_enabled.clone();

    thread::spawn(move || {
        let mut health = None;

        loop {
            // Try and read an updated modified health value from the CLI thread.
            match health_receiver.recv_timeout(Duration::from_millis(100)) {
                // The CLI thread has sent an updated health value.
                Ok(v) => health = Some(v),
                // The CLI thread has not sent an updated health value, so we will continue to
                // use the value previously stored.
                Err(RecvTimeoutError::Timeout) => (),

                // The CLI thread has disconnected, so we exit the thread.
                Err(RecvTimeoutError::Disconnected) => return,
            };

            // If the cheat is not enabled, continue the loop.
            if !is_enabled_sent.load(Ordering::Relaxed) {
                continue;
            }

            if let Some(health) = health {
                let alloc = Allocation::from(alloc);

                let p_health = match alloc.deref_chain_with_base(
                    pointers::LOCAL_PLAYER as _,
                    pointers::OFFS_LOCAL_PLAYER_HEALTH,
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        let _ = error_sender.send(e);
                        continue;
                    }
                };

                if let Err(err) = alloc.write_i32(p_health, health) {
                    let _ = error_sender.send(err);
                }
            }
        }
    });
    SenderTask::new(health_sender, is_enabled, error_receiver)
}

pub struct Tasks {
    health_task: SenderTask<i32, u32>,
}

impl Tasks {
    pub fn health_task(&self) -> &SenderTask<i32, u32> {
        &self.health_task
    }

    pub unsafe fn from_alloc(alloc: SendAlloc) -> Self {
        Self {
            health_task: new_health_task(alloc),
        }
    }
}
