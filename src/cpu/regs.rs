use std::{fmt, str::FromStr};

use crate::utils::s16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Reg {
    Zero = 0,
    Rv = 1,
    Ra = 2,
    A0 = 3,
    A1 = 4,
    A2 = 5,
    S0 = 6,
    S1 = 7,
    S2 = 8,
    T0 = 9,
    T1 = 10,
    T2 = 11,
    K0 = 12,
    K1 = 13,
    Gp = 14,
    Sp = 15,
}

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Zero => "zero",
            Self::Rv => "rv",
            Self::Ra => "ra",
            Self::A0 => "a0",
            Self::A1 => "a1",
            Self::A2 => "a2",
            Self::S0 => "s0",
            Self::S1 => "s1",
            Self::S2 => "s2",
            Self::T0 => "t0",
            Self::T1 => "t1",
            Self::T2 => "t2",
            Self::K0 => "k0",
            Self::K1 => "k1",
            Self::Gp => "gp",
            Self::Sp => "sp",
        };
        write!(f, "{}", name)
    }
}

pub const REG_NAMES: [&str; 16] = [
    "zero", "rv", "ra", "a0", "a1", "a2", "s0", "s1", "s2", "t0", "t1", "t2", "k0", "k1", "gp",
    "sp",
];

impl FromStr for Reg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match if s.starts_with('$') { &s[1..] } else { s } {
            "zero" => Ok(Self::Zero),
            "rv" => Ok(Self::Rv),
            "ra" => Ok(Self::Ra),
            "a0" => Ok(Self::A0),
            "a1" => Ok(Self::A1),
            "a2" => Ok(Self::A2),
            "s0" => Ok(Self::S0),
            "s1" => Ok(Self::S1),
            "s2" => Ok(Self::S2),
            "t0" => Ok(Self::T0),
            "t1" => Ok(Self::T1),
            "t2" => Ok(Self::T2),
            "k0" => Ok(Self::K0),
            "k1" => Ok(Self::K1),
            "gp" => Ok(Self::Gp),
            "sp" => Ok(Self::Sp),
            _ => Err(s.to_string()),
        }
    }
}

impl TryFrom<u8> for Reg {
    type Error = u8;
    fn try_from(id: u8) -> Result<Self, Self::Error> {
        match id {
            0 => Ok(Self::Zero),
            1 => Ok(Self::Rv),
            2 => Ok(Self::Ra),
            3 => Ok(Self::A0),
            4 => Ok(Self::A1),
            5 => Ok(Self::A2),
            6 => Ok(Self::S0),
            7 => Ok(Self::S1),
            8 => Ok(Self::S2),
            9 => Ok(Self::T0),
            10 => Ok(Self::T1),
            11 => Ok(Self::T2),
            12 => Ok(Self::K0),
            13 => Ok(Self::K1),
            14 => Ok(Self::Gp),
            15 => Ok(Self::Sp),
            _ => Err(id),
        }
    }
}
pub struct RegisterFile {
    indexed: [s16; 15],
}

impl RegisterFile {
    pub fn new(start_sp: u16) -> Self {
        let mut regs = Self {
            indexed: [s16::default(); 15],
        };
        regs.set(Reg::Sp, start_sp);
        regs
    }

    /// Returns `None` for the zero register.
    fn reg_offset(reg: Reg) -> Option<usize> {
        (reg as usize).checked_sub(1)
    }

    pub fn get<T: From<s16>>(&self, rd: Reg) -> T {
        // Skip the zero register.
        if let Some(rd) = Self::reg_offset(rd) {
            self.indexed[rd].into()
        } else {
            s16::ZERO.into()
        }
    }

    pub fn set<T: Into<s16>>(&mut self, rd: Reg, value: T) {
        if let Some(rd) = Self::reg_offset(rd) {
            self.indexed[rd] = value.into()
        }
    }
}
