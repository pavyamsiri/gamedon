use crate::{BusReader, BusWriter};
pub(crate) use mbc1::Mbc1;
pub(crate) use nombc::NoMbc;
use thiserror::Error;

/// Implementation of MBC1.
mod mbc1;
/// Implementation of ROM only cartridges.
mod nombc;

/// Addresses mapping to the MBC.
#[macro_export]
macro_rules! mbc_addresses {
    () => {
        0x0000..=0xBFFF
    };
}

/// Errors that can occur when loading a ROM.
#[derive(Error, Debug)]
pub enum RomLoadError {
    /// The given byte slice has a different length compared to the expected ROM size.
    #[error(
        "The cartridge expected a ROM with a size of {expected} bytes but it was given {actual} bytes."
    )]
    WrongSize { expected: usize, actual: usize },
}

/// Errors that can occur when loading saved RAM.
#[allow(dead_code)]
#[derive(Error, Debug)]
pub(crate) enum RamLoadError {
    /// The given byte slice has a different length compared to the expected RAM size.
    #[error(
        "The cartridge expected an external RAM with a size of {expected} bytes but it was given {actual} bytes."
    )]
    WrongSize { expected: usize, actual: usize },
}

/// Errors that can occur when creating a memory bank controller.
#[derive(Error, Debug)]
pub enum MbcCreationError {
    /// The memory bank controller does not support the given ROM size.
    #[error(
        "The MBC only supports ROMs of size {max} bytes but the requested ROM size was {actual} bytes."
    )]
    WrongRomSize { max: usize, actual: usize },
    /// The memory bank controller does not support the given RAM size.
    #[error(
        "The MBC only supports external RAM of size {max} bytes but the requested RAM size was {actual} bytes."
    )]
    WrongRamSize { max: usize, actual: usize },
}

/// Interfaces representing all memory bank controllers.
pub(crate) trait MemoryBankController: BusReader + BusWriter {
    /// Load a ROM represented as a series of bytes into the controller.
    fn load_rom(&mut self, bytes: &[u8]) -> Result<(), RomLoadError>;
    /// Load saved RAM represented as a series of bytes into the controller.
    #[allow(dead_code)]
    fn load_ram(&mut self, bytes: &[u8]) -> Result<(), RamLoadError>;
    /// Dump the RAM into a vector of bytes.
    #[allow(dead_code)]
    fn dump_ram(&self) -> Vec<u8>;
    /// Update the controller by one M-cycle.
    #[allow(dead_code)]
    fn tick(&mut self);
}

/// A memory bank controller along with its tag.
/// This is used to allow dynamic determination of MBC.
#[expect(
    clippy::large_enum_variant,
    reason = "There will only be one MBC per instance so it doesn't matter if it is large."
)]
#[derive(Debug, Clone)]
pub(crate) enum TaggedMbc {
    /// Simple ROM only cartridges with no memory bank controllers.
    RomOnly(NoMbc),
    /// MBC1.
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

    fn tick(&mut self) {
        match self {
            TaggedMbc::RomOnly(mbc) => mbc.tick(),
            TaggedMbc::Mbc1(mbc) => mbc.tick(),
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
