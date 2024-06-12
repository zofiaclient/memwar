use std::borrow::Cow;
use std::ffi::c_void;
use std::mem;

use derive_more::{Display, Error};
use winapi::um::winnt::{
    IMAGE_DOS_SIGNATURE, IMAGE_NT_HEADERS, IMAGE_NT_SIGNATURE, PIMAGE_DOS_HEADER,
    PIMAGE_NT_HEADERS, PIMAGE_SECTION_HEADER,
};

#[derive(Display, Debug, Error)]
pub enum InvalidDllReason {
    #[display("Invalid DOS header, is this payload a DLL?")]
    DosHeader,

    #[display("Invalid NT headers, does this payload contain executable code?")]
    NtHeaders,

    #[display("Invalid logical file address! (e_lfanew)")]
    LogicalFileAddress,
}

#[derive(Debug)]
pub struct Section<'a> {
    virtual_address: u32,
    raw_data_size: usize,
    ptr_to_raw_data: *mut u8,
    name: Cow<'a, str>,
}

impl<'a> Section<'a> {
    pub const fn new(
        virtual_address: u32,
        raw_data_size: usize,
        ptr_to_raw_data: *mut u8,
        name: Cow<'a, str>,
    ) -> Self {
        Self {
            virtual_address,
            raw_data_size,
            ptr_to_raw_data,
            name,
        }
    }

    pub const fn virtual_address(&self) -> u32 {
        self.virtual_address
    }

    pub const fn raw_data_size(&self) -> usize {
        self.raw_data_size
    }

    pub const fn ptr_to_raw_data(&self) -> *mut u8 {
        self.ptr_to_raw_data
    }

    pub const fn name(&self) -> &Cow<'a, str> {
        &self.name
    }
}

pub struct Dll<'a> {
    p_nt_headers: PIMAGE_NT_HEADERS,
    sections: Vec<Section<'a>>,
}

impl<'a> Dll<'a> {
    pub unsafe fn nt_headers(&self) -> IMAGE_NT_HEADERS {
        *self.p_nt_headers
    }

    pub const fn p_nt_headers(&self) -> PIMAGE_NT_HEADERS {
        self.p_nt_headers
    }

    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    pub unsafe fn try_parse(data: &[u8]) -> Result<Self, InvalidDllReason> {
        let (_, p_nt_headers) = get_headers(data)?;
        let sections = get_sections(p_nt_headers);
        Ok(Self::new(p_nt_headers, sections))
    }

    pub const fn new(p_nt_headers: PIMAGE_NT_HEADERS, sections: Vec<Section<'a>>) -> Self {
        Self {
            p_nt_headers,
            sections,
        }
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn image_first_section(nt_headers: PIMAGE_NT_HEADERS) -> PIMAGE_SECTION_HEADER {
    (nt_headers as *mut c_void)
        .add(mem::offset_of!(IMAGE_NT_HEADERS, OptionalHeader))
        .add((*nt_headers).FileHeader.SizeOfOptionalHeader as usize) as _
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn get_sections<'a>(p_nt_headers: PIMAGE_NT_HEADERS) -> Vec<Section<'a>> {
    let mut sections = Vec::new();
    let mut p_section_header = image_first_section(p_nt_headers);

    for _ in (0..(*p_nt_headers).FileHeader.NumberOfSections).rev() {
        let section = Section::new(
            (*p_section_header).VirtualAddress,
            (*p_section_header).SizeOfRawData as _,
            (*p_section_header).PointerToRawData as *mut u8,
            String::from_utf8_lossy(&(*p_section_header).Name),
        );
        sections.push(section);
        p_section_header = p_section_header.add(1);
    }
    sections
}

/// Reads the given DLL's DOS and NT headers.
#[allow(clippy::missing_safety_doc)]
pub unsafe fn get_headers(
    dll_data: &[u8],
) -> Result<(PIMAGE_DOS_HEADER, PIMAGE_NT_HEADERS), InvalidDllReason> {
    let p_dos_header: PIMAGE_DOS_HEADER = dll_data.as_ptr() as _;

    if (*p_dos_header).e_magic != IMAGE_DOS_SIGNATURE {
        return Err(InvalidDllReason::DosHeader);
    }

    if (*p_dos_header).e_lfanew == 0 {
        return Err(InvalidDllReason::LogicalFileAddress);
    }

    let p_nt_header: PIMAGE_NT_HEADERS =
        dll_data.as_ptr().add((*p_dos_header).e_lfanew as usize) as _;

    if (*p_nt_header).Signature != IMAGE_NT_SIGNATURE {
        return Err(InvalidDllReason::NtHeaders);
    }
    Ok((p_dos_header, p_nt_header))
}
