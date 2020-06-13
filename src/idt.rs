#[allow(dead_code)]

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::println;
use crate::pic::PIC;
use crate::apic::APIC;
use crate::interrupt_controller::InterruptController;
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt[32].set_handler_fn(timer_handler);
        idt[35].set_handler_fn(ipi_handler);
        idt[39].set_handler_fn(spurious_handler);

        idt.divide_error.set_handler_fn(generic_handler);
        idt.debug.set_handler_fn(generic_handler);
        idt.non_maskable_interrupt.set_handler_fn(generic_handler);
        idt.breakpoint.set_handler_fn(generic_handler);
        idt.overflow.set_handler_fn(generic_handler);
        idt.bound_range_exceeded.set_handler_fn(generic_handler);
        idt.invalid_opcode.set_handler_fn(generic_handler);
        idt.device_not_available.set_handler_fn(generic_handler);
        idt.x87_floating_point.set_handler_fn(generic_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.virtualization.set_handler_fn(generic_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);

        idt.general_protection_fault.set_handler_fn(gpf_handler);

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(pagefault_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn pagefault_handler(
    _stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode)
{
    println!("page fault -> error: {:?}", error_code);
    println!("fault_address {:?}", x86_64::registers::control::Cr2::read());
    println!("page-table address {:?}", x86_64::registers::control::Cr3::read());
    loop {}
}

extern "x86-interrupt" fn double_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    error_code: u64) -> !
{
    println!("double fault -> error: {:?}", error_code);
    println!("fault_address {:?}", x86_64::registers::control::Cr2::read());
    println!("page-table address {:?}", x86_64::registers::control::Cr3::read());
    loop {}
}

extern "x86-interrupt" fn gpf_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64)
{
    println!("gp fault");
    loop {}
}

extern "x86-interrupt" fn ipi_handler(
    _stack_frame: &mut InterruptStackFrame)
{
    println!("IPI handler!");
    APIC::eoi(0);
}

extern "x86-interrupt" fn spurious_handler(
    _stack_frame: &mut InterruptStackFrame)
{
    println!("Spurious handler!");
    loop {}
}

extern "x86-interrupt" fn generic_handler(
    _stack_frame: &mut InterruptStackFrame)
{
    println!("Generic handler!");
    loop {}
}

extern "x86-interrupt" fn machine_check_handler(
    _stack_frame: &mut InterruptStackFrame) -> !
{
    println!("Machine check handler!");
    loop {}
}

extern "x86-interrupt" fn timer_handler(
    _stack_frame: &mut InterruptStackFrame)
{
    println!("-- isr: {}", PIC::isr());
    println!("-- irr: {}", PIC::irr());

    PIC::eoi(0);
    APIC::eoi(0);
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: &mut InterruptStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}
