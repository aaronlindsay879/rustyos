//! RSDP tags

use std::cursor::Cursor;

use crate::boot::boot_tag::BootTag;

/// Copy of RSDPv1 as defined per ACPI 1.0 or later
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#ACPI-new-RSDP
#[derive(Debug)]
pub struct RSDPv1 {
    /// OEM id
    pub oem_id: &'static str,
    /// ACPI revision
    pub revision: u8,
    /// Physical address of RSDT table
    pub rsdt_addr: u32,
}

impl BootTag for RSDPv1 {
    const TYPE: u32 = 14;

    fn read_from_buffer(buffer: &mut Cursor) -> Option<Self> {
        let _size = buffer.read_u32()?;

        read_rsdpv1(buffer)
    }
}

/// Copy of RSDPv2 as defined per ACPI 2.0 or later
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#ACPI-new-RSDP
#[derive(Debug)]
pub struct RSDPv2 {
    /// Address of XSDT data structure
    pub xsdt_addr: u64,
}

impl BootTag for RSDPv2 {
    const TYPE: u32 = 15;

    fn read_from_buffer(buffer: &mut Cursor) -> Option<Self> {
        let _size = buffer.read_u32()?;

        let v1_tag = read_rsdpv1(buffer)?;

        let length = buffer.read_u32()?;
        let xsdt_addr = buffer.read_u64()?;
        let mut checksum = buffer.read_u8()?;

        let _reserved = [buffer.read_u8()?, buffer.read_u8()?, buffer.read_u8()?];

        // checksum _should_ be calculated using all fields, including v1 fields
        // however, we check the v1 checksum in `read_rsdpv1` so we already know all v1 bytes sum to 0
        // therefore we can ignore those fields and just compute checksum again for v2, checking that all
        // bytes sum to 0

        // checksum + all other bytes should be 0 if valid
        for byte in length
            .to_ne_bytes()
            .iter()
            .chain(xsdt_addr.to_ne_bytes().iter())
        {
            checksum += byte;
        }

        if checksum != 0 {
            return None;
        }

        // TODO: emulate acpi 2.0+ properly so this can be tested

        Some(Self { xsdt_addr })
    }
}

/// Reads a RSDPv1 tag, useful since this otherwise would be duplicated in v1 and v2 code
fn read_rsdpv1(buffer: &mut Cursor) -> Option<RSDPv1> {
    let signature = unsafe { buffer.read_slice(8)? };
    if signature != b"RSD PTR " {
        panic!("incorrect signature!");
    }

    let mut checksum = buffer.read_u8()?;
    let oemid = unsafe { buffer.read_slice(6)? };
    let revision = buffer.read_u8()?;
    let rsdt_addr = buffer.read_u32()?;

    // checksum + all other bytes should be 0 if valid
    for byte in signature
        .iter()
        .chain(oemid)
        .chain(rsdt_addr.to_ne_bytes().iter())
    {
        checksum += byte;
    }
    checksum += revision;

    if checksum != 0 {
        return None;
    }

    Some(RSDPv1 {
        oem_id: unsafe { core::str::from_utf8_unchecked(oemid) },
        revision,
        rsdt_addr,
    })
}
