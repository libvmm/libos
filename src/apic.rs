#[allow(dead_code)]

use crate::msr::*;
use crate::interrupt_controller::InterruptController;

/* APIC Timer Delivery Mode */
const TIMER_MODE_ONE_SHOT: u32      = 0x0 << 17;
const TIMER_MODE_PERIODIC: u32      = 0x1 << 17;
const TIMER_MODE_TSCDEADLINE: u32   = 0x2 << 17;

/* APIC Delivery Mode */
const DELIVERY_MODE_FIXED: u32  = 0x0 << 8;
const DELIVERY_MODE_LP: u32     = 0x1 << 8;
const DELIVERY_MODE_SMI: u32    = 0x2 << 8;
const DELIVERY_MODE_NMI: u32    = 0x4 << 8;
const DELIVERY_MODE_INIT: u32   = 0x5 << 8;
const DELIVERY_MODE_SIPI: u32   = 0x6 << 8;
const DELIVERY_MODE_EXTINT: u32 = 0x7 << 8;

/* APIC Destination Mode */
const DESTINATION_MODE_PHYSICAL: u32    = 0x0 << 11;
const DESTINATION_MODE_LOGICAL: u32     = 0x1 << 11;

/* APIC Level */
const LEVEL_DEASSERT: u32          = 0x0 << 14;
const LEVEL_ASSERT: u32            = 0x1 << 14;


/* APIC Destination shorthand */
const DESTINATION_SHORTHAND_NONE: u32           = 0x0 << 18;
const DESTINATION_SHORTHAND_SELF: u32           = 0x1 << 18;
const DESTINATION_SHORTHAND_ALL_INC_SELF: u32   = 0x2 << 18;
const DESTINATION_SHORTHAND_ALL_EXC_SELF: u32   = 0x3 << 18;

/* APIC Mask */
const NOT_MASKED: u32   = 0 << 16;
const MASKED: u32       = 1 << 16;

/* APIC Trigger mode */
const TRIGGER_MODE_EDGE: u32    = 0 << 15;
const TRIGGER_MODE_LEVEL: u32   = 1 << 15;

/* APIC registers */
const ID: usize         = 0x20;
const VERSION: usize    = 0x30;
const TPR: usize        = 0x80;
const APR: usize        = 0x90;
const PPR: usize        = 0xA0;
const EOI: usize        = 0xB0;
const RRD: usize        = 0xC0;
const LDR: usize        = 0xD0;
const DFR: usize        = 0xE0;
const SIP: usize        = 0xF0;
const ISR: usize        = 0x100;
const TMR: usize        = 0x180;
const IRR: usize        = 0x200;
const ESR: usize        = 0x280;
const LVT_CMCI: usize   = 0x2f0;
const LVT_TR: usize     = 0x320;
const LVT_TMR: usize    = 0x330;
const LVT_PMCR: usize   = 0x340;
const LVT_LINT0: usize  = 0x350;
const LVT_LINT1: usize  = 0x360;
const LVT_ER: usize     = 0x370;
const ICR0: usize       = 0x300;
const ICR1: usize       = 0x310;
const TIMER_ICR: usize  = 0x380;
const TIMER_CCR: usize  = 0x390;
const TIMER_DCR: usize  = 0x3e0;

/* MSR IA32_APIC_BASE fields */
const BSP_SHIFT: u8      = 8;
const BSP_MASK: u64      = 0x1;

const ENABLE_X2APIC_SHIFT: u8   = 10;
const ENABLE_X2APIC_MASK: u64   = 0x1;

const ENABLE_XAPIC_SHIFT: u8   = 11;
const ENABLE_XAPIC_MASK: u64   = 0x1;

const BASE_SHIFT: u8    = 0;
const BASE_MASK: u64    = 0x7fffff000;

const SPURIOUS_VECTOR: u32 = 39;

#[derive(Clone, Copy)]
pub enum APICLVTEntry {
    APIC_LVT_TIMER,
    APIC_LVT_CMCI,
    APIC_LVT_LINT0,
    APIC_LVT_LINT1,
    APIC_LVT_ERROR,
    APIC_LVT_PMCR,
    APIC_LVT_THERMAL,
}

