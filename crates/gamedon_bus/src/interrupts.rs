use crate::{BusReader, BusWriter, ReadByteError, WriteByteError};
use gamedon_bits::{BitShift8, HwReg8};

/// Addresses mapping to the interrupt registers.
#[macro_export]
macro_rules! interrupt_addresses {
    () => {
        0xFF0F | 0xFFFF
    };
}

/// The bit representing `VBlank` interrupts.
const INTERRUPT_VBLANK: BitShift8 = BitShift8::Bit0;
/// The bit representing LCD interrupts.
const INTERRUPT_LCD: BitShift8 = BitShift8::Bit1;
/// The bit representing timer interrupts.
const INTERRUPT_TIMER: BitShift8 = BitShift8::Bit2;
/// The bit representing serial interrupts.
const INTERRUPT_SERIAL: BitShift8 = BitShift8::Bit3;
/// The bit representing joypad interrupts.
const INTERRUPT_JOYPAD: BitShift8 = BitShift8::Bit4;

/// Represents all types of interrupts.
#[derive(Debug, Clone, Copy)]
pub enum Interrupt {
    /// Triggers when the PPU enters `VBlank`.
    VBlank,
    /// Triggers are related to the `STAT` register at $FF41.
    Lcd,
    /// Triggers when the timer overflows.
    Timer,
    /// Triggers when the serial port finishes data transfer.
    Serial,
    /// Triggers when a button is pressed.
    Joypad,
}

impl Interrupt {
    /// Return the address of the interrupt handler associated with the interrupt.
    #[inline]
    pub const fn to_address(self) -> u16 {
        match self {
            Interrupt::VBlank => 0x0040,
            Interrupt::Lcd => 0x0048,
            Interrupt::Timer => 0x0050,
            Interrupt::Serial => 0x0058,
            Interrupt::Joypad => 0x0060,
        }
    }

    /// Return the associated bit of the interrupt.
    const fn to_shift(self) -> BitShift8 {
        match self {
            Interrupt::VBlank => INTERRUPT_VBLANK,
            Interrupt::Lcd => INTERRUPT_LCD,
            Interrupt::Timer => INTERRUPT_TIMER,
            Interrupt::Serial => INTERRUPT_SERIAL,
            Interrupt::Joypad => INTERRUPT_JOYPAD,
        }
    }
}

/// The interrupt registers.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Interrupts {
    /// $FF0F: Bit flags to signal interrupt requests.
    flag: HwReg8,
    /// $FFFF: Bit flags to toggle which interrupts are enabled.
    enable: HwReg8,
}

impl Interrupts {
    /// Whether there is a pending interrupt.
    pub(crate) const fn is_pending(self) -> bool {
        (self.enable.0 & self.flag.0) != 0
    }

    /// Return the pending interrupt if there are any.
    pub(crate) const fn get_pending(self) -> Option<Interrupt> {
        let pending = HwReg8(self.enable.0 & self.flag.0);
        if pending.bit(INTERRUPT_VBLANK) {
            Some(Interrupt::VBlank)
        } else if pending.bit(INTERRUPT_LCD) {
            Some(Interrupt::Lcd)
        } else if pending.bit(INTERRUPT_TIMER) {
            Some(Interrupt::Timer)
        } else if pending.bit(INTERRUPT_SERIAL) {
            Some(Interrupt::Serial)
        } else if pending.bit(INTERRUPT_JOYPAD) {
            Some(Interrupt::Joypad)
        } else {
            None
        }
    }

    /// Set the interrupt flag bit for the given interrupt.
    pub(crate) const fn set_request(&mut self, interrupt: Interrupt) {
        self.flag.set(interrupt.to_shift());
    }

    /// Reset the interrupt flag bit for the given interrupt.
    pub(crate) const fn reset_request(&mut self, interrupt: Interrupt) {
        self.flag.reset(interrupt.to_shift());
    }
}
impl BusReader for Interrupts {
    fn read_byte(&self, address: u16) -> Result<u8, ReadByteError> {
        match address {
            0xFF0F => Ok(self.flag.0),
            0xFFFF => Ok(self.enable.0),
            _ => Err(ReadByteError::InvalidAddressForPeripheral {
                name: "Interrupt Registers",
                address,
            }),
        }
    }
}

impl BusWriter for Interrupts {
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), WriteByteError> {
        match address {
            0xFF0F => {
                self.flag = HwReg8(value);
            }
            0xFFFF => {
                self.enable = HwReg8(value);
            }
            _ => {
                return Err(WriteByteError::InvalidAddressForPeripheral {
                    name: "Interrupt Registers",
                    address,
                });
            }
        }
        Ok(())
    }
}
