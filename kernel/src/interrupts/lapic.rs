use core::cell::OnceCell;
use std::mutex::Mutex;

use acpi::tables::fixed::madt::Madt;
use kernel_shared::{mem::PHYS_MEM_OFFSET, x86::hardware::local_apic::LocalApic};

pub static LAPIC: Mutex<OnceCell<LocalApic>> = Mutex::new(OnceCell::new());

pub fn init(madt_table: &Madt) {
    unsafe {
        // set static LAPIC based on address in madt
        LAPIC
            .lock()
            .set(LocalApic::new(
                madt_table.lapic_addr as usize | PHYS_MEM_OFFSET,
            ))
            .unwrap();

        // and then actually enable
        LAPIC
            .lock()
            .get()
            .unwrap()
            .spurious_interrupt_vector_register()
            .set_spurious_vector(0xFF)
            .set_enabled(true);
    }
}
