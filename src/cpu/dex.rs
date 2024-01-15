//! `dex` stands for Decode+Execute. It implements the CPU's instruction set.

use bitvec::prelude::*;

use crate::{cpu::decode, log_instr, utils::s16};

use super::{opcodes, Cpu};

#[derive(Debug)]
pub enum DexErr {
    RegDecodeError(decode::RegDecodeErr),
    InvalidOpcode(u8),
}

impl From<decode::RegDecodeErr> for DexErr {
    fn from(err: decode::RegDecodeErr) -> Self {
        DexErr::RegDecodeError(err)
    }
}

impl Cpu {
    pub fn decode_and_execute(&mut self) -> Result<(), DexErr> {
        let ir = self.ir.view_bits::<Msb0>();

        // Opcode is most significant 6 bits.
        let opcode = ir[..6].load::<u8>();
        let ir = &ir[6..];

        if cfg!(debug_assertions) {
            print!("pc={}\t", self.pc);
        }

        match opcode {
            opcodes::EXN => {
                let (size, imm10) = decode::imm10(ir);
                log_instr!([size] exn imm10);
                self.breakpoint();
                self.handle_exn(imm10);
                self.pc += size;
            }

            opcodes::HALT => {
                log_instr!([1] halt);
                self.breakpoint();
                std::process::exit(0);
            }

            opcodes::LI => {
                let (size, rd, simm16) = decode::reg_simm(ir)?;
                log_instr!([size] li rd, simm16);
                self.breakpoint();
                self.regs.set(rd, simm16);
                self.pc += size;
            }

            opcodes::ADD => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir)?;
                log_instr!([size] add rd, rs, rt);
                self.breakpoint();
                let sum: i16 = self.regs.get::<i16>(rs) + self.regs.get::<i16>(rt);
                self.regs.set(rd, sum);
                self.pc += size;
            }

