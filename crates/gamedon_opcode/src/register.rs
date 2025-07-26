use core::fmt;
use owo_colors::{OwoColorize, Stream, Style};

macro_rules! apply_colour {
    ($token:expr, $style:expr) => {
        $token.if_supports_color(Stream::Stdout, |text| text.style($style))
    };
}

/// The style to print registers with.
const REGISTER_STYLE: Style = Style::new().blue();

/// The CPU registers.
#[derive(Clone, Copy, Default)]
pub struct Registers {
    /// The 16-bit accumulator.
    a: u8,
    /// The register flags.
    flags: RegFlags,
    /// The general purpose 16-bit BC register.
    bc: u16,
    /// The general purpose 16-bit DE register.
    de: u16,
    /// The general purpose 16-bit HL register.
    hl: u16,
    /// The stack pointer.
    sp: u16,
    /// The program counter.
    pc: u16,
}

impl fmt::Debug for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Registers")
            .field("af", &format_args!("{:#06X}", &self.get_af()))
            .field("bc", &format_args!("{:#06X}", &self.get_bc()))
            .field("de", &format_args!("{:#06X}", &self.get_de()))
            .field("hl", &format_args!("{:#06X}", &self.get_hl()))
            .field("sp", &format_args!("{:#06X}", &self.get_sp()))
            .field("pc", &format_args!("{:#06X}", &self.get_pc()))
            .finish()
    }
}

// 8-bit getters
impl Registers {
    /// Get an 8-bit register value.
    #[inline]
    pub const fn get_reg8(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::B => self.get_b(),
            Reg8::C => self.get_c(),
            Reg8::D => self.get_d(),
            Reg8::E => self.get_e(),
            Reg8::H => self.get_h(),
            Reg8::L => self.get_l(),
            Reg8::A => self.get_a(),
            Reg8::F => self.get_f(),
            Reg8::S => self.get_s(),
            Reg8::P => self.get_p(),
        }
    }

    /// Get the value of the `A` register part of `AF`.
    #[inline]
    pub const fn get_a(&self) -> u8 {
        self.a
    }

    /// Get the value of the `F` register part of `AF`.
    #[inline]
    pub const fn get_f(&self) -> u8 {
        self.flags.bits()
    }

    /// Get the value of the `B` register part of `BC`.
    #[inline]
    pub const fn get_b(&self) -> u8 {
        (self.bc >> 8) as u8
    }

    /// Get the value of the `D` register part of `DE`.
    #[inline]
    pub const fn get_d(&self) -> u8 {
        (self.de >> 8) as u8
    }

    /// Get the value of the `H` register part of `HL`.
    #[inline]
    pub const fn get_h(&self) -> u8 {
        (self.hl >> 8) as u8
    }

    /// Get the value of the `S` register part of `SP.`
    #[inline]
    pub const fn get_s(&self) -> u8 {
        (self.sp >> 8) as u8
    }

    /// Get the value of the `C` register part of `BC`.
    #[inline]
    pub const fn get_c(&self) -> u8 {
        (self.bc & 0x00FF) as u8
    }

    /// Get the value of the `E` register part of `DE`.
    #[inline]
    pub const fn get_e(&self) -> u8 {
        (self.de & 0x00FF) as u8
    }

    /// Get the value of the `L` register part of `HL`.
    #[inline]
    pub const fn get_l(&self) -> u8 {
        (self.hl & 0x00FF) as u8
    }

    /// Get the value of the `P` register part of `SP.`
    #[inline]
    pub const fn get_p(&self) -> u8 {
        (self.sp & 0x00FF) as u8
    }
}

// 8-bit setters
impl Registers {
    /// Set the value of an 8-bit register.
    #[inline]
    pub fn set_reg8(&mut self, reg: Reg8, value: u8) {
        match reg {
            Reg8::B => self.set_b(value),
            Reg8::C => self.set_c(value),
            Reg8::D => self.set_d(value),
            Reg8::E => self.set_e(value),
            Reg8::H => self.set_h(value),
            Reg8::L => self.set_l(value),
            Reg8::A => self.set_a(value),
            Reg8::P => self.set_p(value),
            Reg8::F => self.set_f(value),
            Reg8::S => self.set_s(value),
        }
    }

    /// Set the value of the `A` register.
    #[inline]
    pub const fn set_a(&mut self, value: u8) {
        self.a = value;
    }

    /// Set the value of the `F` register.
    #[inline]
    pub const fn set_f(&mut self, value: u8) {
        self.flags = RegFlags::from_bits(value);
    }

    /// Set the value of the `B` register.
    #[inline]
    pub const fn set_b(&mut self, value: u8) {
        self.bc = (self.bc & 0x00FF) | ((value as u16) << 8);
    }

    /// Set the value of the `D` register.
    #[inline]
    pub const fn set_d(&mut self, value: u8) {
        self.de = (self.de & 0x00FF) | ((value as u16) << 8);
    }

    /// Set the value of the `H` register.
    #[inline]
    pub const fn set_h(&mut self, value: u8) {
        self.hl = (self.hl & 0x00FF) | ((value as u16) << 8);
    }

