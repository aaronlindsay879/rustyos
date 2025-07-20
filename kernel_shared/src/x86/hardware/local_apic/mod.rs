//! Code for handling a processor-local APIC

use crate::x86::hardware::local_apic::svr::SpuriousInterruptVectorRegister;

pub mod svr;

/// Local apic at known address
#[derive(Debug)]
pub struct LocalApic {
    /// Address of start of local APIC
    base_addr: usize,
}

impl LocalApic {
    /// Constructs a new local APIC chip struct from the information at the given address
    ///
    /// ## Safety
    /// `base_addr` **must** point to the base address of a local APIC chip
    pub const unsafe fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    /// Signals that an interrupt has been handled
    pub fn end_of_interrupt(&mut self) {
        unsafe {
            core::ptr::write_volatile((self.base_addr | 0xB0) as *mut u32, 0);
        }
    }

    /// Returns a struct for modifying the Spurious Interrupt Vector Register
    pub const fn spurious_interrupt_vector_register(&self) -> SpuriousInterruptVectorRegister {
        unsafe { SpuriousInterruptVectorRegister::from_base_addr(self.base_addr) }
    }
}
