//! A cursor to allow writing into a buffer while maintaining position.

use core::{ffi::CStr, marker::PhantomData};

/// Supports read operations
pub trait R {}

/// Supports write operations
pub trait W {}

/// Cursor operation mode
pub mod mode {
    /// Read only
    pub struct R;
    impl super::R for R {}

    /// Read + Write
    pub struct RW;
    impl super::R for RW {}
    impl super::W for RW {}
}

/// Wrapper type to support sequential writing operations to a backing buffer type, while storing
/// current offset within it.
pub struct Cursor<'a, MODE = mode::RW> {
    /// Backing data structure, typically constructed from a slice
    backing: *mut u8,
    /// Current offset within the backing data
    offset: usize,
    /// Capacity of backing data
    capacity: usize,
    /// Lifetime information
    phantom: PhantomData<(&'a [u8], MODE)>,
}

/// Read only cursor
pub type CursorR<'a> = Cursor<'a, mode::R>;

/// Helper macro to construct nearly identical write_[type] functions for the cursor type
macro_rules! impl_writes {
    ($($type:ty => $write:ident),*) => {
        $(
            #[doc = concat!("Attempts to write a ", stringify!($type), " to backing buffer, ")]
            #[doc = "returning the number of bytes written (0 if capacity reached)."]
            pub const fn $write(&mut self, value: $type) -> usize {
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
/// Helper macro to construct nearly identical read_[type] functions for the cursor type
macro_rules! impl_reads {
    ($($type:ty => $read:ident),*) => {
        $(
            #[doc = concat!("Attempts to read a ", stringify!($type), " from the backing buffer. ")]
            pub const fn $read(&mut self) -> Option<$type> {
                const SIZE: usize = core::mem::size_of::<$type>();

                if self.offset + SIZE >= self.capacity {
                    return None;
                }

                let mut out = core::mem::MaybeUninit::uninit();

                // SAFETY:
                // backing is safe to read for SIZE bytes, since we check above
                // out is safe to write for SIZE bytes, simple value type
                // data is inherently aligned
                unsafe {
                    core::ptr::copy(
                        self.backing.add(self.offset),
                        out.as_mut_ptr() as *mut u8,
                        SIZE,
                    );

                    self.offset += SIZE;

                    // SAFETY: we know out is initialised because we write to it above
                    Some(out.assume_init())
                }
            }
        )*
    };
}

impl<MODE: R> Cursor<'_, MODE> {
    /// Constructs a cursor from the given slice
    ///
    /// ## Safety
    /// Caller must guarantee cursor is never written to
    pub const unsafe fn from(value: &[u8]) -> Self {
        Self {
            backing: value.as_ptr() as *mut u8,
            offset: 0,
            capacity: value.len(),
            phantom: PhantomData,
        }
    }

    impl_reads! {
        u8 => read_u8,
        u16 => read_u16,
        u32 => read_u32,
        u64 => read_u64,

        i8 => read_i8,
        i16 => read_i16,
        i32 => read_i32,
        i64 => read_i64
    }

    /// Reads a slice from buffer, advancing offset by `len`
    ///
    /// ## Safety
    /// Caller must guarantee that `self.backing[self.offset .. self.offset + len]` is a valid slice
    pub const unsafe fn read_slice(&mut self, len: usize) -> Option<&'static [u8]> {
        if self.offset + len > self.capacity {
            return None;
        }

        let slice = unsafe { core::slice::from_raw_parts(self.backing.add(self.offset), len) };
        self.offset += len;

        Some(slice)
    }

    /// Reads a CStr from buffer, incrementing the offset by the length of the string
    ///
    /// ## Safety
    /// The caller **must** know that the buffer contains a null-terminated string in the selection location.
    pub const unsafe fn read_cstr(&mut self, len: usize) -> Option<&'static CStr> {
        if self.offset + len > self.capacity {
            return None;
        }

        unsafe {
            let slice = core::slice::from_raw_parts(self.backing.add(self.offset), len);
            self.offset += len;

            Some(CStr::from_bytes_with_nul_unchecked(slice))
        }
    }
}

impl<MODE: W> Cursor<'_, MODE> {
    /// Constructs a cursor from the given slice
    pub const fn from_mut(value: &mut [u8]) -> Self {
        Self {
            backing: value.as_mut_ptr(),
            offset: 0,
            capacity: value.len(),
            phantom: PhantomData,
        }
    }

    impl_writes! {
        u8 => write_u8,
        u16 => write_u16,
        u32 => write_u32,
        u64 => write_u64,

        i8 => write_i8,
        i16 => write_i16,
        i32 => write_i32,
        i64 => write_i64
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
}

impl Cursor<'_> {
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

impl<'a, MODE> Cursor<'a, MODE> {
    /// Gets a pointer to the backing array, starting at current offset
    pub const fn as_ptr(&self) -> *const u8 {
        unsafe { self.backing.add(self.offset) }
    }

    /// Gets the current offset within the backing data.
    pub const fn offset(&self) -> usize {
        self.offset
    }

    /// Increments the offset by `value`
    pub const fn increment_offset(&mut self, value: usize) {
        self.offset += value;
    }

    /// Aligns the offset to the next `alignment`, which must be a power of 2
    pub const fn align_offset(&mut self, alignment: usize) {
        self.offset = super::align_up(self.offset, alignment);
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
