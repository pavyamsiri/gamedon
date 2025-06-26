use owo_colors::{OwoColorize, Stream, Style};
use owo_colors::{SupportsColorsDisplay, colors::*};

const REGISTER_STYLE: Style = Style::new().blue();

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
            Self::B => write!(
                f,
                "{}",
                "B".if_supports_color(Stream::Stdout, |text| text.style(REGISTER_STYLE))
            ),
            Self::C => write!(
                f,
                "{}",
                "C".if_supports_color(Stream::Stdout, |text| text.style(REGISTER_STYLE))
            ),
            Self::D => write!(
                f,
                "{}",
                "D".if_supports_color(Stream::Stdout, |text| text.style(REGISTER_STYLE))
            ),
            Self::E => write!(
                f,
                "{}",
                "E".if_supports_color(Stream::Stdout, |text| text.style(REGISTER_STYLE))
            ),
            Self::H => write!(
                f,
                "{}",
                "H".if_supports_color(Stream::Stdout, |text| text.style(REGISTER_STYLE))
            ),
            Self::L => write!(
                f,
                "{}",
                "L".if_supports_color(Stream::Stdout, |text| text.style(REGISTER_STYLE))
            ),
            Self::A => write!(
                f,
                "{}",
                "A".if_supports_color(Stream::Stdout, |text| text.style(REGISTER_STYLE))
            ),
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
            Reg16::BC => write!(
                f,
                "{}",
                "BC".if_supports_color(Stream::Stdout, |text| text.style(REGISTER_STYLE))
            ),
            Reg16::DE => write!(
                f,
                "{}",
                "DE".if_supports_color(Stream::Stdout, |text| text.style(REGISTER_STYLE))
            ),
            Reg16::HL => write!(
                f,
                "{}",
                "HL".if_supports_color(Stream::Stdout, |text| text.style(REGISTER_STYLE))
            ),
            Reg16::SP => write!(
                f,
                "{}",
                "SP".if_supports_color(Stream::Stdout, |text| text.style(REGISTER_STYLE))
            ),
            Reg16::AF => write!(
                f,
                "{}",
                "AF".if_supports_color(Stream::Stdout, |text| text.style(REGISTER_STYLE))
            ),
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
