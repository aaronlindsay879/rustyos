#![feature(abi_x86_interrupt)]
#![no_std]

mod gdt;
mod interrupts;
mod mem;

use core::{
    panic::PanicInfo,
    sync::atomic::{AtomicBool, Ordering},
};

use acpi::tables::fixed::{hpet::Hpet as HpetTable, madt::Madt, rsdt::Rsdt};
use kernel_shared::{
    logger::Logger,
    mem::{
        PHYS_MEM_OFFSET, frame_alloc::bitmap::BitmapFrameAlloc,
        paging::active_table::ActivePageTable,
    },
};
use multiboot::prelude::BootInfo;

static LOGGER: Logger = Logger::new(log::LevelFilter::Trace);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{info}");
    kernel_shared::x86::halt()
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(bootinfo_addr: usize, loader_start: usize, loader_end: usize) {
    // bootinfo is only valid for this scope
    let (_frame_alloc, _active_page_table) = {
        // it is not mapped at lower address anymore, so must mask to access from physical memory mapping
        let bootinfo_addr = bootinfo_addr | PHYS_MEM_OFFSET;
        let bootinfo = unsafe { BootInfo::new(bootinfo_addr as *const u32) }.unwrap();

        init(&bootinfo, loader_start, loader_end).unwrap()
    };

    kernel_shared::x86::halt()
}

fn init(
    bootinfo: &BootInfo,
    loader_start: usize,
    loader_end: usize,
) -> Option<(&'static mut BitmapFrameAlloc, ActivePageTable)> {
    // prevents being called twice
    static INIT_CALLED: AtomicBool = AtomicBool::new(false);

    if INIT_CALLED.swap(true, Ordering::Relaxed) {
        panic!("init must only be called once")
    }

    LOGGER.init().expect("failed to init logger");
    log::info!("entered kernel_main");

    // initialise memory
    let (frame_alloc, page_table) = mem::init(loader_start, loader_end);

    // now find acpi root table
    let rsdt_addr = bootinfo.rsdpv1.as_ref()?.rsdt_addr as usize | PHYS_MEM_OFFSET;
    log::trace!("ACPI RSDT table at {rsdt_addr:#X}");

    let rsdt_table = unsafe { Rsdt::<u32>::from_addr(rsdt_addr) }?;

    let madt_table = rsdt_table.find_table(&Madt::SIGNATURE, PHYS_MEM_OFFSET)?;
    log::trace!("ACPI MADT table at {madt_table:#X}");
    let madt = unsafe { Madt::from_addr(madt_table)? };

    let hpet_table = rsdt_table.find_table(&HpetTable::SIGNATURE, PHYS_MEM_OFFSET)?;
    log::trace!("ACPI HPET table at {hpet_table:#X}");
    let hpet = unsafe { HpetTable::from_addr(hpet_table)? };

    gdt::init();
    interrupts::init(&madt, &hpet);

    Some((frame_alloc, page_table))
}
