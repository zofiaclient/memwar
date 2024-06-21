use std::sync::mpsc::TryRecvError;

use anyhow::{anyhow, bail, Result};
use cnsl::readln;
use memwar::mem::{CVoidPtr, SendAlloc};
use memwar::tasks::Task;
use memwar::{module, process};

use crate::tasks::Tasks;

mod entity;
mod pointers;
mod tasks;

fn handle_thread_error<T>(task: &Task<T, (u32, usize)>) -> Result<()> {
    match task.read_error() {
        Ok(v) => eprintln!("Aimbot returned error {}, status {}", v.0, v.1),
        Err(TryRecvError::Empty) => (),
        Err(TryRecvError::Disconnected) => bail!("Aimbot thread disconnected! Aborting."),
    }
    Ok(())
}

unsafe fn cli(tasks: Tasks) -> Result<()> {
    println!("Type help to get a list of commands");

    loop {
        let input = readln!("$ ");
        let trim = input.trim();

        if trim == "help" {
            println!("help\ntoggle_aimbot");
        }

        if trim == "toggle_aimbot" {
            tasks.aimbot_task().toggle_enabled();
            handle_thread_error(tasks.aimbot_task())?;
        }
    }
}

unsafe fn run() -> Result<()> {
    let pid = process::get_process_by_name("ac_client.exe")
        .map_err(|e| anyhow!("Failed to get process information! OS error: {e}"))?
        .ok_or_else(|| anyhow!("Failed to find ac_client.exe!"))?;

    let h_process = process::open_process_handle(pid)
        .map_err(|e| anyhow!("Failed to open a handle to AssaultCube.exe! OS error: {e}"))?;

    let p_base = module::get_mod_base(pid, "ac_client.exe")
        .map_err(|e| anyhow!("Failed to create snapshot of process! OS error: {e}"))?;

    if p_base.is_null() {
        bail!("Failed to find ac_client.exe module!")
    }

    let alloc = SendAlloc::new(CVoidPtr(h_process), CVoidPtr(p_base));
    let tasks = Tasks::from_alloc(alloc);

    cli(tasks)
}

fn main() -> Result<()> {
    unsafe { run() }
}
