//! Code for manipulating and using Task State Segments

use core::fmt::Display;

/// A task state segment
#[derive(Debug)]
#[repr(C, packed(4))]
pub struct TaskStateSegment {
    /// Reserved
    _reserved1: u32,
    /// Table of stacks for different privilege levels
    pub privilege_stack_table: [usize; 3],
    /// Reserved
    _reserved2: u64,
    /// Table of stacks for different interrupts
    pub interrupt_stack_table: [usize; 7],
    /// Reserved
    _reserved3: u64,
    /// Reserved
    _reserved4: u16,
    /// I/O map base address
    pub base_addr: u16,
}

impl Default for TaskStateSegment {
    fn default() -> Self {
        Self {
            privilege_stack_table: [0; 3],
            interrupt_stack_table: [0; 7],
            base_addr: size_of::<Self>() as u16,
            _reserved1: 0,
            _reserved2: 0,
            _reserved3: 0,
            _reserved4: 0,
        }
    }
}

impl Display for TaskStateSegment {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let privilege_table = self.privilege_stack_table;
        let interrupt_table = self.interrupt_stack_table;

        writeln!(f, "Task state segment:")?;
        writeln!(f, "\tPrivilege stack table: {privilege_table:?}")?;
        writeln!(f, "\tInterrupt stack table: {interrupt_table:?}")?;
        writeln!(f, "\tIomap base addr: {:#X}", self.base_addr)?;

        Ok(())
    }
}
