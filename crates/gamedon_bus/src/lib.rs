use bank::MemoryBank;
use cartridge::{CartridgeHeader, HeaderLoadError, RamSize, RomSize};
use core::{default, fmt, iter};
use dma::Dma;
use interrupts::Interrupts;
use lcd::Lcd;
use mbc::{MbcCreationError, MemoryBankController, NoMbc, RomLoadError, TaggedMbc};
use serial::Serial;
use thiserror::Error;
use timer::Timer;
use vram::VideoRam;

pub(crate) use bank::NameTag;
pub use interrupts::Interrupt;

/// Memory banks or RAM areas.
mod bank;
/// The game cartridge.
mod cartridge;
/// The DMA transfer protocol.
mod dma;
/// The interrupt registers.
mod interrupts;
/// The LCD registers.
mod lcd;
/// The memory bank controller.
mod mbc;
/// The serial port.
mod serial;
/// The timer.
mod timer;
/// Video RAM
mod vram;

/// Addresses mapping to high RAM.
macro_rules! hram_addresses {
    () => {
        0xFF80..=0xFFFE
    };
}

/// Addresses mapping to working RAM.
macro_rules! wram_addresses {
    () => {
        0xC000..=0xDFFF
    };
}

/// Addresses mapping to echo RAM.
macro_rules! echo_ram_addresses {
    () => {
        0xE000..=0xFDFF
    };
}

/// Errors that can occur when reading a byte from the bus.
#[derive(Debug, Error)]
pub enum ReadByteError {
    /// The given `address` is not a valid address for the peripheral that is being read from.
    #[error("The address {address:#06X} is invalid for the peripheral named {name}")]
    InvalidAddressForPeripheral { name: &'static str, address: u16 },
}

/// Errors that can occur when writing a byte to the bus.
#[derive(Debug, Error)]
pub enum WriteByteError {
    /// The given `address` is not a valid address for the peripheral that is being written to.
    #[error("The address {address:#06X} is invalid for the peripheral named {name}")]
    InvalidAddressForPeripheral { name: &'static str, address: u16 },
}

/// Errors that can occur when loading a cartridge.
#[derive(Debug, Error)]
pub enum CartridgeLoadError {
    /// The cartridge header is invalid.
    #[error(transparent)]
    InvalidHeader(#[from] HeaderLoadError),
    /// Could not create the MBC due to a possibly incompatible header.
    #[error(transparent)]
    InvalidMbcSpec(#[from] MbcCreationError),
    /// Failed to load a ROM likely due to it differing to its expected size.
    #[error(transparent)]
    FailedRomLoad(#[from] RomLoadError),
}

/// Interface to read bytes off the bus.
pub trait BusReader {
    /// Read the byte at `address`.
    fn read_byte(&self, address: u16) -> Result<u8, ReadByteError>;
}

/// Interface to write bytes to the bus.
pub trait BusWriter {
    /// Write the `value` to the `address`.
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), WriteByteError>;
}

/// Devices attached to the memory bus that are synced to the system clock.
trait Peripheral {
    /// Execute one M-cycle.
    fn tick(&mut self);
}

/// Represents sources of interrupt requests.
pub trait InterruptSource {
    /// Return a pending interrupt request.
    fn get_interrupt_request(&mut self) -> Option<Interrupt>;
}

/// Working RAM.
#[derive(Debug, Clone)]
struct WorkingRam;
impl NameTag for WorkingRam {
    fn name() -> &'static str {
        "Working RAM"
    }
}

/// High RAM.
#[derive(Debug, Clone)]
struct HighRam;
impl NameTag for HighRam {
    fn name() -> &'static str {
        "High RAM"
    }
}

/// The memory bus.
#[derive(Clone)]
pub struct MemoryBus {
    /// The memory bank controller.
    mbc: TaggedMbc,
    /// The interrupt registers.
    interrupts: Interrupts,
    /// The timer registers.
    timer: Timer,
    /// The working RAM from $C000 to $DFFF.
    working_ram: MemoryBank<0x2000, WorkingRam>,
    /// The serial port.
    serial: Serial,
    /// The video RAM from $8000 to $9FFF and OAM from $FE00-$FE97.
    vram: VideoRam,
    /// The LCD registers.
    lcd: Lcd,
    /// The high RAM from $FF80 to $FFFE.
    hram: MemoryBank<0x007F, HighRam>,
    /// DMA transfer at $FF46.
    dma: Dma,
}

impl default::Default for MemoryBus {
    fn default() -> Self {
        let mbc = NoMbc::new(RomSize::Rom32KiB, RamSize::Ram0KiB)
            .expect("sizes are guaranteed to be supported");
        Self {
            mbc: TaggedMbc::RomOnly(mbc),
            interrupts: Interrupts::default(),
            timer: Timer::default(),
            working_ram: MemoryBank::default(),
            serial: Serial::default(),
            vram: VideoRam::default(),
            hram: MemoryBank::default(),
            lcd: Lcd::default(),
            dma: Dma::default(),
        }
    }
}

impl fmt::Debug for MemoryBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryBus")
            .field("interrupts", &self.interrupts)
            .field("timer", &self.timer)
            .finish_non_exhaustive()
    }
}

