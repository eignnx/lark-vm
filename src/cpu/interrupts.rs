use crate::cpu::{Cpu, MemRw};

use super::regs::Reg;

/// These are addresses in memory where function *pointers* are stored.
#[derive(Clone, Copy)]
#[repr(u16)]
#[expect(non_camel_case_types)]
pub enum Interrupt {
    ILL_INSTR = 0xFFFE, // Illegal Instruction
    DIV_ZERO = 0xFFFC,  // Division by Zero
    KEY_EVENT = 0xFFFA, // Keyboard Event
    TIMER_EXP = 0xFFF8, // Timer Expiration
}

impl Cpu {
    pub fn send_interrupt(&mut self, interrupt: Interrupt) {
        // Disable interrupts.
        self.interrupts_enabled = false;
        // Save the current PC to the K0 register.
        self.regs.set(Reg::K0, self.pc);

        // Jump to the interrupt handler.
        let handler_address = *self.mem.read_s16(interrupt as u16).as_u16();
        self.pc = handler_address;
    }
}
