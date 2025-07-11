//! Module alignment header tag for requesting page aligned modules

use std::cursor::Cursor;

use crate::header::{flags::Flags, header_tag::HeaderTag};

/// Informs multiboot2 that modules must be paged aligned.
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Module-alignment-tag
pub struct ModuleAlignment {
    /// Flags for tag
    pub flags: Flags,
}

impl const HeaderTag for ModuleAlignment {
    const TYPE: u16 = 6;

    fn write_to_buffer(&self, buffer: &mut Cursor) {
        buffer.write_u16(self.flags as u16);
        buffer.write_u32(8); // size = 8
    }
}
