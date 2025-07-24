mod cartridge;
mod interrupts;
mod mbc;
mod ram;
mod serial;
mod timer;

use cartridge::{CartridgeHeader, HeaderLoadError};
use interrupts::Interrupts;
use mbc::{
    MbcCreationError, MemoryBankController, NoMbc, RamSize, RomLoadError, RomSize, TaggedMbc,
};
use ram::RamArea;
use serial::Serial;
use thiserror::Error;

pub use interrupts::Interrupt;
use timer::Timer;

#[derive(Debug, Error)]
pub enum ReadByteError {
    #[error("The address {address:#06X} is invalid for the peripheral named {name}")]
    InvalidAddressForPeripheral { name: &'static str, address: u16 },
}

#[derive(Debug, Error)]
pub enum WriteByteError {
    #[error("The address {address:#06X} is invalid for the peripheral named {name}")]
    InvalidAddressForPeripheral { name: &'static str, address: u16 },
}

#[derive(Debug, Error)]
pub enum CartridgeLoadError {
    #[error(transparent)]
    InvalidHeader(#[from] HeaderLoadError),
    #[error(transparent)]
    InvalidMbcSpec(#[from] MbcCreationError),
    #[error(transparent)]
    FailedRomLoad(#[from] RomLoadError),
}

pub trait BusReader {
    fn read_byte(&self, address: u16) -> Result<u8, ReadByteError>;
}

pub trait BusWriter {
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), WriteByteError>;
}

trait Peripheral {
    fn tick(&mut self);
    fn batch_tick(&mut self, num_m_cycles: usize) {
        for _ in 0..num_m_cycles {
            self.tick();
        }
    }
}

pub trait InterruptSource {
    fn get_interrupt_request(&mut self) -> Option<Interrupt>;
}

#[derive(Clone)]
pub struct MemoryBus {
    mbc: TaggedMbc,
    interrupts: Interrupts,
    timer: Timer,
    working_ram: RamArea<0x2000, 0xC000>,
    serial: Serial,
    vram: RamArea<0x2000, 0x8000>,
    hram: RamArea<0x007F, 0xFF80>,
}

impl core::default::Default for MemoryBus {
    fn default() -> Self {
        let mbc = NoMbc::new(RomSize::Rom32KiB, RamSize::Ram0KiB)
            .expect("sizes are guaranteed to be supported");
        Self {
            mbc: TaggedMbc::RomOnly(mbc),
            interrupts: Interrupts::default(),
            timer: Timer::default(),
            working_ram: RamArea::default(),
            serial: Serial::default(),
            vram: RamArea::default(),
            hram: RamArea::default(),
        }
    }
}

impl core::fmt::Debug for MemoryBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryBus")
            .field("interrupts", &self.interrupts)
            .field("timer", &self.timer)
            .finish_non_exhaustive()
    }
}

impl BusReader for MemoryBus {
    #[inline]
    fn read_byte(&self, address: u16) -> Result<u8, ReadByteError> {
        let value = match address {
            // Interrupts
            0xFF0F | 0xFFFF => self.interrupts.read_byte(address)?,
            // Timer
            0xFF04..=0xFF07 => self.timer.read_byte(address)?,
            // Working RAM
            0xC000..=0xDFFF => self.working_ram.read_byte(address)?,
            // Echo RAM
            0xE000..=0xFDFF => self.read_byte(address - 0x2000)?,
            // Serial
            0xFF01 | 0xFF02 => self.serial.read_byte(address)?,
            // HACK(pavyamsiri): Hard return 0x90 for now so I can debug with doctor
            0xFF44 => 0x90,
            // VRAM
            0x8000..=0x9FFF => self.vram.read_byte(address)?,
            // MBC
            0x0000..=0xBFFF => self.mbc.read_byte(address)?,
            // High RAM
            0xFF80..=0xFFFE => self.hram.read_byte(address)?,
            _ => {
                tracing::trace!(address = address, "Missing read implementation for address");
                0xFF
            }
        };

        tracing::trace!(
            address = address,
            value = value,
            "Read {value:#04X} at {address:#06X}"
        );
        Ok(value)
    }
}

impl BusWriter for MemoryBus {
    #[inline]
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), WriteByteError> {
        tracing::trace!(
            address = address,
            value = value,
            "Writing {value:#04X} to {address:#06X}"
        );
        match address {
            // Interrupts
            0xFF0F | 0xFFFF => self.interrupts.write_byte(address, value),
            // Timer
            0xFF04..=0xFF07 => self.timer.write_byte(address, value),
            // Working RAM
            0xC000..=0xDFFF => self.working_ram.write_byte(address, value),
            // Echo RAM
            0xE000..=0xFDFF => self.write_byte(address - 0x2000, value),
            // Serial
            0xFF01 | 0xFF02 => self.serial.write_byte(address, value),
            // VRAM
            0x8000..=0x9FFF => self.vram.write_byte(address, value),
            // MBC
            0x0000..=0xBFFF => self.mbc.write_byte(address, value),
            // High RAM
            0xFF80..=0xFFFE => self.hram.write_byte(address, value),
            _ => {
                tracing::trace!(address = address, "Missing read implementation for address");
                Ok(())
            }
        }
    }
}

impl MemoryBus {
    #[inline]
    pub fn load_rom(rom: &[u8]) -> Result<Self, CartridgeLoadError> {
        let header = CartridgeHeader::parse(rom)?;
        tracing::debug!("Parsed header: {header:#?}");
        let mut mbc = header.get_mbc()?;

        mbc.load_rom(rom)?;

        Ok(Self {
            mbc,
            interrupts: Interrupts::default(),
            timer: Timer::default(),
            working_ram: RamArea::default(),
            serial: Serial::default(),
            vram: RamArea::default(),
            hram: RamArea::default(),
        })
    }

