mod display;

#[derive(Debug, Clone, Copy)]
pub enum BitShift8 {
    Bit0,
    Bit1,
    Bit2,
    Bit3,
    Bit4,
    Bit5,
    Bit6,
    Bit7,
}

impl BitShift8 {
    #[inline]
    pub const fn get_shift_amount(self) -> usize {
        match self {
            Self::Bit0 => 0,
            Self::Bit1 => 1,
            Self::Bit2 => 2,
            Self::Bit3 => 3,
            Self::Bit4 => 4,
            Self::Bit5 => 5,
            Self::Bit6 => 6,
            Self::Bit7 => 7,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BitShift16 {
    Bit00,
    Bit01,
    Bit02,
    Bit03,
    Bit04,
    Bit05,
    Bit06,
    Bit07,
    Bit08,
    Bit09,
    Bit10,
    Bit11,
    Bit12,
    Bit13,
    Bit14,
    Bit15,
}

impl BitShift16 {
    #[inline]
    pub const fn get_shift_amount(self) -> usize {
        match self {
            Self::Bit00 => 0,
            Self::Bit01 => 1,
            Self::Bit02 => 2,
            Self::Bit03 => 3,
            Self::Bit04 => 4,
            Self::Bit05 => 5,
            Self::Bit06 => 6,
            Self::Bit07 => 7,
            Self::Bit08 => 8,
            Self::Bit09 => 9,
            Self::Bit10 => 10,
            Self::Bit11 => 11,
            Self::Bit12 => 12,
            Self::Bit13 => 13,
            Self::Bit14 => 14,
            Self::Bit15 => 15,
        }
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct HwReg8(pub u8);

impl core::fmt::Debug for HwReg8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#04X}", self.0)
    }
}

impl HwReg8 {
    #[inline]
    pub const fn bit(self, shift: BitShift8) -> bool {
        (self.0 & (1 << shift.get_shift_amount())) != 0
    }

    #[inline]
    pub const fn set(&mut self, shift: BitShift8) {
        self.0 |= 1 << shift.get_shift_amount();
    }

    #[inline]
    pub const fn reset(&mut self, shift: BitShift8) {
        self.0 &= !(1 << shift.get_shift_amount());
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct HwReg16(pub u16);

impl core::fmt::Debug for HwReg16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#06X}", self.0)
    }
}

impl HwReg16 {
    #[inline]
    pub const fn bit(self, shift: BitShift16) -> bool {
        (self.0 & (1 << shift.get_shift_amount())) != 0
    }

    #[inline]
    pub const fn set(&mut self, shift: BitShift16) {
        self.0 |= 1 << shift.get_shift_amount();
    }

    #[inline]
    pub const fn reset(&mut self, shift: BitShift16) {
        self.0 &= !(1 << shift.get_shift_amount());
    }

    #[inline]
    pub const fn upper(self) -> u8 {
        self.0.to_be_bytes()[0]
    }
}
