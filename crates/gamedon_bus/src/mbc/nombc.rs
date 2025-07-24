use super::{
    MbcCreationError, MemoryBankController, RamLoadError, RamSize, RomLoadError, RomSize,
    bank::{MemoryBank, NamedType},
};
use crate::{BusReader, BusWriter, ReadByteError, WriteByteError};

/// The name of the controller.
const NAME: &str = "nombc";
/// The base address for the ROM bank area.
const ROM_BASE: u16 = 0x0000;
/// The base address for the RAM bank area.
const RAM_BASE: u16 = 0xA000;
/// The size of a ROM bank for a cartridge with no MBC.
const ROM_BANK_SIZE: usize = 0x8000;
/// The size of a RAM bank for a cartridge with no MBC.
const RAM_BANK_SIZE: usize = 0x2000;

type RomBank = MemoryBank<ROM_BANK_SIZE, NoMbcName>;
type RamBank = MemoryBank<RAM_BANK_SIZE, NoMbcName>;

struct NoMbcName;

impl NamedType for NoMbcName {
    fn name() -> &'static str {
        NAME
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NoMbc {
    rom: RomBank,
    ram: Option<RamBank>,
}

impl NoMbc {
    pub(crate) const fn new(
        rom_size: RomSize,
        ram_size: RamSize,
    ) -> Result<Self, MbcCreationError> {
        let RomSize::Rom32KiB = rom_size else {
            return Err(MbcCreationError::WrongRomSize {
                max: RomSize::Rom32KiB.get_size_in_bytes(),
                actual: rom_size.get_size_in_bytes(),
            });
        };

        let ram = match ram_size {
            RamSize::Ram0KiB => None,
            RamSize::Ram8KiB => Some(RamBank::new()),
            _ => {
                return Err(MbcCreationError::WrongRamSize {
                    max: RAM_BANK_SIZE,
                    actual: ram_size.get_size_in_bytes(),
                });
            }
        };

        Ok(Self {
            rom: RomBank::new(),
            ram,
        })
    }
}

impl BusReader for NoMbc {
    fn read_byte(&self, address: u16) -> Result<u8, ReadByteError> {
        match address {
            0x0000..=0x7FFF => self.rom.read_byte::<ROM_BASE>(address),
            0xA000..=0xBFFF => match &self.ram {
                Some(ram_bank) => ram_bank.read_byte::<RAM_BASE>(address),
                None => {
                    tracing::warn!("Reading from non-existent external RAM!");
                    Ok(0xFF)
                }
            },
            _ => Err(ReadByteError::InvalidAddressForPeripheral {
                name: NAME,
                address,
            }),
        }
    }
}

impl BusWriter for NoMbc {
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), WriteByteError> {
        match address {
            0x0000..=0x7FFF => {
                tracing::warn!("Writing to ROM for a cartridge with no MBC does nothing.");
                Ok(())
            }
            0xA000..=0xBFFF => match self.ram {
                Some(ref mut ram) => ram.write_byte::<RAM_BASE>(address, value),
                None => {
                    tracing::warn!("Attempting to write {value:#04X} to non-existent external RAM");
                    Ok(())
                }
            },
            _ => Err(WriteByteError::InvalidAddressForPeripheral {
                name: NAME,
                address,
            }),
        }
    }
}

impl MemoryBankController for NoMbc {
    fn load_rom(&mut self, bytes: &[u8]) -> Result<(), RomLoadError> {
        let mut chunks = bytes.chunks_exact(self.rom.get_size());
        let Some(data) = chunks.next() else {
            return Err(RomLoadError::WrongSize {
                expected: self.rom.get_size(),
                actual: bytes.len(),
            });
        };
        self.rom
            .load_bytes(data)
            .expect("chunk is guaranteed to be same size as the ROM bank.");
        Ok(())
    }

    fn load_ram(&mut self, bytes: &[u8]) -> Result<(), RamLoadError> {
        let ram = match (&mut self.ram, bytes.is_empty()) {
            (None, true) => return Ok(()),
            (None, false) => {
                return Err(RamLoadError::WrongSize {
                    expected: 0,
                    actual: bytes.len(),
                });
            }
            (Some(ram), _) => ram,
        };

        let mut chunks = bytes.chunks_exact(ram.get_size());
        let Some(data) = chunks.next() else {
            return Err(RamLoadError::WrongSize {
                expected: self.rom.get_size(),
                actual: bytes.len(),
            });
        };
        ram.load_bytes(data)
            .expect("chunk is guaranteed to be the same size as the RAM bank.");
        Ok(())
    }

    fn dump_ram(&self) -> Vec<u8> {
        if let Some(bank) = &self.ram {
            bank.as_slice().to_owned()
        } else {
            vec![]
        }
    }

    fn update(&mut self, num_m_cycles: usize) {
        let _ = num_m_cycles;
    }
}
