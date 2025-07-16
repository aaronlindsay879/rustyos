//! Code for constructing descriptor table pointers

use core::{arch::asm, marker::PhantomData};

use crate::x86::{gdt::GlobalDescriptorTable, idt::InterruptDescriptorTable};

/// Trait for constructing pointers to descriptor tables
pub trait IntoDescriptorTable
where
    Self: Sized,
{
    /// Constructs a pointer to the descriptor table
    fn as_dtr(&'static self) -> DescriptorTablePointer<Self>;
}

impl IntoDescriptorTable for InterruptDescriptorTable {
    fn as_dtr(&'static self) -> DescriptorTablePointer<Self> {
        DescriptorTablePointer {
            base: self as *const _ as u64,
            limit: size_of::<Self>() as u16,
            phantom: PhantomData {},
        }
    }
}

impl IntoDescriptorTable for GlobalDescriptorTable {
    fn as_dtr(&'static self) -> DescriptorTablePointer<GlobalDescriptorTable> {
        DescriptorTablePointer {
            base: self.table.as_ptr() as u64,
            limit: (self.len * size_of::<u64>() - 1) as u16,
            phantom: PhantomData {},
        }
    }
}

/// A pointer to a descriptor table
#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
pub struct DescriptorTablePointer<T> {
    /// Size of the DT.
    limit: u16,
    /// Pointer to the memory region containing the DT.
    base: u64,
    /// Type information
    phantom: PhantomData<T>,
}

impl DescriptorTablePointer<InterruptDescriptorTable> {
    /// Loads the given descriptor table as an interrupt descriptor table
    ///
    /// # Safety
    /// Descriptor table must point to a valid interrupt descriptor table
    pub unsafe fn load_idt(self) {
        unsafe {
            asm!("lidt [{}]", in(reg) &self, options(readonly, nostack, preserves_flags));
        }
    }
}

impl DescriptorTablePointer<GlobalDescriptorTable> {
    /// Loads the given descriptor table as an global descriptor table
    ///
    /// # Safety
    /// Descriptor table must point to a valid global descriptor table
    pub unsafe fn load_gdt(self) {
        unsafe {
            asm!("lgdt [{}]", in(reg) &self, options(readonly, nostack, preserves_flags));
        }
    }
}
