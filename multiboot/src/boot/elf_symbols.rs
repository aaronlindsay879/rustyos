//! ELF symbols

use std::{cursor::Cursor, elf::section_header::SectionHeader};

use crate::boot::boot_tag::BootTag;

/// ELF symbols for loaded OS image
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#ELF_002dSymbols
#[derive(Debug)]
pub struct ElfSymbols {
    ///  Number of entries
    pub entry_count: u32,
    /// Size of each entry
    pub entry_size: u32,
    /// Which index is the string table
    pub string_table_index: u32,
    /// Section headers
    pub section_headers: &'static [SectionHeader],
}

impl ElfSymbols {
    /// Returns the header for the string table
    pub const fn string_header(&self) -> &SectionHeader {
        &self.section_headers[self.string_table_index as usize]
    }
}

impl BootTag for ElfSymbols {
    const TYPE: u32 = 9;

    fn read_from_buffer(buffer: &mut Cursor) -> Option<Self> {
        let _size = buffer.read_u32()?;

        let entry_count = buffer.read_u32()?;
        let entry_size = buffer.read_u32()?;

        let string_table_index = buffer.read_u32()?;
        let section_headers = unsafe {
            let bytes = buffer.read_slice(entry_count as usize * entry_size as usize)?;

            core::slice::from_raw_parts(
                bytes.as_ptr() as *const SectionHeader,
                entry_count as usize,
            )
        };

        Some(Self {
            entry_count,
            entry_size,
            string_table_index,
            section_headers,
        })
    }
}
