//! Console header tag for requesting specific console options

use std::cursor::Cursor;

use crate::header::{flags::Flags, header_tag::HeaderTag};

/// Specifies information about the requested and support consoles.
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Console-header-tags
pub struct ConsoleFlags {
    /// Flags for tag
    pub flags: Flags,
    /// If true, then at least one supported console must be present
    pub must_be_present: bool,
    /// If true, then OS image has support for EGA text
    pub ega_text_support: bool,
}

impl const HeaderTag for ConsoleFlags {
    const TYPE: u16 = 4;

    fn write_to_buffer(&self, buffer: &mut Cursor) {
        buffer.write_u16(self.flags as u16);
        buffer.write_u32(12); // size = 12

        let mut console_flags = 0;

        if self.must_be_present {
            console_flags |= 0b01;
        }
        if self.ega_text_support {
            console_flags |= 0b10;
        }

        buffer.write_u16(console_flags);
    }
}
