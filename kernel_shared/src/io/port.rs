//! Code for interacting with ports

use core::{arch::asm, marker::PhantomData};

use paste::paste;

/// Trait to indicate a given type can be read from a port
pub trait PortRead {
    /// Reads a `Self` value from the given port.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    unsafe fn read_from_port(port: u16) -> Self;
}

/// Trait to indicate a given type can be written to a port
pub trait PortWrite {
    /// Writes a `Self` value to the given port.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    unsafe fn write_to_port(port: u16, value: Self);
}

/// Helper macro to define repetitive code for read/write port operations
macro_rules! port_definition {
    ($reg:expr, $type:ty) => {
        paste! {
            impl crate::io::port::PortRead for $type {
                unsafe fn read_from_port(port: u16) -> $type {
                    let value: $type;

                    unsafe {
                        asm!(
                            concat!("in ", $reg, ", dx"),
                            out($reg) value,
                            in("dx") port,
                            options(nomem, nostack, preserves_flags)
                        );
                    }

                    value
                }
            }

            impl crate::io::port::PortWrite for $type {
                unsafe fn write_to_port(port: u16, value: $type) {
                    unsafe {
                        asm!(
                            concat!("out dx, ", $reg),
                            in("dx") port,
                            in($reg) value,
                            options(nomem, nostack, preserves_flags)
                        )
                    }
                }
            }
        }
    };
}

port_definition!("al", u8);
port_definition!("ax", u16);
port_definition!("eax", u32);

/// A port that operates with the type `T`
pub struct Port<T> {
    /// Address of port to read/write from
    port: u16,
    /// Type information
    _phantom: PhantomData<T>,
}

impl<T> Port<T> {
    /// Constructs a port with the given memory address
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            _phantom: PhantomData,
        }
    }
}

impl<T: PortRead> Port<T> {
    /// Reads a `T` value from the given port.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    pub unsafe fn read(&mut self) -> T {
        unsafe { T::read_from_port(self.port) }
    }
}

impl<T: PortWrite> Port<T> {
    /// Writes a `T` value to the given port.
    ///
    /// ## Safety
    ///
    /// This function is unsafe because the I/O port could have side effects that violate memory
    /// safety.
    pub unsafe fn write(&mut self, value: T) {
        unsafe { T::write_to_port(self.port, value) }
    }
}
