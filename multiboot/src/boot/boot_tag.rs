//! Base type for all boot info tags

use std::cursor::Cursor;

/// A boot info tag which can be read from a multiboot2 struct
pub trait BootTag
where
    Self: Sized,
{
    /// Numeric type of the tag
    const TYPE: u32;

    /// Reads the tag from a buffer
    fn read_from_buffer(buffer: &mut Cursor) -> Option<Self>;
}
