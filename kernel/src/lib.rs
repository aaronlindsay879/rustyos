#![no_std]

use core::panic::PanicInfo;

use acpi::tables::{
    fixed::{madt::Madt, rsdt::Rsdt},
    header::signature_at_addr,
};
use kernel_shared::{logger::Logger, mem::PHYS_MEM_OFFSET};
use multiboot::prelude::BootInfo;

static LOGGER: Logger = Logger::new(log::LevelFilter::Trace);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{info}");
    kernel_shared::x86::halt()
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(bootinfo_addr: usize, loader_start: usize, loader_end: usize) {
    LOGGER.init().expect("failed to init logger");
    log::info!("entered kernel_main");

    // it is not mapped at lower address anymore, so must mask to access from physical memory mapping
    let bootinfo_addr = bootinfo_addr | PHYS_MEM_OFFSET;
    let bootinfo = unsafe { BootInfo::new(bootinfo_addr as *const u32) }.unwrap();

    // now find acpi root table
    let rsdt_addr = bootinfo.rsdpv1.as_ref().unwrap().rsdt_addr as usize;
    log::trace!("ACPI RSDT table at 0x{rsdt_addr:08X}");

    let rsdt_table = unsafe { Rsdt::<u32>::from_addr(rsdt_addr | PHYS_MEM_OFFSET) }.unwrap();

    // and loop through other tables
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

    kernel_shared::x86::halt()
}
