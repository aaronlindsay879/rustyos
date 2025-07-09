//! Header end tag

use std::cursor::Cursor;

use crate::prelude::Tag;

/// Dummy tag, only used for ending header
pub struct DummyTag;

impl const Tag for DummyTag {
    const TYPE: u16 = 0;

    fn write_to_buffer(&self, buffer: &mut Cursor<'_>) {
        buffer.write_u16(0); // flags = 0
        buffer.write_u32(8); // size = 8
    }
}
