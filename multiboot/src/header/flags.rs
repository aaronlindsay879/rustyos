//! Flag field for all header tags

/// Flag field for all header tags
#[repr(u16)]
#[derive(Copy, Clone)]
pub enum Flags {
    /// Bootloader **must** support the requested tag
    Required = 0b0,
    /// Bootloader may support the requested tag
    Optional = 0b1,
}
