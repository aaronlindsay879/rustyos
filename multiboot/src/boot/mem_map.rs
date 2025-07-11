//! Memory map tag

use core::fmt::Formatter;
use std::cursor::Cursor;

use crate::boot::boot_tag::BootTag;

/// Memory map
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Memory-map
#[derive(Debug)]
pub struct MemoryMap {
    /// Size of each memory entry
    pub entry_size: u32,
    /// Version of entry type, should be 0
    pub entry_version: u32,
    /// Slice of memory map entries
    pub entries: &'static [MemoryMapEntry],
}

impl core::fmt::Display for MemoryMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "MemoryMap: entry size {}, entry version {}",
            self.entry_size, self.entry_version
        )?;

        for entry in self.entries {
            writeln!(f, "{entry}")?;
        }

        Ok(())
    }
}

/// Individual entry within the map, storing information about a single memory region
#[derive(Debug)]
#[repr(C)]
pub struct MemoryMapEntry {
    /// Starting physical address of region
    pub base_addr: u64,
    /// Size of memory region in bytes
    pub length: u64,
    /// Type of entry
    pub entry_type: MemoryEntryType,
    /// Reserved field, should be 0
    pub _reserved: u32,
}

impl core::fmt::Display for MemoryMapEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}: 0x{:016X}-0x{:016X} (len 0x{:X})",
            self.entry_type,
            self.base_addr,
            self.base_addr + self.length,
            self.length,
        )
    }
}

/// What type the memory region is
#[derive(Debug)]
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum MemoryEntryType {
    /// Usable ram
    RAM = 1,
    /// Reserved by system
    RESERVED,
    /// Usable but containing ACPI data
    ACPI,
    /// Must be preserved on hibernation
    PRESERVED_ON_HIBERNATION,
    /// Defective ram modules
    DEFECTIVE,
}

impl core::fmt::Display for MemoryEntryType {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use MemoryEntryType::*;
        write!(
            f,
            "{}",
            match self {
                RAM =>                      "                     RAM",
                RESERVED =>                 "                RESERVED",
                ACPI =>                     "                    ACPI",
                PRESERVED_ON_HIBERNATION => "PRESERVED_ON_HIBERNATION",
                DEFECTIVE =>                "               DEFECTIVE",
            }
        )
    }
}

impl BootTag for MemoryMap {
    const TYPE: u32 = 6;

    fn read_from_buffer(buffer: &mut Cursor) -> Option<Self> {
        let size = buffer.read_u32()?;

        // read entry metadata and make sure it matches up with what we expect
        let entry_size = buffer.read_u32()?;
        let entry_version = buffer.read_u32()?;

        if entry_size as usize != size_of::<MemoryMapEntry>() {
            panic!("entry size does not match expected memory map entry size");
        }

        // then read the correct amount of entries
        let num_entries = (size - 16) / entry_size;
        let entries = unsafe {
            core::slice::from_raw_parts(
                buffer.as_ptr() as *const MemoryMapEntry,
                num_entries as usize,
            )
        };

        // and advance cursor
        buffer.increment_offset(size as usize - 16);

        Some(Self {
            entry_size,
            entry_version,
            entries,
        })
    }
}
