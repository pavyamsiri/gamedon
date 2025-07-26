use crate::{
    BusReader, BusWriter, Interrupt, InterruptSource, Peripheral, ReadByteError, WriteByteError,
};
use core::default;
use gamedon_bits::HwReg8;

/// The serial port state.
#[derive(Debug, Clone)]
enum State {
    /// The normal state.
    Normal,
    /// Byte transfer is being requested.
    RequestTransfer,
}

/// The serial port.
#[derive(Debug, Clone)]
pub(crate) struct Serial {
    /// $FF01: Serial data port.
    data: HwReg8,
    /// $FF02: Serial data control.
    control: HwReg8,

    /// Transfer state.
    state: State,
    // Debug
    /// All of the data written to the serial port.
    output: Vec<u8>,
    /// Whether the serial port's current output has been shown before.
    has_shown: bool,
}

impl default::Default for Serial {
    fn default() -> Self {
        Self {
            has_shown: true,
            data: HwReg8::default(),
            control: HwReg8::default(),
            output: Vec::new(),
            state: State::Normal,
        }
    }
}

impl Serial {
    /// Whether the serial port's current output has been shown yet.
    pub(crate) const fn has_shown(&self) -> bool {
        self.has_shown
    }

    /// Return the serial port's current output as a slice of bytes.
    pub(crate) fn show(&mut self) -> &[u8] {
        self.has_shown = true;
        &self.output
    }
}

impl Peripheral for Serial {
    fn tick(&mut self) {
        match self.state {
            State::Normal => {}
            State::RequestTransfer => {
                self.has_shown = false;
                self.output.push(self.data.0);
                self.control.0 = 0x0;
                self.state = State::Normal;
            }
        }
    }
}

impl InterruptSource for Serial {
    fn get_interrupt_request(&mut self) -> Option<Interrupt> {
        None
    }
}

impl BusReader for Serial {
    fn read_byte(&self, address: u16) -> Result<u8, ReadByteError> {
        match address {
            0xFF01 => Ok(self.data.0),
            0xFF02 => Ok(self.control.0),
            _ => Err(ReadByteError::InvalidAddressForPeripheral {
                name: "Serial",
                address,
            }),
        }
    }
}

impl BusWriter for Serial {
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), WriteByteError> {
        match address {
            0xFF01 => {
                self.data.0 = value;
            }
            0xFF02 => {
                if value == 0x81 {
                    self.state = State::RequestTransfer;
                }
                self.control.0 = value;
            }
            _ => {
                return Err(WriteByteError::InvalidAddressForPeripheral {
                    name: "Serial",
                    address,
                });
            }
        }

        Ok(())
    }
}
