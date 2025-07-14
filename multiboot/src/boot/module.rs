//! Module tag

use core::ffi::CStr;
use std::cursor::Cursor;

use crate::boot::boot_tag::BootTag;

/// Information about a single module
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Modules
#[derive(Debug)]
pub struct Module {
    /// Physical address of module in memory
    pub module_addr: u32,
    /// Length of module in bytes
    pub module_len: u32,
    /// Name of module
    pub module_str: &'static CStr,
}

impl BootTag for Module {
    const TYPE: u32 = 3;

    fn read_from_buffer(buffer: &mut Cursor) -> Option<Self> {
        let size = buffer.read_u32()?;

        let module_addr = buffer.read_u32()?;
        let module_end = buffer.read_u32()?;
        let module_len = module_end - module_addr;

        let str_len = size - 16;

        let module_str = unsafe { buffer.read_cstr(str_len as usize)? };

        Some(Self {
            module_addr,
            module_len,
            module_str,
        })
    }
}
