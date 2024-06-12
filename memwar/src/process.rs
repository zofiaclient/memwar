use std::mem::MaybeUninit;

use sysinfo::System;
use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::{BOOL, LPARAM};
use winapi::shared::windef::HWND;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::{
    HANDLE, PROCESS_SUSPEND_RESUME, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
};
use winapi::um::winuser::{EnumWindows, GetWindowThreadProcessId};

pub struct WindowedProcessInformation {
    pid: u32,
    this_win_tid: u32,
    hwnd: HWND,
}

impl WindowedProcessInformation {
    pub const fn new(pid: u32, this_win_tid: u32, hwnd: HWND) -> Self {
        Self {
            pid,
            this_win_tid,
            hwnd,
        }
    }

    pub const fn pid(&self) -> u32 {
        self.pid
    }

    pub const fn this_win_tid(&self) -> u32 {
        self.this_win_tid
    }

    pub const fn hwnd(&self) -> HWND {
        self.hwnd
    }
}

struct WindowEnumData {
    pid: u32,
    wpinf: MaybeUninit<WindowedProcessInformation>,
}

impl WindowEnumData {
    const fn new(pid: u32) -> Self {
        Self {
            pid,
            wpinf: MaybeUninit::uninit(),
        }
    }
}

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

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let enum_data = &mut *(lparam as *mut WindowEnumData);

    let mut pid = 0;
    let tid = GetWindowThreadProcessId(hwnd, &mut pid);

    if enum_data.pid == pid {
        enum_data
            .wpinf
            .write(WindowedProcessInformation::new(pid, tid, hwnd));
        return 0;
    }
    1
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn find_wpinf_from_pid(pid: u32) -> Result<WindowedProcessInformation, DWORD> {
    let mut enum_data = WindowEnumData::new(pid);

    if EnumWindows(
        Some(enum_windows_callback),
        &mut enum_data as *mut WindowEnumData as isize,
    ) == 0
    {
        return Ok(enum_data.wpinf.assume_init());
    }
    Err(GetLastError())
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn get_process_by_name(
    process_name: &str,
) -> Result<Option<(WindowedProcessInformation, String)>, DWORD> {
    let sys = System::new_all();
    let lower = process_name.to_lowercase();

    for (pid, process) in sys.processes() {
        if process.name().to_lowercase() == lower {
            return Ok(Some((
                find_wpinf_from_pid(pid.as_u32())?,
                process.name().to_string(),
            )));
        }
    }
    Ok(None)
}