            opcodes::ADDU => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir)?;
                log_instr!([size] addu rd, rs, rt);
                self.breakpoint();
                let sum: u16 = self.regs.get::<u16>(rs) + self.regs.get::<u16>(rt);
                self.regs.set(rd, sum);
                self.pc += size;
            }

            opcodes::ADDI => {
                let (size, rd, rs, simm) = decode::reg_reg_simm(ir)?;
                log_instr!([size] addi rd, rs, simm);
                self.breakpoint();
                let sum: i16 = self.regs.get::<i16>(rs) + simm;
                self.regs.set(rd, sum);
                self.pc += size;
            }

            opcodes::SUB => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir)?;
                log_instr!([size] sub rd, rs, rt);
                self.breakpoint();
                let diff: i16 = self.regs.get::<i16>(rs) - self.regs.get::<i16>(rt);
                self.regs.set(rd, diff);
                self.pc += size;
            }

            opcodes::SUBU => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir)?;
                log_instr!([size] subu rd, rs, rt);
                self.breakpoint();
                let diff: u16 = self.regs.get::<u16>(rs) - self.regs.get::<u16>(rt);
                self.regs.set(rd, diff);
                self.pc += size;
            }

            opcodes::SUBI => {
                let (size, rd, rs, simm) = decode::reg_reg_simm(ir)?;
                log_instr!([size] subi rd, rs, simm);
                self.breakpoint();
                let diff: i16 = self.regs.get::<i16>(rs) - simm;
                self.regs.set(rd, diff);
                self.pc += size;
            }

            opcodes::MV => {
                let (size, rd, rs) = decode::reg_reg(ir)?;
                log_instr!([size] mv rd, rs);
                self.breakpoint();
                let value: s16 = self.regs.get(rs);
                self.regs.set(rd, value);
                self.pc += size;
            }

            opcodes::JR => {
                let (size, rs) = decode::reg(ir)?;
                log_instr!([size] jr rs);
                self.breakpoint();
                self.pc = self.regs.get(rs);
            }

            opcodes::J => {
                let (size, offset) = decode::simm16(ir);
                log_instr!([size] j offset);
                self.breakpoint();
                self.pc = (self.pc as i32)
                    .checked_add(offset as i32)
                    .expect("Jump address overflow") as u16;
            }

            // Jump and link.
            // Example: jal $rd, ADDR
            opcodes::JAL => {
                let (size, rd, offset) = decode::reg_simm(ir)?;
                log_instr!([size] jal rd, offset);
                self.breakpoint();
                self.regs.set(rd, self.pc + size);
                self.pc = (self.pc as i32)
                    .checked_add(offset as i32)
                    .expect("Jump address overflow") as u16;
            }

            // Jump register and link.
            // Example: jral $rd, $rs
            //                |    |
            //          save pc    jump address
            opcodes::JRAL => {
                let (size, rd, rs) = decode::reg_reg(ir)?;
                log_instr!([size] jral rd, rs);
                self.breakpoint();
                self.regs.set(rd, self.pc + size);
                self.pc = self.regs.get(rs);
            }

            opcodes::TLT => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir)?;
                log_instr!([size] tlt rd, rs, rt);
                self.breakpoint();
                let value = self.regs.get::<i16>(rs) < self.regs.get(rt);
                self.regs.set(rd, value as u16);
                self.pc += size;
            }

            opcodes::TLTU => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir)?;
                log_instr!([size] tltu rd, rs, rt);
                self.breakpoint();
                let value = self.regs.get::<u16>(rs) < self.regs.get(rt);
                self.regs.set(rd, value as u16);
                self.pc += size;
            }

            opcodes::TGE => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir)?;
                log_instr!([size] tge rd, rs, rt);
                self.breakpoint();
                let value = self.regs.get::<i16>(rs) >= self.regs.get(rt);
                self.regs.set(rd, value as u16);
                self.pc += size;
            }

            opcodes::TGEU => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir)?;
                log_instr!([size] tgeu rd, rs, rt);
                self.breakpoint();
                let value = self.regs.get::<u16>(rs) >= self.regs.get(rt);
                self.regs.set(rd, value as u16);
                self.pc += size;
            }

            opcodes::TEQ => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir)?;
                log_instr!([size] teq rd, rs, rt);
                self.breakpoint();
                let value = self.regs.get::<i16>(rs) == self.regs.get(rt);
                self.regs.set(rd, value as u16);
                self.pc += size;
            }

            opcodes::TNE => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir)?;
                log_instr!([size] tne rd, rs, rt);
                self.breakpoint();
                let value = self.regs.get::<u16>(rs) != self.regs.get(rt);
                self.regs.set(rd, value as u16);
                self.pc += size;
            }

            opcodes::TEZ => {
                let (size, rd, rs) = decode::reg_reg(ir)?;
                log_instr!([size] tez rd, rs);
                self.breakpoint();
                let value = self.regs.get::<u16>(rs) == 0u16;
                self.regs.set(rd, value as u16);
                self.pc += size;
            }

            opcodes::TNZ => {
                let (size, rd, rs) = decode::reg_reg(ir)?;
                log_instr!([size] tnz rd, rs);
                self.breakpoint();
                let value = self.regs.get::<u16>(rs) != 0u16;
                self.regs.set(rd, value as u16);
                self.pc += size;
            }

            // Branch if false.
            opcodes::BF => {
                let (size, rs, addr_offset) = decode::reg_simm(ir)?;
                log_instr!([size] bf rs, addr_offset);
                self.breakpoint();
                if !self.regs.get::<bool>(rs) {
                    self.pc = (self.pc as i32 + addr_offset as i32) as u16;
                } else {
                    self.pc += size;
                }
            }

            // Branch if true.
            opcodes::BT => {
                let (size, rs, addr_offset) = decode::reg_simm(ir)?;
                log_instr!([size] bt rs, addr_offset);
                self.breakpoint();
                if self.regs.get(rs) {
                    self.pc = (self.pc as i32 + addr_offset as i32) as u16;
                } else {
                    self.pc += size;
                }
            }

            opcodes::NOT => {
                let (size, rd, rs) = decode::reg_reg(ir)?;
                log_instr!([size] not rd, rs);
                self.breakpoint();
                let value = !self.regs.get::<bool>(rs);
                self.regs.set(rd, value as u16);
                self.pc += size;
            }

            opcodes::NOP => {
                log_instr!([1] nop);
                self.breakpoint();
                self.pc += 1;
            }

            opcodes::MUL => {
                let (size, rs, rt) = decode::reg_reg(ir)?;
                log_instr!([size] mul rs, rt);
                self.breakpoint();

                let product = self.regs.get::<i16>(rs) as i32 * self.regs.get::<i16>(rt) as i32;
                let product = unsafe { std::mem::transmute::<i32, u32>(product) };
                let product: &BitSlice<u32, Lsb0> = product.view_bits();

                *self.lo.as_i16_mut() = product[0..16].load();
                *self.hi.as_i16_mut() = product[16..32].load();

                self.pc += size;
            }

            opcodes::MULU => {
                let (size, rs, rt) = decode::reg_reg(ir)?;
                log_instr!([size] mulu rs, rt);
                self.breakpoint();

                let product: u32 =
                    self.regs.get::<u16>(rs) as u32 * self.regs.get::<u16>(rt) as u32;
                let product: &BitSlice<u32, Lsb0> = product.view_bits();

                *self.lo.as_u16_mut() = product[0..16].load();
                *self.hi.as_u16_mut() = product[16..32].load();

                self.pc += size;
            }

            opcodes::MVLO => {
                let (size, rd) = decode::reg(ir)?;
                log_instr!([size] mvlo rd);
                self.breakpoint();
                self.regs.set(rd, self.lo);
                self.pc += size;
            }

            opcodes::MVHI => {
                let (size, rd) = decode::reg(ir)?;
                log_instr!([size] mvhi rd);
                self.breakpoint();
                self.regs.set(rd, self.hi);
                self.pc += size;
            }

            opcodes::LW => {
                let (size, rd, rs, addr_offset) = decode::reg_reg_simm(ir)?;
                log_instr!([size] lw rd, rs, addr_offset);
                self.breakpoint();
                let addr_base = self.regs.get(rs);
                let value = self.mem_read_s16(addr_base, addr_offset);
                self.regs.set(rd, value);
                self.pc += size;
            }

            opcodes::LBU => {
                let (size, rd, rs, addr_offset) = decode::reg_reg_simm(ir)?;
                log_instr!([size] lbu rd, rs, addr_offset);
                self.breakpoint();
                let addr_base = self.regs.get(rs);
                let value = self.mem_read_u8(addr_base, addr_offset);
                self.regs.set(rd, value as u16);
                self.pc += size;
            }

            // Stores a word in memory given a address register and an offset.
            // Example: sw -32($t0), $t1
            //              ^   ^     ^
            //              |   |     |
            //         simm10   |     |
            //    base addr reg (rd)  |
            //              value reg (rs)
            opcodes::SW => {
                let (size, rd, rs, addr_offset) = decode::reg_reg_simm(ir)?;
                log_instr!([size] sw rd, rs, addr_offset);
                self.breakpoint();
                let addr_base = self.regs.get(rd);
                let value = self.regs.get(rs);
                self.mem_write_s16(addr_base, addr_offset, value);
                self.pc += size;
            }

            opcodes::SB => {
                let (size, rd, rs, addr_offset) = decode::reg_reg_simm(ir)?;
                log_instr!([size] sb rd, rs, addr_offset);
                self.breakpoint();
                let addr_base = self.regs.get(rd);
                let value = (self.regs.get::<u16>(rs) & 0x00FF) as u8;
                self.mem_write_u8(addr_base, addr_offset, value);
                self.pc += size;
            }

            opcodes::SEB => {
                let (size, rd, rs) = decode::reg_reg(ir)?;
                log_instr!([size] seb rd, rs);
                self.breakpoint();
                let value = (self.regs.get::<u16>(rs) & 0x00FF) as u8;
                let value = unsafe { std::mem::transmute::<u8, i8>(value) };
                let value = value as i16;
                self.regs.set(rd, value);
                self.pc += size;
            }

            opcodes::SHL => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir)?;
                log_instr!([size] shl rd, rs, rt);
                self.breakpoint();
                let value: u16 = self.regs.get::<u16>(rs) << self.regs.get::<u16>(rt);
                self.regs.set(rd, value);
                self.pc += size;
            }

            opcodes::SHR => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir)?;
                log_instr!([size] shr rd, rs, rt);
                self.breakpoint();
                let value: u16 = self.regs.get::<u16>(rs) >> self.regs.get::<u16>(rt);
                self.regs.set(rd, value);
                self.pc += size;
            }

            // Shift right arithmetic.
            opcodes::SHRA => {
                let (size, rd, rs, rt) = decode::reg_reg_reg(ir)?;
                log_instr!([size] shra rd, rs, rt);
                self.breakpoint();
                // Will perform sign-extension after shifting.
                let value: i16 = self.regs.get::<i16>(rs) >> self.regs.get::<u16>(rt);
                self.regs.set(rd, value);
                self.pc += size;
            }

            other => unimplemented!("unimplemented opcode `0x{:X?}` (pc={})", other, self.pc),
        }

        Ok(())
    }
}
