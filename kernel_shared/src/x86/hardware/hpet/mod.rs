//! High Precision Interval Timer

use crate::x86::hardware::hpet::{
    capabilities::Capabilities, configuration::Configuration, timer::Timer,
};

pub mod capabilities;
pub mod configuration;
mod timer;

/// HPET at known address
#[derive(Debug)]
pub struct Hpet {
    /// Address of start of HPET registers
    base_addr: usize,
}

impl Hpet {
    /// Constructs a new HPET struct from the information at the given address
    ///
    /// ## Safety
    /// `base_addr` **must** point to the base address of HPET registers
    pub const unsafe fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    /// Returns a struct for reading the capabilities of the HPET hardware
    pub const fn capabilities(&self) -> Capabilities {
        unsafe { Capabilities::from_base_addr(self.base_addr) }
    }

    /// Returns a struct for modifying the configuration of the HPET hardware
    pub const fn configuration(&self) -> Configuration {
        unsafe { Configuration::from_base_addr(self.base_addr) }
    }

    /// Returns a struct for configuring a specific timer, returning None if out of bounds
    pub fn timer(&self, timer_number: u8) -> Option<Timer> {
        if timer_number < self.capabilities().timer_count() {
            unsafe { Some(Timer::from_base_addr(self.base_addr, timer_number)) }
        } else {
            None
        }
    }

    /// Gets the current counter value
    pub fn counter_value(&self) -> u64 {
        unsafe { core::ptr::read_volatile((self.base_addr | 0xF0) as *const u64) }
    }
}
