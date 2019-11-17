#[allow(dead_code)]

use x86_64::instructions::interrupts;
use x86_64::registers::rflags;
use x86_64::registers::rflags::RFlags;

pub struct CPU;

impl CPU {
    pub fn irq_save() -> RFlags {
        let mut flags = rflags::read();
        let backup = flags.clone();

        if flags.contains(RFlags::INTERRUPT_FLAG) {
            flags.remove(RFlags::INTERRUPT_FLAG);
            rflags::write(flags);
        }

        backup
    }

    pub fn irq_restore(flags: RFlags) {
        rflags::write(flags);
    }

    pub fn irq_enable() {
        interrupts::enable();
    }

    pub fn irq_disable() {
        interrupts::disable();
    }
}