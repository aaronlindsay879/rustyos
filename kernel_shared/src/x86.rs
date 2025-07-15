//! Wrapper functions for x86 intrinsics

use core::arch::asm;

use crate::mem::frame::Frame;

/// CR3 register
pub struct CR3;

impl CR3 {
    /// Reads the frame and flags from CR3 register
    pub fn read() -> (Frame, u16) {
        let val: u64;

        unsafe {
            asm!("mov {}, cr3", out(reg) val, options(nostack, preserves_flags));
        }

        let addr = val & 0x_000F_FFFF_FFFF_F000;
        let frame = Frame::containing_address(addr as usize);

        (frame, (val & 0xFFF) as u16)
    }

    /// Writes the provided frame and flags to CR3 register
    ///
    /// # Safety
    /// `frame` and `flags` must be valid to write to `CR3`.
    pub unsafe fn write(frame: Frame, flags: u16) {
        let addr = frame.start_address();
        let val = addr as u64 | flags as u64;

        unsafe {
            asm!("mov cr3, {}", in(reg) val, options(nostack, preserves_flags));
        }
    }

    /// Invalidate the TLB by reloading the CR3 register
    pub fn flush_tlb() {
        unsafe {
            asm!(
            "mov {temp:r}, cr3",
            "mov cr3, {temp:r}",
            temp = out(reg) _,
            options(nostack, preserves_flags)
            )
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
