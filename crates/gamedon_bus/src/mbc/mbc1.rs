use super::{
    MbcCreationError, MemoryBankController, RamLoadError, RamSize, RomLoadError, RomSize,
    bank::{MemoryBank, NamedType},
};
use crate::{BusReader, BusWriter, ReadByteError, WriteByteError};

/// Name of the Device.
const NAME: &str = "mbc1";
/// The size of a single ROM bank for MBC1.
const ROM_BANK_SIZE: usize = 0x4000;
/// The size of a single RAM bank for MBC1.
const RAM_BANK_SIZE: usize = 0x2000;
/// The base address for the first ROM bank area.
const FIRST_ROM_BANK_BASE: u16 = 0x0000;
/// The base address for the second ROM bank area.
const SECOND_ROM_BANK_BASE: u16 = 0x4000;
/// The base address for the RAM bank area.
const RAM_BANK_BASE: u16 = 0xA000;

type RomBank = MemoryBank<ROM_BANK_SIZE, Mbc1Name>;
type RamBank = MemoryBank<RAM_BANK_SIZE, Mbc1Name>;

struct Mbc1Name;

impl NamedType for Mbc1Name {
    fn name() -> &'static str {
        NAME
    }
}

#[derive(Debug, Clone, Copy)]
enum SizeMode {
    One,
    Two,
    Three,
    Four,
    Five,
    More,
}

