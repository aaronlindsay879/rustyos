//! Basic memory information tag

use std::cursor::Cursor;

use crate::boot::boot_tag::BootTag;

/// Basic memory information
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Basic-memory-information
#[derive(Debug)]
pub struct BasicMemInfo {
    /// Amount of lower memory in kb
    pub mem_lower: u32,
    /// Amount of upper memory in kb
    pub mem_upper: u32,
}

impl BootTag for BasicMemInfo {
    const TYPE: u32 = 4;

    fn read_from_buffer(buffer: &mut Cursor) -> Option<Self> {
        let _size = buffer.read_u32()?;

        let mem_lower = buffer.read_u32()?;
        let mem_upper = buffer.read_u32()?;

        Some(Self {
            mem_lower,
            mem_upper,
        })
    }
}
