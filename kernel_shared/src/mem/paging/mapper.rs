//! Code for mapping a virtual address to a physical address

use core::ptr::NonNull;
use std::is_aligned;

use crate::{
    mem::{
        align_down_to_page,
        frame::Frame,
        frame_alloc::FrameAllocator,
        page::{HUGE_L2_PAGE_SIZE, HUGE_L3_PAGE_SIZE, PAGE_SIZE, Page},
        paging::{
            ENTRY_COUNT,
            entry::EntryFlags,
            table::{Level4, Table},
        },
    },
    x86::invalidate_address,
};

/// A struct to map addresses with the stored L4 table
pub struct Mapper {
    /// Base L4 table to use
    table: NonNull<Table<Level4>>,
}

impl Mapper {
    /// Creates a new mapper with the given page 4 table
    ///
    /// # Safety
    /// This should only ever be called with a valid table
    pub unsafe fn new(table: *mut Table<Level4>) -> Self {
        Self {
            table: unsafe { NonNull::new_unchecked(table) },
        }
    }

    /// Returns a reference to the level 4 table
    pub fn p4(&self) -> &Table<Level4> {
        unsafe { self.table.as_ref() }
    }

    /// Returns a mutable reference to the level 4 table
    pub fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.table.as_mut() }
    }

    /// Translates a given virtual address to its physical address
    pub fn translate(&self, virt_addr: usize) -> Option<usize> {
        let offset = virt_addr % PAGE_SIZE;

        self.translate_page(Page::containing_address(virt_addr))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    /// Finds the frame that a given page points to
    pub fn translate_page(&self, page: Page) -> Option<Frame> {
        self.p4()
            .next_table(page.p4_index())
            .and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].pointed_frame())
            .or_else(|| self.translate_huge_page(page))
    }

    /// Translates a page, with support for huge pages
    fn translate_huge_page(&self, page: Page) -> Option<Frame> {
        self.p4().next_table(page.p4_index()).and_then(|p3| {
            let p3_entry = &p3[page.p3_index()];
            // 1GiB page?
            if let Some(start_frame) = p3_entry.pointed_frame()
                && p3_entry.flags().contains(EntryFlags::HUGE_PAGE)
            {
                // address must be 1GiB aligned
                assert_eq!(start_frame.number % (ENTRY_COUNT * ENTRY_COUNT), 0);
                let frame = Some(Frame {
                    number: start_frame.number + page.p2_index() * ENTRY_COUNT + page.p1_index(),
                });
                return frame;
            }

            if let Some(p2) = p3.next_table(page.p3_index()) {
                let p2_entry = &p2[page.p2_index()];
                // 2MiB page?
                if let Some(start_frame) = p2_entry.pointed_frame()
                    && p2_entry.flags().contains(EntryFlags::HUGE_PAGE)
                {
                    // address must be 2MiB aligned
                    assert_eq!(start_frame.number % ENTRY_COUNT, 0);
                    return Some(Frame {
                        number: start_frame.number + page.p1_index(),
                    });
                }
            }
            None
        })
    }

    /// Maps a given page to any available frame, using the provided flags
    pub fn map<A: FrameAllocator>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A) {
        let frame = allocator.allocate_frame().expect("out of memory");
        self.map_to(page, frame, flags, allocator)
    }

    /// Maps a given page to a given frame, using the provided flags
    pub fn map_to<A: FrameAllocator>(
        &mut self,
        page: Page,
        frame: Frame,
        flags: EntryFlags,
        allocator: &mut A,
    ) {
        let p4 = self.p4_mut();
        let p3 = p4.next_table_create(page.p4_index(), allocator);
        let p2 = p3.next_table_create(page.p3_index(), allocator);
        let p1 = p2.next_table_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].is_unused());

        p1[page.p1_index()].set(frame, flags | EntryFlags::PRESENT);
    }

    /// Maps a given page to a given frame, using the provided flags and a 2MiB page entry
    pub fn map_to_huge_l2<A: FrameAllocator>(
        &mut self,
        page: Page,
        frame: Frame,
        flags: EntryFlags,
        allocator: &mut A,
    ) {
        let p4 = self.p4_mut();
        let p3 = p4.next_table_create(page.p4_index(), allocator);
        let p2 = p3.next_table_create(page.p3_index(), allocator);

        assert_eq!(page.p1_index(), 0);
        assert!(p2[page.p2_index()].is_unused());

        p2[page.p2_index()].set(frame, flags | EntryFlags::PRESENT | EntryFlags::HUGE_PAGE);
    }

    /// Maps a given page to a given frame, using the provided flags and a 1GiB page entry
    pub fn map_to_huge_l3<A: FrameAllocator>(
        &mut self,
        page: Page,
        frame: Frame,
        flags: EntryFlags,
        allocator: &mut A,
    ) {
        let p4 = self.p4_mut();
        let p3 = p4.next_table_create(page.p4_index(), allocator);

        assert_eq!(page.p1_index(), 0);
        assert_eq!(page.p2_index(), 0);
        assert!(p3[page.p3_index()].is_unused());

        p3[page.p3_index()].set(frame, flags | EntryFlags::PRESENT | EntryFlags::HUGE_PAGE);
    }

    /// Identity maps a given frame, using the provided flags
    pub fn identity_map<A: FrameAllocator>(
        &mut self,
        frame: Frame,
        flags: EntryFlags,
        allocator: &mut A,
    ) {
        let page = Page::containing_address(frame.start_address());
        self.map_to(page, frame, flags, allocator)
    }

    /// Maps a range of addresses. `use_huge_tables` should be used carefully since they can not currently be unmapped
    pub fn map_range<A: FrameAllocator>(
        &mut self,
        phys_range: (usize, usize),
        virt_range: (usize, usize),
        flags: EntryFlags,
        allocator: &mut A,
        use_huge_tables: bool,
    ) {
        // first make sure to align to pages
        let start_phys = align_down_to_page(phys_range.0);
        let end_phys = align_down_to_page(phys_range.1);

        let start_virt = align_down_to_page(virt_range.0);
        let end_virt = align_down_to_page(virt_range.1);

        log::trace!("mapping {start_virt:#X}-{end_virt:#X} to {start_phys:#X}-{end_phys:#X}");

        // check how addresses are aligned relative to each other to check if huge tables are even possible
        let huge_l3_possible =
            use_huge_tables && is_aligned(start_virt - start_phys, HUGE_L3_PAGE_SIZE);
        let huge_l2_possible =
            use_huge_tables && is_aligned(start_virt - start_phys, HUGE_L2_PAGE_SIZE);

        let to_map = (end_phys - start_phys).min(start_virt - end_virt);
        let mut mapped = 0;

        while mapped <= to_map {
            if huge_l3_possible
                && to_map - mapped >= HUGE_L3_PAGE_SIZE
                && is_aligned(start_phys + mapped, HUGE_L3_PAGE_SIZE)
                && is_aligned(start_virt + mapped, HUGE_L3_PAGE_SIZE)
            {
                // if need to map more than HUGE_L3_PAGE_SIZE and addresses are aligned, map a 1GiB page
                self.map_to_huge_l3(
                    Page::containing_address(start_virt + mapped),
                    Frame::containing_address(start_phys + mapped),
                    flags,
                    allocator,
                );

                mapped += HUGE_L3_PAGE_SIZE;
            } else if huge_l2_possible
                && to_map - mapped >= HUGE_L2_PAGE_SIZE
                && is_aligned(start_phys + mapped, HUGE_L2_PAGE_SIZE)
                && is_aligned(start_virt + mapped, HUGE_L2_PAGE_SIZE)
            {
                // then repeat for 2MiB page
                self.map_to_huge_l2(
                    Page::containing_address(start_virt + mapped),
                    Frame::containing_address(start_phys + mapped),
                    flags,
                    allocator,
                );

                mapped += HUGE_L2_PAGE_SIZE;
            } else {
                // otherwise just map a normal 4KiB page
                self.map_to(
                    Page::containing_address(start_virt + mapped),
                    Frame::containing_address(start_phys + mapped),
                    flags,
                    allocator,
                );

                mapped += PAGE_SIZE;
            }
        }
    }

    /// Unmaps a given page
    pub fn unmap<A>(&mut self, page: Page, allocator: &mut A, free_unused_tables: bool)
    where
        A: FrameAllocator,
    {
        assert!(self.translate(page.start_address()).is_some());

        let p3 = self
            .p4_mut()
            .next_table_mut(page.p4_index())
            .expect("mapping code does not support huge pages");
        let p2 = p3
            .next_table_mut(page.p3_index())
            .expect("mapping code does not support huge pages");
        let p1 = p2
            .next_table_mut(page.p2_index())
            .expect("mapping code does not support huge pages");

        let frame = p1[page.p1_index()].pointed_frame().unwrap();
        p1[page.p1_index()].set_unused();

        invalidate_address(frame.start_address());
        allocator.deallocate_frame(frame);

        // TODO: remove repeated code
        if free_unused_tables {
            if p1.is_empty() {
                let p1_frame = p2[page.p2_index()].pointed_frame().unwrap();
                p2[page.p2_index()].set_unused();

                log::trace!("freeing unused p1 table at frame {p1_frame:?}");

                invalidate_address(p1_frame.start_address());
                allocator.deallocate_frame(p1_frame);
            }

            if p2.is_empty() {
                let p2_frame = p3[page.p3_index()].pointed_frame().unwrap();
                p3[page.p3_index()].set_unused();

                log::trace!("freeing unused p2 table at frame {p2_frame:?}");
                invalidate_address(p2_frame.start_address());
                allocator.deallocate_frame(p2_frame);
            }

            if p3.is_empty() {
                let p3_frame = self.p4()[page.p4_index()].pointed_frame().unwrap();
                self.p4_mut()[page.p4_index()].set_unused();

                log::trace!("freeing unused p3 table at frame {p3_frame:?}");
                invalidate_address(p3_frame.start_address());
                allocator.deallocate_frame(p3_frame);
            }
        }
    }
}
