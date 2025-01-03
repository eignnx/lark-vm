#![allow(dead_code)]

use std::convert::TryInto;

use bitvec::{prelude::*, slice::BitSlice};

use super::regs::Reg;

type Bits<'a> = &'a BitSlice<u32, Msb0>;

/// The size in bytes of an instruction.
type InstrSize = u16;

const OPCODE_BITS: usize = 6;
const REG_BITS: usize = 4;

#[derive(Debug, Clone, Copy)]
pub enum RegDecodeErr {
    /// The instruction's destination register is invalid.
    Rd(u8),
    /// The instruction's first source register is invalid.
    Rs(u8),
    /// The instruction's second source register is invalid.
    Rt(u8),
    /// The instruction's single destination/source register is invalid.
    Reg(u8),
}

impl std::fmt::Display for RegDecodeErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegDecodeErr::Rd(rd) => write!(f, "decoded an invalid rd: {}", rd),
            RegDecodeErr::Rs(rs) => write!(f, "decoded an invalid rs: {}", rs),
            RegDecodeErr::Rt(rt) => write!(f, "decoded an invalid rt: {}", rt),
            RegDecodeErr::Reg(reg) => write!(f, "decoded an invalid reg: {}", reg),
        }
    }
}

pub type RegDecodeResult<T> = Result<T, RegDecodeErr>;

const fn instr_size(arg_bits: usize) -> InstrSize {
    ceil_div(OPCODE_BITS + arg_bits, 8) as InstrSize
}

/// Decodes an immediate instruction.
pub fn simm10(instr: Bits) -> (InstrSize, i16) {
    const IMM_BITS: usize = 10;
    let imm = instr[0..IMM_BITS].load_le::<i16>();
    (instr_size(IMM_BITS), imm)
}

pub fn imm10(instr: Bits) -> (InstrSize, u16) {
    const IMM_BITS: usize = 10;
    let imm = instr[0..IMM_BITS].load_le::<u16>();
    (instr_size(IMM_BITS), imm)
}

pub fn reg(instr: Bits) -> RegDecodeResult<(InstrSize, Reg)> {
    let reg = instr[0..4]
        .load_le::<u8>()
        .try_into()
        .map_err(RegDecodeErr::Reg)?;
    Ok((instr_size(REG_BITS), reg))
}

pub fn reg_reg(instr: Bits) -> RegDecodeResult<(InstrSize, Reg, Reg)> {
    let rd = instr[0..4]
        .load_le::<u8>()
        .try_into()
        .map_err(RegDecodeErr::Rd)?;
    let rs = instr[4..8]
        .load_le::<u8>()
        .try_into()
        .map_err(RegDecodeErr::Rs)?;
    Ok((instr_size(2 * REG_BITS), rd, rs))
}

/// Decodes a register-register-register instruction.
pub fn reg_reg_reg(instr: Bits) -> RegDecodeResult<(InstrSize, Reg, Reg, Reg)> {
    let rd = instr[0..4]
        .load_le::<u8>()
        .try_into()
        .map_err(RegDecodeErr::Rd)?;
    let rs = instr[4..8]
        .load_le::<u8>()
        .try_into()
        .map_err(RegDecodeErr::Rs)?;
    let rt = instr[8..12]
        .load_le::<u8>()
        .try_into()
        .map_err(RegDecodeErr::Rt)?;
    Ok((instr_size(3 * REG_BITS), rd, rs, rt))
}

/// Decodes an instruction with an address-offset immediate.
pub fn addr(instr: Bits) -> (InstrSize, i16) {
    const ADDR_BITS: usize = 16;
    let imm = instr[0..ADDR_BITS].load_le::<i16>();
    (instr_size(ADDR_BITS), imm)
}

/// Decodes a register-immediate instruction. Used for address immediates and
/// value immediates.
pub fn reg_simm(instr: Bits) -> RegDecodeResult<(InstrSize, Reg, i16)> {
    const SIMM_BITS: usize = 16;
    let rd = instr[0..REG_BITS]
        .load_le::<u8>()
        .try_into()
        .map_err(RegDecodeErr::Rd)?;
    let imm = instr[REG_BITS..][..SIMM_BITS].load_le::<i16>();
    Ok((instr_size(REG_BITS + SIMM_BITS), rd, imm))
}

/// Decodes a register-register-immediate instruction.
pub fn reg_reg_simm(instr: Bits) -> RegDecodeResult<(InstrSize, Reg, Reg, i16)> {
    const SIMM_BITS: usize = 10;
    let rd = instr[0..4]
        .load_le::<u8>()
        .try_into()
        .map_err(RegDecodeErr::Rd)?;
    let rs = instr[4..8]
        .load_le::<u8>()
        .try_into()
        .map_err(RegDecodeErr::Rs)?;
    let imm = instr[8..][..SIMM_BITS].load_le::<i16>();
    Ok((instr_size(2 * REG_BITS + SIMM_BITS), rd, rs, imm))
}

/// Decodes a signed 16-bit immediate.
pub fn simm16(ir: &BitSlice<u32, Msb0>) -> (InstrSize, i16) {
    const IMM_BITS: usize = 16;
    let imm = ir[0..IMM_BITS].load_le::<i16>();
    (instr_size(IMM_BITS), imm)
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
