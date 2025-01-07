#![allow(dead_code)]

use std::convert::TryInto;

use bitvec::{prelude::*, slice::BitSlice};

use crate::cpu::instr::ops::*;

use super::{instr::Instr, regs::Reg};

type Bits<'a> = &'a BitSlice<u32, Msb0>;

/// The size in bytes of an instruction.
type InstrSize = u16;

const OPCODE_BITS: usize = 6;
const REG_BITS: usize = 4;
const ADDR_BITS: usize = 16;
const IMM10_BITS: usize = 10;
const IMM16_BITS: usize = 16;

#[derive(Debug, Clone, Copy)]
pub enum DecodeErr {
    /// The instruction's opcode is invalid.
    Opcode(u8),
    /// The instruction's destination register is invalid.
    Rd(u8),
    /// The instruction's first source register is invalid.
    Rs(u8),
    /// The instruction's second source register is invalid.
    Rt(u8),
    /// The instruction's single destination/source register is invalid.
    Reg(u8),
}

impl std::fmt::Display for DecodeErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeErr::Opcode(opcode) => write!(f, "decoded an invalid opcode: {}", opcode),
            DecodeErr::Rd(rd) => write!(f, "decoded an invalid rd: {}", rd),
            DecodeErr::Rs(rs) => write!(f, "decoded an invalid rs: {}", rs),
            DecodeErr::Rt(rt) => write!(f, "decoded an invalid rt: {}", rt),
            DecodeErr::Reg(reg) => write!(f, "decoded an invalid reg: {}", reg),
        }
    }
}

pub type DecodeResult<T> = Result<T, DecodeErr>;

/// Returns the size of an instruction in bytes.
const fn instr_size(arg_bits: usize) -> InstrSize {
    ceil_div(OPCODE_BITS + arg_bits, 8) as InstrSize
}

/// Decodes an immediate instruction.
pub fn simm10(instr: Bits) -> (InstrSize, i16) {
    let imm = instr[0..IMM10_BITS].load_le::<i16>();
    (instr_size(IMM10_BITS), imm)
}

pub fn imm10(instr: Bits) -> (InstrSize, u16) {
    let imm = instr[0..IMM10_BITS].load_le::<u16>();
    (instr_size(IMM10_BITS), imm)
}

pub fn reg(instr: Bits) -> DecodeResult<(InstrSize, Reg)> {
    let reg = instr[0..4]
        .load_le::<u8>()
        .try_into()
        .map_err(DecodeErr::Reg)?;
    Ok((instr_size(REG_BITS), reg))
}

pub fn reg_reg(instr: Bits) -> DecodeResult<(InstrSize, Reg, Reg)> {
    let rd = instr[0..4]
        .load_le::<u8>()
        .try_into()
        .map_err(DecodeErr::Rd)?;
    let rs = instr[4..8]
        .load_le::<u8>()
        .try_into()
        .map_err(DecodeErr::Rs)?;
    Ok((instr_size(2 * REG_BITS), rd, rs))
}

/// Decodes a register-register-register instruction.
pub fn reg_reg_reg(instr: Bits) -> DecodeResult<(InstrSize, Reg, Reg, Reg)> {
    let rd = instr[0..4]
        .load_le::<u8>()
        .try_into()
        .map_err(DecodeErr::Rd)?;
    let rs = instr[4..8]
        .load_le::<u8>()
        .try_into()
        .map_err(DecodeErr::Rs)?;
    let rt = instr[8..12]
        .load_le::<u8>()
        .try_into()
        .map_err(DecodeErr::Rt)?;
    Ok((instr_size(3 * REG_BITS), rd, rs, rt))
}

/// Decodes an instruction with an address-offset immediate.
pub fn addr(instr: Bits) -> (InstrSize, i16) {
    let imm = instr[0..ADDR_BITS].load_le::<i16>();
    (instr_size(ADDR_BITS), imm)
}

/// Decodes a register-immediate instruction. Used for address immediates and
/// value immediates.
pub fn reg_simm(instr: Bits) -> DecodeResult<(InstrSize, Reg, i16)> {
    let rd = instr[0..REG_BITS]
        .load_le::<u8>()
        .try_into()
        .map_err(DecodeErr::Rd)?;
    let imm = instr[REG_BITS..][..IMM16_BITS].load_le::<i16>();
    Ok((instr_size(REG_BITS + IMM16_BITS), rd, imm))
}

