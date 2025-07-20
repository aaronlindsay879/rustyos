//! High Precision Event Timer table

use std::cursor::CursorR;

use crate::tables::{AcpiAddress, header::Header};

/// High Precision Event Timer table
#[derive(Debug)]
pub struct Hpet {
    /// HPET header
    pub header: &'static Header,
    /// Block ID - contains information that _could_ be broken down into a struct,
    /// but tends to be inaccurate so we simply read the raw data and get what we need from registers
    pub block_id: u32,
    /// Address of first HPET block
    pub address: AcpiAddress,
    /// Number of current HPET block
    pub hpet_number: u8,
    /// Minimum clock tick in periodic mode
    pub minimum_clock_tick: u16,
    /// Page protection flags
    pub page_protection: u8,
}

impl Hpet {
    /// Signature of MADT: "APIC"
    pub const SIGNATURE: [u8; 4] = *b"HPET";

    /// Constructs a HPET table, assuming it is at the given address
    ///
    /// ## Safety
    /// `addr` must point to a valid HPET table.
    /// This function _does_ check it contains a HPET APIC signature, but only **after** already reading
    /// the header, so if the pointer is invalid then it will still be UB.
    pub unsafe fn from_addr(addr: usize) -> Option<Self> {
        unsafe {
            let (header, remaining) = Header::from_addr(addr)?;

            // even though we assume caller has checked signature, it doesn't hurt to double check
            if header.signature != Self::SIGNATURE {
                return None;
            }

            let mut cursor = CursorR::from(remaining);

            let block_id = cursor.read_u32()?;
            let address = {
                let address_space_id = cursor.read_u8()?;
                let register_bit_width = cursor.read_u8()?;
                let register_bit_offset = cursor.read_u8()?;
                let _reserved = cursor.read_u8()?;
                let address = cursor.read_u64()?;

                AcpiAddress {
                    address_space_id,
                    register_bit_width,
                    register_bit_offset,
                    _reserved,
                    address,
                }
            };
            let hpet_number = cursor.read_u8()?;
            let minimum_clock_tick = cursor.read_u16()?;
            let page_protection = cursor.read_u8()?;

            Some(Self {
                header,
                block_id,
                address,
                hpet_number,
                minimum_clock_tick,
                page_protection,
            })
        }
    }
}
