use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use winapi::um::winuser::GetAsyncKeyState;

use memwar::mem::{Allocation, SendAlloc};
use memwar::tasks::Task;

use crate::entity::Entity;

unsafe fn new_aimbot_task(alloc: SendAlloc) -> Task<(), (u32, usize)> {
    let (sender, _) = mpsc::channel();
    let (error_sender, error_receiver) = mpsc::channel();

    let is_enabled = Arc::<AtomicBool>::default();
    let is_enabled_sent = is_enabled.clone();

    thread::spawn(move || {
        let alloc = Allocation::from(alloc);

        loop {
            if !is_enabled_sent.load(Ordering::Relaxed) {
                continue;
            }

            // F key.
            if GetAsyncKeyState(0x46) == 0 {
                continue;
            }

            let local_player = match Entity::read_local_player(&alloc) {
                Ok(v) => v,
                Err(e) => {
                    let _ = error_sender.send((e, 0));
                    continue;
                }
            };

            let entities = match Entity::from_list(&alloc) {
                Ok(v) => v,
                Err(e) => {
                    let _ = error_sender.send((e, 1));
                    continue;
                }
            };

            let mut entities: Vec<_> = entities
                .into_iter()
                .filter(|e| e.is_alive() && e.is_blue_team() != local_player.is_blue_team())
                .collect();

            entities.sort_by(|l, r| {
                local_player
                    .calc_distance(l)
                    .partial_cmp(&local_player.calc_distance(r))
                    .expect("Distances returned NAN!")
            });

            if let Some(entity) = entities.first() {
                if let Err(err) = local_player.aim_at(entity, &alloc) {
                    let _ = error_sender.send((err, 2));
                    continue;
                }
            }
        }
    });
    Task::new(sender, is_enabled, error_receiver)
}

pub struct Tasks {
    aimbot_task: Task<(), (u32, usize)>,
}

impl Tasks {
    pub unsafe fn from_alloc(alloc: SendAlloc) -> Self {
        Self {
            aimbot_task: new_aimbot_task(alloc),
        }
    }

    pub const fn aimbot_task(&self) -> &Task<(), (u32, usize)> {
        &self.aimbot_task
    }
}
