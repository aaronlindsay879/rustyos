//! Code for handling frames of memory

use core::iter::Step;

/// Size of a frame in bytes
pub const FRAME_SIZE: usize = 4096;

/// A single physical frame of memory
#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct Frame {
    /// The number (**NOT** address) of the frame
    pub number: usize,
}

impl Frame {
    /// Returns the frame which contains a given address
    pub fn containing_address(address: usize) -> Self {
        Self {
            number: address / FRAME_SIZE,
        }
    }

    /// Returns the start address of the frame
    pub fn start_address(&self) -> usize {
        self.number * FRAME_SIZE
    }
}

impl Step for Frame {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        let steps = end.number - start.number;

        (steps, Some(steps))
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        let number = start.number.checked_add(count)?;
        Some(Frame { number })
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        let number = start.number.checked_sub(count)?;
        Some(Frame { number })
    }
}
