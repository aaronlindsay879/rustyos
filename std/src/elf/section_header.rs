//! Information about a section within an ELF file

use core::ffi::CStr;

/// A header for an individual ELF section
#[derive(Debug)]
#[repr(C)]
pub struct SectionHeader {
    /// Offset in bytes to the section name in string table
    pub section_name: u32,
    /// Section type
    pub section_type: SectionType,
    /// Section flags
    pub flags: u64,
    /// Virtual address of the beginning of section, 0 if should not be allocated
    pub addr: u64,
    /// Offset in bytes of the beginning of section contents within file
    pub offset: u64,
    /// Size in bytes of the section
    pub size: u64,
    /// Section index of associated section
    pub link: u32,
    /// Extra info about the section
    pub info: u32,
    /// Required alignment of the section
    pub align: u64,
    /// Size in bytes of each entry for sections that have a table structure
    pub entry_size: u64,
}

impl SectionHeader {
    /// Returns the name of the header using the provided string table
    pub fn name(&self, string_header: &Self) -> &'static CStr {
        let location = string_header.addr as *const i8;

        unsafe { CStr::from_ptr(location.add(self.section_name as usize)) }
    }

    /// Whether section is allocated
    pub fn allocated(&self) -> bool {
        self.flags & 0x2 != 0
    }
}

/// Type of the section
#[repr(u32)]
#[derive(Debug)]
pub enum SectionType {
    /// Unused section header
    Null = 0,
    /// Information defined by program
    Progbits,
    /// Linker symbol table
    Symtab,
    /// String table
    Strtab,
    /// "Rela" type relocation table
    Rela,
    /// Symbol hash table
    Hash,
    /// Dynamic linking table
    Dynamic,
    /// Note information
    Note,
    /// Uninitialized space, occupies no space within file
    Nobits,
    /// "Rel" type relocation entries
    Rel,
    /// Reserved
    Shlib,
    /// Dynamic loader symbol table
    Dynsym,
    /// Environment-specific use
    LoOs = 0x60000000,
    /// Environment-specific use
    HiOs = 0x6FFFFFFF,
    /// Processor-specific use
    LoProc = 0x70000000,
    /// Processor-specific use
    HiProc = 0x7FFFFFFF,
}
