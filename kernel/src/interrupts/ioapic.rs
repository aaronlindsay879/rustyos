use acpi::tables::fixed::madt::{Madt, MadtField};
use kernel_shared::{
    mem::PHYS_MEM_OFFSET,
    x86::hardware::io_apic::{DeliveryMode, DestinationMode, IoApic, RedirectionEntry},
};

pub fn init(madt_table: &Madt) {
    let mut io_apic = find_ioapic(madt_table).expect("no IOAPIC detected!");
    log::trace!("\t* IO APIC found");

    let mut timer_idx = 0;
    let mut table_idx = 0;

    while let Some(table) = madt_table.get_table_entry(table_idx) {
        if let MadtField::InterruptSourceOverride {
            size: _,
            bus: _,
            source,
            global_system_interrupt,
            flags,
        } = table
        {
            // set up each redirection entry from interrupt source override tables, but without enabling yet
            let mut redirection_entry = RedirectionEntry::default();
            redirection_entry
                .set_interrupt_vector(source + 32)
                .set_delivery_mode(DeliveryMode::Fixed)
                .set_destination_mode(DestinationMode::Physical)
                .set_irq_relaxed(true)
                .set_active_high(flags.polarity() <= 1)
                .set_edge_triggered(flags.trigger_mode() <= 1)
                .set_mask(true)
                .set_destination(0);

            // IRQ source 0 is timer, so we've found it
            if source == 0 {
                timer_idx = global_system_interrupt;
            }

            log::trace!("\t\t* setting IO APIC redirect {global_system_interrupt} -> {source}");
            io_apic.set_redirection_entry(global_system_interrupt as u8, redirection_entry);
        }

        table_idx += 1;
    }

    // also configure keyboard
    io_apic.modify_redirection_entry(1, |entry| {
        entry
            .set_interrupt_vector(1 + 32)
            .set_irq_relaxed(true)
            .set_mask(false)
            .set_active_high(true)
            .set_edge_triggered(true);
    });
    log::trace!("\t\t* setting IO APIC keyboard redirect");

    // and enable timer
    io_apic.mask_redirection_entry(timer_idx as u8, false);
    log::trace!("\t\t* enabling IO APIC timer redirect");
}

fn find_ioapic(madt_table: &Madt) -> Option<IoApic> {
    let mut io_apic = None;
    let mut table_idx = 0;

    // simply iterate through madt table until we find io apic entry
    while let Some(table) = madt_table.get_table_entry(table_idx) {
        if let MadtField::IoApic {
            size: _,
            apic_id: _,
            _reserved: _,
            apic_addr,
            global_system_interrupt_base: _,
        } = table
        {
            io_apic = Some(unsafe { IoApic::new(apic_addr as usize | PHYS_MEM_OFFSET) });
            break;
        }

        table_idx += 1;
    }

    io_apic
}
