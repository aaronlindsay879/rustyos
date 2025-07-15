//! Code for memory management, such as paging and frame allocation.

use std::{align_down, align_up};

use crate::mem::page::PAGE_SIZE;

pub mod frame;
pub mod frame_alloc;
pub mod page;
pub mod paging;

/// Offset of physical memory within mappings
pub const PHYS_MEM_OFFSET: usize = 0xFFFF800000000000;

/// Align downwards - returns the greatest _x_ with alignment of page size
/// such that _x_ <= addr. `align` must be power of 2
pub fn align_down_to_page(addr: usize) -> usize {
    align_down(addr, PAGE_SIZE)
}

/// Align upwards - returns the smallest _x_ with alignment of page size
/// such that _x_ >= addr. `align` must be power of 2
pub fn align_up_to_page(addr: usize) -> usize {
    align_up(addr, PAGE_SIZE)
}
