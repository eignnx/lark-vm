use bitvec::{prelude::*, slice::BitSlice};

type Bits<'a> = &'a BitSlice<u32, Msb0>;

/// The size in bytes of an instruction.
type InstrSize = u16;

const OPCODE_BITS: usize = 6;
const REG_BITS: usize = 4;

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

pub fn reg(instr: Bits) -> (InstrSize, u8) {
    let reg = instr[0..4].load_le::<u8>();
    (instr_size(REG_BITS), reg)
}

pub fn reg_reg(instr: Bits) -> (InstrSize, u8, u8) {
    let rd = instr[0..4].load_le::<u8>();
    let rs = instr[4..8].load_le::<u8>();
    (instr_size(2 * REG_BITS), rd, rs)
}

/// Decodes a register-register-register instruction.
pub fn reg_reg_reg(instr: Bits) -> (InstrSize, u8, u8, u8) {
    let rd = instr[0..4].load_le::<u8>();
    let rs = instr[4..8].load_le::<u8>();
    let rt = instr[8..12].load_le::<u8>();
    (instr_size(3 * REG_BITS), rd, rs, rt)
}

/// Decodes an instruction with an address-offset immediate.
pub fn addr(instr: Bits) -> (InstrSize, i16) {
    const ADDR_BITS: usize = 16;
    let imm = instr[0..ADDR_BITS].load_le::<i16>();
    (instr_size(ADDR_BITS), imm)
}

/// Decodes a register-immediate instruction. Used for address immediates and
/// value immediates.
pub fn reg_simm(instr: Bits) -> (InstrSize, u8, i16) {
    const SIMM_BITS: usize = 16;
    let rd = instr[0..REG_BITS].load_le::<u8>();
    let imm = instr[REG_BITS..][..SIMM_BITS].load_le::<i16>();
    (instr_size(REG_BITS + SIMM_BITS), rd, imm)
}

/// Decodes a register-register-immediate instruction.
pub fn reg_reg_simm(instr: Bits) -> (InstrSize, u8, u8, i16) {
    const SIMM_BITS: usize = 10;
    let rd = instr[0..4].load_le::<u8>();
    let rs = instr[4..8].load_le::<u8>();
    let imm = instr[8..][..SIMM_BITS].load_le::<i16>();
    (instr_size(2 * REG_BITS + SIMM_BITS), rd, rs, imm)
}

/// Decodes a signed 16-bit immediate.
pub fn simm16(ir: &BitSlice<u32, Msb0>) -> (InstrSize, i16) {
    const IMM_BITS: usize = 16;
    let imm = ir[0..IMM_BITS].load_le::<i16>();
    (instr_size(IMM_BITS), imm)
}
