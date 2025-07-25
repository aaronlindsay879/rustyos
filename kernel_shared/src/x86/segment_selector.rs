//! Code for reading and manipulating segment selector

use core::arch::asm;

use crate::x86::PrivilegeLevel;

/// Segment selector register
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    /// Reads the current CS register
    pub fn read_cs() -> SegmentSelector {
        let val: u16;

        unsafe {
            asm!("mov {:x}, cs", out(reg) val, options(nostack, preserves_flags));
        }

        SegmentSelector(val)
    }

    /// Writes a new value to CS register
    ///
    /// # Safety
    /// `self` must be a valid segment selector to write to `CS`
    pub unsafe fn write_cs(&self) {
        unsafe {
            asm!(
            "push {sel}",
            "lea {tmp}, [3f + rip]",
            "push {tmp}",
            "retfq",
            "3:",
            sel = in(reg) u64::from(self.0),
            tmp = lateout(reg) _,
            options(preserves_flags),
            );
        }
    }

    /// # Safety
    /// `self` must be a valid segment selector to write to `SS`
    pub unsafe fn write_ss(&self) {
        unsafe {
            asm!("mov ss, {:x}", in(reg) self.0);
        }
    }

    /// Writes a new value to SS register
    ///
    /// # Safety
    /// `self` must be a valid segment selector to load as tss.
    pub unsafe fn load_tss(&self) {
        unsafe {
            asm!("ltr {0:x}", in(reg) self.0, options(nostack, preserves_flags));
        }
    }

    /// Constructs a new segment selector value with the given index and privilege level
    pub const fn new(index: u16, rpl: PrivilegeLevel) -> Self {
        Self(index << 3 | (rpl as u16))
    }

    /// Returns the GDT index
    pub const fn index(&self) -> u16 {
        self.0 >> 3
    }

    /// Returns the requested privilege level
    pub const fn rpl(&self) -> PrivilegeLevel {
        PrivilegeLevel::from_u16(self.0 & 0b11)
    }
}
