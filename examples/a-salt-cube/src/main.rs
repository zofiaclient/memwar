mod pointers;
mod tasks;

use std::sync::mpsc::TryRecvError;

use anyhow::{anyhow, bail, Result};
use cnsl::readln;
use memwar::mem::{CVoidPtr, SendAlloc};
use memwar::{module, process};

use crate::tasks::Tasks;

unsafe fn cli(tasks: Tasks) -> Result<()> {
    println!("Type help to get a list of commands");

    loop {
        let input = readln!("$ ");
        let trim = input.trim();

        if trim == "help" {
            println!("help\ntoggle_health");
            println!("health");
            println!(" \\ value: i32")
        }

        if trim == "toggle_health" {
            tasks.health_task().toggle_enabled();
        }

        if trim == "health" {
            println!("New health value:");

            let health = loop {
                let health_value = readln!("$ ");

                match health_value.parse() {
                    Ok(v) => break v,
                    Err(e) => eprintln!("{e}"),
                }
            };

            tasks.health_task().send_data(health)?;

            match tasks.health_task().read_error() {
                Ok(err) => {
                    eprintln!("Thread raised error {err}")
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => bail!("Thread disconnected! Aborting."),
            }
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
