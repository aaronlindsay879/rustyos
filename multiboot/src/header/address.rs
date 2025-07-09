//! Address header tag for requesting specific memory layout

use std::cursor::Cursor;

use crate::header::{flags::Flags, tag::Tag};

/// Requests a specific memory layout. All addresses in this struct are physical addresses.
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Address-header-tag
pub struct Address {
    /// Flags for tag
    pub flags: Flags,
    /// Address where multiboot2 header is supposed to be loaded
    pub header_addr: u32,
    /// Beginning of text segment, must be <= header_addr
    pub load_addr: u32,
    /// End of text segment
    pub load_end_addr: u32,
    /// End of bss segment
    pub bss_end_addr: u32,
}

impl const Tag for Address {
    const TYPE: u16 = 2;

    fn write_to_buffer(&self, buffer: &mut Cursor) {
        buffer.write_u16(self.flags as u16);
        buffer.write_u32(24); // size = 24

        // then write each address
        buffer.write_u32(self.header_addr);
        buffer.write_u32(self.load_addr);
        buffer.write_u32(self.load_end_addr);
        buffer.write_u32(self.bss_end_addr);
    }
}
