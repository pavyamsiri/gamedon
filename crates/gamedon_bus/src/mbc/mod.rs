use crate::{BusReader, BusWriter};
pub(crate) use mbc1::Mbc1;
pub(crate) use nombc::NoMbc;
use thiserror::Error;

mod bank;
mod mbc1;
mod nombc;

/// The number of bytes in a kibibyte.
const KIB_IN_BYTES: usize = 1024;

#[derive(Error, Debug)]
pub enum RomLoadError {
    #[error(
        "The cartridge expected a ROM with a size of {expected} bytes but it was given {actual} bytes."
    )]
    WrongSize { expected: usize, actual: usize },
}

#[derive(Error, Debug)]
pub(crate) enum RamLoadError {
    #[error(
        "The cartridge expected an external RAM with a size of {expected} bytes but it was given {actual} bytes."
    )]
    WrongSize { expected: usize, actual: usize },
}

#[derive(Error, Debug)]
pub enum MbcCreationError {
    #[error(
        "The MBC only supports ROMs of size {max} bytes but the requested ROM size was {actual} bytes."
    )]
    WrongRomSize { max: usize, actual: usize },
    #[error(
        "The MBC only supports external RAM of size {max} bytes but the requested RAM size was {actual} bytes."
    )]
    WrongRamSize { max: usize, actual: usize },
}

pub(crate) trait MemoryBankController: BusReader + BusWriter {
    fn load_rom(&mut self, bytes: &[u8]) -> Result<(), RomLoadError>;
    fn load_ram(&mut self, bytes: &[u8]) -> Result<(), RamLoadError>;
    fn dump_ram(&self) -> Vec<u8>;
    fn update(&mut self, num_m_cycles: usize);
}

#[derive(Debug)]
pub(crate) enum MbcKind {
    /// For games that are 32KiB or less, the ROM gets directly mapped
    /// to memory at $0000-$7FFF.
    RomOnly {
        has_battery: bool,
    },
    Mbc1 {
        has_battery: bool,
    },
    Mbc2 {
        has_battery: bool,
    },
    Mbc3 {
        has_timer: bool,
        has_battery: bool,
    },
    Mbc5 {
        has_battery: bool,
        has_rumble: bool,
    },
    Mbc6,
    Mbc7,
    Camera,
    BandaiTama5,
    Mmm01 {
        has_battery: bool,
    },
    M161,
    HuC1,
    HuCDash3,
}

impl MbcKind {
    pub(crate) const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 | 0x08 => Some(Self::RomOnly { has_battery: false }),
            0x01 | 0x02 => Some(Self::Mbc1 { has_battery: false }),
            0x03 => Some(Self::Mbc1 { has_battery: true }),
            0x05 => Some(Self::Mbc2 { has_battery: false }),
            0x06 => Some(Self::Mbc2 { has_battery: true }),
            0x09 => Some(Self::RomOnly { has_battery: true }),
            0x0B | 0x0C => Some(Self::Mmm01 { has_battery: false }),
            0x0D => Some(Self::Mmm01 { has_battery: true }),
            0x0F | 0x10 => Some(Self::Mbc3 {
                has_timer: true,
                has_battery: true,
            }),
            0x11 | 0x12 => Some(Self::Mbc3 {
                has_timer: false,
                has_battery: false,
            }),
            0x13 => Some(Self::Mbc3 {
                has_timer: false,
                has_battery: true,
            }),
            0x19 | 0x1A => Some(Self::Mbc5 {
                has_battery: false,
                has_rumble: false,
            }),
            0x1B => Some(Self::Mbc5 {
                has_battery: true,
                has_rumble: false,
            }),
            0x1C | 0x1D => Some(Self::Mbc5 {
                has_battery: false,
                has_rumble: true,
            }),
            0x1E => Some(Self::Mbc5 {
                has_battery: true,
                has_rumble: true,
            }),
            0x20 => Some(Self::Mbc6),
            0x22 => Some(Self::Mbc7),
            0xFC => Some(Self::Camera),
            0xFD => Some(Self::BandaiTama5),
            0xFE => Some(Self::HuCDash3),
            0xFF => Some(Self::HuC1),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum TaggedMbc {
    RomOnly(NoMbc),
    Mbc1(Mbc1),
}

impl MemoryBankController for TaggedMbc {
    fn load_rom(&mut self, bytes: &[u8]) -> Result<(), RomLoadError> {
        match self {
            TaggedMbc::RomOnly(mbc) => mbc.load_rom(bytes),
            TaggedMbc::Mbc1(mbc) => mbc.load_rom(bytes),
        }
    }

