use crate::cpu::regs::Reg;

use crate::cpu::instr::{
    Instr, OpcodeAddr, OpcodeImm, OpcodeOp, OpcodeReg, OpcodeRegImm, OpcodeRegReg, OpcodeRegRegImm,
    OpcodeRegRegReg,
};

use super::stg_loc::StgLoc;

impl Instr {
    /// - `defs` are storage locations that would be overwritten by the instruction `self`.
    /// - `uses` are storage locations that would need to be read from by the instruction `self`.
    pub fn defs_and_uses(&self, defs: &mut impl Extend<StgLoc>, uses: &mut impl Extend<StgLoc>) {
        match *self {
            Instr::O { opcode } => match opcode {
                OpcodeOp::HALT
                | OpcodeOp::NOP
                | OpcodeOp::INRE
                | OpcodeOp::INRD
                | OpcodeOp::KRET => {}
            },

            Instr::A { opcode, offset: _ } => match opcode {
                OpcodeAddr::J => {}
            },

            Instr::I { opcode, imm10: _ } => match opcode {
                OpcodeImm::EXN | OpcodeImm::KCALL => {}
            },

            Instr::R { opcode, reg } => match opcode {
                OpcodeReg::JR => uses.extend([reg.into()]),
                OpcodeReg::MVLO | OpcodeReg::MVHI => defs.extend([reg.into()]),
            },

            Instr::RI {
                opcode,
                reg,
                imm: _,
            } => match opcode {
                OpcodeRegImm::JAL => {
                    let link_reg = reg;
                    defs.extend([link_reg.into()]);
                    defs.extend(Reg::CALLER_SAVED.iter().map(|&r| r.into()));
                }
                OpcodeRegImm::LI => defs.extend([reg.into()]),
                OpcodeRegImm::BT | OpcodeRegImm::BF => uses.extend([reg.into()]),
            },

            Instr::RR { opcode, reg1, reg2 } => match opcode {
                OpcodeRegReg::JRAL => {
                    let (link_reg, jump_addr_reg) = (reg1, reg2);

                    defs.extend([link_reg.into()]);
                    defs.extend(Reg::CALLER_SAVED.iter().map(|&r| r.into()));

                    uses.extend([jump_addr_reg.into()]);
                }
                OpcodeRegReg::MV
                | OpcodeRegReg::NOT
                | OpcodeRegReg::NEG
                | OpcodeRegReg::SEB
                | OpcodeRegReg::TEZ
                | OpcodeRegReg::TNZ => {
                    let (rd, rs) = (reg1, reg2);
                    defs.extend([rd.into()]);
                    uses.extend([rs.into()]);
                }
                OpcodeRegReg::MUL | OpcodeRegReg::MULU | OpcodeRegReg::DIV | OpcodeRegReg::DIVU => {
                    todo!("How to handle $LO/$HI regs?")
                }
            },

            Instr::RRR {
                opcode,
                reg1: rd,
                reg2: rs,
                reg3: rt,
            } => match opcode {
                OpcodeRegRegReg::ADD
                | OpcodeRegRegReg::SUB
                | OpcodeRegRegReg::OR
                | OpcodeRegRegReg::XOR
                | OpcodeRegRegReg::AND
                | OpcodeRegRegReg::ADDU
                | OpcodeRegRegReg::SUBU
                | OpcodeRegRegReg::SHL
                | OpcodeRegRegReg::SHR
                | OpcodeRegRegReg::SHRA
                | OpcodeRegRegReg::TLT
                | OpcodeRegRegReg::TGE
                | OpcodeRegRegReg::TEQ
                | OpcodeRegRegReg::TNE
                | OpcodeRegRegReg::TLTU
                | OpcodeRegRegReg::TGEU => {
                    defs.extend([rd.into()]);
                    uses.extend([rs.into(), rt.into()]);
                }
            },

            Instr::RRI {
                opcode,
                reg1,
                reg2,
                imm10,
            } => match opcode {
                OpcodeRegRegImm::LW | OpcodeRegRegImm::LBS | OpcodeRegRegImm::LBU => {
                    let (rd, src_addr_reg) = (reg1, reg2);
                    defs.extend([rd.into()]);

                    match src_addr_reg {
                        Reg::Sp => {
                            uses.extend([src_addr_reg.into(), StgLoc::stack_var(imm10.as_i16())]);
                        }
                        Reg::Gp => {
                            uses.extend([src_addr_reg.into(), StgLoc::global_var(imm10.as_i16())]);
                        }
                        _ => uses.extend([src_addr_reg.into()]),
                    }
                }
                OpcodeRegRegImm::SW | OpcodeRegRegImm::SB => {
                    let (dest_addr_reg, rs) = (reg1, reg2);
                    uses.extend([dest_addr_reg.into(), rs.into()]);
                    match dest_addr_reg {
                        Reg::Sp => defs.extend([StgLoc::stack_var(imm10.as_i16())]),
                        Reg::Gp => defs.extend([StgLoc::global_var(imm10.as_i16())]),
                        _ => {}
                    }
                }
                OpcodeRegRegImm::ADDI
                | OpcodeRegRegImm::SUBI
                | OpcodeRegRegImm::ORI
                | OpcodeRegRegImm::XORI
                | OpcodeRegRegImm::ANDI => {
                    let (rd, rs) = (reg1, reg2);
                    defs.extend([rd.into()]);
                    uses.extend([rs.into()]);
                }
            },
        }
    }
}
