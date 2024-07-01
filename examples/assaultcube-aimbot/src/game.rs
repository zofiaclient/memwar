use memwar::mem::Allocation;
use memwar::tasks::ReceiverTask;
use memwar::{module, process};

use crate::entity::{Entity, LocalPlayer};
use crate::tasks;

pub struct GameData {
    local_player: LocalPlayer,
    entities: Vec<Entity>,
}

impl GameData {
    pub const fn local_player(&self) -> &LocalPlayer {
        &self.local_player
    }

    pub const fn entities(&self) -> &Vec<Entity> {
        &self.entities
    }

    pub unsafe fn read_from(ac_client_mod_alloc: &Allocation) -> Result<Self, String> {
        Ok(Self {
            local_player: LocalPlayer::read_from(ac_client_mod_alloc)?,
            entities: Entity::from_list(ac_client_mod_alloc)?,
        })
    }
}

pub struct GameManager {
    ac_client_mod_alloc: Allocation,
    game_data: Result<GameData, String>,
    aimbot_task: ReceiverTask<String, String>,
}

impl GameManager {
    pub const fn ac_client_mod_alloc(&self) -> &Allocation {
        &self.ac_client_mod_alloc
    }

    pub const fn game_data(&self) -> &Result<GameData, String> {
        &self.game_data
    }

    pub const fn aimbot_task(&self) -> &ReceiverTask<String, String> {
        &self.aimbot_task
    }

    pub unsafe fn setup() -> Result<Self, String> {
        let pid = process::get_process_by_name("ac_client.exe")
            .map_err(|e| format!("({e}) Failed to get process information!"))?
            .ok_or_else(|| "Failed to find ac_client.exe!".to_string())?;

        let h_process = process::open_process_handle(pid)
            .map_err(|e| format!("({e}) Failed to open a handle to AssaultCube.exe!"))?;

        let p_base = module::get_mod_base(pid, "ac_client.exe")
            .map_err(|e| format!("({e}) Failed to create snapshot of process!"))?;

        if p_base.is_null() {
            return Err("Failed to find ac_client.exe module!".to_string());
        }

        let ac_client_mod_alloc = Allocation::existing(h_process, p_base);
        let game_data = GameData::read_from(&ac_client_mod_alloc);

        Ok(Self {
            ac_client_mod_alloc,
            game_data,
            aimbot_task: tasks::new_aimbot_task(),
        })
    }
}
