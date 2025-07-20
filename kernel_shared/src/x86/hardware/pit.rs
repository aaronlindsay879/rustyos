//! Programmable Interval Timer

use crate::io::port::Port;

/// Struct to represent the programmable interval timer
#[allow(unused)]
pub struct ProgrammableIntervalTimer {
    /// Port for channel 0
    channel0_port: Port<u8>,
    /// Port for channel 1
    channel1_port: Port<u8>,
    /// Port for channel 2
    channel2_port: Port<u8>,
    /// Port for controlling operation
    mode_command_register: Port<u8>,
}

impl Default for ProgrammableIntervalTimer {
    fn default() -> Self {
        Self {
            channel0_port: Port::new(0x40),
            channel1_port: Port::new(0x41),
            channel2_port: Port::new(0x42),
            mode_command_register: Port::new(0x43),
        }
    }
}

impl ProgrammableIntervalTimer {
    /// Disables the PIT from sending any interrupts
    pub fn disable_irq(&mut self) {
        unsafe {
            self.mode_command_register.write(0b00111010);
        }
    }
}
