use memwar::{module, process};
use memwar::mem::Allocation;

pub struct GameManager {
    ac_client_mod_alloc: Allocation,
}

impl GameManager {
    pub const fn ac_client_mod_alloc(&self) -> &Allocation {
        &self.ac_client_mod_alloc
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

        Ok(Self {
            ac_client_mod_alloc,
        })
    }
}
