use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, SendError, Sender, TryRecvError};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use memwar::mem::{Allocation, SendAlloc};

use crate::pointers;

pub struct HealthTask {
    is_enabled: Arc<AtomicBool>,
    health_sender: Sender<u32>,
    error_receiver: Receiver<u32>,
}

impl HealthTask {
    pub fn set_health(&self, new_health: u32) -> Result<(), SendError<u32>> {
        self.health_sender.send(new_health)
    }
    
    pub fn toggle_enabled(&self) {
        self.is_enabled.store(!self.is_enabled.load(Ordering::Relaxed), Ordering::Relaxed)
    }
    
    pub fn read_error(&self) -> Result<Option<u32>, TryRecvError> {
        match self.error_receiver.try_recv() {
            Ok(error) => Ok(Some(error)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(TryRecvError::Disconnected)
        }
    }

    unsafe fn from_alloc(alloc: SendAlloc) -> Self {
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
                
                    let p_health = match alloc
                        .deref_chain(pointers::LOCAL_PLAYER, pointers::OFFS_LOCAL_PLAYER_HEALTH)
                    {
                        Ok(v) => v,
                        Err(e) => {
                            let _ = error_sender.send(e);
                            continue;
                        }
                    };
                    
                    if let Err(err) = alloc.write_u32(p_health, health) {
                        let _ = error_sender.send(err);
                    }
                }
            }
        });
        Self {
            is_enabled,
            health_sender,
            error_receiver,
        }
    }
}

pub struct Tasks {
    health_task: HealthTask,
}

impl Tasks {
    pub fn health_task(&self) -> &HealthTask {
        &self.health_task
    }

    pub unsafe fn from_alloc(alloc: SendAlloc) -> Self {
        Self {
            health_task: HealthTask::from_alloc(alloc),
        }
    }
}
