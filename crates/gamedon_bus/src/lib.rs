mod interrupts;
mod ram;
mod serial;
mod timer;

use interrupts::Interrupts;
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
    rom: [u8; 65536],
    interrupts: Interrupts,
    timer: Timer,
    working_ram: RamArea<0x2000, 0xC000>,
    serial: Serial,
}

impl core::fmt::Debug for MemoryBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryBus")
            .field("interrupts", &self.interrupts)
            .field("timer", &self.timer)
            .finish_non_exhaustive()
    }
}

impl core::default::Default for MemoryBus {
    fn default() -> Self {
        Self {
            rom: [0u8; 65536],
            interrupts: Interrupts::default(),
            timer: Timer::default(),
            working_ram: RamArea::default(),
            serial: Serial::default(),
        }
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
            _ => self.rom[address as usize],
        };

        Ok(value)
    }
}

impl BusWriter for MemoryBus {
    #[inline]
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), WriteByteError> {
        self.write_byte_raw(address, value)
    }
}

impl MemoryBus {
    #[inline]
    pub fn has_new_serial_output(&mut self) -> Option<&[u8]> {
        if self.serial.has_shown() {
            None
        } else {
            Some(self.serial.show())
        }
    }

    #[inline]
    pub fn write_byte_raw(&mut self, address: u16, value: u8) -> Result<(), WriteByteError> {
        match address {
            // Interrupts
            0xFF0F | 0xFFFF => {
                self.interrupts.write_byte(address, value)?;
            }
            // Timer
            0xFF04..=0xFF07 => self.timer.write_byte(address, value)?,
            // Working RAM
            0xC000..=0xDFFF => self.working_ram.write_byte(address, value)?,
            // Echo RAM
            0xE000..=0xFDFF => self.write_byte_raw(address - 0x2000, value)?,
            // Serial
            0xFF01 | 0xFF02 => self.serial.write_byte(address, value)?,
            _ => {
                self.rom[address as usize] = value;
            }
        }
        Ok(())
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
