use crate::{BusReader, BusWriter, ReadByteError, WriteByteError};

#[derive(Debug, Clone)]
pub(crate) struct RamArea<const N: usize, const BASE: u16> {
    data: [u8; N],
}

impl<const N: usize, const BASE: u16> core::default::Default for RamArea<N, BASE> {
    fn default() -> Self {
        Self { data: [0u8; N] }
    }
}

impl<const N: usize, const BASE: u16> BusReader for RamArea<N, BASE> {
    fn read_byte(&self, address: u16) -> Result<u8, ReadByteError> {
        let translated_address = address.checked_sub(BASE).unwrap() as usize;
        self.data
            .get(translated_address)
            .ok_or(ReadByteError::InvalidAddressForPeripheral {
                name: "RAM",
                address,
            })
            .copied()
    }
}

impl<const N: usize, const BASE: u16> BusWriter for RamArea<N, BASE> {
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), WriteByteError> {
        let translated_address = address.checked_sub(BASE).unwrap() as usize;
        let byte = self.data.get_mut(translated_address).ok_or(
            WriteByteError::InvalidAddressForPeripheral {
                name: "RAM",
                address,
            },
        )?;
        *byte = value;
        Ok(())
    }
}