    #[inline]
    pub fn has_new_serial_output(&mut self) -> Option<&[u8]> {
        if self.serial.has_shown() {
            None
        } else {
            Some(self.serial.show())
        }
    }

    #[inline]
    pub fn get_serial_output(&self) -> &[u8] {
        self.serial.output()
    }

    #[inline]
    pub fn write_word(&mut self, address: u16, value: u16) -> Result<(), WriteByteError> {
        let lo = (value & 0xFF) as u8;
        let hi = (value >> 8) as u8;
        match self.write_byte(address, lo) {
            Ok(()) => {}
            Err(err) => return Err(err),
        }
        match self.write_byte(address + 1, hi) {
            Ok(()) => {}
            Err(err) => return Err(err),
        }
        Ok(())
    }

    #[inline]
    pub const fn is_pending_interrupts(&self) -> bool {
        self.interrupts.is_pending()
    }

    #[inline]
    pub const fn get_pending_interrupt(&self) -> Option<Interrupt> {
        self.interrupts.get_pending()
    }

    #[inline]
    pub const fn reset_interrupt_request(&mut self, interrupt: Interrupt) {
        self.interrupts.reset_request(interrupt);
    }

    #[inline]
    pub const fn iter_from(&self, address: u16) -> MemoryBusIterator<'_> {
        MemoryBusIterator {
            bus: self,
            address: Some(address),
        }
    }

    #[inline]
    pub fn iter(&self) -> MemoryBusIterator<'_> {
        self.into_iter()
    }
}

// Peripherals
impl MemoryBus {
    pub fn timer_tick(&mut self, num_m_cycles: usize) {
        self.timer.batch_tick(num_m_cycles);
    }

    pub fn serial_tick(&mut self, num_m_cycles: usize) {
        self.serial.batch_tick(num_m_cycles);
    }

    pub fn update_interrupt_requests(&mut self) {
        if let Some(interrupt) = self.timer.get_interrupt_request() {
            self.interrupts.set_request(interrupt);
        }

        if let Some(interrupt) = self.serial.get_interrupt_request() {
            self.interrupts.set_request(interrupt);
        }
    }
}

impl<'a> core::iter::IntoIterator for &'a MemoryBus {
    type Item = u8;

    type IntoIter = MemoryBusIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            bus: self,
            address: Some(0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryBusIterator<'a> {
    bus: &'a MemoryBus,
    address: Option<u16>,
}

impl core::iter::Iterator for MemoryBusIterator<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.address {
            Some(address) => {
                self.address = address.checked_add(1);
                Some(self.bus.read_byte(address).unwrap_or(0x00))
            }
            None => None,
        }
    }
}
