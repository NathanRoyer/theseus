//! CPU Interface, GICv3 style
//!
//! Included functionality:
//! - Initializing the CPU interface
//! - Setting and getting the minimum interrupt priority
//! - Acknowledging interrupt requests
//! - Sending End-Of-Interrupts signals
//! - Generating software interrupts

use core::arch::asm;
use super::IpiTargetCpu;
use super::Priority;
use super::InterruptNumber;

const SGIR_TARGET_ALL_OTHER_PE: u64 = 1 << 40;
const IGRPEN_ENABLED: u64 = 1;

/// Enables routing of group 1 interrupts for the current CPU and configures
/// the end-of-interrupt mode
pub fn init() {
    let mut icc_ctlr: u64;

    unsafe { asm!("mrs {}, ICC_CTLR_EL1", out(reg) icc_ctlr) };
    // clear bit 1 (EOIMode) so that eoi signals both
    // priority drop & interrupt deactivation
    icc_ctlr &= !0b10;
    unsafe { asm!("msr ICC_CTLR_EL1, {}", in(reg) icc_ctlr) };

    // Enable Group 0
    // bit 0 = group 0 enable
    // unsafe { asm!("msr ICC_IGRPEN0_EL1, {}", in(reg) IGRPEN_ENABLED) };

    // Enable Groupe 1 (non-secure)
    // bit 0 = group 1 (non-secure) enable
    unsafe { asm!("msr ICC_IGRPEN1_EL1, {}", in(reg) IGRPEN_ENABLED) };
}

/// Interrupts have a priority; if their priority
/// is lower or equal to this one, they're queued
/// until this CPU or another one is ready to handle
/// them
pub fn get_minimum_priority() -> Priority {
    let mut reg_value: u64;
    unsafe { asm!("mrs {}, ICC_PMR_EL1", out(reg) reg_value) };
    u8::MAX - (reg_value as u8)
}

/// Interrupts have a priority; if their priority
/// is lower or equal to this one, they're queued
/// until this CPU or another one is ready to handle
/// them
pub fn set_minimum_priority(priority: Priority) {
    let reg_value = (u8::MAX - priority) as u64;
    unsafe { asm!("msr ICC_PMR_EL1, {}", in(reg) reg_value) };
}

/// Signals to the controller that the currently processed interrupt has
/// been fully handled, by zeroing the current priority level of this CPU.
/// This implies that the CPU is ready to process interrupts again.
pub fn end_of_interrupt(int: InterruptNumber) {
    let reg_value = int as u64;
    unsafe { asm!("msr ICC_EOIR1_EL1, {}", in(reg) reg_value) };
}

/// Acknowledge the currently serviced interrupt
/// and fetches its number; this tells the GIC that
/// the requested interrupt is being handled by
/// this CPU.
pub fn acknowledge_interrupt() -> (InterruptNumber, Priority) {
    let int_num: u64;
    let priority: u64;

    // Reading the interrupt number has the side effect
    // of acknowledging the interrupt.
    unsafe {
        asm!("mrs {}, ICC_IAR1_EL1", out(reg) int_num);
        asm!("mrs {}, ICC_RPR_EL1", out(reg) priority);
    }

    let int_num = int_num & 0xffffff;
    let priority = priority & 0xff;
    (int_num as InterruptNumber, priority as u8)
}

pub fn send_ipi(int_num: InterruptNumber, target: IpiTargetCpu) {
    let mut value = match target {
        IpiTargetCpu::Specific(cpu) => {
            let mpidr: cpu::MpidrValue = cpu.into();

            // level 3 affinity in bits [48:55]
            let aff3 = mpidr.affinity(cpu::AffinityShift::LevelThree) << 48;

            // level 2 affinity in bits [32:39]
            let aff2 = mpidr.affinity(cpu::AffinityShift::LevelTwo) << 32;

            // level 1 affinity in bits [16:23]
            let aff1 = mpidr.affinity(cpu::AffinityShift::LevelOne) << 16;

            // level 0 affinity as a GICv2-style target list
            let aff0 = mpidr.affinity(cpu::AffinityShift::LevelZero);
            let target_list = if aff0 >= 16 {
                panic!("[GIC driver] cannot send an IPI to a core with Aff0 >= 16");
            } else {
                1 << aff0
            };
            aff3 | aff2 | aff1 | target_list
        },
        // bit 31: Interrupt Routing Mode
        // value of 1 to target any available cpu
        IpiTargetCpu::AllOtherCpus => SGIR_TARGET_ALL_OTHER_PE,
        IpiTargetCpu::GICv2TargetList(_) => {
            panic!("Cannot use IpiTargetCpu::GICv2TargetList with GICv3!");
        },
    };

    value |= (int_num as u64) << 24;
    unsafe { asm!("msr ICC_SGI1R_EL1, {}", in(reg) value) };
}