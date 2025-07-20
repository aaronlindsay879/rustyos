//! HPET general configuration

/// HPET general configuration
pub struct Configuration {
    /// Pointer to register
    register: *mut u64,
}

impl Configuration {
    /// Constructs a configuration register from the given base address
    ///
    /// ## Safety
    /// Base address must be the valid base address to HPET structure
    pub const unsafe fn from_base_addr(base_addr: usize) -> Self {
        Self {
            register: (base_addr | 0x10) as *mut u64,
        }
    }

    /// Gets whether the HPET is enabled
    pub fn get_enabled(&mut self) -> bool {
        unsafe { core::ptr::read_volatile(self.register) & 1 != 0 }
    }

    /// Sets whether the HPET is enabled
    pub fn set_enabled(&mut self, enabled: bool) -> &mut Self {
        unsafe {
            let value = core::ptr::read_volatile(self.register);
            let value = (value & !1u64) | (enabled as u64);

            core::ptr::write_volatile(self.register, value);
        }

        self
    }

    /// Gets whether the HPET is using legacy routing
    pub fn get_legacy_routing(&mut self) -> bool {
        unsafe { core::ptr::read_volatile(self.register) & 0b10 != 0 }
    }

    /// Sets whether the HPET is using legacy routing
    pub fn set_legacy_routing(&mut self, enabled: bool) -> &mut Self {
        unsafe {
            let value = core::ptr::read_volatile(self.register);
            let value = (value & !(&0b10u64)) | ((enabled as u64) << 1);

            core::ptr::write_volatile(self.register, value);
        }

        self
    }
}