impl BusReader for MemoryBus {
    #[inline]
    fn read_byte(&self, address: u16) -> Result<u8, ReadByteError> {
        // If doing DMA transfer and address is not in high RAM address space.
        if self.dma.is_active() && !matches!(address, hram_addresses!()) {
            Ok(0xFF)
        } else {
            self.read_byte_unchecked(address)
        }
    }
}

impl BusWriter for MemoryBus {
    #[inline]
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), WriteByteError> {
        if !self.dma.is_active() || matches!(address, hram_addresses!()) {
            self.write_byte_unchecked(address, value)
        } else {
            Ok(())
        }
    }
}

impl MemoryBus {
    /// Read the byte at `address`.
    /// This function does not restrict access during DMA transfer.
    #[inline]
    pub fn read_byte_unchecked(&self, address: u16) -> Result<u8, ReadByteError> {
        let value = match address {
            interrupt_addresses!() => self.interrupts.read_byte(address)?,
            timer_addresses!() => self.timer.read_byte(address)?,
            serial_addresses!() => self.serial.read_byte(address)?,
            lcd_addresses!() => self.lcd.read_byte(address)?,
            vram_addresses!() => self.vram.read_byte(address)?,
            mbc_addresses!() => self.mbc.read_byte(address)?,
            hram_addresses!() => self.hram.read_byte::<0xFF80>(address)?,
            wram_addresses!() => self.working_ram.read_byte::<0xC000>(address)?,
            echo_ram_addresses!() => self.read_byte(address - 0x2000)?,
            dma_addresses!() => self.dma.read_byte(address)?,
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

    /// Write the `value` to the `address` in the memory bus.
    /// This function does not restrict access during DMA transfer.
    #[inline]
    pub fn write_byte_unchecked(&mut self, address: u16, value: u8) -> Result<(), WriteByteError> {
        tracing::trace!(
            address = address,
            value = value,
            "Writing {value:#04X} to {address:#06X}"
        );
        match address {
            interrupt_addresses!() => self.interrupts.write_byte(address, value),
            timer_addresses!() => self.timer.write_byte(address, value),
            serial_addresses!() => self.serial.write_byte(address, value),
            vram_addresses!() => self.vram.write_byte(address, value),
            mbc_addresses!() => self.mbc.write_byte(address, value),
            hram_addresses!() => self.hram.write_byte::<0xFF80>(address, value),
            wram_addresses!() => self.working_ram.write_byte::<0xC000>(address, value),
            echo_ram_addresses!() => self.write_byte(address - 0x2000, value),
            _ => {
                tracing::trace!(address = address, "Missing read implementation for address");
                Ok(())
            }
        }
    }

    /// Load the `rom` represented as a slice of bytes.
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
            working_ram: MemoryBank::default(),
            serial: Serial::default(),
            vram: VideoRam::default(),
            hram: MemoryBank::default(),
            lcd: Lcd::default(),
            dma: Dma::default(),
        })
    }

    /// Return the bytes outputted to the serial port if a new byte was written to it since the
    /// last time this function has been called.
    #[inline]
    pub fn has_new_serial_output(&mut self) -> Option<&[u8]> {
        if self.serial.has_shown() {
            None
        } else {
            Some(self.serial.show())
        }
    }

    /// Write the word `value` to `address`.
    /// Specifically the low byte of `value` is written to `address` while the high byte is written
    /// to `address + 1`.
    #[inline]
    pub fn write_word(&mut self, address: u16, value: u16) -> Result<(), WriteByteError> {
        let [hi, lo] = value.to_be_bytes();
        self.write_byte(address, lo)?;
        self.write_byte(address + 1, hi)?;
        Ok(())
    }

    /// Return whether there are pending interrupts.
    #[inline]
    pub const fn is_pending_interrupts(&self) -> bool {
        self.interrupts.is_pending()
    }

    /// Return the pending interrupt if there are any.
    #[inline]
    pub const fn get_pending_interrupt(&self) -> Option<Interrupt> {
        self.interrupts.get_pending()
    }

    /// Reset the interrupt request bit for `interrupt`.
    #[inline]
    pub const fn reset_interrupt_request(&mut self, interrupt: Interrupt) {
        self.interrupts.reset_request(interrupt);
    }

    /// Create an iterator over the bytes in the memory bus starting from `address`.
    #[inline]
    pub const fn iter_from(&self, address: u16) -> MemoryBusIterator<'_> {
        MemoryBusIterator {
            bus: self,
            address: Some(address),
        }
    }

    /// Create an iterator over the bytes in the memory bus starting from $0000.
    #[inline]
    pub fn iter(&self) -> MemoryBusIterator<'_> {
        self.into_iter()
    }
}

