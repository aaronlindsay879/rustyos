//! Code for memory management, such as paging and frame allocation.

/// Size of a standard (non-huge) page
pub const STANDARD_PAGE_SIZE: usize = 4096;

pub mod frame;
pub mod frame_alloc;
