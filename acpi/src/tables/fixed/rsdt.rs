//! Root System Description Table

use core::marker::PhantomData;

use crate::tables::header::Header;

/// Root System Description Table
#[derive(Debug)]
pub struct Rsdt<PTR> {
    /// RSDT header
    pub header: &'static Header,
    /// Number of addresses
    pub num_addresses: usize,
    /// List of addresses of other tables
    pub tables: *const u8,
    /// Phantom type info
    _phantom: PhantomData<PTR>,
}

impl<PTR> Rsdt<PTR>
where
    PTR: TryInto<usize> + Copy,
{
    /// Constructs the RSDT table from the memory at the given address, **without** any checks.
    ///
    /// ## Safety
    /// The caller **must** ensure there is a valid RSDT table at the current position within the cursor
    pub unsafe fn from_addr(addr: usize) -> Option<Self> {
        unsafe {
            let (header, remaining) = Header::from_addr(addr)?;
            let num_addresses = remaining.len() / size_of::<PTR>();

            Some(Self {
                header,
                num_addresses,
                tables: remaining.as_ptr(),
                _phantom: PhantomData,
            })
        }
    }

    /// Returns the table at the given address within RSDT tables field, returning None if out of bounds
    pub fn table(&self, address: usize) -> Option<PTR> {
        if !(0..self.num_addresses).contains(&address) {
            return None;
        }

        unsafe {
            Some(core::ptr::read_unaligned(
                (self.tables as *const PTR).add(address),
            ))
        }
    }

    /// Attempts to find the table with the given signature, returning pointer to start of table if it exists
    pub fn find_table(&self, signature: &[u8]) -> Option<PTR> {
        let signature: [u8; 4] = signature.try_into().ok()?;

        for i in 0..self.num_addresses {
            let table_addr = self.table(i).unwrap();

            let (table, _) = unsafe { Header::from_addr(table_addr.try_into().ok()?) }.unwrap();

            if table.signature == signature {
                return Some(table_addr);
            }
        }

        None
    }
}
