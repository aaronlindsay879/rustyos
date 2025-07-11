//! Framebuffer header tag for requesting pixel-based framebuffer support

use std::cursor::Cursor;

use crate::header::{flags::Flags, header_tag::HeaderTag};

/// Requests a graphical framebuffer with the specified dimensions.
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Console-header-tags
pub struct Framebuffer {
    /// Flags for tag
    pub flags: Flags,
    /// Width of framebuffer in pixels, where 0 means no preference
    pub width: u32,
    /// Height of framebuffer in pixels, where 0 means no preference
    pub height: u32,
    /// Number of bits per pixel, where 0 means no preference
    pub depth: u32,
}

impl const HeaderTag for Framebuffer {
    const TYPE: u16 = 5;

    fn write_to_buffer(&self, buffer: &mut Cursor) {
        buffer.write_u16(self.flags as u16);
        buffer.write_u32(20); // size = 20

        buffer.write_u32(self.width);
        buffer.write_u32(self.height);
        buffer.write_u32(self.depth);
    }
}
