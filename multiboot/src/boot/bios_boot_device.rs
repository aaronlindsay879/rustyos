//! BIOS boot device tag

use std::cursor::Cursor;

use crate::boot::boot_tag::BootTag;

/// BIOS boot device
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#BIOS-Boot-device
#[derive(Debug)]
pub struct BiosBootDevice {
    /// BIOS drive number as understood by the BIOS INT 0x13 low-level disk interface
    pub biosdev: u32,
    /// Top-level partition number
    pub partition: u32,
    /// Sub-partition within the top-level partition
    pub sub_partition: u32,
}

impl BootTag for BiosBootDevice {
    const TYPE: u32 = 5;

    fn read_from_buffer(buffer: &mut Cursor) -> Option<Self> {
        let _size = buffer.read_u32()?;

        let biosdev = buffer.read_u32()?;
        let partition = buffer.read_u32()?;
        let sub_partition = buffer.read_u32()?;

        Some(Self {
            biosdev,
            partition,
            sub_partition,
        })
    }
}
