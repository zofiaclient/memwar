use std::ffi::c_void;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ptr::null_mut;

use winapi::shared::minwindef::DWORD;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualAllocEx, WriteProcessMemory};
use winapi::um::winnt::{HANDLE, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};

/// A wrapper struct, for cleaner calls to the Read/WriteProcessMemory API functions.
pub struct Allocation {
    h_process: HANDLE,
    base: *mut c_void,
}

impl Allocation {
    /// Reads the data into the given buffer.
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn read(&self, buf: *mut c_void, buf_size: usize) -> Result<usize, DWORD> {
        self.read_offset(0, buf, buf_size)
    }

    /// Reads the data at the allocation base plus the offset into the given buffer.
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn read_offset(
        &self,
        offset: isize,
        buf: *mut c_void,
        buf_size: usize,
    ) -> Result<usize, DWORD> {
        let mut read = 0;

        if ReadProcessMemory(
            self.h_process,
            self.base.offset(offset),
            buf,
            buf_size as _,
            &mut read,
        ) == 0
        {
            return Err(GetLastError());
        }
        Ok(read)
    }

    /// Fully writes the given data to this allocation in buffers of `buf_size`, else returns an
    /// [Err] containing the last OS error.
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn write_all_bytes_buffered(
        &self,
        data: &[u8],
        buf_size: usize,
    ) -> Result<(), DWORD> {
        self.write_all_bytes_buffered_offset(0, data, buf_size)
    }

    /// Fully writes the given data to this allocation, (offset by the `offset` parameter), in
    /// buffers of `buf_size`, else returns an [Err] containing the last OS error.
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn write_all_bytes_buffered_offset(
        &self,
        offset: isize,
        data: &[u8],
        buf_size: usize,
    ) -> Result<(), DWORD> {
        let mut buf: Vec<u8> = Vec::with_capacity(buf_size);
        let mut remaining = data.len();
        let mut total_written = 0;
        let mut written;

        while remaining > 0 {
            let real_remains = remaining.min(buf_size);

            buf.set_len(real_remains);
            buf.copy_from_slice(&data[total_written..total_written + real_remains]);

            written = self.write_offset(
                total_written as isize + offset,
                buf.as_ptr() as _,
                real_remains,
            )?;
            total_written += written;
            remaining -= written;
        }
        Ok(())
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn write(&self, data: *mut c_void, data_size: usize) -> Result<usize, DWORD> {
        self.write_offset(0, data, data_size)
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn write_offset(
        &self,
        offset: isize,
        data: *mut c_void,
        data_size: usize,
    ) -> Result<usize, DWORD> {
        let mut written = 0;

        if WriteProcessMemory(
            self.h_process,
            self.base.offset(offset),
            data,
            data_size,
            &mut written,
        ) == 0
        {
            return Err(GetLastError());
        }
        Ok(written)
    }

    pub const fn inner(&self) -> *mut c_void {
        self.base
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn alloc_remote_anywhere(h_process: HANDLE, size: usize) -> Result<Self, DWORD> {
        Self::alloc_remote(h_process, null_mut(), size)
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn alloc_remote(
        h_process: HANDLE,
        base_addr: *mut c_void,
        size: usize,
    ) -> Result<Self, DWORD> {
        let base = VirtualAllocEx(
            h_process,
            base_addr,
            size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        );

        if base.is_null() {
            return Err(GetLastError());
        }
        Ok(Self::existing(h_process, base))
    }

    pub const fn existing(h_process: HANDLE, base: *mut c_void) -> Self {
        Self { h_process, base }
    }
}

impl Debug for Allocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:02X}", self.base as usize)
    }
}
