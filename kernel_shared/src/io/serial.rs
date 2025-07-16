//! Module for sending data across a serial connection

use core::fmt::Write;

use crate::io::port::Port;

/// Module containing constants for serial ports
#[allow(missing_docs)]
pub mod ports {
    use std::mutex::Mutex;

    use crate::io::serial::SerialPort;

    pub static COM1: Mutex<SerialPort<0x3F8>> = Mutex::new(SerialPort);
    pub static COM2: Mutex<SerialPort<0x2F8>> = Mutex::new(SerialPort);
    pub static COM3: Mutex<SerialPort<0x3E8>> = Mutex::new(SerialPort);
    pub static COM4: Mutex<SerialPort<0x2E8>> = Mutex::new(SerialPort);
    pub static COM5: Mutex<SerialPort<0x5F8>> = Mutex::new(SerialPort);
    pub static COM6: Mutex<SerialPort<0x4F8>> = Mutex::new(SerialPort);
    pub static COM7: Mutex<SerialPort<0x5E8>> = Mutex::new(SerialPort);
    pub static COM8: Mutex<SerialPort<0x4E8>> = Mutex::new(SerialPort);
}

pub use ports::*;

use crate::x86::without_interrupts;

/// Waits until `self` contains the OUTPUT_EMPTY flag
macro_rules! wait_for_output_empty {
    ($self:expr) => {
        while !$self.line_status().contains(LineStatusFlags::OUTPUT_EMPTY) {
            core::hint::spin_loop()
        }
    };
}

/// Wrapper type for a port with serial functionality
pub struct SerialPort<const PORT: u16>;

impl<const PORT: u16> SerialPort<PORT> {
    /// Initialises the port as a serial port.
    ///
    /// ## Safety
    /// The caller must guarantee the port is a valid serial port which will not cause
    /// undefined behaviour when written to or read from.
    pub unsafe fn init(&mut self) {
        unsafe {
            // disable interrupts while initialising port
            self.port_int_enable().write(0x00);

            // enable DLAB
            self.port_line_ctrl().write(0x80);

            // set divisor to 3 for baud rate of 38400
            self.port_data().write(0x03);
            self.port_int_enable().write(0x00);

            // disable DLAB and set data word length to 8 bits
            self.port_line_ctrl().write(0x03);

            // enable FIFO, clear queues, and set interrupt watermark at 14 bytes
            self.port_fifo_control().write(0xC7);

            // mark data terminal ready, signal request to send
            // and enable output #2 (interrupt line)
            self.port_modem_ctrl().write(0x0B);

            // enable interrupts
            self.port_int_enable().write(0x01);
        }
    }

    /// Sends a byte down the serial port
    ///
    /// ## Safety
    /// The caller must guarantee the port is a valid serial port which will not cause
    /// undefined behaviour when written to or read from.
    pub unsafe fn send(&mut self, data: u8) {
        unsafe {
            match data {
                8 | 0x7F => {
                    // special code to handle backspace
                    wait_for_output_empty!(self);
                    self.port_data().write(8);

                    wait_for_output_empty!(self);
                    self.port_data().write(b' ');

                    wait_for_output_empty!(self);
                    self.port_data().write(8);
                }
                _ => {
                    // otherwise just send data
                    wait_for_output_empty!(self);
                    self.port_data().write(data);
                }
            }
        }
    }

    /// R+W data port
    const fn port_data(&self) -> Port<u8> {
        Port::new(PORT)
    }

    /// W interrupt enable port
    const fn port_int_enable(&self) -> Port<u8> {
        Port::new(PORT + 1)
    }

    /// W fifo control port
    const fn port_fifo_control(&self) -> Port<u8> {
        Port::new(PORT + 2)
    }

    /// W line control port
    const fn port_line_ctrl(&self) -> Port<u8> {
        Port::new(PORT + 3)
    }

    /// W modem control port
    const fn port_modem_ctrl(&self) -> Port<u8> {
        Port::new(PORT + 4)
    }

    /// R port line status
    const fn port_line_status(&self) -> Port<u8> {
        Port::new(PORT + 5)
    }

    /// Line status
    ///
    /// ## Safety
    /// The caller must guarantee the port is a valid serial port which will not cause
    /// undefined behaviour when written to or read from.
    unsafe fn line_status(&self) -> LineStatusFlags {
        unsafe { LineStatusFlags::from_bits_truncate(self.port_line_status().read()) }
    }
}

impl<const PORT: u16> Write for SerialPort<PORT> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            unsafe {
                self.send(byte);
            }
        }

        Ok(())
    }
}

/// Flags for line status
struct LineStatusFlags(u8);

impl LineStatusFlags {
    /// Input full
    const INPUT_FULL: u8 = 0b0000_0001;
    /// Output empty
    const OUTPUT_EMPTY: u8 = 0b0010_0000;

    /// Construct from bits, discarding any unknown flags
    pub const fn from_bits_truncate(bits: u8) -> Self {
        Self(bits & (Self::INPUT_FULL | Self::OUTPUT_EMPTY))
    }

    /// Checks if `self` contains the given flags
    pub const fn contains(&self, flags: u8) -> bool {
        self.0 & flags != 0
    }
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    without_interrupts(|| {
        COM1.lock()
            .write_fmt(args)
            .expect("Printing to serial failed")
    });
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::io::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n\r"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n\r")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n\r"), $($arg)*));
}
