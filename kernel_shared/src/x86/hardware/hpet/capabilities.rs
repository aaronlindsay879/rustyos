//! HPET general capabilities

/// General capabilities of HPET
pub struct Capabilities {
    /// Pointer to register
    register: *mut u64,
}

impl Capabilities {
    /// Constructs capabilities register from the given base address
    ///
    /// ## Safety
    /// Base address must be the valid base address to HPET structure
    pub const unsafe fn from_base_addr(base_addr: usize) -> Self {
        Self {
            register: base_addr as *mut u64,
        }
    }

    /// Revision of HPET
    pub fn revision_id(&self) -> u8 {
        unsafe { core::ptr::read_volatile(self.register) as u8 }
    }

    /// Number of timers
    pub fn timer_count(&self) -> u8 {
        // register returns number - 1, so need to increment
        unsafe { ((core::ptr::read_volatile(self.register) >> 8) & 0xF) as u8 + 1 }
    }

    /// Whether the counter is 64 bits (false = 32 bits)
    pub fn counter_is_64bits(&self) -> bool {
        unsafe { core::ptr::read_volatile(self.register) & (1 << 13) != 0 }
    }

    /// Whether legacy IRQ routing is supported
    pub fn supports_legacy_routing(&self) -> bool {
        unsafe { core::ptr::read_volatile(self.register) & (1 << 15) != 0 }
    }

    /// Vendor ID
    pub fn vendor_id(&self) -> u16 {
        unsafe { (core::ptr::read_volatile(self.register) >> 16) as u16 }
    }

    /// Clock period in femtoseconds
    pub fn clock_period(&self) -> u32 {
        unsafe { (core::ptr::read_volatile(self.register) >> 32) as u32 }
    }
}
