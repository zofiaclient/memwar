use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use memwar::tasks::ReceiverTask;
use winapi::um::winuser::GetAsyncKeyState;

use crate::entity::Entity;
use crate::game::{GameData, GameManager};

fn aimbot(game_manager: &GameManager, data: &GameData) -> Result<String, String> {
    let local_player_entity = data.local_player().entity();

    let mut entities: Vec<&Entity> = data
        .entities()
        .iter()
        .filter(|e| e.health() > 0 && e.is_blue_team() != local_player_entity.is_blue_team())
        .collect();

    entities.sort_by(|l, r| {
        local_player_entity
            .calc_distance(l)
            .partial_cmp(&local_player_entity.calc_distance(r))
            .expect("Distance returned NAN!")
    });

    if let Some(entity) = entities.first() {
        unsafe {
            data.local_player()
                .aim_at(entity, game_manager.ac_client_mod_alloc())?;

            return Ok(entity.name_as_string());
        }
    }
    Err("Found no entities.".to_string())
}

pub fn new_aimbot_task() -> ReceiverTask<String, String> {
    let (sender, receiver) = mpsc::channel();
    let (err_sender, err_receiver) = mpsc::channel();

    let is_enabled = Arc::<AtomicBool>::default();
    let is_enabled_sent = is_enabled.clone();

    thread::spawn(move || unsafe {
        let mut game_manager = GameManager::setup();

        loop {
            thread::sleep(Duration::from_millis(50));

            if !is_enabled_sent.load(Ordering::Relaxed) {
                continue;
            }

            if GetAsyncKeyState(0x46) == 0 {
                continue;
            }

            let game_manager = match &game_manager {
                Ok(v) => v,
                Err(e) => {
                    let _ = err_sender.send(e.to_string());
                    thread::sleep(Duration::from_secs(1));

                    game_manager = GameManager::setup();
                    continue;
                }
            };

            let game_data = match GameData::read_from(game_manager.ac_client_mod_alloc()) {
                Ok(v) => v,
                Err(e) => {
                    let _ = err_sender.send(e);
                    continue;
                }
            };

            match aimbot(game_manager, &game_data) {
                Ok(entity_name) => {
                    if let Err(err) = sender.send(entity_name) {
                        let _ = err_sender.send(format!("Thread sender error: {err}"));
                    }
                }
                Err(err) => {
                    let _ = err_sender.send(err);
                }
            }
        }
    });
    ReceiverTask::new(receiver, is_enabled, err_receiver)
}
