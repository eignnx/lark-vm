//! `dex` stands for Decode+Execute. It implements the CPU's instruction set.

use bitvec::prelude::*;

use crate::{cpu::decode, log_instr, utils::s16};

use super::{
    instr::{
        Instr, OpcodeAddr, OpcodeImm, OpcodeOp, OpcodeReg, OpcodeRegImm, OpcodeRegReg,
        OpcodeRegRegImm, OpcodeRegRegReg,
    },
    regs::Reg,
    Cpu, Signal,
};

#[derive(Debug)]
pub enum DexErr {
    RegDecodeError(decode::DecodeErr),
    InvalidOpcode(u8),
}

impl From<decode::DecodeErr> for DexErr {
    fn from(err: decode::DecodeErr) -> Self {
        DexErr::RegDecodeError(err)
    }
}

impl Cpu {
    pub fn decode_and_execute(&mut self) -> Result<(), DexErr> {
        let ir = self.ir.view_bits::<Msb0>();
        let instr = Instr::from_bits(ir)?;
        let size = instr.instr_size();

        match instr {
            Instr::O { opcode } => match opcode {
                OpcodeOp::HALT => {
                    self.log(log_instr!([size] halt));
                    self.breakpoint();
                    self.signal(Signal::Halt);
                }
                OpcodeOp::NOP => {
                    self.log(log_instr!([size] nop));
                    self.breakpoint();
                    self.pc += 1;
                }
                OpcodeOp::KRET => {
                    self.log(log_instr!([size] kret));
                    self.breakpoint();
                    // Re-enable interrupts.
                    self.interrupts_enabled = true;
                    // Restore the PC from the K0 register.
                    self.pc = self.regs.get(Reg::K0);
                }
                OpcodeOp::INRE => {
                    self.log(log_instr!([size] inre));
                    self.breakpoint();
                    self.interrupts_enabled = true;
                    self.pc += 1;
                }
                OpcodeOp::INRD => {
                    self.log(log_instr!([size] inrd));
                    self.breakpoint();
                    self.interrupts_enabled = false;
                    self.pc += 1;
                }
            },

            Instr::A { opcode, offset } => match opcode {
                OpcodeAddr::J => {
                    self.log(log_instr!([size] j offset));
                    self.breakpoint();
                    self.pc = (self.pc as i32)
                        .checked_add(offset as i32)
                        .expect("Jump address overflow") as u16;
                }
            },

            Instr::I { opcode, imm10 } => match opcode {
                OpcodeImm::EXN => {
                    self.log(log_instr!([size] exn imm10));
                    self.breakpoint();
                    self.handle_exn(imm10);
                    self.pc += size;
                }
                OpcodeImm::KCALL => unimplemented!(),
            },

            Instr::R { opcode, reg } => match opcode {
                OpcodeReg::JR => {
                    let rs = reg;
                    self.log(log_instr!([size] jr rs));
                    self.breakpoint();
                    self.pc = self.regs.get(rs);
                }
                OpcodeReg::MVLO => {
                    let rd = reg;
                    self.log(log_instr!([size] mvlo rd));
                    self.breakpoint();
                    self.regs.set(rd, self.lo);
                    self.pc += size;
                }
                OpcodeReg::MVHI => {
                    let rd = reg;
                    self.log(log_instr!([size] mvhi rd));
                    self.breakpoint();
                    self.regs.set(rd, self.hi);
                    self.pc += size;
                }
            },

            Instr::RI { opcode, reg, imm } => match opcode {
                OpcodeRegImm::JAL => {
                    // Jump and link.
                    // Example: jal $rd, ADDR
                    let (rd, offset) = (reg, imm.as_i16());
                    self.log(log_instr!([size] jal rd, offset));
                    self.breakpoint();
                    self.regs.set(rd, self.pc + size);
                    self.pc = (self.pc as i32)
                        .checked_add(offset as i32)
                        .expect("Jump address overflow") as u16;
                }
                OpcodeRegImm::BT => {
                    let (rs, addr_offset) = (reg, imm.as_i16());
                    self.log(log_instr!([size] bt rs, addr_offset));
                    self.breakpoint();
                    if self.regs.get(rs) {
                        self.pc = (self.pc as i32 + addr_offset as i32) as u16;
                    } else {
                        self.pc += size;
                    }
                }
                OpcodeRegImm::BF => {
                    let (rs, addr_offset) = (reg, imm.as_i16());
                    self.log(log_instr!([size] bf rs, addr_offset));
                    self.breakpoint();
                    if !self.regs.get::<bool>(rs) {
                        self.pc = (self.pc as i32 + addr_offset as i32) as u16;
                    } else {
                        self.pc += size;
                    }
                }
                OpcodeRegImm::LI => {
                    let (rd, simm16) = (reg, imm.as_i16());
                    self.log(log_instr!([size] li rd, simm16));
                    self.breakpoint();
                    self.regs.set(rd, simm16);
                    self.pc += size;
                }
            },

            Instr::RR { opcode, reg1, reg2 } => match opcode {
                OpcodeRegReg::JRAL => {
                    // Jump register and link.
                    // Example: jral $rd, $rs
                    //                |    |
                    //          save pc    jump address
                    let rd = reg1;
                    let rs = reg2;
                    self.log(log_instr!([size] jral rd, rs));
                    self.breakpoint();
                    self.regs.set(rd, self.pc + size);
                    self.pc = self.regs.get(rs);
                }
                OpcodeRegReg::MV => {
                    let rd = reg1;
                    let rs = reg2;
                    self.log(log_instr!([size] mv rd, rs));
                    self.breakpoint();
                    let value: s16 = self.regs.get(rs);
                    self.regs.set(rd, value);
                    self.pc += size;
                }
                OpcodeRegReg::MUL => {
                    let rs = reg1;
                    let rt = reg2;
                    self.log(log_instr!([size] mul rs, rt));
                    self.breakpoint();

                    let product = self.regs.get::<i16>(rs) as i32 * self.regs.get::<i16>(rt) as i32;
                    let product = unsafe { std::mem::transmute::<i32, u32>(product) };
                    let product: &BitSlice<u32, Lsb0> = product.view_bits();

                    *self.lo.as_i16_mut() = product[0..16].load();
                    *self.hi.as_i16_mut() = product[16..32].load();

                    self.pc += size;
                }
                OpcodeRegReg::MULU => {
                    let rs = reg1;
                    let rt = reg2;
                    self.log(log_instr!([size] mulu rs, rt));
                    self.breakpoint();

                    let product: u32 =
                        self.regs.get::<u16>(rs) as u32 * self.regs.get::<u16>(rt) as u32;
                    let product: &BitSlice<u32, Lsb0> = product.view_bits();

                    *self.lo.as_u16_mut() = product[0..16].load();
                    *self.hi.as_u16_mut() = product[16..32].load();

                    self.pc += size;
                }
                OpcodeRegReg::DIV => unimplemented!(),
                OpcodeRegReg::DIVU => unimplemented!(),
                OpcodeRegReg::NOT => {
                    let rd = reg1;
                    let rs = reg2;
                    self.log(log_instr!([size] not rd, rs));
                    self.breakpoint();
                    let value = !self.regs.get::<bool>(rs);
                    self.regs.set(rd, value as u16);
                    self.pc += size;
                }
                OpcodeRegReg::NEG => {
                    let rd = reg1;
                    let rs = reg2;
                    self.log(log_instr!([size] neg rd, rs));
                    self.breakpoint();
                    let value = -self.regs.get::<i16>(rs);
                    self.regs.set(rd, value);
                    self.pc += size;
                }
                OpcodeRegReg::SEB => {
                    let rd = reg1;
                    let rs = reg2;
                    self.log(log_instr!([size] seb rd, rs));
                    self.breakpoint();
                    let value = (self.regs.get::<u16>(rs) & 0x00FF) as u8;
                    let value = unsafe { std::mem::transmute::<u8, i8>(value) };
                    let value = value as i16;
                    self.regs.set(rd, value);
                    self.pc += size;
                }
                OpcodeRegReg::TEZ => {
                    let rd = reg1;
                    let rs = reg2;
                    self.log(log_instr!([size] tez rd, rs));
                    self.breakpoint();
                    let value = self.regs.get::<u16>(rs) == 0u16;
                    self.regs.set(rd, value as u16);
                    self.pc += size;
                }
                OpcodeRegReg::TNZ => {
                    let rd = reg1;
                    let rs = reg2;
                    self.log(log_instr!([size] tnz rd, rs));
                    self.breakpoint();
                    let value = self.regs.get::<u16>(rs) != 0u16;
                    self.regs.set(rd, value as u16);
                    self.pc += size;
                }
            },

            Instr::RRR {
                opcode,
                reg1: rd,
                reg2: rs,
                reg3: rt,
            } => match opcode {
                OpcodeRegRegReg::ADD => {
                    self.log(log_instr!([size] add rd, rs, rt));
                    self.breakpoint();
                    let x = self.regs.get::<i16>(rs);
                    let y = self.regs.get::<i16>(rt);
                    let sum: i16 = x.wrapping_add(y);
                    self.regs.set(rd, sum);
                    self.pc += size;
                }
                OpcodeRegRegReg::ADDU => {
                    self.log(log_instr!([size] addu rd, rs, rt));
                    self.breakpoint();
                    let x = self.regs.get::<u16>(rs);
                    let y = self.regs.get::<u16>(rt);
                    let sum: u16 = x.wrapping_add(y);
                    self.regs.set(rd, sum);
                    self.pc += size;
                }
                OpcodeRegRegReg::SUB => {
                    self.log(log_instr!([size] sub rd, rs, rt));
                    self.breakpoint();
                    let x = self.regs.get::<i16>(rs);
                    let y = self.regs.get::<i16>(rt);
                    let diff: i16 = x.wrapping_sub(y);
                    self.regs.set(rd, diff);
                    self.pc += size;
                }
                OpcodeRegRegReg::SUBU => {
                    self.log(log_instr!([size] subu rd, rs, rt));
                    self.breakpoint();
                    let x = self.regs.get::<u16>(rs);
                    let y = self.regs.get::<u16>(rt);
                    let diff: u16 = x.wrapping_sub(y);
                    self.regs.set(rd, diff);
                    self.pc += size;
                }
                OpcodeRegRegReg::OR => unimplemented!(),
                OpcodeRegRegReg::XOR => unimplemented!(),
                OpcodeRegRegReg::AND => unimplemented!(),
                OpcodeRegRegReg::SHL => {
                    self.log(log_instr!([size] shl rd, rs, rt));
                    self.breakpoint();
                    let value: u16 = self.regs.get::<u16>(rs) << self.regs.get::<u16>(rt);
                    self.regs.set(rd, value);
                    self.pc += size;
                }
                OpcodeRegRegReg::SHR => {
                    self.log(log_instr!([size] shr rd, rs, rt));
                    self.breakpoint();
                    let value: u16 = self.regs.get::<u16>(rs) >> self.regs.get::<u16>(rt);
                    self.regs.set(rd, value);
                    self.pc += size;
                }
                OpcodeRegRegReg::SHRA => {
                    self.log(log_instr!([size] shra rd, rs, rt));
                    self.breakpoint();
                    // Will perform sign-extension after shifting.
                    let value: i16 = self.regs.get::<i16>(rs) >> self.regs.get::<u16>(rt);
                    self.regs.set(rd, value);
                    self.pc += size;
                }
                OpcodeRegRegReg::TLT => {
                    self.log(log_instr!([size] tlt rd, rs, rt));
                    self.breakpoint();
                    let value = self.regs.get::<i16>(rs) < self.regs.get(rt);
                    self.regs.set(rd, value as u16);
                    self.pc += size;
                }
                OpcodeRegRegReg::TLTU => {
                    self.log(log_instr!([size] tltu rd, rs, rt));
                    self.breakpoint();
                    let value = self.regs.get::<u16>(rs) < self.regs.get(rt);
                    self.regs.set(rd, value as u16);
                    self.pc += size;
                }
                OpcodeRegRegReg::TGE => {
                    self.log(log_instr!([size] tge rd, rs, rt));
                    self.breakpoint();
                    let value = self.regs.get::<i16>(rs) >= self.regs.get(rt);
                    self.regs.set(rd, value as u16);
                    self.pc += size;
                }
                OpcodeRegRegReg::TGEU => {
                    self.log(log_instr!([size] tgeu rd, rs, rt));
                    self.breakpoint();
                    let value = self.regs.get::<u16>(rs) >= self.regs.get(rt);
                    self.regs.set(rd, value as u16);
                    self.pc += size;
                }
                OpcodeRegRegReg::TEQ => {
                    self.log(log_instr!([size] teq rd, rs, rt));
                    self.breakpoint();
                    let value = self.regs.get::<i16>(rs) == self.regs.get(rt);
                    self.regs.set(rd, value as u16);
                    self.pc += size;
                }
                OpcodeRegRegReg::TNE => {
                    self.log(log_instr!([size] tne rd, rs, rt));
                    self.breakpoint();
                    let value = self.regs.get::<u16>(rs) != self.regs.get(rt);
                    self.regs.set(rd, value as u16);
                    self.pc += size;
                }
            },

            Instr::RRI {
                opcode,
                reg1,
                reg2,
                imm10,
            } => match opcode {
                OpcodeRegRegImm::LW => {
                    let (rd, rs, addr_offset) = (reg1, reg2, imm10.as_i16());
                    self.log(log_instr!([size] lw rd, addr_offset, rs));
                    self.breakpoint();
                    let addr_base = self.regs.get(rs);
                    let value = self.mem_read_s16(addr_base, addr_offset);
                    self.regs.set(rd, value);
                    self.pc += size;
                }
                OpcodeRegRegImm::LBS => unimplemented!(),
                OpcodeRegRegImm::LBU => {
                    let (rd, rs, addr_offset) = (reg1, reg2, imm10.as_i16());
                    self.log(log_instr!([size] lbu rd, addr_offset, rs));
                    self.breakpoint();
                    let addr_base = self.regs.get(rs);
                    let value = self.mem_read_u8(addr_base, addr_offset);
                    self.regs.set(rd, value as u16);
                    self.pc += size;
                }
                OpcodeRegRegImm::SW => {
                    // Stores a word in memory given a address register and an offset.
                    // Example: sw -32($t0), $t1
                    //              ^   ^     ^
                    //              |   |     |
                    //         simm10   |     |
                    //    base addr reg (rd)  |
                    //              value reg (rs)
                    let (rd, rs, addr_offset) = (reg1, reg2, imm10.as_i16());
                    self.log(log_instr!([size] sw addr_offset, rd, rs ));
                    self.breakpoint();
                    let addr_base = self.regs.get(rd);
                    let value = self.regs.get(rs);
                    self.mem_write_s16(addr_base, addr_offset, value);
                    self.pc += size;
                }
                OpcodeRegRegImm::SB => {
                    let (rd, rs, addr_offset) = (reg1, reg2, imm10.as_i16());
                    self.log(log_instr!([size] sb addr_offset, rd, rs));
                    self.breakpoint();
                    let addr_base = self.regs.get(rd);
                    let value = (self.regs.get::<u16>(rs) & 0x00FF) as u8;
                    self.mem_write_u8(addr_base, addr_offset, value);
                    self.pc += size;
                }
                OpcodeRegRegImm::ADDI => {
                    let (rd, rs, simm) = (reg1, reg2, imm10.as_i16());
                    self.log(log_instr!([size] addi rd, rs, simm));
                    self.breakpoint();
                    let sum: i16 = self.regs.get::<i16>(rs).wrapping_add(simm);
                    self.regs.set(rd, sum);
                    self.pc += size;
                }
                OpcodeRegRegImm::SUBI => {
                    let (rd, rs, simm) = (reg1, reg2, imm10.as_i16());
                    self.log(log_instr!([size] subi rd, rs, simm));
                    self.breakpoint();
                    let diff: i16 = self.regs.get::<i16>(rs).wrapping_sub(simm);
                    self.regs.set(rd, diff);
                    self.pc += size;
                }
                OpcodeRegRegImm::ORI => unimplemented!(),
                OpcodeRegRegImm::XORI => unimplemented!(),
                OpcodeRegRegImm::ANDI => unimplemented!(),
            },
        }

        Ok(())
    }
}
