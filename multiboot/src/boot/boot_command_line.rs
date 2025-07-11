//! Boot command line tag

use core::ffi::CStr;
use std::cursor::Cursor;

use crate::boot::boot_tag::BootTag;

/// Boot command line
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Boot-command-line
#[derive(Debug)]
pub struct BootCommandLine {
    /// Command that OS image was booted with
    pub command: &'static CStr,
}

impl BootTag for BootCommandLine {
    const TYPE: u32 = 1;

    fn read_from_buffer(buffer: &mut Cursor) -> Option<Self> {
        let size = buffer.read_u32()?;

        // size is 8 bytes for tag + size fields, so any more past that is string length
        let str_len = size - 8;

        let command = unsafe {
            // safety: we know this is a boot command line tag, so we expect a cstr
            buffer.read_cstr(str_len as usize)?
        };

        Some(Self { command })
    }
}
