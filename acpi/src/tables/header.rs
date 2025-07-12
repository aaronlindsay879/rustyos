//! Shared header for all tables

use std::cursor::Cursor;

/// ACPI table header
#[derive(Debug)]
#[repr(C, packed)]
pub struct Header {
    /// 4-byte signature to describe table
    pub signature: [u8; 4],
    /// Length of table, including header
    pub length: u32,
    /// Revision of table
    pub revision: u8,
    /// Header checksum field, includes all bytes of table including header
    pub checksum: u8,
    /// OEM-supplied string
    pub oem_id: [u8; 6],
    /// OEM-supplied string for the specific table
    pub oem_table_id: [u8; 8],
    /// OEM-supplied revision number
    pub oem_revision: u32,
    /// Vendor ID of utility that created table
    pub creator_id: u32,
    /// Vendor revision of utility that created table
    pub creator_revision: u32,
}

impl Header {
    /// Constructs a header from the given cursor, **without** any checks
    ///
    /// ## Safety
    /// The caller **must** ensure there is a valid ACPI header at the current position within the cursor
    pub unsafe fn from_bytes(cursor: &mut Cursor) -> Option<(&'static Self, &'static [u8])> {
        unsafe {
            let buffer = cursor.read_slice(size_of::<Self>())?;
            let header = &*(buffer.as_ptr() as *const Self);

            let remaining = header.length as usize - size_of::<Self>();
            let remaining_slice = { core::slice::from_raw_parts(cursor.as_ptr(), remaining) };

            Some((header, remaining_slice))
        }
    }

    /// Constructs a header from the memory at the given address, **without** any checks.
    /// Returns the header and the slice of remaining data within the table
    ///
    /// ## Safety
    /// The caller **must** ensure there is a valid ACPI header at the current position within the cursor
    pub unsafe fn from_addr(addr: usize) -> Option<(&'static Self, &'static [u8])> {
        unsafe {
            let slice = core::slice::from_raw_parts_mut(addr as *mut u8, size_of::<Self>());
            let mut cursor = Cursor::from_mut(slice);

            Self::from_bytes(&mut cursor)
        }
    }

    /// Returns `self.signature` as a string
    pub const fn signature(&self) -> &str {
        // safety: we assume that self was constructed from an actual header, in which case self.signature is a valid
        // utf-8 string with length 4
        unsafe { core::str::from_utf8_unchecked(&self.signature) }
    }

    /// Returns `self.oem_id` as a string
    pub const fn oem_id(&self) -> &str {
        // safety: we assume that self was constructed from an actual header, in which case self.oem_id is a valid
        // utf-8 string with length 6
        unsafe { core::str::from_utf8_unchecked(&self.oem_id) }
    }

    /// Returns `self.oem_table_id` as a string
    pub const fn oem_table_id(&self) -> &str {
        // safety: we assume that self was constructed from an actual header, in which case self.oem_table_id is a valid
        // utf-8 string with length 8
        unsafe { core::str::from_utf8_unchecked(&self.oem_table_id) }
    }
}

/// Returns the ACPI table signature at the given address
///
/// ## Safety
/// This is essentially a raw pointer dereference, and the caller must guarantee that
/// `addr` is valid to read for 4 bytes.
pub const unsafe fn signature_at_addr(addr: usize) -> [u8; 4] {
    unsafe { *(addr as *const [u8; 4]) }
}
