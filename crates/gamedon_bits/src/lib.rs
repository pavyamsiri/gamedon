use core::fmt;

pub use latch::FlagLatch;

mod display;
mod latch;

/// Represents all possible bit shifts for an 8-bit value.
#[derive(Debug, Clone, Copy)]
pub enum BitShift8 {
    /// Shift by 0 bits.
    Bit0,
    /// Shift by 1 bit.
    Bit1,
    /// Shift by 2 bits.
    Bit2,
    /// Shift by 3 bits.
    Bit3,
    /// Shift by 4 bits.
    Bit4,
    /// Shift by 5 bits.
    Bit5,
    /// Shift by 6 bits.
    Bit6,
    /// Shift by 7 bits.
    Bit7,
}

impl BitShift8 {
    /// The amount of bits to shift.
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

/// Represents all possible bit shifts for an 16-bit value.
#[derive(Debug, Clone, Copy)]
pub enum BitShift16 {
    /// Shift by 0 bits.
    Bit00,
    /// Shift by 1 bits.
    Bit01,
    /// Shift by 2 bits.
    Bit02,
    /// Shift by 3 bits.
    Bit03,
    /// Shift by 4 bits.
    Bit04,
    /// Shift by 5 bits.
    Bit05,
    /// Shift by 6 bits.
    Bit06,
    /// Shift by 7 bits.
    Bit07,
    /// Shift by 8 bits.
    Bit08,
    /// Shift by 9 bits.
    Bit09,
    /// Shift by 10 bits.
    Bit10,
    /// Shift by 11 bits.
    Bit11,
    /// Shift by 12 bits.
    Bit12,
    /// Shift by 13 bits.
    Bit13,
    /// Shift by 14 bits.
    Bit14,
    /// Shift by 15 bits.
    Bit15,
}

impl BitShift16 {
    /// The amount of bits to shift.
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

/// Represents an 8-bit value.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct HwReg8(pub u8);

impl fmt::Debug for HwReg8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#04X}", self.0)
    }
}

impl HwReg8 {
    /// Return whether a bit is set.
    #[inline]
    pub const fn bit(self, shift: BitShift8) -> bool {
        (self.0 & (1 << shift.get_shift_amount())) != 0
    }

    /// Set a bit.
    #[inline]
    pub const fn set(&mut self, shift: BitShift8) {
        self.0 |= 1 << shift.get_shift_amount();
    }

    /// Reset a bit.
    #[inline]
    pub const fn reset(&mut self, shift: BitShift8) {
        self.0 &= !(1 << shift.get_shift_amount());
    }

    /// Set a bit to a boolean value.
    #[inline]
    pub const fn set_value(&mut self, shift: BitShift8, value: bool) {
        if value {
            self.set(shift);
        } else {
            self.reset(shift);
        }
    }
}

/// Represents an 16-bit value.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct HwReg16(pub u16);

impl core::fmt::Debug for HwReg16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#06X}", self.0)
    }
}

impl HwReg16 {
    /// Return whether a bit is set.
    #[inline]
    pub const fn bit(self, shift: BitShift16) -> bool {
        (self.0 & (1 << shift.get_shift_amount())) != 0
    }

    /// Set a bit.
    #[inline]
    pub const fn set(&mut self, shift: BitShift16) {
        self.0 |= 1 << shift.get_shift_amount();
    }

    /// Reset a bit.
    #[inline]
    pub const fn reset(&mut self, shift: BitShift16) {
        self.0 &= !(1 << shift.get_shift_amount());
    }

    /// Return the upper byte.
    #[inline]
    pub const fn upper(self) -> u8 {
        self.0.to_be_bytes()[0]
    }
}
