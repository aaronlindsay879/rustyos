mod ioapic;
mod lapic;
mod pic_8259;
mod timers;

use acpi::tables::fixed::{hpet::Hpet, madt::Madt};
use bitflags::bitflags;
use kernel_shared::x86::{
    enable_interrupts, exception::ExceptionStackFrame, halt, idt::InterruptDescriptorTable,
    registers::CR2,
};
use lazy_static::lazy_static;

use crate::{
    gdt,
    interrupts::{lapic::LAPIC, pic_8259::PICS},
};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::default();

        idt.divide_error.set(divide_by_zero_handler);
        idt.breakpoint.set(breakpoint_handler);
        idt.invalid_opcode.set(invalid_opcode_handler);
        idt.page_fault.set(page_fault_handler);
        idt.general_protection_fault
            .set(general_protection_fault_handler);
        unsafe {
            idt.double_fault
                .set(double_fault)
                .set_ist_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt[0x20].set(timer_interrupt_handler);

        idt
    };
}

extern "x86-interrupt" fn divide_by_zero_handler(stack_frame: ExceptionStackFrame) {
    log::error!("EXCEPTION: DIVIDE BY ZERO\n{stack_frame}");

    halt();
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: ExceptionStackFrame) {
    log::error!(
        "EXCEPTION: INVALID OPCODE at {:#X}\n{}",
        stack_frame.instruction_pointer,
        stack_frame
    );

    halt();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: ExceptionStackFrame) {
    log::warn!(
        "EXCEPTION: BREAKPOINT at {:#X}\n{}",
        stack_frame.instruction_pointer,
        stack_frame
    );
}

extern "x86-interrupt" fn double_fault(stack_frame: ExceptionStackFrame, err: u64) -> ! {
    log::error!("DOUBLE FAULT with err {err}\n{stack_frame}");
    panic!("\nDOUBLE FAULT with err {}\n{}", err, stack_frame);
}

bitflags! {
    #[derive(Debug)]
    struct PageFaultErrorCode: u64 {
        const PROTECTION_VIOLATION = 1 << 0;
        const CAUSED_BY_WRITE = 1 << 1;
        const USER_MODE = 1 << 2;
        const MALFORMED_TABLE = 1 << 3;
        const INSTRUCTION_FETCH = 1 << 4;
    }
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: ExceptionStackFrame, error_code: u64) {
    log::error!(
        "EXCEPTION: PAGE FAULT while accessing {:#X}\
        \nerror code: {:?}\n{}",
        CR2::read(),
        PageFaultErrorCode::from_bits(error_code).unwrap(),
        stack_frame
    );

    halt();
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: ExceptionStackFrame) {
    log::trace!("timer interrupt.");

    LAPIC.lock().get_mut().unwrap().end_of_interrupt();
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: ExceptionStackFrame,
    error_code: u64,
) {
    log::error!(
        "EXCEPTION: GENERAL PROTECTION FAULT while accessing {:#X}\
        \nerror code: {:?}\n{}",
        CR2::read(),
        error_code,
        stack_frame
    );

    halt();
}

pub fn init(madt_table: &Madt, hpet_table: &Hpet) {
    log::trace!("initialising interrupts");

    IDT.load();
    log::trace!("\t* loaded IDT");

    // disable 8259 PIC
    unsafe {
        // disable all 8259 PIC interrupts by fully masking
        PICS.lock().write_masks(0xFF, 0xFF);

        // even though we're disabling the 8259s with the masking above, we still need to initialise them
        // in order to remap correctly.
        //
        // this is important so that if any spurious interrupts are generated, they are mapped to correct
        // interrupts instead of CPU assuming they are exceptions.
        PICS.lock().init();
    }
    log::trace!("\t* 8259 PICs disabled");

    lapic::init(madt_table);
    log::trace!("\t* LAPIC enabled");

    ioapic::init(madt_table);
    log::trace!("\t* IOAPIC programmed");

    timers::init(hpet_table);
    log::trace!("\t* timers programmed");

    enable_interrupts();
    log::trace!("\t* enabled interrupts");
    log::trace!("interrupts initialised");
}
