//! Code for allocating physical memory using a bitmap, where one frame = one bit

use multiboot::prelude::{MemoryEntryType, MemoryMapEntry};

use crate::mem::{
    PHYS_MEM_OFFSET,
    frame::{FRAME_SIZE, Frame},
    frame_alloc::FrameAllocator,
};

/// Stores information about frames within a single region of usable memory
#[repr(C)]
struct BitmapRegion {
    /// Base memory address of region
    region_base_addr: usize,
    /// Length of region in bytes
    region_size: usize,
    /// Number of entries within bitmap
    bitmap_length: usize,
    /// Bitmap array
    bitmap: [usize],
}

impl BitmapRegion {
    /// Value to shift by in order to get offset into array
    const SHIFT_VALUE: u32 = usize::BITS.ilog2();

    /// Value to and with in order to get offset into usize
    const AND_MASK: usize = (1 << Self::SHIFT_VALUE) - 1;

    /// Gets the frame at a given index
    fn get_frame(&self, index: usize) -> Option<Frame> {
        // make sure we're actually in range
        if index > self.bitmap_length * 64 {
            return None;
        }

        Some(Frame::containing_address(
            index * FRAME_SIZE + self.region_base_addr,
        ))
    }

    /// Checks if a given frame is contained within this region, returning the index if it is
    fn frame_index(&self, frame: Frame) -> Option<usize> {
        let frame_addr = frame.start_address();

        // make sure frame is within our region
        if !(self.region_base_addr..self.region_base_addr + self.region_size).contains(&frame_addr)
        {
            return None;
        }

        // remove base_addr, and divide by 4096 to remove empty bits (frames are always aligned to 4096)
        Some((frame_addr - self.region_base_addr) / 4096)
    }

    /// Finds the index of the first unset bit, returning None if all set
    fn find_first_unset(&self) -> Option<usize> {
        for i in 0..self.bitmap_length {
            let entry = unsafe { *self.bitmap.get_unchecked(i) };

            // if all bits set, skip
            if entry == !0 {
                continue;
            }

            // otherwise at least one bit is unset
            let unset_bit_idx = entry.trailing_ones() as usize;

            return Some((i << Self::SHIFT_VALUE) | unset_bit_idx);
        }
        None
    }

    /// Sets a given bit to 1
    fn set_bit(&mut self, index: usize) {
        // make sure we're actually in range
        if index > self.bitmap_length * 64 {
            return;
        }

        // find indices
        let array_index = index >> Self::SHIFT_VALUE;
        let entry_index = index & Self::AND_MASK;

        // and finally set bit
        unsafe { *self.bitmap.get_unchecked_mut(array_index) |= 1 << entry_index };
    }

    /// Sets a given bit to 0
    fn unset_bit(&mut self, index: usize) {
        // make sure we're actually in range
        if index > self.bitmap_length * 64 {
            return;
        }

        // find indices
        let array_index = index >> Self::SHIFT_VALUE;
        let entry_index = index & Self::AND_MASK;

        // and finally set bit
        unsafe { *self.bitmap.get_unchecked_mut(array_index) &= !(1 << entry_index) };
    }

    /// Sets all entries to '1' (used) in unavailable memory
    fn block_unavailable_regions(&mut self) {
        let final_index = self.region_size / 4096;

        let array_index = final_index >> Self::SHIFT_VALUE;
        let entry_index = final_index & Self::AND_MASK;

        unsafe {
            *self.bitmap.get_unchecked_mut(array_index) |=
                !(!0usize << (64 - entry_index)) << entry_index
        };
    }
}

/// Handles allocating frames, tracking and freeing them as needed
#[repr(C)]
pub struct BitmapFrameAlloc {
    /// Number of memory regions we keep track of
    pub region_count: usize,
    /// Pointer to first entry in region array
    first_region: *mut BitmapRegion,
}