    fn load_ram(&mut self, bytes: &[u8]) -> Result<(), RamLoadError> {
        match self {
            TaggedMbc::RomOnly(mbc) => mbc.load_ram(bytes),
            TaggedMbc::Mbc1(mbc) => mbc.load_ram(bytes),
        }
    }

    fn dump_ram(&self) -> Vec<u8> {
        match self {
            TaggedMbc::RomOnly(mbc) => mbc.dump_ram(),
            TaggedMbc::Mbc1(mbc) => mbc.dump_ram(),
        }
    }

    fn update(&mut self, num_m_cycles: usize) {
        match self {
            TaggedMbc::RomOnly(mbc) => mbc.update(num_m_cycles),
            TaggedMbc::Mbc1(mbc) => mbc.update(num_m_cycles),
        }
    }
}

impl BusReader for TaggedMbc {
    fn read_byte(&self, address: u16) -> Result<u8, crate::ReadByteError> {
        match self {
            TaggedMbc::RomOnly(mbc) => mbc.read_byte(address),
            TaggedMbc::Mbc1(mbc) => mbc.read_byte(address),
        }
    }
}

impl BusWriter for TaggedMbc {
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), crate::WriteByteError> {
        match self {
            TaggedMbc::RomOnly(mbc) => mbc.write_byte(address, value),
            TaggedMbc::Mbc1(mbc) => mbc.write_byte(address, value),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RomSize {
    Rom32KiB,
    Rom64KiB,
    Rom128KiB,
    Rom256KiB,
    Rom512KiB,
    Rom1024KiB,
    Rom2048KiB,
    Rom4096KiB,
    Rom8192KiB,
}

impl RomSize {
    pub(crate) const fn get_size_in_bytes(&self) -> usize {
        match self {
            Self::Rom32KiB => 32 * KIB_IN_BYTES,
            Self::Rom64KiB => 64 * KIB_IN_BYTES,
            Self::Rom128KiB => 128 * KIB_IN_BYTES,
            Self::Rom256KiB => 256 * KIB_IN_BYTES,
            Self::Rom512KiB => 512 * KIB_IN_BYTES,
            Self::Rom1024KiB => 1024 * KIB_IN_BYTES,
            Self::Rom2048KiB => 2048 * KIB_IN_BYTES,
            Self::Rom4096KiB => 4096 * KIB_IN_BYTES,
            Self::Rom8192KiB => 8192 * KIB_IN_BYTES,
        }
    }

    pub(crate) const fn get_number_of_banks(&self) -> usize {
        match self {
            Self::Rom32KiB => 2,
            Self::Rom64KiB => 4,
            Self::Rom128KiB => 8,
            Self::Rom256KiB => 16,
            Self::Rom512KiB => 32,
            Self::Rom1024KiB => 64,
            Self::Rom2048KiB => 128,
            Self::Rom4096KiB => 256,
            Self::Rom8192KiB => 512,
        }
    }

    pub(crate) const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::Rom32KiB),
            0x01 => Some(Self::Rom64KiB),
            0x02 => Some(Self::Rom128KiB),
            0x03 => Some(Self::Rom256KiB),
            0x04 => Some(Self::Rom512KiB),
            0x05 => Some(Self::Rom1024KiB),
            0x06 => Some(Self::Rom2048KiB),
            0x07 => Some(Self::Rom4096KiB),
            0x08 => Some(Self::Rom8192KiB),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RamSize {
    Ram0KiB,
    Ram8KiB,
    Ram32KiB,
    Ram64KiB,
    Ram128KiB,
}

impl RamSize {
    pub(crate) const fn get_size_in_bytes(&self) -> usize {
        match self {
            RamSize::Ram0KiB => 0,
            RamSize::Ram8KiB => 8 * KIB_IN_BYTES,
            RamSize::Ram32KiB => 32 * KIB_IN_BYTES,
            RamSize::Ram64KiB => 64 * KIB_IN_BYTES,
            RamSize::Ram128KiB => 128 * KIB_IN_BYTES,
        }
    }

    pub(crate) const fn get_number_of_banks(&self) -> usize {
        match self {
            RamSize::Ram0KiB => 0,
            RamSize::Ram8KiB => 1,
            RamSize::Ram32KiB => 4,
            RamSize::Ram64KiB => 8,
            RamSize::Ram128KiB => 16,
        }
    }

    pub(crate) const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::Ram0KiB),
            0x01 => Some(Self::Ram8KiB),
            0x02 => Some(Self::Ram8KiB),
            0x03 => Some(Self::Ram32KiB),
            0x04 => Some(Self::Ram128KiB),
            0x05 => Some(Self::Ram64KiB),
            _ => None,
        }
    }
}
