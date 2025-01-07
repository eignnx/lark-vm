use core::fmt;

use super::regs::Reg;
use crate::utils::s16;

/// You probably want to `use ops::*;` since theres a lot of these.
pub mod ops {
    use core::fmt;

    use num_enum::TryFromPrimitive;

    use crate::cpu::opcodes;

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

    impl fmt::Display for OpcodeOp {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let name = match self {
                Self::HALT => "halt",
                Self::NOP => "nop",
                Self::KRET => "kret",
                Self::INRE => "inre",
                Self::INRD => "inrd",
            };
            write!(f, "{}", name)
        }
    }

    impl TryFrom<&str> for OpcodeOp {
        type Error = ();

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            match value {
                "halt" => Ok(Self::HALT),
                "nop" => Ok(Self::NOP),
                "kret" => Ok(Self::KRET),
                "inre" => Ok(Self::INRE),
                "inrd" => Ok(Self::INRD),
                _ => Err(()),
            }
        }
    }

    #[derive(Debug, Clone, Copy, TryFromPrimitive)]
    #[repr(u8)]
    pub enum OpcodeAddr {
        /// Jump (absolute)
        J = opcodes::J,
    }

    impl fmt::Display for OpcodeAddr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let name = match self {
                Self::J => "j",
            };
            write!(f, "{}", name)
        }
    }

    impl TryFrom<&str> for OpcodeAddr {
        type Error = ();

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            match value {
                "j" => Ok(Self::J),
                _ => Err(()),
            }
        }
    }

    #[derive(Debug, Clone, Copy, TryFromPrimitive)]
    #[repr(u8)]
    pub enum OpcodeImm {
        /// Exception
        EXN = opcodes::EXN,
        /// Kernel Call
        KCALL = opcodes::KCALL,
    }

    impl fmt::Display for OpcodeImm {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let name = match self {
                Self::EXN => "exn",
                Self::KCALL => "kcall",
            };
            write!(f, "{}", name)
        }
    }

    impl TryFrom<&str> for OpcodeImm {
        type Error = ();

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            match value {
                "exn" => Ok(Self::EXN),
                "kcall" => Ok(Self::KCALL),
                _ => Err(()),
            }
        }
    }

    #[derive(Debug, Clone, Copy, TryFromPrimitive)]
    #[repr(u8)]
    pub enum OpcodeReg {
        /// Jump Register
        JR = opcodes::JR,
        /// Move Low ($LO)
        MVLO = opcodes::MVLO,
        /// Move High ($HI)
        MVHI = opcodes::MVHI,
    }

    impl fmt::Display for OpcodeReg {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let name = match self {
                Self::JR => "jr",
                Self::MVLO => "mvlo",
                Self::MVHI => "mvhi",
            };
            write!(f, "{}", name)
        }
    }

    impl TryFrom<&str> for OpcodeReg {
        type Error = ();

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            match value {
                "jr" => Ok(Self::JR),
                "mvlo" => Ok(Self::MVLO),
                "mvhi" => Ok(Self::MVHI),
                _ => Err(()),
            }
        }
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

    impl fmt::Display for OpcodeRegImm {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let name = match self {
                Self::JAL => "jal",
                Self::BT => "bt",
                Self::BF => "bf",
                Self::LI => "li",
            };
            write!(f, "{}", name)
        }
    }

    impl TryFrom<&str> for OpcodeRegImm {
        type Error = ();

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            match value {
                "jal" => Ok(Self::JAL),
                "bt" => Ok(Self::BT),
                "bf" => Ok(Self::BF),
                "li" => Ok(Self::LI),
                _ => Err(()),
            }
        }
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

    impl fmt::Display for OpcodeRegReg {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let name = match self {
                Self::JRAL => "jral",
                Self::MV => "mv",
                Self::MUL => "mul",
                Self::DIV => "div",
                Self::NOT => "not",
                Self::NEG => "neg",
                Self::MULU => "mulu",
                Self::DIVU => "divu",
                Self::SEB => "seb",
                Self::TEZ => "tez",
                Self::TNZ => "tnz",
            };
            write!(f, "{}", name)
        }
    }

    impl TryFrom<&str> for OpcodeRegReg {
        type Error = ();

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            match value {
                "jral" => Ok(Self::JRAL),
                "mv" => Ok(Self::MV),
                "mul" => Ok(Self::MUL),
                "div" => Ok(Self::DIV),
                "not" => Ok(Self::NOT),
                "neg" => Ok(Self::NEG),
                "mulu" => Ok(Self::MULU),
                "divu" => Ok(Self::DIVU),
                "seb" => Ok(Self::SEB),
                "tez" => Ok(Self::TEZ),
                "tnz" => Ok(Self::TNZ),
                _ => Err(()),
            }
        }
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

    impl fmt::Display for OpcodeRegRegReg {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let name = match self {
                Self::ADD => "add",
                Self::SUB => "sub",
                Self::OR => "or",
                Self::XOR => "xor",
                Self::AND => "and",
                Self::ADDU => "addu",
                Self::SUBU => "subu",
                Self::SHL => "shl",
                Self::SHR => "shr",
                Self::SHRA => "shra",
                Self::TLT => "tlt",
                Self::TGE => "tge",
                Self::TEQ => "teq",
                Self::TNE => "tne",
                Self::TLTU => "tltu",
                Self::TGEU => "tgeu",
            };
            write!(f, "{}", name)
        }
    }

    impl TryFrom<&str> for OpcodeRegRegReg {
        type Error = ();

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            match value {
                "add" => Ok(Self::ADD),
                "sub" => Ok(Self::SUB),
                "or" => Ok(Self::OR),
                "xor" => Ok(Self::XOR),
                "and" => Ok(Self::AND),
                "addu" => Ok(Self::ADDU),
                "subu" => Ok(Self::SUBU),
                "shl" => Ok(Self::SHL),
                "shr" => Ok(Self::SHR),
                "shra" => Ok(Self::SHRA),
                "tlt" => Ok(Self::TLT),
                "tge" => Ok(Self::TGE),
                "teq" => Ok(Self::TEQ),
                "tne" => Ok(Self::TNE),
                "tltu" => Ok(Self::TLTU),
                "tgeu" => Ok(Self::TGEU),
                _ => Err(()),
            }
        }
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

    impl fmt::Display for OpcodeRegRegImm {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let name = match self {
                Self::LW => "lw",
                Self::LBS => "lbs",
                Self::LBU => "lbu",
                Self::SW => "sw",
                Self::SB => "sb",
                Self::ADDI => "addi",
                Self::SUBI => "subi",
                Self::ORI => "ori",
                Self::XORI => "xori",
                Self::ANDI => "andi",
            };
            write!(f, "{}", name)
        }
    }

    impl TryFrom<&str> for OpcodeRegRegImm {
        type Error = ();

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            match value {
                "lw" => Ok(Self::LW),
                "lbs" => Ok(Self::LBS),
                "lbu" => Ok(Self::LBU),
                "sw" => Ok(Self::SW),
                "sb" => Ok(Self::SB),
                "addi" => Ok(Self::ADDI),
                "subi" => Ok(Self::SUBI),
                "ori" => Ok(Self::ORI),
                "xori" => Ok(Self::XORI),
                "andi" => Ok(Self::ANDI),
                _ => Err(()),
            }
        }
    }
}