#[derive(Clone, Copy)]
pub enum APICDeliveryMode {
    APIC_DELIVERY_NA,
    APIC_DELIVERY_FIXED,
    APIC_DELIVERY_LP,
    APIC_DELIVERY_SMI,
    APIC_DELIVERY_NMI,
    APIC_DELIVERY_INIT,
    APIC_DELIVERY_SIPI,
    APIC_DELIVERY_EXTINT,
}

#[derive(Clone, Copy)]
pub enum APICTimerMode {
    APIC_TIMER_NA,
    APIC_TIMER_ONESHOT,
    APIC_TIMER_PERIODIC,
    APIC_TIMER_TSCDEADLINE,
}

#[derive(Clone, Copy)]
pub enum APICLevel {
    APIC_LEVEL_ASSERT,
    APIC_LEVEL_DEASSERT,
    APIC_LEVEL_NA,
}

#[derive(Clone, Copy)]
pub enum APICTriggerMode {
    APIC_TRIGGER_NA,
    APIC_TRIGGER_EDGE,
    APIC_TRIGGER_LEVEL,
}

#[derive(Clone, Copy)]
pub enum APICDestinationMode {
    APIC_DESTINATION_PHYSICAL,
    APIC_DESTINATION_LOGICAL,
    APIC_DESTINATION_NA,
}

#[derive(Clone, Copy)]
pub enum APICDestinationShorthand {
    APIC_DESTINATION_SHORTHAND_NONE,
    APIC_DESTINATION_SHORTHAND_SELF,
    APIC_DESTINATION_SHORTHAND_ALL_INC_SELF,
    APIC_DESTINATION_SHORTHAND_ALL_EXC_SELF,
    APIC_DESTINATION_SHORTHAND_NA,
}

#[derive(Clone, Copy)]
pub struct APICInterrupt {
    vector: u8,
    delivery: APICDeliveryMode,
    destination: APICDestinationMode,
    level: APICLevel,
    trigger: APICTriggerMode,
    dest_shorthand: APICDestinationShorthand,
    timer_mode: APICTimerMode,
    masked: bool,
}

pub struct APIC;

impl APIC {
    pub const ADDRESS: u32 = 0xfee00000;

    fn read32(index: usize) -> Option<u32> {
        if (index & 0xf) != 0 {
            return None;
        }

        return Some(
            unsafe {
                let apic_page: *mut u32 = APIC::ADDRESS as *mut u32;
                apic_page.offset((index >> 2) as isize).read_volatile()
            }
        )
    }

    fn write32(index: usize, value: u32) {
        if (index & 0xf) != 0 {
            return
        }

        unsafe {
            let apic_page: *mut u32 = APIC::ADDRESS as *mut u32;
            apic_page.offset((index >> 2) as isize).write_volatile(value)
        };
    }

    fn update_base(base: u32, xapic: bool, x2apic: bool) {
        let mut value = (base as u64) << BASE_SHIFT;

        if xapic {
            value |= 1 << ENABLE_XAPIC_SHIFT;
        }

        if x2apic {
            value |= 1 << ENABLE_X2APIC_SHIFT;
        }

        unsafe {
            MSR::IA32_APIC_BASE.write(value)
        }
    }

    fn isr_of(offset: usize) -> usize {
        (offset * 0x10) + ISR
    }

    fn tmr_of(offset: usize) -> usize {
        (offset * 0x10) + TMR
    }

    fn irr_of(offset: usize) -> usize {
        (offset * 0x10) + IRR
    }

    fn delivery(delivery: APICDeliveryMode) -> u32 {
        match delivery {
            APICDeliveryMode::APIC_DELIVERY_FIXED => DELIVERY_MODE_FIXED,
            APICDeliveryMode::APIC_DELIVERY_LP => DELIVERY_MODE_LP,
            APICDeliveryMode::APIC_DELIVERY_NMI => DELIVERY_MODE_NMI,
            APICDeliveryMode::APIC_DELIVERY_SMI => DELIVERY_MODE_SMI,
            APICDeliveryMode::APIC_DELIVERY_INIT => DELIVERY_MODE_INIT,
            APICDeliveryMode::APIC_DELIVERY_EXTINT => DELIVERY_MODE_EXTINT,
            APICDeliveryMode::APIC_DELIVERY_SIPI => DELIVERY_MODE_SIPI,
            APICDeliveryMode::APIC_DELIVERY_NA => 0,
        }
    }

