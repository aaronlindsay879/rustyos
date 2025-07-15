#![no_std]
#![feature(const_trait_impl, used_with_arg)]

use core::{arch::asm, ops::DerefMut, panic::PanicInfo};
use std::{
    align_up,
    elf::{
        file_header::FileHeader,
        section_header::{SectionHeader, SectionType},
    },
};

use kernel_shared::{
    io::serial,
    logger::Logger,
    mem::{
        PHYS_MEM_OFFSET, align_down_to_page,
        frame::{FRAME_SIZE, Frame},
        frame_alloc::{FrameAllocator, bitmap::BitmapFrameAlloc},
        page::{PAGE_SIZE, Page},
        paging::{
            active_table::ActivePageTable, entry::EntryFlags, inactive_table::InactivePageTable,
            mapper::Mapper,
        },
    },
};
use multiboot::{multiboot_header, prelude::*};

multiboot_header! {
    arch: 0,
    tags: [
        ModuleAlignment {
            flags: Flags::Required
        },
        ConsoleFlags {
            flags: Flags::Required,
            must_be_present: true,
            ega_text_support: true,
        },
    ]
}

static LOGGER: Logger = Logger::new(log::LevelFilter::Trace);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{info}");
    kernel_shared::x86::halt()
}

#[unsafe(no_mangle)]
extern "C" fn loader_main(bootinfo_addr: usize) {
    unsafe {
        serial::COM1.lock().init();
    }
    LOGGER.init().unwrap();

    let bootinfo = unsafe { BootInfo::new((bootinfo_addr) as *const u32).unwrap() };
    let memory_map = bootinfo.memory_map.as_ref().unwrap();

    let (bootinfo_start, bootinfo_end) = (bootinfo.addr, bootinfo.addr + bootinfo.size);
    log::trace!("bootinfo start: 0x{bootinfo_start:X}, end: 0x{bootinfo_end:X}");

    let (loader_start, loader_end) =
        loader_range(bootinfo.elf_symbols.as_ref().unwrap().section_headers);
    log::trace!("loader start: 0x{loader_start:X}, end: 0x{loader_end:X}");

    let kernel_module = bootinfo.module(c"kernel").unwrap();
    let (kernel_start, kernel_end) = (
        kernel_module.module_addr as usize,
        (kernel_module.module_addr + kernel_module.module_len) as usize,
    );
    log::trace!("kernel start: 0x{kernel_start:X}, end 0x{kernel_end:X}");

    // if we have extended memory at 0x0000000100000000, then we can simply start frame alloc there
    // otherwise we have to place it after everything multiboot2 loaded
    let frame_alloc_phys_addr = if memory_map.contains_extended_memory_three() {
        0x0000000100000000
    } else {
        align_up(bootinfo_end.max(loader_end).max(kernel_end), FRAME_SIZE)
    };

    let frame_alloc_addr = frame_alloc_phys_addr | PHYS_MEM_OFFSET;

    let (frame_alloc, frame_alloc_size) = unsafe {
        BitmapFrameAlloc::new(frame_alloc_phys_addr, frame_alloc_addr, memory_map.entries)
    };

    let bootinfo_region =
        Frame::containing_address(bootinfo_start)..=Frame::containing_address(bootinfo_end);
    log::trace!(
        "blocking bootinfo region 0x{:X}-0x{:X}",
        bootinfo_region.start().start_address(),
        bootinfo_region.end().start_address()
    );
    frame_alloc.block_region(bootinfo_region);

    let loader_region =
        Frame::containing_address(loader_start)..=Frame::containing_address(loader_end);
    log::trace!(
        "blocking loader region 0x{:X}-0x{:X}",
        loader_region.start().start_address(),
        loader_region.end().start_address()
    );
    frame_alloc.block_region(loader_region);

    let kernel_region =
        Frame::containing_address(kernel_start)..=Frame::containing_address(kernel_end);
    log::trace!(
        "blocking kernel region 0x{:X}-0x{:X}",
        kernel_region.start().start_address(),
        kernel_region.end().start_address()
    );
    frame_alloc.block_region(kernel_region);

    // if we place the L4 frame at physical address 0, then things break
    // so make sure frame 0 cant be handed out
    frame_alloc.block_frame(Frame::containing_address(0));

    // now we can start remapping
    let table_frame = frame_alloc
        .allocate_frame()
        .expect("failed to allocate a frame for level 4 table");

    let mut table = unsafe { InactivePageTable::new(table_frame) };

    // identity map bootinfo and loader
    identity_map(
        "bootinfo",
        frame_alloc,
        &mut table,
        bootinfo_start,
        bootinfo_end,
    );
    identity_map("loader", frame_alloc, &mut table, loader_start, loader_end);

    // also make sure to map allocator
    map_frame_allocator(
        frame_alloc,
        &mut table,
        Frame::containing_address(frame_alloc_phys_addr),
        Frame::containing_address(frame_alloc_phys_addr + frame_alloc_size),
    );

    // set up stack, descending from end of kernel space
    log::trace!("setting up stack at {:#X}", usize::MAX);
    let start_page = Page::containing_address(usize::MAX - kernel_shared::STACK_SIZE + 1);
    let end_page = Page::containing_address(usize::MAX);

    for page in start_page..=end_page {
        table.map(page, EntryFlags::WRITABLE, frame_alloc);
    }

    // now map kernel sections
    let kernel_elf = unsafe { FileHeader::from_addr(kernel_start) }.unwrap();
    let string_header = kernel_elf.string_header();

    for section_header in kernel_elf.section_headers() {
        // only map sections that need allocating
        if !section_header.allocated() {
            log::trace!(
                "skipping mapping kernel section {:?}",
                section_header.name(string_header, kernel_start)
            );
            continue;
        }

        let flags = EntryFlags::from_elf_section_flags(section_header);

        let start_phys = section_header.offset as usize + kernel_start;
        let end_phys = (section_header.offset + section_header.size - 1) as usize + kernel_start;

        let start_virt = section_header.addr as usize;
        let end_virt = (section_header.addr + section_header.size - 1) as usize;

        log::trace!(
            "mapping kernel section {:?} at {:#X}-{:#X} with flags `{}`",
            section_header.name(string_header, kernel_start),
            align_down_to_page(start_virt),
            align_down_to_page(end_virt),
            flags
        );

        assert_eq!(
            section_header.addr as usize % PAGE_SIZE,
            0,
            "sections need to be page aligned, addr {:#X}",
            section_header.addr
        );

        // if SHT_NOBITS, we need to manually zero
        if section_header.section_type == SectionType::Nobits {
            unsafe {
                core::ptr::write_bytes(
                    align_down_to_page(start_phys) as *mut u8,
                    0,
                    section_header.size as usize,
                )
            };
        }

        // finally actually map
        table.map_range(
            (start_phys, end_phys),
            (start_virt, end_virt),
            flags,
            frame_alloc,
            true,
        );
    }

    // and heap/phys memory
    map_heap(frame_alloc, &mut table, kernel_shared::HEAP_SIZE);
    map_phys_memory(frame_alloc, &mut table, memory_map);

    // now we're ready to hop to kernel!
    // first switch out active table, and then jump

    let entrypoint = kernel_elf.entry;

    let mut active_table = unsafe { ActivePageTable::new() };
    active_table.switch(table);

    log::trace!("switched active table!");

    log::trace!("jumping to kernel at {entrypoint:#X}");
    unsafe {
        asm!(
            "mov rsp, 0xFFFFFFFFFFFFFFFF",
            "jmp {}",
            in(reg) entrypoint,
            in("rdi") bootinfo_addr | PHYS_MEM_OFFSET,
            in("rsi") loader_start,
            in("rdx") loader_end
        )
    }
}

