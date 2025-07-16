//! Code for manipulating an active L4 table

use core::ops::{Deref, DerefMut};

use crate::{
    mem::paging::{
        inactive_table::InactivePageTable,
        mapper::Mapper,
        table::{Level4, Table},
    },
    x86::registers::CR3,
};

/// The active, loaded L4 page table
pub struct ActivePageTable {
    /// Mapper
    mapper: Mapper,
}

impl ActivePageTable {
    /// Creates a new mapper with the active page 4 table
    ///
    /// # Safety
    /// This should only ever be called once
    pub unsafe fn new() -> Self {
        let table = CR3::read().0.start_address() + super::PHYS_MEM_OFFSET;
        let table = table as *mut Table<Level4>;

        Self {
            mapper: unsafe { Mapper::new(table) },
        }
    }

    /// Switches the currently loaded table to the provided inactive table
    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        let (frame, flags) = CR3::read();

        let old_table = unsafe { InactivePageTable::new(frame) };
        let new_table_frame = new_table.frame();

        unsafe { CR3::write(new_table_frame, flags) }

        old_table
    }
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Self::Target {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mapper
    }
}