    fn destination(destination: APICDestinationMode) -> u32 {
        match destination {
            APICDestinationMode::APIC_DESTINATION_PHYSICAL => DESTINATION_MODE_PHYSICAL,
            APICDestinationMode::APIC_DESTINATION_LOGICAL => DESTINATION_MODE_LOGICAL,
            APICDestinationMode::APIC_DESTINATION_NA => 0,
        }
    }

    fn masked(masked: bool) -> u32 {
        match masked {
            true => MASKED,
            false => NOT_MASKED,
        }
    }

    fn trigger(trigger: APICTriggerMode) -> u32 {
        match trigger {
            APICTriggerMode::APIC_TRIGGER_EDGE => TRIGGER_MODE_EDGE,
            APICTriggerMode::APIC_TRIGGER_LEVEL => TRIGGER_MODE_LEVEL,
            APICTriggerMode::APIC_TRIGGER_NA => 0,
        }
    }

    fn timer_mode(mode: APICTimerMode) -> u32 {
        match mode {
            APICTimerMode::APIC_TIMER_ONESHOT => TIMER_MODE_ONE_SHOT,
            APICTimerMode::APIC_TIMER_PERIODIC => TIMER_MODE_PERIODIC,
            APICTimerMode::APIC_TIMER_TSCDEADLINE => TIMER_MODE_TSCDEADLINE,
            APICTimerMode::APIC_TIMER_NA => 0,
        }
    }

    fn destination_shorthand(shorthand: APICDestinationShorthand) -> u32 {
        match shorthand {
            APICDestinationShorthand::APIC_DESTINATION_SHORTHAND_NONE => DESTINATION_SHORTHAND_NONE,
            APICDestinationShorthand::APIC_DESTINATION_SHORTHAND_SELF => DESTINATION_SHORTHAND_SELF,
            APICDestinationShorthand::APIC_DESTINATION_SHORTHAND_ALL_INC_SELF => DESTINATION_SHORTHAND_ALL_INC_SELF,
            APICDestinationShorthand::APIC_DESTINATION_SHORTHAND_ALL_EXC_SELF => DESTINATION_SHORTHAND_ALL_EXC_SELF,
            APICDestinationShorthand::APIC_DESTINATION_SHORTHAND_NA => 0,
        }
    }

    fn lvt_register(entry: APICLVTEntry) -> usize {
        match entry {
            APICLVTEntry::APIC_LVT_TIMER => LVT_TR,
            APICLVTEntry::APIC_LVT_CMCI => LVT_CMCI,
            APICLVTEntry::APIC_LVT_ERROR => LVT_ER,
            APICLVTEntry::APIC_LVT_LINT0 => LVT_LINT0,
            APICLVTEntry::APIC_LVT_LINT1 => LVT_LINT1,
            APICLVTEntry::APIC_LVT_PMCR => LVT_PMCR,
            APICLVTEntry::APIC_LVT_THERMAL => LVT_TMR,
        }
    }

    fn level(level: APICLevel) -> u32 {
        match level {
            APICLevel::APIC_LEVEL_DEASSERT => LEVEL_DEASSERT,
            APICLevel::APIC_LEVEL_ASSERT => LEVEL_ASSERT,
            APICLevel::APIC_LEVEL_NA => 0x0,
        }
    }

    fn interrupt_entry(interrupt: &APICInterrupt) -> u32 {
        let mut value;

        value = interrupt.vector as u32;
        value |= APIC::delivery(interrupt.delivery);
        value |= APIC::destination(interrupt.destination);
        value |= APIC::level(interrupt.level);
        value |= APIC::masked(interrupt.masked);
        value |= APIC::trigger(interrupt.trigger);
        value |= APIC::timer_mode(interrupt.timer_mode);

        value
    }

