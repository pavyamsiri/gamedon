use crate::{BusReader, BusWriter, ReadByteError, WriteByteError};

/// Addresses mapping to the MBC.
#[macro_export]
macro_rules! dma_addresses {
    () => {
        0xFF46
    };
}

/// The number of bytes transferred during a DMA transfer.
const NUM_TRANSFER_BYTES: u8 = 160;

/// The bytes to transfer over the course of one M-cycle during a DMA transfer.
pub(crate) struct TransferRequest {
    /// The high byte XX of the address of the source $XXYY.
    pub(crate) src: u8,
    /// The low byte YY of the source address $XXYY and the destination address $FEYY.
    pub(crate) dst: u8,
}

/// The DMA state.
#[derive(Debug, Clone, Default)]
enum State {
    /// The DMA is inactive.
    #[default]
    Inactive,
    /// A transfer is scheduled to occur in two M-cycles.
    /// The byte `src` is the high byte of the source address.
    ScheduledCycle1 { src: u8 },
    /// A transfer is scheduled to occur in one M-cycle.
    /// The byte `src` is the high byte of the source address.
    ScheduledCycle2 { src: u8 },
    /// A transfer is occurring.
    /// The byte `src` is the high byte of the source address.
    /// The number of bytes `remaining` indicate the bytes yet to transfer.
    Active { src: u8, remaining: u8 },
}

/// The DMA transfer protocol.
#[derive(Debug, Clone, Default)]
pub(crate) struct Dma {
    /// $FF46: The high byte of the source address.
    reg: u8,
    /// The state.
    state: State,
}

impl BusReader for Dma {
    #[inline]
    fn read_byte(&self, address: u16) -> Result<u8, ReadByteError> {
        match address {
            0xFF46 => Ok(self.reg),
            _ => Err(ReadByteError::InvalidAddressForPeripheral {
                name: "DMA",
                address,
            }),
        }
    }
}

impl BusWriter for Dma {
    #[inline]
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), WriteByteError> {
        match address {
            0xFF46 => {
                self.prepare_dma_transfer(value);
                Ok(())
            }
            _ => Err(WriteByteError::InvalidAddressForPeripheral {
                name: "DMA",
                address,
            }),
        }
    }
}

impl Dma {
    fn prepare_dma_transfer(&mut self, value: u8) {
        match self.state {
            State::Active { .. } => {}
            _ => {
                let src = if (0xE0..=0xFD).contains(&value) {
                    value - 0x20
                } else {
                    value
                };
                self.state = State::ScheduledCycle1 { src };
            }
        }
    }

    /// Update the DMA transfer returning the high byte of the source address and the low byte of both
    /// the source and destination address.
    pub(crate) fn tick(&mut self) -> Option<TransferRequest> {
        match self.state {
            State::Inactive => {}
            State::ScheduledCycle1 { src } => self.state = State::ScheduledCycle2 { src },
            State::ScheduledCycle2 { src } => {
                self.state = State::Active {
                    src,
                    remaining: NUM_TRANSFER_BYTES,
                }
            }
            State::Active { src, remaining } => {
                assert!(
                    remaining <= NUM_TRANSFER_BYTES,
                    "DMA can only transfer {NUM_TRANSFER_BYTES} bytes.",
                );
                if remaining > 0 {
                    self.state = State::Active {
                        src,
                        remaining: remaining - 1,
                    };
                } else {
                    self.state = State::Inactive;
                }
                let offset = NUM_TRANSFER_BYTES - remaining;
                return Some(TransferRequest { src, dst: offset });
            }
        }
        None
    }

    /// Return whether the DMA is in the process of transferring.
    #[inline]
    pub(crate) const fn is_active(&self) -> bool {
        !matches!(self.state, State::Active { .. })
    }
}
