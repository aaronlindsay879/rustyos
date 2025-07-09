//! Information request header tag for requesting certain information to be returned by multiboot2

use std::cursor::Cursor;

use crate::header::{flags::Flags, tag::Tag};

/// Requests certain information within the multiboot2 response.
///
/// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Information-request-header-tag
pub struct InformationRequest {
    /// Flags for tag
    pub flags: Flags,
    /// List of type values that the OS image requests to be present in the returned multiboot2 data structure
    pub requests: &'static [u32],
}

impl const Tag for InformationRequest {
    const TYPE: u16 = 1;

    fn write_to_buffer(&self, buffer: &mut Cursor<'_>) {
        buffer.write_u16(self.flags as u16);

        // size is 8 bytes from type, flags, and size field and then 4 bytes per request
        buffer.write_u32(8 + 4 * self.requests.len() as u32);

        // then write each request to the buffer
        let mut i = 0;
        while i < self.requests.len() {
            buffer.write_u32(self.requests[i]);
            i += 1;
        }
    }
}
