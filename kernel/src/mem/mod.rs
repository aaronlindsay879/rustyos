use kernel_shared::mem::{
    frame_alloc::bitmap::BitmapFrameAlloc, page::Page, paging::active_table::ActivePageTable,
};

/// Initialises memory for kernel
pub fn init(
    loader_start: usize,
    loader_end: usize,
) -> (&'static mut BitmapFrameAlloc, ActivePageTable) {
    log::info!("initialising memory");

    let frame_alloc = unsafe { BitmapFrameAlloc::from_address(0xFFFFFFFF00000000) };
    let mut active_table = unsafe { ActivePageTable::new() };

    unsafe {
        free_region(&mut active_table, frame_alloc, loader_start, loader_end);
    }

    log::trace!("\t* loader memory freed");
    log::info!("memory initialised");

    (frame_alloc, active_table)
}

pub unsafe fn free_region(
    active_table: &mut ActivePageTable,
    frame_alloc: &mut BitmapFrameAlloc,
    addr_start: usize,
    addr_end: usize,
) {
    let start_page = Page::containing_address(addr_start);
    let end_page = Page::containing_address(addr_end);

    for page in start_page..=end_page {
        active_table.unmap(page, frame_alloc, true);
    }
}
