use num_enum::TryFromPrimitive;

use super::{opcodes, regs::Reg};
use crate::utils::s16;

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeOp {
    /// Halt
    HALT = opcodes::HALT,
    /// No-Op
    NOP = opcodes::NOP,
    /// Kernel Return (return from interrupt)
    KRET = opcodes::KRET,
    /// Interrupts Enable
    INRE = opcodes::INRE,
    /// Interrupts Disable
    INRD = opcodes::INRD,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeAddr {
    /// Jump (absolute)
    J = opcodes::J,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeImm {
    /// Exception
    EXN = opcodes::EXN,
    /// Kernel Call
    KCALL = opcodes::KCALL,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeReg {
    /// Jump Register
    JR = opcodes::JR,
    /// Move Low ($LO)
    MVLO = opcodes::MVLO,
    /// Move Hight ($HI)
    MVHI = opcodes::MVHI,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeRegImm {
    /// Jump And Link
    JAL = opcodes::JAL,
    /// Branch if True
    BT = opcodes::BT,
    /// Branch if False
    BF = opcodes::BF,
    /// Load Immediate
    LI = opcodes::LI,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeRegReg {
    /// Jump Register And Link
    JRAL = opcodes::JRAL,
    /// Move
    MV = opcodes::MV,
    /// Multiply
    MUL = opcodes::MUL,
    /// Divide
    DIV = opcodes::DIV,
    /// Not (boolean operation)
    NOT = opcodes::NOT,
    /// Negate
    NEG = opcodes::NEG,
    /// Multiply Unsigned
    MULU = opcodes::MULU,
    /// Divide Unsigned
    DIVU = opcodes::DIVU,
    /// Sign Extend Byte
    SEB = opcodes::SEB,
    /// Test Equal to Zero
    TEZ = opcodes::TEZ,
    /// Test Non-Zero
    TNZ = opcodes::TNZ,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeRegRegReg {
    /// Add
    ADD = opcodes::ADD,
    /// Subtract
    SUB = opcodes::SUB,
    /// Or (boolean operation)
    OR = opcodes::OR,
    /// Xor (exclusive-or)
    XOR = opcodes::XOR,
    /// And (boolean operation)
    AND = opcodes::AND,
    /// Add Unsigned
    ADDU = opcodes::ADDU,
    /// Subtract Unsigned
    SUBU = opcodes::SUBU,
    /// Shift Left
    SHL = opcodes::SHL,
    /// Shift Right (shifts zeros into the most-significant-bit of the word)
    SHR = opcodes::SHR,
    /// Shift Right Arithmetic (shifts sign bit into the most-significant-bit of the word)
    SHRA = opcodes::SHRA,
    /// Test Less Than
    TLT = opcodes::TLT,
    /// Test Greater or Equal
    TGE = opcodes::TGE,
    /// Test Equal
    TEQ = opcodes::TEQ,
    /// Test Not Equal
    TNE = opcodes::TNE,
    /// Test Less Than Unsigned
    TLTU = opcodes::TLTU,
    /// Test Greater or Equal Unsigned
    TGEU = opcodes::TGEU,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeRegRegImm {
    /// Load Word
    LW = opcodes::LW,
    /// Load Byte Signed
    LBS = opcodes::LBS,
    /// Load Byte Unsigned
    LBU = opcodes::LBU,
    /// Store Word
    SW = opcodes::SW,
    /// Store Byte
    SB = opcodes::SB,
    /// Add Immediate
    ADDI = opcodes::ADDI,
    /// Subtract Immediate
    SUBI = opcodes::SUBI,
    /// Or Immediate
    ORI = opcodes::ORI,
    /// Xor Immediate
    XORI = opcodes::XORI,
    /// And Immediate
    ANDI = opcodes::ANDI,
}

#[derive(Debug, Clone, Copy)]
pub enum Instr {
    /// No arguments, opcode only
    O { opcode: OpcodeOp },
    /// Single address argument
    A { opcode: OpcodeAddr, offset: i16 },
    /// Single immediate argument
    I {
        opcode: OpcodeImm,
        /// A 10-bit immediate.
        imm10: u16,
    },
    /// Single register argument
    R { opcode: OpcodeReg, reg: Reg },
    /// One register and one immediate arguments
    RI {
        opcode: OpcodeRegImm,
        reg: Reg,
        imm: s16,
    },
    /// Two register arguments
    RR {
        opcode: OpcodeRegReg,
        reg1: Reg,
        reg2: Reg,
    },
    /// Three register arguments
    RRR {
        opcode: OpcodeRegRegReg,
        reg1: Reg,
        reg2: Reg,
        reg3: Reg,
    },
    /// Two register and one immediate arguments
    RRI {
        opcode: OpcodeRegRegImm,
        reg1: Reg,
        reg2: Reg,
        /// A 10-bit immediate.
        imm10: s16,
    },
}