    fn update_lvt(interrupt: &APICInterrupt, entry: APICLVTEntry) {
        APIC::write32(APIC::lvt_register(entry), APIC::interrupt_entry(interrupt));
    }

    pub fn set_timer_periodic_mode() {
        let interrupt = APICInterrupt {
            vector: 32,
            delivery: APICDeliveryMode::APIC_DELIVERY_NA,
            destination: APICDestinationMode::APIC_DESTINATION_NA,
            level: APICLevel::APIC_LEVEL_NA,
            masked: false,
            trigger: APICTriggerMode::APIC_TRIGGER_NA,
            dest_shorthand: APICDestinationShorthand::APIC_DESTINATION_SHORTHAND_NA,
            timer_mode: APICTimerMode::APIC_TIMER_PERIODIC
        };

        APIC::update_lvt(&interrupt, APICLVTEntry::APIC_LVT_TIMER);
    }

    pub fn set_timer_period(value: u32) {
        APIC::write32(TIMER_ICR, value);
    }

    pub fn set_timer_oneshot_mode() {
        let interrupt = APICInterrupt {
            vector: 32,
            delivery: APICDeliveryMode::APIC_DELIVERY_NA,
            destination: APICDestinationMode::APIC_DESTINATION_NA,
            level: APICLevel::APIC_LEVEL_NA,
            masked: false,
            trigger: APICTriggerMode::APIC_TRIGGER_NA,
            dest_shorthand: APICDestinationShorthand::APIC_DESTINATION_SHORTHAND_NA,
            timer_mode: APICTimerMode::APIC_TIMER_ONESHOT,
        };

        APIC::update_lvt(&interrupt, APICLVTEntry::APIC_LVT_TIMER);
    }

    pub fn set_timer_oneshot(value: u32) {
        APIC::write32(TIMER_ICR, value);
    }

    pub fn set_timer_tscdeadline_mode() {
        let interrupt = APICInterrupt {
            vector: 32,
            delivery: APICDeliveryMode::APIC_DELIVERY_NA,
            destination: APICDestinationMode::APIC_DESTINATION_NA,
            level: APICLevel::APIC_LEVEL_NA,
            masked: false,
            trigger: APICTriggerMode::APIC_TRIGGER_NA,
            dest_shorthand: APICDestinationShorthand::APIC_DESTINATION_SHORTHAND_NA,
            timer_mode: APICTimerMode::APIC_TIMER_TSCDEADLINE
        };

        APIC::update_lvt(&interrupt, APICLVTEntry::APIC_LVT_TIMER);
    }

    pub fn set_timer_tscdeadline(value: u64) {
        unsafe { MSR::IA32_TSC_DEADLINE.write(value) }
    }

    pub fn self_ipi(ipi: u8) {
        let interrupt = APICInterrupt {
            vector: ipi,
            delivery: APICDeliveryMode::APIC_DELIVERY_FIXED,
            destination: APICDestinationMode::APIC_DESTINATION_PHYSICAL,
            level: APICLevel::APIC_LEVEL_NA,
            trigger: APICTriggerMode::APIC_TRIGGER_NA,
            dest_shorthand: APICDestinationShorthand::APIC_DESTINATION_SHORTHAND_SELF,
            timer_mode: APICTimerMode::APIC_TIMER_NA,
            masked: false,
        };

        APIC::write32(ICR1, 0);
        APIC::write32(ICR0, APIC::interrupt_entry(&interrupt));
    }

