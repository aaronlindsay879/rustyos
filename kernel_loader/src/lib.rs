#![no_std]
#![feature(const_trait_impl, used_with_arg)]

use core::panic::PanicInfo;

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

    let meminfo = bootinfo.memory_map.unwrap();
    serial_println!("{}", meminfo);

    let textbuffer = 0xB8000 as *mut u32;

    unsafe {
        *textbuffer = 0x2f4b2f4f;
    }
}
