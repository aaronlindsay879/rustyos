//! Relocatable header tag for informing multiboot2 that the image can be relocated

use std::cursor::Cursor;

use crate::header::{flags::Flags, header_tag::HeaderTag};

/// Preference of where to load image within the allowable range.
#[repr(u32)]
#[derive(Copy, Clone)]
pub enum LocationPreference {
    /// No preference
    None,
    /// Lowest possible address in the allowed range
    LowestPossible,
    /// Highest possible address in the allowed range
    HighestPossible,
}

/// Informs multiboot2 that the image is relocatable. All addresses in this struct are physical addresses.
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Relocatable-header-tag
pub struct Relocatable {
    /// Flags for tag
    pub flags: Flags,
    /// Lowest possible address at which image should be loaded
    pub min_addr: u32,
    /// Highest possible address at which loaded image should end
    pub max_addr: u32,
    /// Image alignment in memory
    pub align: u32,
    /// Load address placement suggestion
    pub preference: LocationPreference,
}

impl const HeaderTag for Relocatable {
    const TYPE: u16 = 10;

    fn write_to_buffer(&self, buffer: &mut Cursor) {
        buffer.write_u16(self.flags as u16);
        buffer.write_u32(24); // size = 24

        buffer.write_u32(self.min_addr);
        buffer.write_u32(self.max_addr);
        buffer.write_u32(self.align);
        buffer.write_u32(self.preference as u32);
    }
}
