//! Provides functions and macros for constructing a multiboot2 header.

use std::cursor::Cursor;

use crate::prelude::HeaderTag;

pub mod address;
pub mod console_flags;
pub mod dummy;
pub mod efi_boot_services;
pub mod entry_address;
pub mod flags;
pub mod framebuffer;
pub mod header_tag;
pub mod information_request;
pub mod module_alignment;
pub mod relocatable;

/// Struct to build a multiboot2 compliant header, by writing tags to an internal buffer which is then emitted as an
/// array using [HeaderBuilder::as_bytes()]
#[repr(C)]
pub struct HeaderBuilder {
    /// Architecture: 0 for i386, 4 for MIPS
    arch: u32,
    /// Backing data storage
    out: [u8; Self::SIZE],
    /// Cursor for backing data storage
    out_cursor: Cursor<'static>,
}

impl HeaderBuilder {
    /// Multiboot2 header magic value
    const MAGIC: u32 = 0xE85250D6;

    /// Size of backing array
    pub const SIZE: usize = 4096;

    /// Constructs a new header **without** initialising pointers.
    /// Arch field indicates the architecture: 0 for i386, 4 for MIPS.
    /// To ensure correct usage, [Self::set_cursor] must be called before any other methods
    pub const fn new(arch: u32) -> Self {
        Self {
            arch,
            out: [0; Self::SIZE],
            out_cursor: Cursor::default(),
        }
    }

    /// Sets the pointer within the cursor to point at the buffer
    pub const fn set_cursors(&mut self) -> &mut Self {
        self.out_cursor = Cursor::from_mut(&mut self.out);

        self
    }

    /// Partially writes the header, with unfilled checksum/size values
    pub const fn write_header(&mut self) -> &mut Self {
        // write initial header (magic, arch, and 0 for size/checksum)
        self.out_cursor.write_u32(Self::MAGIC);
        self.out_cursor.write_u32(self.arch);
        self.out_cursor.write_u32(0);
        self.out_cursor.write_u32(0);

        self
    }

    /// Writes a given tag to the multiboot header
    pub const fn write_tag(&mut self, tag: &impl ~const HeaderTag) -> &mut Self {
        tag.write_tag(&mut self.out_cursor);

        self
    }

    /// Return the bytes representing multiboot header, setting the size and checksum fields
    pub const fn as_bytes(&mut self) -> [u8; Self::SIZE] {
        let written = self.out_cursor.offset();
        let checksum = (0x100000000 - (Self::MAGIC + self.arch + written as u32) as u64) as u32;

        // now update size and checksum within out
        // size is bytes 8-12
        // checksum is bytes 12-16

        unsafe {
            let out_ptr = self.out.as_mut_ptr();

            core::ptr::copy_nonoverlapping(written.to_ne_bytes().as_ptr(), out_ptr.offset(8), 4);
            core::ptr::copy_nonoverlapping(checksum.to_ne_bytes().as_ptr(), out_ptr.offset(12), 4);
        }

        self.out
    }
}

/// Constructs a multiboot header with the given architecture and (optionally) tags.
///
/// Creates a static `HEADER` variable in the `.multiboot` section
#[macro_export]
macro_rules! multiboot_header {
    (arch: $arch:expr) => {
        use multiboot::prelude::*;

        #[used(linker)]
        #[unsafe(no_mangle)]
        #[unsafe(link_section = ".multiboot")]
        pub static HEADER: [u8; HeaderBuilder::SIZE] =
            HeaderBuilder::new($arch)
                .set_cursors()
                .write_header()
                .write_tag(&DummyTag)
                .as_bytes();
    };
    (
        arch: $arch:expr,
        tags: [
            $( $tag:expr, )*
        ]
    ) => {
        use multiboot::prelude::*;

        #[used(linker)]
        #[unsafe(no_mangle)]
        #[unsafe(link_section = ".multiboot")]
        pub static HEADER: [u8; HeaderBuilder::SIZE] =
            HeaderBuilder::new($arch)
                .set_cursors()
                .write_header()
                $(
                    .write_tag(&$tag)
                )*
                .write_tag(&DummyTag)
                .as_bytes();
    };
}
