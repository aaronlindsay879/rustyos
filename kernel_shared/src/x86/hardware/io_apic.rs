//! Code for programming an I/O APIC chip

use core::fmt::{Display, Formatter};

/// Struct containing information about an I/O APIC chip
#[derive(Debug)]
pub struct IoApic {
    /// Base address of IOAPIC
    base_addr: usize,
    /// How many IRQs this IOAPIC can handle - 1
    max_redirection_entry: u8,
}

impl IoApic {
    /// Constructs a new I/O APIC chip struct from the information at the given address
    ///
    /// ## Safety
    /// `base_addr` **must** point to the base address of an I/O APIC chip
    pub unsafe fn new(base_addr: usize) -> Self {
        unsafe {
            let io_reg_sel = base_addr as *mut u32;
            let io_reg = (base_addr + 0x10) as *mut u32;

            core::ptr::write_volatile(io_reg_sel, 1);
            let max_redirection_entry = core::ptr::read_volatile(io_reg);

            // now mask out the info we want
            let max_redirection_entry = (max_redirection_entry >> 16) & 0xFF;

            Self {
                base_addr,
                max_redirection_entry: max_redirection_entry as u8,
            }
        }
    }

    /// Gets the redirection entry for the given `irq_number`, returning None if out of bounds
    pub fn get_redirection_entry(&mut self, irq_number: u8) -> Option<RedirectionEntry> {
        if irq_number > self.max_redirection_entry {
            return None;
        }

        let io_reg_sel = self.base_addr as *mut u32;
        let io_reg = (self.base_addr + 0x10) as *mut u32;

        unsafe {
            core::ptr::write_volatile(io_reg_sel, 0x10 + (irq_number * 2) as u32);
            let low = core::ptr::read_volatile(io_reg);

            core::ptr::write_volatile(io_reg_sel, 0x10 + (irq_number * 2) as u32 + 1);
            let high = core::ptr::read_volatile(io_reg);

            Some(RedirectionEntry { low, high })
        }
    }

    /// Sets the redirection entry for the given `irq_number`, returning None if out of bounds
    pub fn set_redirection_entry(&mut self, irq_number: u8, entry: RedirectionEntry) -> Option<()> {
        if irq_number > self.max_redirection_entry {
            return None;
        }

        let io_reg_sel = self.base_addr as *mut u32;
        let io_reg = (self.base_addr + 0x10) as *mut u32;

        unsafe {
            core::ptr::write_volatile(io_reg_sel, 0x10 + (irq_number * 2) as u32);
            core::ptr::write_volatile(io_reg, entry.low);

            core::ptr::write_volatile(io_reg_sel, 0x10 + (irq_number * 2) as u32 + 1);
            core::ptr::write_volatile(io_reg, entry.high);
        }

        Some(())
    }

    /// Modifies the redirection entry at the given number
    pub fn modify_redirection_entry<F: FnOnce(&mut RedirectionEntry)>(
        &mut self,
        irq_number: u8,
        entry_fn: F,
    ) -> Option<()> {
        let mut entry = self.get_redirection_entry(irq_number)?;
        entry_fn(&mut entry);

        self.set_redirection_entry(irq_number, entry)
    }

    /// Sets the mask for a given redirection entry
    pub fn mask_redirection_entry(&mut self, irq_number: u8, mask: bool) -> Option<()> {
        self.modify_redirection_entry(irq_number, |entry| {
            entry.set_mask(mask);
        })
    }
}

/// A single redirection entry for the IO APIC
#[derive(Debug, Default)]
pub struct RedirectionEntry {
    /// Low 32 bits
    low: u32,
    /// High 32 bits
    high: u32,
}

impl RedirectionEntry {
    /// Gets the interrupt vector that will be raised on the CPU
    pub fn get_interrupt_vector(&self) -> u8 {
        (self.low & 0xFF) as u8
    }

    /// Sets the interrupt vector that will be raised on the CPU
    pub fn set_interrupt_vector(&mut self, vector: u8) -> &mut Self {
        self.low = (self.low & !0xFF) | (vector as u32);

        self
    }

    /// Gets the delivery mode of the interrupt
    pub fn get_delivery_mode(&self) -> DeliveryMode {
        unsafe { core::mem::transmute(((self.low >> 8) & 0b111) as u8) }
    }

