//! Code for representing exceptions

use core::fmt::Display;

use crate::x86::{registers::CpuFlags, segment_selector::SegmentSelector};

/// Stack frame for an exception
#[derive(Debug)]
#[repr(C)]
pub struct ExceptionStackFrame {
    /// Instruction pointer at time of exception
    pub instruction_pointer: u64,
    /// Code segment at time of exception
    pub code_segment: SegmentSelector,
    /// Reserved
    _reserved1: [u8; 6],
    /// Cpu flags at time of exception
    pub cpu_flags: CpuFlags,
    /// Stack pointer at time of exception
    pub stack_pointer: u64,
    /// Stack segment at time of exception
    pub stack_segment: SegmentSelector,
    /// Reserved
    _reserved2: [u8; 6],
}

impl Display for ExceptionStackFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Exception stack frame:")?;
        writeln!(f, "\tInstruction pointer: {:#X}", self.instruction_pointer)?;
        writeln!(f, "\tCode segment: {:?}", self.code_segment)?;
        writeln!(f, "\tCpu flags: {}", self.cpu_flags)?;
        writeln!(f, "\tStack pointer: {:#X}", self.stack_pointer)?;
        writeln!(f, "\tStack segment: {:?}", self.stack_segment)?;

        Ok(())
    }
}
