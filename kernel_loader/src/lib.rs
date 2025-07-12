#![no_std]
#![feature(const_trait_impl, used_with_arg)]

use core::panic::PanicInfo;

use acpi::tables::{
    fixed::{madt::Madt, rsdt::Rsdt},
    header::signature_at_addr,
};
use kernel_shared::{io::serial, serial_println};
use multiboot::{multiboot_header, prelude::*};

multiboot_header! {
    arch: 0,
    tags: [
        ConsoleFlags {
            flags: Flags::Required,
            must_be_present: true,
            ega_text_support: true,
        },
    ]
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    loop {}
}

#[unsafe(no_mangle)]
extern "C" fn loader_main(bootinfo_addr: *const u32) {
    unsafe {
        serial::COM1.lock().init();
    }

    let bootinfo = unsafe { BootInfo::new(bootinfo_addr).unwrap() };
    serial_println!("{:#X?}", bootinfo);

    let rsdt_addr = bootinfo.rsdpv1.unwrap().rsdt_addr as usize;
    let rsdt_table = unsafe { Rsdt::<u32>::from_addr(rsdt_addr) }.unwrap();

    serial_println!(
        "RSDT header at 0x{:X} with signature `{}`",
        rsdt_addr,
        rsdt_table.header.signature()
    );

    for table_addr in (0..rsdt_table.num_addresses).map(|i| rsdt_table.table(i).unwrap() as usize) {
        let signature = unsafe { signature_at_addr(table_addr) };

        serial_println!("table at 0x{:X} with signature `{}`", table_addr, unsafe {
            core::str::from_utf8_unchecked(&signature)
        });

        #[allow(clippy::single_match)]
        match signature {
            Madt::SIGNATURE => {
                let madt = unsafe { Madt::from_addr(table_addr).unwrap() };

                serial_println!("{:#X?}", madt);

                for i in 0.. {
                    let field = madt.get_table_entry(i);

                    if let Some(field) = field {
                        serial_println!("{:#X?}", field);
                    } else {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
}
