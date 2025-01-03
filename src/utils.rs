use core::fmt;

/// Represents a signed or unsigned 16-bit number. The sign is determined by the
/// user.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub union s16 {
    u16: u16,
    i16: i16,
}

impl s16 {
    pub const ZERO: Self = Self { u16: 0 };

    pub fn as_u16(&self) -> u16 {
        unsafe { self.u16 }
    }

    pub fn as_u16_mut(&mut self) -> &mut u16 {
        unsafe { &mut self.u16 }
    }

    pub fn as_i16(&self) -> i16 {
        unsafe { self.i16 }
    }

    pub fn as_i16_mut(&mut self) -> &mut i16 {
        unsafe { &mut self.i16 }
    }
}

impl Default for s16 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<u16> for s16 {
    fn from(value: u16) -> Self {
        Self { u16: value }
    }
}

impl From<i16> for s16 {
    fn from(value: i16) -> Self {
        Self { i16: value }
    }
}

impl From<u8> for s16 {
    fn from(value: u8) -> Self {
        Self { u16: value.into() }
    }
}

impl From<s16> for u16 {
    fn from(value: s16) -> Self {
        unsafe { value.u16 }
    }
}

impl From<s16> for i16 {
    fn from(value: s16) -> Self {
        unsafe { value.i16 }
    }
}

impl From<s16> for bool {
    fn from(value: s16) -> Self {
        unsafe { value.u16 != 0 }
    }
}

impl From<bool> for s16 {
    fn from(value: bool) -> Self {
        Self { u16: value as u16 }
    }
}

impl fmt::Debug for s16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let signed = self.as_i16();
        let unsigned = self.as_u16();
        if signed < 0 {
            write!(f, "s16{{{unsigned}u, {signed}i}}")
        } else {
            write!(f, "s16{{{unsigned}}}")
        }
    }
}
