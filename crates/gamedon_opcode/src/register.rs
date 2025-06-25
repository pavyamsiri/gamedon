#[derive(Debug, Clone, Copy)]
pub enum Reg8 {
    B,
    C,
    D,
    E,
    H,
    L,
    A,
}

impl core::fmt::Display for Reg8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reg8::B => write!(f, "B"),
            Reg8::C => write!(f, "C"),
            Reg8::D => write!(f, "D"),
            Reg8::E => write!(f, "E"),
            Reg8::H => write!(f, "H"),
            Reg8::L => write!(f, "L"),
            Reg8::A => write!(f, "A"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Reg16 {
    BC,
    DE,
    HL,
    SP,
    AF,
}

impl core::fmt::Display for Reg16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reg16::BC => write!(f, "BC"),
            Reg16::DE => write!(f, "DE"),
            Reg16::HL => write!(f, "HL"),
            Reg16::SP => write!(f, "SP"),
            Reg16::AF => write!(f, "AF"),
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct RegFlags(u8);

impl RegFlags {
    pub const Z: Self = Self(1 << 7);
    pub const N: Self = Self(1 << 6);
    pub const H: Self = Self(1 << 5);
    pub const C: Self = Self(1 << 4);
}

impl RegFlags {
    #[inline]
    pub const fn bits(self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn contains(&self, other: Self) -> bool {
        self.bits() & other.bits() == other.bits()
    }

    #[inline]
    pub const fn insert(&mut self, other: Self) {
        self.0 = self.bits() | other.bits();
    }

    #[inline]
    pub const fn remove(&mut self, other: Self) {
        self.0 = self.bits() & !other.bits();
    }

    #[inline]
    pub const fn set(&mut self, other: Self, value: bool) {
        if value {
            self.insert(other);
        } else {
            self.remove(other);
        }
    }
}
