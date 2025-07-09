//! EFI boot services header tag for indicating OS image can boot without terminating boot services.

use std::cursor::Cursor;

use crate::header::{flags::Flags, tag::Tag};

/// Informs multiboot2 that payload supports starting without terminating boot services.
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#EFI-boot-services-tag
pub struct EfiBootServices {
    /// Flags for tag
    pub flags: Flags,
}

impl const Tag for EfiBootServices {
    const TYPE: u16 = 7;

    fn write_to_buffer(&self, buffer: &mut Cursor) {
        buffer.write_u16(self.flags as u16);
        buffer.write_u32(8); // size = 8
    }
}
