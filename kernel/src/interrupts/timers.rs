use std::duration::Duration;

use acpi::tables::fixed::hpet::Hpet as HpetTable;
use kernel_shared::{
    mem::PHYS_MEM_OFFSET,
    x86::hardware::{hpet::Hpet, pit::ProgrammableIntervalTimer},
};

const DESIRED_TIME: Duration = Duration::from_milliseconds(500);

pub fn init(hpet_table: &HpetTable) {
    log::trace!("\t* programming timers");

    let mut pit = ProgrammableIntervalTimer::default();
    pit.disable_irq();
    log::trace!("\t\t* PIT disabled");

    let hpet = unsafe { Hpet::new(hpet_table.address.address as usize | PHYS_MEM_OFFSET) };
    let mut timer = hpet.timer(0).unwrap();

    let clock_period_fs = hpet.capabilities().clock_period() as u64;
    let ticks_required = DESIRED_TIME.as_femtoseconds() as u64 / clock_period_fs;

    timer
        .set_interrupt_routing(0)
        .allow_accumulator_write()
        .set_timer_periodic(true)
        .set_interrupt_enabled(true);

    // need to write twice to update both comparator register and accumulator
    timer.set_comparator_value(hpet.counter_value() + ticks_required);
    timer.set_comparator_value(ticks_required);
    log::trace!(
        "\t\t* HPET timer 0 programmed with interval of {}μs",
        DESIRED_TIME.as_microseconds()
    );

    hpet.configuration().set_enabled(true);
    log::trace!("\t\t* HPET enabled");
}
