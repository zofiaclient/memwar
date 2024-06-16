mod pointers;
mod tasks;

use anyhow::{anyhow, bail, Result};
use cnsl::readln;
use memwar::{module, process};
use memwar::mem::{CVoidPtr, SendAlloc};
use crate::tasks::Tasks;

unsafe fn cli(tasks: Tasks) -> Result<()> {
    println!("Type help to get a list of commands");
    
    loop {
        let input = readln!("$ ");
        let trim = input.trim();
        
        if trim == "help" {
            println!("help\ntoggle_health");
            println!("health");
            println!(" \\ value: u32")
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
            
            tasks.health_task().set_health(health)?;
            
            if let Some(err) = tasks.health_task().read_error()? {
                eprintln!("Thread raised error {err}")
            }
        }
    }
}

unsafe fn run() -> Result<()> {
    let (wpinf, _) = process::get_process_by_name("ac_client.exe")
        .map_err(|e| anyhow!("Failed to get window! OS error: {e}"))?
        .ok_or_else(|| anyhow!("Failed to find ac_client.exe!"))?;

    let h_process = process::open_process_handle(wpinf.pid())
        .map_err(|e| anyhow!("Failed to open a handle to AssaultCube.exe! OS error: {e}"))?;

    let p_base = module::get_mod_base(wpinf.pid(), "ac_client.exe");
    
    if p_base.is_null() {
        bail!("Could not get ac_client.exe module base address!")
    }
    
    let alloc = SendAlloc::new(CVoidPtr(h_process), CVoidPtr(p_base));
    let tasks = Tasks::from_alloc(alloc);
    
    cli(tasks)
}

fn main() -> Result<()> {
    unsafe { run() }
}