    /// Set the value of the `S` register.
    #[inline]
    pub const fn set_s(&mut self, value: u8) {
        self.sp = (self.sp & 0x00FF) | ((value as u16) << 8);
    }

    /// Set the value of the `C` register.
    #[inline]
    pub fn set_c(&mut self, value: u8) {
        self.bc = (self.bc & 0xFF00) | (value as u16);
    }

    /// Set the value of the `E` register.
    #[inline]
    pub const fn set_e(&mut self, value: u8) {
        self.de = (self.de & 0xFF00) | (value as u16);
    }

    /// Set the value of the `L` register.
    #[inline]
    pub const fn set_l(&mut self, value: u8) {
        self.hl = (self.hl & 0xFF00) | (value as u16);
    }

    /// Set the value of the `P` register.
    #[inline]
    pub const fn set_p(&mut self, value: u8) {
        self.sp = (self.sp & 0xFF00) | (value as u16);
    }
}
// 16-bit getters
impl Registers {
    /// Get a 16-bit register value.
    #[inline]
    pub const fn get_reg16(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::BC => self.get_bc(),
            Reg16::DE => self.get_de(),
            Reg16::HL => self.get_hl(),
            Reg16::SP => self.get_sp(),
            Reg16::AF => self.get_af(),
        }
    }

    /// Get the value of the `BC` register.
    #[inline]
    pub const fn get_bc(&self) -> u16 {
        self.bc
    }

    /// Get the value of the `DE` register.
    #[inline]
    pub const fn get_de(&self) -> u16 {
        self.de
    }

    /// Get the value of the `HL` register.
    #[inline]
    pub const fn get_hl(&self) -> u16 {
        self.hl
    }

    /// Get the value of the `SP` register.
    #[inline]
    pub const fn get_sp(&self) -> u16 {
        self.sp
    }

    /// Get the value of the `PC` register.
    #[inline]
    pub const fn get_pc(&self) -> u16 {
        self.pc
    }

    /// Get the value of the `AF` register.
    #[inline]
    pub const fn get_af(&self) -> u16 {
        let a = self.get_a();
        let f = self.flags.bits();
        ((a as u16) << 8) | (f as u16)
    }
}

// 16-bit setters
impl Registers {
    /// Set the value of 16-bit register.
    #[inline]
    pub const fn set_reg16(&mut self, reg: Reg16, value: u16) {
        match reg {
            Reg16::BC => self.set_bc(value),
            Reg16::DE => self.set_de(value),
            Reg16::HL => self.set_hl(value),
            Reg16::SP => self.set_sp(value),
            Reg16::AF => self.set_af(value),
        }
    }

    /// Set the value of the `BC` register.
    #[inline]
    pub const fn set_bc(&mut self, value: u16) {
        self.bc = value;
    }

    /// Set the value of the `DE` register.
    #[inline]
    pub const fn set_de(&mut self, value: u16) {
        self.de = value;
    }

    /// Set the value of the `HL` register.
    #[inline]
    pub const fn set_hl(&mut self, value: u16) {
        self.hl = value;
    }

    /// Set the value of the `SP` register.
    #[inline]
    pub const fn set_sp(&mut self, value: u16) {
        self.sp = value;
    }

    /// Set the value of the `PC` register.
    #[inline]
    pub const fn set_pc(&mut self, value: u16) {
        self.pc = value;
    }

    /// Set the value of the `AF` register.
    #[inline]
    pub const fn set_af(&mut self, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = (value & 0xFF) as u8;
        self.a = hi;
        self.flags = RegFlags::from_bits(lo);
    }
}

// flag getters
impl Registers {
    /// Return the zero flag.
    #[inline]
    pub const fn get_zero_flag(&self) -> bool {
        self.flags.contains(RegFlags::Z)
    }

    /// Return the subtraction flag.
    #[inline]
    pub const fn get_subtraction_flag(&self) -> bool {
        self.flags.contains(RegFlags::N)
    }

    /// Return the half carry flag.
    #[inline]
    pub const fn get_half_carry_flag(&self) -> bool {
        self.flags.contains(RegFlags::H)
    }

    /// Return the carry flag.
    #[inline]
    pub const fn get_carry_flag(&self) -> bool {
        self.flags.contains(RegFlags::C)
    }
}

// flag setters
impl Registers {
    /// Set multiple flags at once.
    #[inline]
    pub const fn set_flags(&mut self, other: RegFlags, value: bool) {
        self.flags.set(other, value);
    }

    /// Set the zero flag.
    #[inline]
    pub const fn set_zero_flag(&mut self, value: bool) {
        self.flags.set(RegFlags::Z, value);
    }

    /// Set the subtraction flag.
    #[inline]
    pub const fn set_subtraction_flag(&mut self, value: bool) {
        self.flags.set(RegFlags::N, value);
    }

    /// Set the half carry flag.
    #[inline]
    pub const fn set_half_carry_flag(&mut self, value: bool) {
        self.flags.set(RegFlags::H, value);
    }

