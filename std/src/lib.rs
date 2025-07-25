//! Standard library for rustyos

#![no_std]
#![warn(missing_docs, clippy::missing_docs_in_private_items)]

pub mod cursor;
pub mod duration;
pub mod elf;
pub mod mutex;

/// Align downwards - returns the greatest _x_ with alignment `align`
/// such that _x_ <= addr. `align` must be power of 2
pub const fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be power of two")
    }
}

/// Align upwards - returns the smallest _x_ with alignment `align`
/// such that _x_ >= addr. `align` must be power of 2
pub const fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}

/// Checks if an address is aligned to a given boundary
pub const fn is_aligned(addr: usize, alignment: usize) -> bool {
    align_up(addr, alignment) == addr
}
