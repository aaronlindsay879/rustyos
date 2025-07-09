//! A cursor to allow writing into a buffer while maintaining position.

use core::marker::PhantomData;

/// Wrapper type to support sequential writing operations to a backing buffer type, while storing
/// current offset within it.
pub struct Cursor<'a> {
    /// Backing data structure, typically constructed from a slice
    backing: *mut u8,
    /// Current offset within the backing data
    offset: usize,
    /// Capacity of backing data
    capacity: usize,
    /// Lifetime information
    phantom: PhantomData<&'a [u8]>,
}

/// Helper macro to construct nearly identical write_[type] functions for the cursor type
macro_rules! write_functions {
    ($($name:ident: $type:ty),*) => {
        $(
            #[doc = concat!("Attempts to write a ", stringify!($type), " to backing buffer, ")]
            #[doc = "returning the number of bytes written (0 if capacity reached)."]
            pub const fn $name(&mut self, value: $type) -> usize {
                const SIZE: usize = core::mem::size_of::<$type>();

                if self.offset + SIZE >= self.capacity {
                    return 0;
                }

                // SAFETY:
                // value is safe to read for SIZE bytes, since it's simply a value type
                // check above guarantees that backing has room to write that many bytes
                // data is inherently aligned
                unsafe {
                    core::ptr::copy(
                        value.to_ne_bytes().as_ptr(),
                        self.backing.add(self.offset),
                        SIZE,
                    );
                }
                self.offset += SIZE;

                SIZE
            }
        )*
    }
}

impl Cursor<'_> {
    /// Constructs a cursor from the given slice
    pub const fn from(value: &mut [u8]) -> Self {
        Self {
            backing: value.as_mut_ptr(),
            offset: 0,
            capacity: value.len(),
            phantom: PhantomData,
        }
    }

    /// Constructs a default cursor, with no backing data structure and a capacity of 0
    pub const fn default() -> Self {
        Self {
            backing: core::ptr::null_mut(),
            offset: 0,
            capacity: 0,
            phantom: PhantomData,
        }
    }
}

impl<'a> Cursor<'a> {
    write_functions! {
        write_u8: u8,
        write_u16: u16,
        write_u32: u32,
        write_u64: u64,

        write_i8: i8,
        write_i16: i16,
        write_i32: i32,
        write_i64: i64
    }

    /// Attempts to write an entire slice to the cursor, returning number of bytes successfully written.
    pub const fn write_slice(&mut self, value: &[u8]) -> usize {
        if self.offset + value.len() > self.capacity {
            return 0;
        }

        // SAFETY:
        // value is safe to read for len bytes, since we can assume it's a standard safe slice
        // check above guarantees that backing has room to write that many bytes
        // data is inherently aligned
        unsafe {
            core::ptr::copy(value.as_ptr(), self.backing.add(self.offset), value.len());
        }
        self.offset += value.len();

        value.len()
    }

    /// Gets the current offset within the backing data.
    pub const fn offset(&self) -> usize {
        self.offset
    }

    /// Resets the offset back to the start of the backing data.
    pub const fn reset_offset(&mut self) {
        self.offset = 0;
    }
}

impl AsRef<[u8]> for Cursor<'_> {
    fn as_ref(&self) -> &[u8] {
        // SAFETY:
        // cursor can only be initialised with backing pointing to a standard, safe slice
        // offset will always be below the capacity of that slice as it can only be incremented in other safe methods
        // rust lifetime rules will prevent the backing data being mutated as long as this slice lives
        unsafe { core::slice::from_raw_parts(self.backing, self.offset) }
    }
}