impl SizeMode {
    const fn is_large(self) -> bool {
        matches!(self, Self::More)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Mbc1 {
    /// Up to 128 banks.
    rom_banks: Vec<RomBank>,
    /// Up to 4 banks.
    ram_banks: Vec<RamBank>,
    /// The size mode to determine how many bits to mask in the bank number register.
    size: SizeMode,
    /// Comprised of two different registers.
    /// 1. A 2-bit register (`0bX11X_XXXX`) in bits 5 and 6.
    /// 2. A 5-bit register (`0bXXX1_1111`) in bits 0 to 4.
    bank_number: u8,
    /// RAM enable flag.
    ram_enable: bool,
    /// The banking mode.
    is_simple_mode: bool,
}

impl Mbc1 {
    pub(crate) fn new(rom_size: RomSize, ram_size: RamSize) -> Result<Self, MbcCreationError> {
        let size = match rom_size {
            RomSize::Rom32KiB => SizeMode::One,
            RomSize::Rom64KiB => SizeMode::Two,
            RomSize::Rom128KiB => SizeMode::Three,
            RomSize::Rom256KiB => SizeMode::Four,
            RomSize::Rom512KiB => SizeMode::Five,
            RomSize::Rom1024KiB | RomSize::Rom2048KiB => SizeMode::More,
            _ => {
                return Err(MbcCreationError::WrongRomSize {
                    max: RomSize::Rom2048KiB.get_size_in_bytes(),
                    actual: rom_size.get_size_in_bytes(),
                });
            }
        };

        if ram_size > RamSize::Ram32KiB {
            return Err(MbcCreationError::WrongRamSize {
                max: RamSize::Ram32KiB.get_size_in_bytes(),
                actual: ram_size.get_size_in_bytes(),
            });
        }

        let rom_banks = vec![RomBank::new(); rom_size.get_number_of_banks()];
        let ram_banks = vec![RamBank::new(); ram_size.get_number_of_banks()];
        Ok(Self {
            rom_banks,
            ram_banks,
            size,
            bank_number: 0x00,
            ram_enable: false,
            is_simple_mode: true,
        })
    }
    const fn get_lower_bank(&self) -> u8 {
        self.bank_number & 0b0001_1111
    }

    const fn get_upper_bank(&self) -> u8 {
        self.bank_number & 0b0110_0000
    }

    const fn get_rom_size(&self) -> usize {
        self.rom_banks.len() * RomBank::get_size_from_type()
    }

    const fn get_ram_size(&self) -> usize {
        self.ram_banks.len() * RamBank::get_size_from_type()
    }
}

impl BusReader for Mbc1 {
    fn read_byte(&self, address: u16) -> Result<u8, ReadByteError> {
        match address {
            0x0000..=0x3FFF => {
                let bank_index = if !self.is_simple_mode && self.size.is_large() {
                    self.get_upper_bank()
                } else {
                    0
                };
                let bank = self
                    .rom_banks
                    .get(bank_index as usize)
                    .expect("can only be OOBs due to programmer error.");

                bank.read_byte::<FIRST_ROM_BANK_BASE>(address)
            }
            0x4000..=0x7FFF => {
                // If the lower five bits are 0 then it behaves as it it were 1.
                let lower_bank = self.get_lower_bank();
                let upper_bank = self.get_upper_bank();
                let adjusted_lower_bank = if lower_bank == 0 { 1 } else { lower_bank };

                let bank_index = match self.size {
                    SizeMode::One => adjusted_lower_bank & 0b0000_0001,
                    SizeMode::Two => adjusted_lower_bank & 0b0000_0011,
                    SizeMode::Three => adjusted_lower_bank & 0b0000_0111,
                    SizeMode::Four => adjusted_lower_bank & 0b0000_1111,
                    SizeMode::Five => adjusted_lower_bank & 0b0001_1111,
                    SizeMode::More => adjusted_lower_bank & 0b0001_1111 | upper_bank,
                };

                let bank = self
                    .rom_banks
                    .get(bank_index as usize)
                    .expect("can only be OOBs due to programmer error.");

                bank.read_byte::<SECOND_ROM_BANK_BASE>(address)
            }
            0xA000..=0xBFFF => {
                if !self.ram_enable {
                    tracing::warn!("Reading from disabled RAM!");
                    return Ok(0xFF);
                }

                let bank_index = if self.is_simple_mode {
                    0
                } else {
                    self.get_upper_bank() >> 5
                };

                let bank = self
                    .ram_banks
                    .get(bank_index as usize)
                    .expect("can only be OOBs due to programmer error.");
                bank.read_byte::<RAM_BANK_BASE>(address)
            }
            _ => Err(ReadByteError::InvalidAddressForPeripheral {
                name: NAME,
                address,
            }),
        }
    }
}

impl BusWriter for Mbc1 {
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), WriteByteError> {
        match address {
            // RAM enable flag.
            0x0000..=0x1FFF => {
                self.ram_enable = (value & 0x0F) == 0x0A;
                Ok(())
            }
            // ROM bank number.
            0x2000..=0x3FFF => {
                self.bank_number = self.get_upper_bank() | (value & 0b0001_1111);
                Ok(())
            }
            // RAM bank number.
            0x4000..=0x5FFF => {
                self.bank_number = ((value & 0b0000_0011) << 5) | self.get_lower_bank();
                Ok(())
            }
            // Banking mode.
            0x6000..=0x7FFF => {
                self.is_simple_mode = (value & 0b0000_0001) == 0;
                Ok(())
            }
            // RAM
            0xA000..=0xBFFF => {
                if !self.ram_enable {
                    tracing::warn!("Attempting to write {value:#04X} to disabled RAM.");
                    return Ok(());
                } else if self.ram_banks.is_empty() {
                    tracing::warn!("Attempting to write {value:#04X} to MBC1 with no RAM.");
                    return Ok(());
                }
                let bank_index = if self.is_simple_mode {
                    0
                } else {
                    self.get_upper_bank() >> 5
                };

                let bank = self
                    .ram_banks
                    .get_mut(bank_index as usize)
                    .expect("can only be OOBs due to programmer error.");
                bank.write_byte::<RAM_BANK_BASE>(address, value)
            }
            _ => Err(WriteByteError::InvalidAddressForPeripheral {
                name: NAME,
                address,
            }),
        }
    }
}

impl MemoryBankController for Mbc1 {
    fn load_rom(&mut self, bytes: &[u8]) -> Result<(), RomLoadError> {
        if self.get_rom_size() != bytes.len() {
            return Err(RomLoadError::WrongSize {
                expected: self.get_rom_size(),
                actual: bytes.len(),
            });
        }

        let bank_iter = bytes.chunks_exact(RomBank::get_size_from_type());
        for (current_data, current_bank) in bank_iter.zip(self.rom_banks.iter_mut()) {
            current_bank
                .load_bytes(current_data)
                .expect("data is guaranteed to be the same size as the bank.");
        }

        Ok(())
    }

    fn load_ram(&mut self, bytes: &[u8]) -> Result<(), RamLoadError> {
        if self.get_ram_size() != bytes.len() {
            return Err(RamLoadError::WrongSize {
                expected: self.get_ram_size(),
                actual: bytes.len(),
            });
        }

        let bank_iter = bytes.chunks_exact(RamBank::get_size_from_type());
        for (current_data, current_bank) in bank_iter.zip(self.ram_banks.iter_mut()) {
            current_bank
                .load_bytes(current_data)
                .expect("data is guaranteed to be the same size as the bank.");
        }

        Ok(())
    }

    fn dump_ram(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.get_ram_size());
        for bank in &self.ram_banks {
            data.extend_from_slice(bank.as_slice());
        }
        data
    }

    fn update(&mut self, num_m_cycles: usize) {
        let _ = num_m_cycles;
    }
}