use ops::*;

#[derive(Debug, Clone, Copy)]
pub enum Instr<R = Reg, Imm = s16> {
    /// No arguments, opcode only
    O { opcode: OpcodeOp },
    /// Single address argument
    A { opcode: OpcodeAddr, offset: Imm },
    /// Single immediate argument
    I {
        opcode: OpcodeImm,
        /// A 10-bit immediate.
        imm10: Imm,
    },
    /// Single register argument
    R { opcode: OpcodeReg, reg: R },
    /// One register and one immediate arguments
    RI {
        opcode: OpcodeRegImm,
        reg: R,
        imm: Imm,
    },
    /// Two register arguments
    RR {
        opcode: OpcodeRegReg,
        reg1: R,
        reg2: R,
    },
    /// Three register arguments
    RRR {
        opcode: OpcodeRegRegReg,
        reg1: R,
        reg2: R,
        reg3: R,
    },
    /// Two register and one immediate arguments
    RRI {
        opcode: OpcodeRegRegImm,
        reg1: R,
        reg2: R,
        /// A 10-bit immediate.
        imm10: Imm,
    },
}

impl<R: fmt::Display, Imm: fmt::Display> fmt::Display for Instr<R, Imm> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instr::O { opcode } => write!(f, "{opcode}"),
            Instr::A { opcode, offset } => write!(f, "{opcode} {offset}"),
            Instr::I { opcode, imm10 } => write!(f, "{opcode} {imm10}"),
            Instr::R { opcode, reg } => write!(f, "{opcode} {reg}"),
            Instr::RI { opcode, reg, imm } => write!(f, "{opcode} {reg}, {imm}"),
            Instr::RR { opcode, reg1, reg2 } => write!(f, "{opcode} {reg1}, {reg2}"),
            Instr::RRR {
                opcode,
                reg1,
                reg2,
                reg3,
            } => write!(f, "{opcode} {reg1}, {reg2}, {reg3}"),
            Instr::RRI {
                opcode,
                reg1,
                reg2,
                imm10,
            } => write!(f, "{opcode} {reg1}, {reg2}, imm={imm10}"),
        }
    }
}
