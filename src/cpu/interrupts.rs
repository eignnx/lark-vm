use crate::cpu::{Cpu, MemRw};

/// These are addresses in memory where function *pointers* are stored.
#[derive(Clone, Copy)]
#[repr(u16)]
pub enum Interrupt {
    ILL_INSTR = 0xFFFF, // Illegal Instruction
    DIV_ZERO = 0xFFFE,  // Division by Zero
    KEY_EVENT = 0xFFFD, // Keyboard Event
    TIMER_EXP = 0xFFFC, // Timer Expiration
}

impl Cpu {
    pub fn send_interrupt(&mut self, interrupt: Interrupt) {
        let handler_address = *self.mem.read_s16(interrupt as u16).as_u16();
        self.pc = handler_address;
    }
}
