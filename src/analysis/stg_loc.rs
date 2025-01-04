use crate::cpu::regs::Reg;

/// Represents a Storage Location.
/// These will only be used for intra-procedural analysis (within one subroutine), so hopefully
/// `$sp` and `$gp` can be assumed to be constant throughout.
pub enum StgLoc {
    Reg(Reg),
    /// A (possibly unbound) alias to another storage location.
    Tmp(String),
    StackVar {
        simm10_offset: i16,
    },
    GlobalVar {
        simm10_offset: i16,
    },
}

impl From<Reg> for StgLoc {
    fn from(reg: Reg) -> Self {
        Self::Reg(reg)
    }
}

impl StgLoc {
    pub fn stack_var(simm10_offset: i16) -> Self {
        Self::StackVar { simm10_offset }
    }
    pub fn global_var(simm10_offset: i16) -> Self {
        Self::GlobalVar { simm10_offset }
    }
}
