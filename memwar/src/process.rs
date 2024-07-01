use sysinfo::System;
use winapi::shared::minwindef::DWORD;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::{
    HANDLE, PROCESS_SUSPEND_RESUME, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
};

#[allow(clippy::missing_safety_doc)]
pub unsafe fn open_process_handle(pid: u32) -> Result<HANDLE, DWORD> {
    let h_process = OpenProcess(
        PROCESS_SUSPEND_RESUME | PROCESS_VM_OPERATION | PROCESS_VM_WRITE | PROCESS_VM_READ,
        0,
        pid,
    );

    if h_process.is_null() {
        return Err(GetLastError());
    }
    Ok(h_process)
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn get_process_by_name(process_name: &str) -> Result<Option<u32>, DWORD> {
    let sys = System::new_all();
    let lower = process_name.to_lowercase();

    for (pid, process) in sys.processes() {
        if process.name().to_lowercase() == lower {
            return Ok(Some(pid.as_u32()));
        }
    }
    Ok(None)
}
