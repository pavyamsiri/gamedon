use crate::{ReadByteError, WriteByteError};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum MemoryBankError {
    #[error("The byte slice to load has a different size to the memory bank!")]
    WrongSize { expected: usize, actual: usize },
}

pub(crate) trait NamedType {
    fn name() -> &'static str;
}
pub(crate) struct MemoryBank<const N: usize, Device> {
    data: [u8; N],
    _marker: std::marker::PhantomData<Device>,
}

impl<const N: usize, Device: NamedType> core::fmt::Debug for MemoryBank<N, Device> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryBank")
            .field("data", &self.data)
            .field("size", &N)
            .field("name", &Device::name())
            .finish()
    }
}

impl<const N: usize, Device> core::clone::Clone for MemoryBank<N, Device> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<const N: usize, Device> MemoryBank<N, Device> {
    pub(crate) const fn new() -> Self {
        Self {
            data: [0u8; N],
            _marker: std::marker::PhantomData,
        }
    }

    pub(crate) const fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub(crate) const fn get_size(&self) -> usize {
        N
    }

    pub(crate) const fn get_size_from_type() -> usize {
        N
    }

    pub(crate) const fn load_bytes(&mut self, bytes: &[u8]) -> Result<(), MemoryBankError> {
        if self.data.len() == bytes.len() {
            self.data.copy_from_slice(bytes);
            Ok(())
        } else {
            Err(MemoryBankError::WrongSize {
                expected: self.data.len(),
                actual: bytes.len(),
            })
        }
    }
}

impl<const N: usize, Device: NamedType> MemoryBank<N, Device> {
    pub(crate) fn read_byte<const BASE: u16>(&self, address: u16) -> Result<u8, ReadByteError> {
        let Some(translated_address) = address.checked_sub(BASE) else {
            return Err(ReadByteError::InvalidAddressForPeripheral {
                name: Device::name(),
                address,
            });
        };
        self.data
            .get(translated_address as usize)
            .ok_or(ReadByteError::InvalidAddressForPeripheral {
                name: Device::name(),
                address,
            })
            .copied()
    }

    pub(crate) fn write_byte<const BASE: u16>(
        &mut self,
        address: u16,
        value: u8,
    ) -> Result<(), WriteByteError> {
        let Some(translated_address) = address.checked_sub(BASE) else {
            return Err(WriteByteError::InvalidAddressForPeripheral {
                name: Device::name(),
                address,
            });
        };
        let byte = self.data.get_mut(translated_address as usize).ok_or(
            WriteByteError::InvalidAddressForPeripheral {
                name: Device::name(),
                address,
            },
        )?;

        *byte = value;
        Ok(())
    }
}
