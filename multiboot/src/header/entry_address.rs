//! Entry address header tag for informing multiboot2 about entrypoint

use core::marker::PhantomData;
use std::cursor::Cursor;

use crate::header::{flags::Flags, header_tag::HeaderTag};

/// Trait to indicate different possible entry address types.
pub trait EntryAddressType {
    /// Multiboot2 type field for given entry address type
    const TYPE_FIELD: u16;
}

/// Standard entry address: use whenever possible.
pub struct Standard;
impl EntryAddressType for Standard {
    const TYPE_FIELD: u16 = 3;
}

/// This tag is taken into account only on EFI i386 platforms when Multiboot2 image header contains EFI boot services tag.
/// Then entry point specified in ELF header and the entry address tag of Multiboot2 header are ignored.
pub struct I386;
impl EntryAddressType for I386 {
    const TYPE_FIELD: u16 = 8;
}

/// This tag is taken into account only on EFI amd64 platforms when Multiboot2 image header contains EFI boot services tag.
/// Then entry point specified in ELF header and the entry address tag of Multiboot2 header are ignored.
pub struct Amd64;
impl EntryAddressType for Amd64 {
    const TYPE_FIELD: u16 = 9;
}

/// Informs multiboot2 where to jump to start running the operating system.
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#The-entry-address-tag-of-Multiboot2-header
pub struct EntryAddress<E: EntryAddressType> {
    /// Flags for tag
    pub flags: Flags,
    /// The physical address which contains the start point of OS image
    pub entry_addr: u32,
    /// Type information
    pub _phantom: PhantomData<E>,
}

impl<E: EntryAddressType> const HeaderTag for EntryAddress<E> {
    const TYPE: u16 = E::TYPE_FIELD;

    fn write_to_buffer(&self, buffer: &mut Cursor) {
        buffer.write_u16(self.flags as u16);
        buffer.write_u32(12); // size = 12

        // then write the address
        buffer.write_u32(self.entry_addr);
    }
}
