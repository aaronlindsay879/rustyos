//! Multiple APIC Description Table

use core::fmt::{Debug, Formatter};
use std::cursor::CursorR;

use crate::tables::header::Header;

/// Multiple APIC Description Table
#[derive(Debug)]
pub struct Madt {
    /// MADT header
    pub header: &'static Header,
    /// Physical address at which each process can access LAPIC
    pub lapic_addr: u32,
    /// Whether 8259 PIC exists and must be disabled before LAPIC can be used
    pub old_pic_exists: bool,
    /// Pointer to first field
    fields: *const MadtField,
    /// Field length in bytes
    field_length: usize,
}

/// Flags:
/// * First two bits: polarity of APIC IO input signals
///     * 00: conforms to specifications of bus (for example, EISA is active-low for level-triggered interrupts)
///     * 01: active high
///     * 10: reserved
///     * 11: active low
/// * Next two bits: trigger mode of APIC IO input signals
///     * 00: conforms to specifications of bus (for example, ISA is edge-triggered)
///     * 01: edge-triggered
///     * 10: reserved
///     * 11: level-triggered
pub struct MpsIntiFlags(u16);

impl MpsIntiFlags {
    /// Polarity of flags:
    /// * 00: conforms to specifications of bus (for example, EISA is active-low for level-triggered interrupts)
    /// * 01: active high
    /// * 10: reserved
    /// * 11: active low
    pub const fn polarity(&self) -> u8 {
        (self.0 & 0b11) as u8
    }

    /// Trigger mode of flags:
    /// * 00: conforms to specifications of bus (for example, ISA is edge-triggered)
    /// * 01: edge-triggered
    /// * 10: reserved
    /// * 11: level-triggered
    pub const fn trigger_mode(&self) -> u8 {
        ((self.0 >> 2) & 0b11) as u8
    }
}

impl Debug for MpsIntiFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "MPS_INTI_FLAGS {{")?;
        writeln!(
            f,
            "\tPolarity: `{}`",
            match self.polarity() {
                0b00 => "Bus specification",
                0b01 => "Edge-triggered",
                0b11 => "Level-triggered",
                _ => "Reserved",
            }
        )?;
        writeln!(
            f,
            "\tTrigger Mode: `{}`",
            match self.trigger_mode() {
                0b00 => "Bus specification",
                0b01 => "Active high",
                0b11 => "Active low",
                _ => "Reserved",
            }
        )?;
        write!(f, "}}")
    }
}

// TODO: i have only implemented the entries present on my actual computer, so rest need to be done
/// Enum representing each potential field within the MADT table
#[repr(u8)]
#[derive(Debug)]
pub enum MadtField {
    /// A processor-local APIC
    ProcessorLocalAPIC {
        /// Length in bytes
        size: u8,
        /// Id of associated processor
        acpi_processor_uid: u8,
        /// Processor's local APIC id
        apic_id: u8,
        /// Flags:
        /// * bit 0: enabled - if bit set, then processor is ready for use
        /// * bit 1: online capable - whether processor can be taken online
        flags: u32,
    } = 0,
    /// An I/O APIC
    IoApic {
        /// Length in bytes
        size: u8,
        /// IO APIC id
        apic_id: u8,
        /// Reserved, must be 0
        _reserved: u8,
        /// 32-bit physical address of where to access the IO APIC
        apic_addr: u32,
        /// The Global System Interrupt number where this IO APIC’s interrupt inputs start.
        global_system_interrupt_base: u32,
    } = 1,
    /// Interrupt source override to describe variances between platform implementation and 8259 implementation
    InterruptSourceOverride {
        /// Length in bytes
        size: u8,
        /// 0 Constant, meaning ISA
        bus: u8,
        /// Bus-relative interrupt source (IRQ)
        source: u8,
        /// The Global System Interrupt that this bus-relative interrupt source will signal.
        global_system_interrupt: u32,
        /// Flags
        flags: MpsIntiFlags,
    } = 2,
    /// Specifies which interrupt inputs should be enabled as non-maskable
    NmiSource {
        /// Length in bytes
        size: u8,
        /// Flags
        flags: MpsIntiFlags,
        /// The Global System Interrupt that this NMI will signal.
        global_system_interrupt: u32,
    } = 3,
    /// This structure describes the Local APIC interrupt input (LINTn) that NMI is connected to for each of the
    /// processors in the system where such a connection exists
    LocalApicNmi {
        /// Length in bytes
        size: u8,
        /// Processor id for connection, 0xFF means all processors
        acpi_processor_uid: u8,
        /// Flags
        flags: MpsIntiFlags,
        /// Local APIC interrupt input LINTn to which NMI is connected
        local_apic_lint: u8,
    } = 4,
}