/// Finds where loader lies within memory
fn loader_range(section_headers: &'static [SectionHeader]) -> (usize, usize) {
    let start = section_headers
        .iter()
        .filter(|header| header.allocated())
        .map(|header| header.addr)
        .min()
        .unwrap() as usize;

    let end = section_headers
        .iter()
        .filter(|header| header.allocated())
        .map(|header| header.addr + header.size)
        .max()
        .unwrap() as usize;

    (start, end)
}

/// Helper function for identity mapping a region
fn identity_map<A: FrameAllocator, T: DerefMut<Target = Mapper>>(
    log_str: &'static str,
    alloc: &mut A,
    table: &mut T,
    start_addr: usize,
    end_addr: usize,
) {
    let start_frame = Frame::containing_address(start_addr);
    let end_frame = Frame::containing_address(end_addr);

    log::trace!(
        "mapping {log_str} at {:#X}-{:#X}",
        start_frame.start_address(),
        end_frame.start_address()
    );

    for frame in start_frame..=end_frame {
        table.identity_map(frame, EntryFlags::WRITABLE, alloc);
    }
}

/// Maps frame allocator to 0xFFFFFFFF00000000
fn map_frame_allocator<A: FrameAllocator, T: DerefMut<Target = Mapper>>(
    alloc: &mut A,
    table: &mut T,
    start_frame: Frame,
    end_frame: Frame,
) {
    log::trace!(
        "mapping frame allocator at phys addr {:#X}-{:#X}",
        start_frame.start_address(),
        end_frame.start_address()
    );

    table.map_range(
        (start_frame.start_address(), end_frame.start_address()),
        (0xFFFFFFFF00000000, 0xFFFFFFFF1FFFFFFF),
        EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE,
        alloc,
        true,
    );
}

/// Maps heap to 0xFFFFFFFF20000000
fn map_heap<A: FrameAllocator, T: DerefMut<Target = Mapper>>(
    alloc: &mut A,
    table: &mut T,
    size: usize,
) {
    log::trace!("mapping heap");

    let end_addr = (0xFFFFFFFF20000000 + size).min(0xFFFFFFFF3FFFFFFF);

    let start_page = Page::containing_address(0xFFFFFFFF20000000);
    let end_page = Page::containing_address(end_addr);

    for page in start_page..=end_page {
        table.map(page, EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE, alloc);
    }
}

/// Maps physical memory to 0xFFFF800000000000
fn map_phys_memory<A: FrameAllocator, T: DerefMut<Target = Mapper>>(
    alloc: &mut A,
    table: &mut T,
    memory_map: &MemoryMap,
) {
    log::trace!("mapping physical memory");

    let highest_address = memory_map
        .entries
        .iter()
        .map(|entry| entry.base_addr + entry.length)
        .max()
        .unwrap() as usize;

    table.map_range(
        (0, highest_address),
        (0xFFFF800000000000, 0xFFFFBFFFFFFFFFFF),
        EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE,
        alloc,
        true,
    );
}
