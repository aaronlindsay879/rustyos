//! Page table entry

use std::elf::section_header::SectionHeader;

use bitflags::bitflags;

use crate::mem::frame::Frame;

bitflags! {
    /// Stores possible flags for a page entry
    #[derive(Clone, Copy)]
    pub struct EntryFlags: u64 {
        /// Whether page is present
        const PRESENT = 1 << 0;

        /// Whether page is writable
        const WRITABLE = 1 << 1;

        /// Whether page can be accessed by ring 3
        const USER_ACCESSIBLE = 1 << 2;

        /// Whether write-through caching is enabled
        const WRITE_THROUGH = 1 << 3;

        /// Whether caching is disabled
        const NO_CACHE = 1 << 4;

        /// Whether the page has been accessed
        const ACCESSED = 1 << 5;

        /// Whether the page has been written to
        const DIRTY = 1 << 6;

        /// Whether the page is huge
        const HUGE_PAGE = 1 << 7;

        /// Whether the page is always present
        const GLOBAL = 1 << 8;

        /// Whether execution from this page should be disabled
        const NO_EXECUTE = 1 << 63;
    }
}

impl EntryFlags {
    /// Set flags based on the flags used in ELF header
    pub fn from_elf_section_flags(section: &SectionHeader) -> Self {
        let mut flags = EntryFlags::NO_EXECUTE;

        if section.allocated() {
            flags.insert(EntryFlags::PRESENT);
        }
        if section.writable() {
            flags.insert(EntryFlags::WRITABLE);
        }
        if section.executable() {
            flags.remove(EntryFlags::NO_EXECUTE);
        }

        flags
    }
}

impl core::fmt::Display for EntryFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for string in self.iter_names().map(|(a, _)| a).intersperse(" | ") {
            f.write_str(string)?;
        }

        Ok(())
    }
}

/// An individual page table entry
pub struct Entry(u64);

impl Entry {
    /// Mask to extract address from entry
    const ADDRESS_MASK: usize = 0x000FFFFF_FFFFF000;

    /// Checks if PRESENT flag is set
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    /// Removes address and flags
    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    /// Sets the entry to the given frame and flags
    pub fn set(&mut self, frame: Frame, flags: EntryFlags) {
        // ensure address is page aligned and smaller than 2^52
        assert_eq!(frame.start_address() & !Self::ADDRESS_MASK, 0);

        self.0 = (frame.start_address() as u64) | flags.bits();
    }

    /// Returns the flags
    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    /// Returns the frame the entry points to, if it exists
    pub fn pointed_frame(&self) -> Option<Frame> {
        if self.flags().contains(EntryFlags::PRESENT) {
            Some(Frame::containing_address(
                self.0 as usize & Self::ADDRESS_MASK,
            ))
        } else {
            None
        }
    }
}