    pub fn wake_ap(apic_id: u32, address: u32) {
        let mut interrupt;

        interrupt = APICInterrupt {
            vector: 0,
            delivery: APICDeliveryMode::APIC_DELIVERY_INIT,
            destination: APICDestinationMode::APIC_DESTINATION_PHYSICAL,
            level: APICLevel::APIC_LEVEL_ASSERT,
            trigger: APICTriggerMode::APIC_TRIGGER_LEVEL,
            dest_shorthand: APICDestinationShorthand::APIC_DESTINATION_SHORTHAND_NONE,
            timer_mode: APICTimerMode::APIC_TIMER_NA,
            masked: false,
        };
        APIC::write32(ICR1, apic_id << 24);
        APIC::write32(ICR0, APIC::interrupt_entry(&interrupt));

        interrupt = APICInterrupt {
            vector: 0,
            delivery: APICDeliveryMode::APIC_DELIVERY_INIT,
            destination: APICDestinationMode::APIC_DESTINATION_PHYSICAL,
            level: APICLevel::APIC_LEVEL_DEASSERT,
            trigger: APICTriggerMode::APIC_TRIGGER_LEVEL,
            dest_shorthand: APICDestinationShorthand::APIC_DESTINATION_SHORTHAND_NONE,
            timer_mode: APICTimerMode::APIC_TIMER_NA,
            masked: false,
        };
        APIC::write32(ICR1, apic_id << 24);
        APIC::write32(ICR0, APIC::interrupt_entry(&interrupt));

        /* The CPU is now ready to receive the startup IPI */
        for _ in 0..2 {
            interrupt = APICInterrupt {
                vector: (address >> 12) as u8,
                delivery: APICDeliveryMode::APIC_DELIVERY_SIPI,
                destination: APICDestinationMode::APIC_DESTINATION_PHYSICAL,
                level: APICLevel::APIC_LEVEL_NA,
                trigger: APICTriggerMode::APIC_TRIGGER_EDGE,
                dest_shorthand: APICDestinationShorthand::APIC_DESTINATION_SHORTHAND_NONE,
                timer_mode: APICTimerMode::APIC_TIMER_NA,
                masked: false,
            };
            APIC::write32(ICR1, apic_id << 24);
            APIC::write32(ICR0, APIC::interrupt_entry(&interrupt));
        }
    }

    pub fn id() -> u32 {
        APIC::read32(ID).unwrap() >> 24
    }

    pub fn start() {
        //APIC::set_timer_oneshot_mode();
        //APIC::set_timer_oneshot(0x5fffffff);

        APIC::set_timer_periodic_mode();
        APIC::set_timer_period(0xffffff);

        //APIC::set_timer_tscdeadline_mode();
        //APIC::set_timer_tscdeadline(TSC::read() + 0x5fffffff);
    }
}

impl InterruptController for APIC {
    fn enable() {
        /* no support for x2apic for now */
        let x2apic = false;

        if x2apic {
            APIC::update_base(APIC::ADDRESS, true, true);
        } else {
            APIC::update_base(APIC::ADDRESS, true, false);
        }
    }

    fn disable() {
        APIC::update_base(APIC::ADDRESS, true, false);
    }

    fn reset() {
        APIC::disable();

        APIC::write32(DFR, 0xFFFFFFFF);
        let ldr = (APIC::read32(LDR).unwrap() & 0x0FFFFFF) | 1;
        APIC::write32(LDR, ldr);

        let masked_interrupt = APICInterrupt {
            vector: 0,
            timer_mode: APICTimerMode::APIC_TIMER_NA,
            destination: APICDestinationMode::APIC_DESTINATION_NA,
            level: APICLevel::APIC_LEVEL_NA,
            trigger: APICTriggerMode::APIC_TRIGGER_NA,
            dest_shorthand: APICDestinationShorthand::APIC_DESTINATION_SHORTHAND_NA,
            delivery: APICDeliveryMode::APIC_DELIVERY_NA,
            masked: true,
        };

        for entry in &[ APICLVTEntry::APIC_LVT_TIMER,
            APICLVTEntry::APIC_LVT_ERROR,
            APICLVTEntry::APIC_LVT_THERMAL,
            APICLVTEntry::APIC_LVT_PMCR,
            APICLVTEntry::APIC_LVT_LINT0,
            APICLVTEntry::APIC_LVT_LINT1] {
            APIC::update_lvt(&masked_interrupt, *entry);
        }

        APIC::write32(ESR, 0);
        APIC::write32(ESR, 0);
        APIC::write32(TPR, 0);
        APIC::write32(SIP, SPURIOUS_VECTOR | 0x100);

        APIC::eoi(0);
    }

    fn eoi(_irq: u32) {
        APIC::write32(EOI, 0);
    }

    fn spurious_irq() -> u32 {
        SPURIOUS_VECTOR
    }

    fn mask(_irq: u32) {

    }

    fn unmask(_irq: u32) {

    }
}