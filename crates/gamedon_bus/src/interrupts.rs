use crate::{BusReader, BusWriter, ReadByteError, WriteByteError};
use gamedon_bits::{BitShift8, HwReg8};

const INTERRUPT_VBLANK: BitShift8 = BitShift8::Bit0;
const INTERRUPT_LCD: BitShift8 = BitShift8::Bit1;
const INTERRUPT_TIMER: BitShift8 = BitShift8::Bit2;
const INTERRUPT_SERIAL: BitShift8 = BitShift8::Bit3;
const INTERRUPT_JOYPAD: BitShift8 = BitShift8::Bit4;

#[derive(Debug, Clone, Copy)]
pub enum Interrupt {
    VBlank,
    Lcd,
    Timer,
    Serial,
    Joypad,
}

impl Interrupt {
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

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Interrupts {
    /// 0xFF0F: Bit flags to signal interrupt requests.
    flag: HwReg8,
    /// 0xFFFF: Bit flags to toggle which interrupts are enabled.
    enable: HwReg8,
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

impl Interrupts {
    pub(crate) const fn is_pending(self) -> bool {
        (self.enable.0 & self.flag.0) != 0
    }

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

    pub(crate) const fn set_request(&mut self, interrupt: Interrupt) {
        self.flag.set(interrupt.to_shift());
    }

    pub(crate) const fn reset_request(&mut self, interrupt: Interrupt) {
        self.flag.reset(interrupt.to_shift());
    }
}