/// Decodes a register-register-immediate instruction.
pub fn reg_reg_simm(instr: Bits) -> DecodeResult<(InstrSize, Reg, Reg, i16)> {
    let rd = instr[0..4]
        .load_le::<u8>()
        .try_into()
        .map_err(DecodeErr::Rd)?;
    let rs = instr[4..8]
        .load_le::<u8>()
        .try_into()
        .map_err(DecodeErr::Rs)?;
    let imm = instr[8..][..IMM10_BITS].load_le::<i16>();
    Ok((instr_size(2 * REG_BITS + IMM10_BITS), rd, rs, imm))
}

/// Decodes a signed 16-bit immediate.
pub fn simm16(instr: Bits) -> (InstrSize, i16) {
    let imm = instr[0..IMM16_BITS].load_le::<i16>();
    (instr_size(IMM16_BITS), imm)
}

impl Instr {
    pub fn from_bits(bits: Bits) -> DecodeResult<Self> {
        let opcode = bits[0..OPCODE_BITS].load_le::<u8>();
        let bits = &bits[OPCODE_BITS..];

        if let Ok(opcode) = OpcodeOp::try_from(opcode) {
            return Ok(Instr::O { opcode });
        }

        if let Ok(opcode) = OpcodeAddr::try_from(opcode) {
            let (_size, offset) = addr(bits);
            return Ok(Instr::A {
                opcode,
                offset: offset.into(),
            });
        }

        if let Ok(opcode) = OpcodeImm::try_from(opcode) {
            let (_size, imm10) = imm10(bits);
            return Ok(Instr::I {
                opcode,
                imm10: imm10.into(),
            });
        }

        if let Ok(opcode) = OpcodeReg::try_from(opcode) {
            let (_size, reg) = reg(bits)?;
            return Ok(Instr::R { opcode, reg });
        }

        if let Ok(opcode) = OpcodeRegImm::try_from(opcode) {
            let (_size, reg, simm) = reg_simm(bits)?;
            return Ok(Instr::RI {
                opcode,
                reg,
                imm: simm.into(),
            });
        }

        if let Ok(opcode) = OpcodeRegReg::try_from(opcode) {
            let (_size, reg1, reg2) = reg_reg(bits)?;
            return Ok(Instr::RR { opcode, reg1, reg2 });
        }

        if let Ok(opcode) = OpcodeRegRegReg::try_from(opcode) {
            let (_size, reg1, reg2, reg3) = reg_reg_reg(bits)?;
            return Ok(Instr::RRR {
                opcode,
                reg1,
                reg2,
                reg3,
            });
        }

        if let Ok(opcode) = OpcodeRegRegImm::try_from(opcode) {
            let (_size, reg1, reg2, simm) = reg_reg_simm(bits)?;
            return Ok(Instr::RRI {
                opcode,
                reg1,
                reg2,
                imm10: simm.into(),
            });
        }

        Err(DecodeErr::Opcode(opcode))
    }

    pub const fn instr_size(&self) -> InstrSize {
        match self {
            Instr::O { .. } => instr_size(0),
            Instr::A { .. } => instr_size(ADDR_BITS),
            Instr::I { .. } => instr_size(IMM10_BITS),
            Instr::R { .. } => instr_size(REG_BITS),
            Instr::RI { .. } => instr_size(REG_BITS + IMM16_BITS),
            Instr::RR { .. } => instr_size(2 * REG_BITS),
            Instr::RRR { .. } => instr_size(3 * REG_BITS),
            Instr::RRI { .. } => instr_size(2 * REG_BITS + IMM10_BITS),
        }
    }
}

const fn ceil_div(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ceil_div() {
        assert_eq!(ceil_div(10, 3), 4);
        assert_eq!(ceil_div(15, 5), 3);
        assert_eq!(ceil_div(7, 2), 4);
        assert_eq!(ceil_div(100, 10), 10);
        assert_eq!(ceil_div(0, 5), 0);
    }
}
