#![no_std]
#![feature(const_trait_impl, used_with_arg)]

use core::panic::PanicInfo;
use std::{
    align_up,
    elf::{file_header::FileHeader, section_header::SectionHeader},
};

use acpi::tables::{
    fixed::{madt::Madt, rsdt::Rsdt},
    header::signature_at_addr,
};
use kernel_shared::{
    io::serial,
    logger::Logger,
    mem::{
        frame::{FRAME_SIZE, Frame},
        frame_alloc::bitmap::BitmapFrameAlloc,
    },
    serial_println,
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

const PHYS_MEM_OFFSET: usize = 0xFFFF800000000000;
static LOGGER: Logger = Logger::new(log::LevelFilter::Trace);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    loop {}
}

#[unsafe(no_mangle)]
extern "C" fn loader_main(bootinfo_addr: usize) {
    unsafe {
        serial::COM1.lock().init();
    }
    LOGGER.init().unwrap();

    let bootinfo = unsafe { BootInfo::new((bootinfo_addr) as *const u32).unwrap() };

    let string_table = bootinfo.elf_symbols.as_ref().unwrap().string_header();
    for loader_section in bootinfo.elf_symbols.as_ref().unwrap().section_headers {
        log::trace!(
            "loader ELF section at 0x{:08X}-0x{:08X} with name {:?}",
            loader_section.addr,
            loader_section.addr + loader_section.size,
            loader_section.name(string_table),
        );
    }

    let rsdt_addr = bootinfo.rsdpv1.as_ref().unwrap().rsdt_addr as usize;
    log::trace!("ACPI RSDT table at 0x{rsdt_addr:08X}");

    let rsdt_table = unsafe { Rsdt::<u32>::from_addr(rsdt_addr | PHYS_MEM_OFFSET) }.unwrap();

    for table_addr in (0..rsdt_table.num_addresses).map(|i| rsdt_table.table(i).unwrap() as usize) {
        let table_addr = table_addr | PHYS_MEM_OFFSET;
        let signature = unsafe { signature_at_addr(table_addr) };

        log::trace!(
            "ACPI table at 0x{:X} with signature `{}`",
            table_addr,
            unsafe { core::str::from_utf8_unchecked(&signature) }
        );

        #[allow(clippy::single_match)]
        match signature {
            Madt::SIGNATURE => {
                let madt = unsafe { Madt::from_addr(table_addr).unwrap() };

                for i in 0.. {
                    if let Some(_field) = madt.get_table_entry(i) {
                        log::trace!("{_field:#?}");
                    } else {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    let memory_map = bootinfo.memory_map.as_ref().unwrap();
    log::trace!("{memory_map}");

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

    let frame_alloc = unsafe {
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
