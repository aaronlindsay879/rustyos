//! A page table of any level

use core::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::mem::{
    frame_alloc::FrameAllocator,
    paging::{
        ENTRY_COUNT, PHYS_MEM_OFFSET,
        entry::{Entry, EntryFlags},
    },
};

/// Marker trait for recursive table levels
pub trait TableLevel {}

/// Level 4 page table
pub enum Level4 {}

/// Level 3 page table
pub enum Level3 {}

/// Level 2 page table
pub enum Level2 {}

/// Level 1 page table
pub enum Level1 {}

impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

/// Marker trait to indicate which table levels have sub-levels
pub trait HierarchicalLevel: TableLevel {
    /// Next lower level
    type NextLevel: TableLevel;
}

impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}

impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
}

impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}

/// Stores a page table of a specific level
pub struct Table<L: TableLevel> {
    /// Entries within page table
    entries: [Entry; ENTRY_COUNT],
    /// Level type
    level: PhantomData<L>,
}

impl<L: TableLevel> Table<L> {
    /// Sets all entries in table to unused
    pub fn zero(&mut self) {
        for entry in &mut self.entries {
            entry.set_unused();
        }
    }

    /// Checks if the page table is empty by iterating over each entry
    pub fn is_empty(&self) -> bool {
        self.entries.iter().all(Entry::is_unused)
    }
}

impl<L: HierarchicalLevel> Table<L> {
    /// Finds address of the next level table at the given index
    fn next_table_address(&self, index: usize) -> Option<usize> {
        let entry_flags = self[index].flags();

        if !entry_flags.contains(EntryFlags::HUGE_PAGE) {
            self[index]
                .pointed_frame()
                .map(|frame| frame.start_address() | PHYS_MEM_OFFSET)
        } else {
            None
        }
    }

    /// Finds the next level table at the given index
    pub fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &*(address as *const _) })
    }

    /// Finds the next level table at the given index
    pub fn next_table_mut(&mut self, index: usize) -> Option<&mut Table<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &mut *(address as *mut _) })
    }

    /// Finds the next level table with the specified index, creating a blank table if it doesn't exist
    pub fn next_table_create<A: FrameAllocator>(
        &mut self,
        index: usize,
        allocator: &mut A,
    ) -> &mut Table<L::NextLevel> {
        // create table if doesnt exist
        if self.next_table(index).is_none() {
            assert!(
                !self.entries[index].flags().contains(EntryFlags::HUGE_PAGE),
                "mapping code does not support huge pages"
            );

            // allocate a frame, point to it, and make sure its zeroed
            let frame = allocator.allocate_frame().expect("no available frames");

            self.entries[index].set(frame, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            self.next_table_mut(index).unwrap().zero();
        }

        // we know next table either already existed, or we created it
        self.next_table_mut(index).unwrap()
    }
}

impl<L: TableLevel> Index<usize> for Table<L> {
    type Output = Entry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<L: TableLevel> IndexMut<usize> for Table<L> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}
