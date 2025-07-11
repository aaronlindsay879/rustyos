//! Base type for all header tags

use std::cursor::Cursor;

/// A header tag which can be written into the multiboot2 header
#[const_trait]
pub trait HeaderTag {
    /// Numeric type of the tag
    const TYPE: u16;

    /// Writes the tag into a buffer, without caring about alignment or padding tags.
    fn write_to_buffer(&self, buffer: &mut Cursor);

    /// Writes the tag to an output slice, respecting
    /// [required multiboot2 alignment](https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Header-tags).
    fn write_tag(&self, out: &mut Cursor) {
        // write tag to output buffer
        out.write_u16(Self::TYPE);
        self.write_to_buffer(out);

        // align up to next multiple of 8 bytes
        let position = out.offset();
        let mut padding_bytes_required = ((position + 7) & !7) - position;

        while padding_bytes_required > 0 {
            out.write_u8(0);
            padding_bytes_required -= 1;
        }
    }
}
