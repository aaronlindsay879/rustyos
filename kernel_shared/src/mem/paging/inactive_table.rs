//! Code for manipulating an inactive L4 table

use core::ops::{Deref, DerefMut};

use crate::mem::{
    frame::Frame,
    paging::{
        mapper::Mapper,
        table::{Level4, Table},
    },
};

/// An inactive, unloaded L4 page table
pub struct InactivePageTable {
    /// Mapper
    mapper: Mapper,
    /// Frame that L4 table lies in
    frame: Frame,
}

impl InactivePageTable {
    /// Creates a new mapper in the given frame
    ///
    /// # Safety
    /// This should only ever be called with a valid frame
    pub unsafe fn new(frame: Frame) -> Self {
        unsafe {
            core::ptr::write_bytes(frame.start_address() as *mut u64, 0, 512);
        }
        let table = frame.start_address() as *mut Table<Level4>;

        Self {
            mapper: unsafe { Mapper::new(table) },
            frame,
        }
    }

    /// Returns the frame that L4 table lies in
    pub fn frame(self) -> Frame {
        self.frame
    }
}

impl Deref for InactivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Self::Target {
        &self.mapper
    }
}

impl DerefMut for InactivePageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mapper
    }
}
