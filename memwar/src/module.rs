use std::ffi::{c_void, CStr, CString};
use std::mem;
use std::ptr::null_mut;

use winapi::um::handleapi::CloseHandle;
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Module32First, Module32Next, MODULEENTRY32, TH32CS_SNAPMODULE,
    TH32CS_SNAPMODULE32,
};

/// Returns the base address of the module under the given process.
#[allow(clippy::missing_safety_doc)]
pub unsafe fn get_mod_base(pid: u32, mod_name: &str) -> *mut c_void {
    let c_mod_name = CString::new(mod_name).expect("Could not create CString");

    let mut mod_base = null_mut();
    let h_snap = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid);

    if h_snap.is_null() {
        return null_mut();
    }

    let mut mod_entry: MODULEENTRY32 = mem::zeroed();
    mod_entry.dwSize = size_of_val(&mod_entry) as _;

    if Module32First(h_snap, &mut mod_entry) > 0 {
        loop {
            if CStr::from_ptr(mod_entry.szModule.as_ptr() as _) == c_mod_name.as_c_str() {
                mod_base = mod_entry.modBaseAddr as _;
            }

            if Module32Next(h_snap, &mut mod_entry) == 0 {
                break;
            }
        }
    }
    CloseHandle(h_snap);
    mod_base
}
