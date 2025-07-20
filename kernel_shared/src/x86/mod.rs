//! Wrapper functions for x86 intrinsics

pub mod descriptor_table_pointer;
pub mod exception;
pub mod gdt;
pub mod hardware;
pub mod idt;
pub mod registers;
pub mod segment_selector;
pub mod tss;

use core::arch::asm;

use crate::x86::registers::CpuFlags;

/// Privilege level
pub enum PrivilegeLevel {
    /// Ring 0 (kernel)
    Ring0 = 0,
    /// Ring 1 (unused)
    Ring1,
    /// Ring 2 (unused)
    Ring2,
    /// Ring 3 (userspace
    Ring3,
}

impl PrivilegeLevel {
    /// Constructs a privilege level from an integer
    pub const fn from_u16(pl: u16) -> Self {
        match pl {
            0 => Self::Ring0,
            1 => Self::Ring1,
            2 => Self::Ring2,
            3 => Self::Ring3,
            _ => panic!("invalid privilege level"),
        }
    }
}

/// Invalidates a given address in the TLB
pub fn invalidate_address(addr: usize) {
    unsafe {
        asm!(
        "invlpg [{}]",
        in(reg) addr as u64,
        options(nostack, preserves_flags)
        )
    }
}

/// Halts execution
pub fn halt() -> ! {
    loop {
        unsafe { asm!("hlt") }
    }
}

/// Returns true if interrupts are enabled
pub fn are_interrupts_enabled() -> bool {
    CpuFlags::read().contains(CpuFlags::INTERRUPT_FLAG)
}

/// Enable interrupts
pub fn enable_interrupts() {
    unsafe {
        asm!("sti", options(nostack, preserves_flags));
    }
}

/// Disable interrupts
pub fn disable_interrupts() {
    unsafe {
        asm!("cli", options(nostack, preserves_flags));
    }
}

/// Run closure with interrupts disabled, re-enabling them afterwards if they were enabled before
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let saved_input_flag = are_interrupts_enabled();

    if saved_input_flag {
        disable_interrupts();
    }

    let ret = f();

    if saved_input_flag {
        enable_interrupts();
    }

    ret
}