impl BitmapFrameAlloc {
    /// Constructs a new bitmap frame allocator, storing the data at `addr` and returning the number of bytes written
    ///
    /// ## Safety
    /// This function uses a **lot** of raw memory operations - both `addr` and `memory_map_entries` must be valid.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn new(
        phys_addr: usize,
        addr: usize,
        memory_map_entries: &'static [MemoryMapEntry],
    ) -> (&'static mut Self, usize) {
        log::trace!("constructing frame allocator at physical addr 0x{phys_addr:016X}");

        use core::ptr::*;
        let write_addr = addr as *mut usize;

        // start by writing the frame alloc itself
        let region_count = memory_map_entries
            .iter()
            .filter(|region| region.entry_type == MemoryEntryType::RAM)
            .count();

        unsafe {
            write(write_addr, region_count);
            write(write_addr.add(1) as *mut *mut usize, write_addr.add(2));
        }

        // now move to first memory region
        let mut write_addr = unsafe { write_addr.add(2) };

        for region in memory_map_entries
            .iter()
            .filter(|region| region.entry_type == MemoryEntryType::RAM)
        {
            log::trace!(
                "setting up memory region at base addr 0x{:016X} with length 0x{:X}",
                region.base_addr,
                region.length
            );
            // write each field of region, setting all pages as '0' (free)
            unsafe {
                write(write_addr, region.base_addr as usize);
                write(write_addr.add(1), region.length as usize);

                let entries_needed = (region.length as usize).div_ceil(FRAME_SIZE * 64);
                write(write_addr.add(2), entries_needed);

                write_bytes(write_addr.add(3), 0, entries_needed);

                // and finally set addr to start of next region
                write_addr = write_addr.add(3 + entries_needed);
            }
        }

        let bitmap_alloc = unsafe { &mut *(addr as *mut BitmapFrameAlloc) };

        // block all unavailable regions
        let mut region = bitmap_alloc.first_region;
        for _ in 0..bitmap_alloc.region_count {
            let region_ref = unsafe { &mut *region };

            log::trace!(
                "blocking unusable memory in region with base addr 0x{:016X}",
                region_ref.region_base_addr
            );
            region_ref.block_unavailable_regions();

            // move to next region
            region = unsafe { region.byte_add(24 + region_ref.bitmap_length * size_of::<usize>()) };
        }

        // also block allocator memory
        // write_addr.addr() - addr gives us number of bytes written
        let start_frame = Frame::containing_address(phys_addr);
        let end_frame = Frame::containing_address(phys_addr + (write_addr.addr() - addr));

        log::trace!(
            "blocking allocator memory from 0x{:016X} to 0x{:016X}",
            start_frame.start_address(),
            end_frame.start_address()
        );
        bitmap_alloc.block_region(start_frame..=end_frame);

        (bitmap_alloc, write_addr.addr() - addr)
    }

    /// Finds the first free frame, returning the region it lies in and the index within that region if it exists
    fn first_free_frame(&mut self) -> Option<(&mut BitmapRegion, usize)> {
        let mut region = self.first_region;

        for _ in 0..self.region_count {
            let region_ref = unsafe { &mut *region };

            if let Some(index) = region_ref.find_first_unset() {
                return Some((region_ref, index));
            }

            // move to next region
            region = unsafe { region.byte_add(24 + region_ref.bitmap_length * size_of::<usize>()) };
        }

        None
    }

    /// Finds the bitmap region and index of a given frame, if it exists within this allocator's scope
    fn find_frame_index(&mut self, frame: Frame) -> Option<(&mut BitmapRegion, usize)> {
        let mut region = self.first_region;
        for _ in 0..self.region_count {
            let region_ref = unsafe { &mut *region };

            if let Some(index) = region_ref.frame_index(frame) {
                return Some((region_ref, index));
            }

            // move to next region
            region = unsafe { region.byte_add(24 + region_ref.bitmap_length * size_of::<usize>()) };
        }

        None
    }

    /// Blocks an individual frame from being assigned
    pub fn block_frame(&mut self, frame: Frame) {
        if let Some((region, index)) = self.find_frame_index(frame) {
            region.set_bit(index);
        }
    }

    /// Blocks a range of frames from being handed out
    pub fn block_region<R>(&mut self, frame_range: R)
    where
        R: IntoIterator<Item = Frame>,
    {
        // doing it bit-by-bit is absolutely not the fastest way to do this,
        // but its the easiest to prove is correct
        // TODO?: replace with proper masking method

        for frame in frame_range {
            if let Some((region, index)) = self.find_frame_index(frame) {
                region.set_bit(index);
            }
        }
    }

    /// Returns if the frame is tracked by this frame allocator
    pub fn is_frame_tracked(&self, frame: Frame) -> bool {
        let frame_addr = frame.start_address();

        let mut region = self.first_region;
        for _ in 0..self.region_count {
            let region_ref = unsafe { &mut *region };

            if (region_ref.region_base_addr..region_ref.region_base_addr + region_ref.region_size)
                .contains(&frame_addr)
            {
                return true;
            }

            // move to next region
            region = unsafe { region.byte_add(24 + region_ref.bitmap_length * size_of::<usize>()) };
        }

        false
    }
}

impl FrameAllocator for BitmapFrameAlloc {
    fn allocate_frame(&mut self) -> Option<Frame> {
        let (region, index) = self.first_free_frame()?;

        region.set_bit(index);
        region.get_frame(index)
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        let (region, index) = self.find_frame_index(frame).unwrap();

        #[cfg(feature = "ZERO_OUT_FREED_MEMORY")]
        {
            let addr = frame.start_address() | PHYS_MEM_OFFSET;

            log::trace!("zeroing memory at {addr:#X}");
            unsafe { core::ptr::write_bytes(addr as *mut u8, 0, FRAME_SIZE) };
        }

        region.unset_bit(index);
    }
}
