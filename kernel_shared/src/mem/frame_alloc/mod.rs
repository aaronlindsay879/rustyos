//! Code for handling allocating physical frames

pub mod bitmap;

use crate::mem::frame::Frame;

/// A trait for a type which is capable of allocating and deallocating physical frames
pub trait FrameAllocator {
    /// Allocates a frame, returning None if not possible
    fn allocate_frame(&mut self) -> Option<Frame>;

    /// Deallocates a frame, freeing it for future use
    fn deallocate_frame(&mut self, frame: Frame);
}