    /// Sets the delivery mode of the interrupt
    pub fn set_delivery_mode(&mut self, mode: DeliveryMode) -> &mut Self {
        self.low = (self.low & !(0b111 << 8)) | ((mode as u32) << 8);

        self
    }

    /// Gets the destination mode of the interrupt
    pub fn get_destination_mode(&self) -> DestinationMode {
        unsafe { core::mem::transmute(((self.low >> 11) & 0b1) as u8) }
    }

    /// Sets the destination mode of the interrupt
    pub fn set_destination_mode(&mut self, mode: DestinationMode) -> &mut Self {
        self.low = (self.low & !(0b11 << 11)) | ((mode as u32) << 11);

        self
    }

    /// Gets whether the IRQ is currently relaxed (not waiting for IRQ to be delivered)
    pub fn get_irq_relaxed(&self) -> bool {
        (self.low >> 12) & 1 == 0
    }

    /// Sets whether the IRQ is currently relaxed (not waiting for IRQ to be delivered)
    pub fn set_irq_relaxed(&mut self, value: bool) -> &mut Self {
        self.low = (self.low & !(1 << 12)) | (((!value) as u32) << 12);

        self
    }

    /// Gets whether the IRQ is active high (so false = active low)
    pub fn get_active_high(&self) -> bool {
        self.low >> 13 == 0
    }

    /// Sets whether the IRQ is active high (so false = active low)
    pub fn set_active_high(&mut self, value: bool) -> &mut Self {
        self.low = (self.low & !(1 << 13)) | (((!value) as u32) << 13);

        self
    }

    /// Gets whether the IRQ is edge triggered (so false = level triggered)
    pub fn get_edge_triggered(&self) -> bool {
        (self.low >> 15) & 1 == 0
    }

    /// Sets whether the IRQ is edge triggered (so false = level triggered)
    pub fn set_edge_triggered(&mut self, value: bool) -> &mut Self {
        self.low = (self.low & !(1 << 15)) | (((!value) as u32) << 15);

        self
    }

    /// Gets whether the IRQ is currently masked out
    pub fn get_mask(&self) -> bool {
        (self.low >> 16) & 1 == 1
    }

    /// Sets whether the IRQ is currently masked out
    pub fn set_mask(&mut self, value: bool) -> &mut Self {
        self.low = (self.low & !(1 << 16)) | ((value as u32) << 16);

        self
    }

    /// Gets the destination of the IRQ, where the meaning depends on destination mode
    pub fn get_destination(&self) -> u8 {
        ((self.high >> 24) & 0xFF) as u8
    }

    /// Sets the destination of the IRQ, where the meaning depends on destination mode
    pub fn set_destination(&mut self, destination: u8) -> &mut Self {
        self.high = (self.high & !(0xFF << 24)) | ((destination as u32) << 24);

        self
    }
}

impl Display for RedirectionEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Redirection entry:")?;
        writeln!(f, "\tIRQ vector: {:#X}", self.get_interrupt_vector())?;
        writeln!(f, "\tDelivery mode: {:?}", self.get_delivery_mode())?;
        writeln!(f, "\tDestination Mode: {:?}", self.get_destination_mode())?;
        writeln!(f, "\tIRQ relaxed: {}", self.get_irq_relaxed())?;
        writeln!(f, "\tActive High: {}", self.get_active_high())?;
        writeln!(f, "\tEdge Triggered: {}", self.get_edge_triggered())?;
        writeln!(f, "\tMask: {}", self.get_mask())?;
        write!(f, "\tDestination: {:#b}", self.get_destination())
    }
}

/// The delivery mode of the interrupt
#[repr(u8)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum DeliveryMode {
    /// Signal is delivered on the INTR signal of all CPUs
    Fixed = 0b000,
    /// Signal is delivered on the INTR signal of CPU with the lowest priority
    LowestPriority = 0b001,
    /// System Management Interrupt
    SMI = 0b010,
    /// Non Maskable interrupt
    NMI = 0b100,
    /// Signal is delivered by asserting INIT signal on all CPUs
    INIT = 0b101,
    /// Signal is delievered on the INTR signal of all CPUs, listed as an interrupt that originated in external
    /// interrupt controller
    ExtINT = 0b111,
}

/// The destination mode of the interrupt
#[repr(u8)]
#[derive(Debug)]
pub enum DestinationMode {
    /// Destination is an APIC id
    Physical = 0,
    /// Destination is a set of processors
    Logical,
}
