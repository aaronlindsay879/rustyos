#![no_std]
#![feature(const_trait_impl, used_with_arg)]

use core::panic::PanicInfo;

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
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
extern "C" fn loader_main() {
    let textbuffer = 0xB8000 as *mut u32;

    unsafe {
        *textbuffer = 0x2f4b2f4f;
    }
}