    /// Set the carry flag.
    #[inline]
    pub const fn set_carry_flag(&mut self, value: bool) {
        self.flags.set(RegFlags::C, value);
    }
}

/// Represents addressable CPU 8-bit registers.
#[derive(Debug, Clone, Copy)]
pub enum Reg8 {
    /// The `B` register forming the upper byte of the `BC` register.
    B,
    /// The `C` register forming the lower byte of the `BC` register.
    C,
    /// The `D` register forming the upper byte of the `DE` register.
    D,
    /// The `E` register forming the lower byte of the `DE` register.
    E,
    /// The `H` register forming the upper byte of the `HL` register.
    H,
    /// The `L` register forming the lower byte of the `HL` register.
    L,
    /// The `A` register forming the upper byte of the `AF` register.
    A,
    /// The `F` register forming the lower byte of the `AF` register.
    F,
    /// The `S` register forming the upper byte of the `SP` register.
    S,
    /// The `P` register forming the lower byte of the `SP` register.
    P,
}

impl fmt::Display for Reg8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::B => write!(f, "{}", apply_colour!("B", REGISTER_STYLE)),
            Self::C => write!(f, "{}", apply_colour!("C", REGISTER_STYLE)),
            Self::D => write!(f, "{}", apply_colour!("D", REGISTER_STYLE)),
            Self::E => write!(f, "{}", apply_colour!("E", REGISTER_STYLE)),
            Self::H => write!(f, "{}", apply_colour!("H", REGISTER_STYLE)),
            Self::L => write!(f, "{}", apply_colour!("L", REGISTER_STYLE)),
            Self::A => write!(f, "{}", apply_colour!("A", REGISTER_STYLE)),
            Self::P => write!(f, "{}", apply_colour!("P", REGISTER_STYLE)),
            Self::F => write!(f, "{}", apply_colour!("F", REGISTER_STYLE)),
            Self::S => write!(f, "{}", apply_colour!("S", REGISTER_STYLE)),
        }
    }
}

/// Represents addressable CPU 16-bit registers.
#[derive(Debug, Clone, Copy)]
pub enum Reg16 {
    /// The `AF` register.
    AF,
    /// The `BC` register.
    BC,
    /// The `DE` register.
    DE,
    /// The `HL` register.
    HL,
    /// The `SP` register.
    SP,
}

impl Reg16 {
    /// Return the 8-bit register that contains the lower byte.
    pub const fn lo(self) -> Reg8 {
        match self {
            Reg16::BC => Reg8::C,
            Reg16::DE => Reg8::E,
            Reg16::HL => Reg8::L,
            Reg16::SP => Reg8::P,
            Reg16::AF => Reg8::F,
        }
    }

    /// Return the 8-bit register that contains the upper byte.
    pub const fn hi(self) -> Reg8 {
        match self {
            Reg16::BC => Reg8::B,
            Reg16::DE => Reg8::D,
            Reg16::HL => Reg8::H,
            Reg16::SP => Reg8::S,
            Reg16::AF => Reg8::A,
        }
    }
}

impl fmt::Display for Reg16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reg16::BC => write!(f, "{}", apply_colour!("BC", REGISTER_STYLE)),
            Reg16::DE => write!(f, "{}", apply_colour!("DE", REGISTER_STYLE)),
            Reg16::HL => write!(f, "{}", apply_colour!("HL", REGISTER_STYLE)),
            Reg16::SP => write!(f, "{}", apply_colour!("SP", REGISTER_STYLE)),
            Reg16::AF => write!(f, "{}", apply_colour!("AF", REGISTER_STYLE)),
        }
    }
}

/// Represents the four CPU arithmetic flags.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RegFlags(u8);

impl RegFlags {
    /// The zero flag bit.
    pub const Z: Self = Self(1 << 7);
    /// The subtraction flag bit.
    pub const N: Self = Self(1 << 6);
    /// The half carry flag bit.
    pub const H: Self = Self(1 << 5);
    /// The carry flag bit.
    pub const C: Self = Self(1 << 4);
}

impl RegFlags {
    /// Construct from a byte.
    #[inline]
    pub const fn from_bits(value: u8) -> Self {
        Self(value & 0xF0)
    }

    /// Convert into a byte.
    #[inline]
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Construct an empty flags struct.
    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Return whether the current flags contain the given flags.
    #[inline]
    pub const fn contains(&self, other: Self) -> bool {
        self.bits() & other.bits() == other.bits()
    }

    /// Insert a set of flags.
    #[inline]
    pub const fn insert(&mut self, other: Self) {
        self.0 = self.bits() | other.bits();
    }

    /// Remove a set of flags.
    #[inline]
    pub const fn remove(&mut self, other: Self) {
        self.0 = self.bits() & !other.bits();
    }

    /// Set multiple flags at once.
    #[inline]
    pub const fn set(&mut self, other: Self, value: bool) {
        if value {
            self.insert(other);
        } else {
            self.remove(other);
        }
    }
}