// Peripherals
impl MemoryBus {
    /// Update the bus and its peripherals by one M-cycle.
    #[inline]
    pub fn tick(&mut self) {
        // DMA
        if let Some(req) = self.dma.tick() {
            // Starting address of DMA transfer is $XX00 where XX is the byte written to $FF46.
            let offset = req.dst;
            let src = u16::from_be_bytes([req.src, offset]);
            let value = self
                .read_byte_unchecked(src)
                .expect("The src should always be a valid address.");
            self.vram
                .write_to_oam(offset, value)
                .expect("The dst should always be a valid OAM address.");
        }

        self.serial_tick();
        self.timer_tick();
    }

    /// Update the timer by one M-cycle.
    fn timer_tick(&mut self) {
        self.timer.tick();
    }

    /// Update the serial port by one M-cycle.
    fn serial_tick(&mut self) {
        self.serial.tick();
    }

    /// Update the interrupt registers by polling all interrupt sources.
    pub fn update_interrupt_requests(&mut self) {
        if let Some(interrupt) = self.timer.get_interrupt_request() {
            self.interrupts.set_request(interrupt);
        }

        if let Some(interrupt) = self.serial.get_interrupt_request() {
            self.interrupts.set_request(interrupt);
        }
    }
}

impl<'a> iter::IntoIterator for &'a MemoryBus {
    type Item = (u16, u8);
    type IntoIter = MemoryBusIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            bus: self,
            address: Some(0),
        }
    }
}

/// An iterator over the bytes of the memory bus.
#[derive(Debug, Clone)]
pub struct MemoryBusIterator<'a> {
    /// The memory bus.
    bus: &'a MemoryBus,
    /// The current address.
    address: Option<u16>,
}

impl iter::Iterator for MemoryBusIterator<'_> {
    type Item = (u16, u8);

    fn next(&mut self) -> Option<Self::Item> {
        match self.address {
            Some(address) => {
                self.address = address.checked_add(1);
                Some((address, self.bus.read_byte(address).unwrap_or(0x00)))
            }
            None => None,
        }
    }
}