impl Madt {
    /// Signature of MADT: "APIC"
    pub const SIGNATURE: [u8; 4] = *b"APIC";

    /// Constructs a MADT, assuming it is at the given address
    ///
    /// ## Safety
    /// `addr` must point to a valid MADT.
    /// This function _does_ check it contains a valid APIC signature, but only **after** already reading
    /// the header, so if the pointer is invalid then it will still be UB.
    pub unsafe fn from_addr(addr: usize) -> Option<Self> {
        unsafe {
            let (header, remaining) = Header::from_addr(addr)?;

            // even though we assume caller has checked signature, it doesn't hurt to double check
            if header.signature != Self::SIGNATURE {
                return None;
            }

            let mut cursor = CursorR::from(remaining);

            let lapic_addr = cursor.read_u32()?;
            let flags = cursor.read_u32()?;

            Some(Self {
                header,
                lapic_addr,
                old_pic_exists: flags & 1 != 0,
                fields: cursor.as_ptr() as *const MadtField,
                field_length: header.length as usize - 8,
            })
        }
    }

    /// Gets the table entry at the given index, returning None if out of bounds
    pub fn get_table_entry(&self, index: usize) -> Option<MadtField> {
        let mut cursor = unsafe {
            CursorR::from(core::slice::from_raw_parts(
                self.fields as *const u8,
                self.field_length,
            ))
        };

        for i in 0.. {
            // first two fields are always type (enum discriminant) and size
            let field_type = cursor.read_u8()?;
            let field_size = cursor.read_u8()?;

            // if we're not at the correct index yet, skip this entry
            if index != i {
                cursor.increment_offset(field_size as usize - 2);
                continue;
            }

            // TODO: i have only implemented the entries present on my actual computer, so rest need to be done
            return match field_type {
                0 => Some(MadtField::ProcessorLocalAPIC {
                    size: field_size,
                    acpi_processor_uid: cursor.read_u8()?,
                    apic_id: cursor.read_u8()?,
                    flags: cursor.read_u32()?,
                }),
                1 => Some(MadtField::IoApic {
                    size: field_size,
                    apic_id: cursor.read_u8()?,
                    _reserved: cursor.read_u8()?,
                    apic_addr: cursor.read_u32()?,
                    global_system_interrupt_base: cursor.read_u32()?,
                }),
                2 => Some(MadtField::InterruptSourceOverride {
                    size: field_size,
                    bus: cursor.read_u8()?,
                    source: cursor.read_u8()?,
                    global_system_interrupt: cursor.read_u32()?,
                    flags: MpsIntiFlags(cursor.read_u16()?),
                }),
                3 => Some(MadtField::NmiSource {
                    size: field_size,
                    flags: MpsIntiFlags(cursor.read_u16()?),
                    global_system_interrupt: cursor.read_u32()?,
                }),
                4 => Some(MadtField::LocalApicNmi {
                    size: field_size,
                    acpi_processor_uid: cursor.read_u8()?,
                    flags: MpsIntiFlags(cursor.read_u16()?),
                    local_apic_lint: cursor.read_u8()?,
                }),
                _ => None,
            };
        }

        None
    }
}
