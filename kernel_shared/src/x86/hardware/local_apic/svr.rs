//!Spurious-Interrupt Vector Register (SVR)

/// Spurious-Interrupt Vector Register (SVR)
pub struct SpuriousInterruptVectorRegister {
    /// Pointer to register
    register: *mut u32,
}

impl SpuriousInterruptVectorRegister {
    /// Constructs a SVR from the given base address
    ///
    /// ## Safety
    /// Base address must be the valid base address to a local APIC structure
    pub const unsafe fn from_base_addr(base_addr: usize) -> Self {
        Self {
            register: (base_addr | 0xF0) as *mut u32,
        }
    }

    /// Sets the spurious vector field
    pub fn set_spurious_vector(&mut self, vector: u8) -> &mut Self {
        unsafe {
            let value = core::ptr::read_volatile(self.register);
            let value = (value & !0xFF) | (vector as u32);

            core::ptr::write_volatile(self.register, value);
        }

        self
    }

    /// Sets whether the LAPIC is enabled
    pub fn set_enabled(&mut self, enabled: bool) -> &mut Self {
        unsafe {
            let value = core::ptr::read_volatile(self.register);
            let value = (value & !(1 << 8)) | ((enabled as u32) << 8);

            core::ptr::write_volatile(self.register, value);
        }

        self
    }
}
