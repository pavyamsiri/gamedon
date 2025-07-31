use crate::{
    BusReader, BusWriter, Interrupt, InterruptSource, Peripheral, ReadByteError, WriteByteError,
};
use core::default;
use gamedon_bits::{BitShift8, BitShift16, HwReg8, HwReg16};

/// Addresses mapping to the timer registers.
#[macro_export]
macro_rules! timer_addresses {
    () => {
        0xFF04..=0xFF07
    };
}

/// The timer state.
#[derive(Debug, Clone, Default)]
enum State {
    #[default]
    Normal,
    ReloadScheduled,
    Reloading,
}

/// The timer.
#[derive(Debug, Clone)]
pub(crate) struct Timer {
    /// $FF04: The system counter where the upper 8 bits are visible as DIV or the divider register
    /// addressed as $FF04.
    sys_counter: HwReg16,
    /// 0xFF05: TIMA - timer counter.
    tima: HwReg8,
    /// 0xFF06: TMA - timer modulo.
    tma: HwReg8,
    /// 0xFF07: TAC - timer control.
    tac: HwReg8,

    /// Pending interrupt.
    pending_interrupt: bool,
    /// Timer state.
    state: State,
}

impl default::Default for Timer {
    fn default() -> Self {
        Self {
            sys_counter: HwReg16(0xABCC),
            tima: HwReg8(0x00),
            tma: HwReg8(0x00),
            tac: HwReg8(0x00),
            pending_interrupt: false,
            state: State::Normal,
        }
    }
}

impl BusReader for Timer {
    #[inline]
    fn read_byte(&self, address: u16) -> Result<u8, ReadByteError> {
        match address {
            0xFF04 => Ok(self.sys_counter.upper()),
            0xFF05 => Ok(self.tima.0),
            0xFF06 => Ok(self.tma.0),
            0xFF07 => Ok(self.tac.0 | 0xF8),
            _ => Err(ReadByteError::InvalidAddressForPeripheral {
                name: "Timer",
                address,
            }),
        }
    }
}

impl BusWriter for Timer {
    #[inline]
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), WriteByteError> {
        match address {
            0xFF04 => {
                self.reset_counter();
            }
            0xFF05 => {
                self.write_tima(value);
            }
            0xFF06 => {
                self.write_tma(value);
            }
            0xFF07 => {
                self.write_tac(value);
            }
            _ => {
                return Err(WriteByteError::InvalidAddressForPeripheral {
                    name: "Timer",
                    address,
                });
            }
        }
        Ok(())
    }
}

impl Peripheral for Timer {
    fn tick(&mut self) {
        let rate_bit = Self::rate_bit_select(self.tac.0);

        // TIMA reload lasts one cycle, so it's ok to reset this
        // at the beginning of each tick.
        if matches!(self.state, State::Reloading) {
            self.state = State::Normal;
        }

        // If a reload was scheduled and not canceled, set the IRQ flag and
        // reload TIMA with TMA. This also causes the timer to enter a cycle
        // in which writes to TIMA are ignored.
        if matches!(self.state, State::ReloadScheduled) {
            self.state = State::Reloading;
            self.tima = self.tma;
            self.pending_interrupt = true;
        }

        let new_value = self.sys_counter.0.wrapping_add(4);
        if self.is_running() {
            let old = self.sys_counter;
            self.sys_counter.0 = new_value;
            let new = self.sys_counter;

            // TIMA is incremented when the rate bit changes from 1 to 0.
            if old.bit(rate_bit) && !new.bit(rate_bit) {
                self.inc_tima();
            }
        } else {
            self.sys_counter.0 = new_value;
        }
    }
}

impl InterruptSource for Timer {
    fn get_interrupt_request(&mut self) -> Option<Interrupt> {
        if self.pending_interrupt {
            self.pending_interrupt = false;
            Some(Interrupt::Timer)
        } else {
            None
        }
    }
}

impl Timer {
    /// Reset the system counter.
    fn reset_counter(&mut self) {
        // HW_BUG: Resetting DIV while the multiplexer bit corresponding to the current tick rate is set causes
        // TIMA to increment if it is running.
        if self.is_running() && self.rate_bit() {
            self.inc_tima();
        }

        self.sys_counter.0 = 0;
    }

    /// Write a `new_value` to `TAC`.
    fn write_tac(&mut self, new_value: u8) {
        let new_value = HwReg8(new_value);
        // HW_BUG: When changing TAC register value, if the old multiplexer bit is 0 but the new one is 1 and the new
        // enable bit is set, it will increase TIMA.
        let first_bug = new_value.bit(BitShift8::Bit2)
            && !self.rate_bit()
            && self.sys_counter.bit(Self::rate_bit_select(new_value.0));

        // HW_BUG: Whenever half the clocks of the count are reached, TIMA will increase when disabling the timer.
        let second_bug = self.is_running() && !new_value.bit(BitShift8::Bit2) && self.rate_bit();

        if first_bug || second_bug {
            self.inc_tima();
        }

        self.tac = new_value;
    }

    /// Write a `new_value` to `TIMA`.
    const fn write_tima(&mut self, new_value: u8) {
        // When TIMA is being reloaded, writes are ignored.
        if matches!(self.state, State::Reloading) {
            return;
        }

        self.tima.0 = new_value;

        // If a write to TIMA happens when a reload is scheduled to happen, the reload is cancelled
        // with TIMA set to the written value and the interrupt request is cancelled.
        if matches!(self.state, State::ReloadScheduled) {
            self.state = State::Normal;
        }
    }

    /// Write a `new_value` to `TMA`.
    const fn write_tma(&mut self, new_value: u8) {
        self.tma.0 = new_value;

        // If a write to TMA happens while TIMA is being reloaded, the new value is loaded instead.
        if matches!(self.state, State::Reloading) {
            self.tima = self.tma;
        }
    }

    /// Increment `TIMA`.
    const fn inc_tima(&mut self) {
        let new_value = self.tima.0.wrapping_add(1);
        self.tima.0 = new_value;

        // When TIMA overflows, an interrupt request is fired and TIMA is set to the value of TMA.
        // This happens over a full M cycle and so we need to delay the reload as before this happens
        // the value is still 0.
        if new_value == 0 {
            self.state = State::ReloadScheduled;
        }
    }

    /// Return whether the timer is running.
    const fn is_running(&self) -> bool {
        // Check bit 2 of TAC
        self.tac.bit(BitShift8::Bit2)
    }

    /// Return which bit is selected in the system counter due to the currently selected rate.
    fn rate_bit_select(tac: u8) -> BitShift16 {
        match tac & 0x3 {
            0b00 => BitShift16::Bit09,
            0b01 => BitShift16::Bit03,
            0b10 => BitShift16::Bit05,
            0b11 => BitShift16::Bit07,
            _ => unreachable!("impossible due to bit mask"),
        }
    }

    /// Check the bit in sys counter currently selected by TAC
    /// and the subsequently the multiplexer.
    /// See <https://gbdev.io/pandocs/Timer_Obscure_Behaviour.html>.
    fn rate_bit(&self) -> bool {
        let shift = Self::rate_bit_select(self.tac.0);
        self.sys_counter.bit(shift)
    }
}
