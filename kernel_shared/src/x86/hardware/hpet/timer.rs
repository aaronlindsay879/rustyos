//! Structs for programming an individual HPET timer

/// An individual HPET timer
#[allow(unused)]
pub struct Timer {
    /// Register for querying capabilities and changing config
    configuration_capability_register: *mut u64,
    /// Register for the comparator value
    comparator_register: *mut u64,
    /// Register for configuring FSB interrupts
    fsb_interrupt_register: *mut u64,
}

impl Timer {
    /// Constructs a timer from the given base address
    ///
    /// ## Safety
    /// Base address must be the valid base address to HPET structure
    pub const unsafe fn from_base_addr(base_addr: usize, timer_number: u8) -> Self {
        let base_addr = base_addr | (0x100 + 0x20 * timer_number as usize);

        Self {
            configuration_capability_register: base_addr as *mut u64,
            comparator_register: (base_addr | 0x08) as *mut u64,
            fsb_interrupt_register: (base_addr | 0x10) as *mut u64,
        }
    }

    /// Returns if the timer interrupts are edge-triggered
    pub fn is_level_triggered(&self) -> bool {
        let config = unsafe { core::ptr::read_volatile(self.configuration_capability_register) };

        config & (1 << 1) != 0
    }

    /// Sets if the timer interrupts are edge-triggered
    pub fn set_level_triggered(&mut self, level_triggered: bool) -> &mut Self {
        let config = unsafe { core::ptr::read_volatile(self.configuration_capability_register) };
        let config = (config & !(1 << 1)) | ((level_triggered as u64) << 1);

        unsafe { core::ptr::write_volatile(self.configuration_capability_register, config) }

        self
    }

    /// Returns if the timer interrupts are enabled
    pub fn is_interrupt_enabled(&self) -> bool {
        let config = unsafe { core::ptr::read_volatile(self.configuration_capability_register) };

        config & (1 << 2) != 0
    }

    /// Sets if the timer interrupts are enabled
    pub fn set_interrupt_enabled(&mut self, interrupt_enabled: bool) -> &mut Self {
        let config = unsafe { core::ptr::read_volatile(self.configuration_capability_register) };
        let config = (config & !(1 << 2)) | ((interrupt_enabled as u64) << 2);

        unsafe { core::ptr::write_volatile(self.configuration_capability_register, config) }

        self
    }

    /// Returns if the timer interrupts are periodic
    pub fn is_timer_periodic(&self) -> bool {
        let config = unsafe { core::ptr::read_volatile(self.configuration_capability_register) };

        config & (1 << 3) != 0
    }

    /// Sets if the timer interrupts are periodic
    pub fn set_timer_periodic(&mut self, periodic: bool) -> &mut Self {
        let config = unsafe { core::ptr::read_volatile(self.configuration_capability_register) };
        let config = (config & !(1 << 3)) | ((periodic as u64) << 3);

        unsafe { core::ptr::write_volatile(self.configuration_capability_register, config) }

        self
    }

    /// Allows the next write to the accumulator directly
    pub fn allow_accumulator_write(&mut self) -> &mut Self {
        let config = unsafe { core::ptr::read_volatile(self.configuration_capability_register) };
        let config = config | (1 << 6);

        unsafe { core::ptr::write_volatile(self.configuration_capability_register, config) }

        self
    }

    /// Sets the interrupt routing for IO APIC
    pub fn set_interrupt_routing(&mut self, route: u8) -> &mut Self {
        assert!(route < 32);

        let config = unsafe { core::ptr::read_volatile(self.configuration_capability_register) };
        let config = (config & !(0b11111 << 9)) | (((route & 0b11111) as u64) << 9);

        unsafe { core::ptr::write_volatile(self.configuration_capability_register, config) }

        self
    }

    /// Reads the current comparator value
    pub fn get_comparator_value(&self) -> u64 {
        unsafe { core::ptr::read_volatile(self.comparator_register) }
    }

    /// Sets the current comparator value
    pub fn set_comparator_value(&mut self, value: u64) -> &mut Self {
        unsafe { core::ptr::write_volatile(self.comparator_register, value) }

        self
    }
}
