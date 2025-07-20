//! Code for parsing ACPI tables

pub mod fixed;
pub mod header;

/// An ACPI address struct
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct AcpiAddress {
    /// Address space:
    /// * 0 - system memory
    /// * 1 - system I/O
    pub address_space_id: u8,
    /// Register bit width
    pub register_bit_width: u8,
    /// Register bit offset
    pub register_bit_offset: u8,
    /// Reserved
    _reserved: u8,
    /// Address
    pub address: u64,
}
