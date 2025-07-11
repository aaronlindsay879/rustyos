//! Provides functionality for reading and processing the returned multiboot2 information

use std::cursor::Cursor;

use crate::{
    boot::boot_tag::BootTag,
    prelude::{BasicMemInfo, BiosBootDevice, BootCommandLine, MemoryMap, RSDPv1, RSDPv2},
};

pub mod basic_mem_info;
pub mod bios_boot_device;
pub mod boot_command_line;
pub mod boot_tag;
pub mod mem_map;
pub mod rsdp;

/// Returned multiboot2 information
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Boot-information-format
#[derive(Default, Debug)]
pub struct BootInfo {
    /// Basic memory information
    pub basic_mem_info: Option<BasicMemInfo>,
    /// Device the OS image was loaded from
    pub bios_boot_device: Option<BiosBootDevice>,
    /// Command line that OS image was booted with
    pub boot_command_line: Option<BootCommandLine>,
    /// Memory map of system
    pub memory_map: Option<MemoryMap>,
    /// RSDPv1 tag
    pub rsdpv1: Option<RSDPv1>,
    /// RSDPv2 tag
    pub rsdpv2: Option<RSDPv2>,
}

impl BootInfo {
    /// Creates a new bootinfo struct from the given address
    ///
    /// # Safety
    /// This is **very** unsafe and must only ever be called with the address returned by multiboot2
    pub unsafe fn new(addr: *const u32) -> Option<Self> {
        let backing_slice = unsafe {
            let total_size = *addr;
            core::slice::from_raw_parts_mut(addr as *mut u8, total_size as usize)
        };
        let mut cursor = Cursor::from(backing_slice);

        let _total_size = cursor.read_u32()?;
        let _reserved = cursor.read_u32()?;

        let mut info = Self::default();

        while let Some(tag) = cursor.read_u32() {
            match tag {
                BasicMemInfo::TYPE => {
                    info.basic_mem_info = BasicMemInfo::read_from_buffer(&mut cursor);
                }
                BiosBootDevice::TYPE => {
                    info.bios_boot_device = BiosBootDevice::read_from_buffer(&mut cursor);
                }
                BootCommandLine::TYPE => {
                    info.boot_command_line = BootCommandLine::read_from_buffer(&mut cursor);
                }
                MemoryMap::TYPE => {
                    info.memory_map = MemoryMap::read_from_buffer(&mut cursor);
                }
                RSDPv1::TYPE => {
                    info.rsdpv1 = RSDPv1::read_from_buffer(&mut cursor);
                }
                RSDPv2::TYPE => {
                    info.rsdpv2 = RSDPv2::read_from_buffer(&mut cursor);
                }
                _ => {
                    // we don't know this tag, so read another byte for size and skip that many
                    if let Some(size) = cursor.read_u32() {
                        cursor.increment_offset(size as usize - 8);
                    }
                }
            }

            if cursor.offset() % 8 != 0 {
                cursor.align_offset(8);
            }
        }

        Some(info)
    }
}
