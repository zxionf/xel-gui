use std::ops::BitOr;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct UIDebugFlag(u8);

impl UIDebugFlag {
    pub const NONE: Self = Self(0);
    pub const DEBUG_WIDGETS: Self = Self(0b0000_0001);
    pub const DEBUG_EVENTS: Self = Self(0b0000_0010);

    pub fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }

    pub fn insert(&mut self, flag: Self) {
        self.0 |= flag.0;
    }
}

impl BitOr for UIDebugFlag {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}