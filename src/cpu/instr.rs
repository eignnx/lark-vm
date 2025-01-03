use num_enum::TryFromPrimitive;

use super::regs::Reg;
use crate::utils::s16;

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeOp {
    /// Halt
    HALT = 0x01,
    /// No-Op
    NOP = 0x02,
    /// Kernel Return (return from interrupt)
    KRET = 0x0D,
    /// Interrupts Enable
    INRE = 0x1C,
    /// Interrupts Disable
    INRD = 0x1D,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeAddr {
    /// Jump (absolute)
    J = 0x08,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeImm {
    /// Exception
    EXN = 0x00,
    /// Kernel Call
    KCALL = 0x0E,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeReg {
    /// Jump Register
    JR = 0x09,
    /// Move Low ($LO)
    MVLO = 0x2A,
    /// Move Hight ($HI)
    MVHI = 0x2B,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeRegImm {
    /// Jump And Link
    JAL = 0x0A,
    /// Branch if True
    BT = 0x0C,
    /// Branch if False
    BF = 0x0F,
    /// Load Immediate
    LI = 0x10,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeRegReg {
    /// Jump Register And Link
    JRAL = 0x0B,
    /// Move
    MV = 0x14,
    /// Multiply
    MUL = 0x22,
    /// Divide
    DIV = 0x23,
    /// Not (boolean operation)
    NOT = 0x27,
    /// Negate
    NEG = 0x2F,
    /// Multiply Unsigned
    MULU = 0x32,
    /// Divide Unsigned
    DIVU = 0x33,
    /// Sign Extend Byte
    SEB = 0x37,
    /// Test Equal to Zero
    TEZ = 0x3E,
    /// Test Non-Zero
    TNZ = 0x3F,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeRegRegReg {
    /// Add
    ADD = 0x20,
    /// Subtract
    SUB = 0x21,
    /// Or (boolean operation)
    OR = 0x24,
    /// Xor (exclusive-or)
    XOR = 0x25,
    /// And (boolean operation)
    AND = 0x26,
    /// Add Unsigned
    ADDU = 0x30,
    /// Subtract Unsigned
    SUBU = 0x31,
    /// Shift Left
    SHL = 0x34,
    /// Shift Right (shifts zeros into the most-significant-bit of the word)
    SHR = 0x35,
    /// Shift Right Arithmetic (shifts sign bit into the most-significant-bit of the word)
    SHRA = 0x36,
    /// Test Less Than
    TLT = 0x38,
    /// Test Greater or Equal
    TGE = 0x39,
    /// Test Equal
    TEQ = 0x3A,
    /// Test Not Equal
    TNE = 0x3B,
    /// Test Less Than Unsigned
    TLTU = 0x3C,
    /// Test Greater or Equal Unsigned
    TGEU = 0x3D,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum OpcodeRegRegImm {
    /// Load Word
    LW = 0x11,
    /// Load Byte Signed
    LBS = 0x12,
    /// Load Byte Unsigned
    LBU = 0x13,
    /// Store Word
    SW = 0x15,
    /// Store Byte
    SB = 0x16,
    /// Add Immediate
    ADDI = 0x28,
    /// Subtract Immediate
    SUBI = 0x29,
    /// Or Immediate
    ORI = 0x2C,
    /// Xor Immediate
    XORI = 0x2D,
    /// And Immediate
    ANDI = 0x2E,
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
