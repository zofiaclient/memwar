use std::ffi::{c_void, CStr, CString};
use std::mem;
use std::ptr::null_mut;

use winapi::shared::minwindef::DWORD;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Module32First, Module32Next, MODULEENTRY32, TH32CS_SNAPMODULE,
    TH32CS_SNAPMODULE32,
};

/// Returns a list of modules in the given process.
#[allow(clippy::missing_safety_doc)]
pub unsafe fn get_modules(pid: u32) -> Result<Vec<MODULEENTRY32>, DWORD> {
    let mut modules = Vec::new();
    let h_snap = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid);

    if h_snap == INVALID_HANDLE_VALUE {
        return Err(GetLastError());
    }

    let mut mod_entry: MODULEENTRY32 = mem::zeroed();
    mod_entry.dwSize = size_of_val(&mod_entry) as _;

    if Module32First(h_snap, &mut mod_entry) > 0 {
        loop {
            modules.push(mod_entry);

            if Module32Next(h_snap, &mut mod_entry) == 0 {
                break;
            }
        }
    }
    CloseHandle(h_snap);
    Ok(modules)
}

/// Returns the base address of the module with the given name in the process.
///
/// This function will return Ok([null_mut]) if the module with the name provided could not be
/// found.
#[allow(clippy::missing_safety_doc)]
pub unsafe fn get_mod_base(pid: u32, mod_name: &str) -> Result<*mut c_void, DWORD> {
    let c_mod_name = CString::new(mod_name).expect("Could not create CString");

    for module in get_modules(pid)? {
        if CStr::from_ptr(module.szModule.as_ptr()) == c_mod_name.as_c_str() {
            return Ok(module.modBaseAddr as _);
        }
    }
    Ok(null_mut())
}
