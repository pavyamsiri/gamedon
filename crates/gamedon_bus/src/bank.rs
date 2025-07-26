use crate::{ReadByteError, WriteByteError};
use core::{clone, default, fmt};
use std::marker::PhantomData;
use thiserror::Error;

/// Interface to allow types to be tagged with a name.
pub(crate) trait NameTag {
    /// The name of the tag.
    fn name() -> &'static str;
}

/// Errors that can occur when loading bytes into a memory bank.
#[derive(Error, Debug)]
pub(crate) enum MemoryBankLoadError {
    /// The size of the memory bank differs with the size of the given byte slice.
    #[error("The byte slice to load has a different size to the memory bank!")]
    WrongSize { expected: usize, actual: usize },
}

/// A bank of addressable memory.
pub(crate) struct MemoryBank<const N: usize, Tag> {
    /// The underlying data.
    data: [u8; N],
    /// Store a type tag.
    _marker: PhantomData<Tag>,
}

impl<const N: usize, Tag> MemoryBank<N, Tag> {
    /// Create a new bank with memory zeroed out.
    pub(crate) const fn new() -> Self {
        Self {
            data: [0u8; N],
            _marker: PhantomData,
        }
    }

    /// Return the data in the bank.
    #[allow(dead_code)]
    pub(crate) const fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Return the size of the memory bank (as a method).
    pub(crate) const fn get_size(&self) -> usize {
        let _ = self;
        N
    }

    /// Return the size of the memory bank (as a static method).
    pub(crate) const fn get_size_from_type() -> usize {
        N
    }

    /// Load the `bytes` into the memory bank if the given slice has the same length as the bank.
    pub(crate) const fn load_bytes(&mut self, bytes: &[u8]) -> Result<(), MemoryBankLoadError> {
        if self.data.len() == bytes.len() {
            self.data.copy_from_slice(bytes);
            Ok(())
        } else {
            Err(MemoryBankLoadError::WrongSize {
                expected: self.data.len(),
                actual: bytes.len(),
            })
        }
    }
}
impl<const N: usize, Tag> clone::Clone for MemoryBank<N, Tag> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            _marker: PhantomData,
        }
    }
}

impl<const N: usize, Tag: NameTag> fmt::Debug for MemoryBank<N, Tag> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryBank")
            .field("name", &Tag::name())
            .finish_non_exhaustive()
    }
}

impl<const N: usize, Tag> default::Default for MemoryBank<N, Tag> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize, Tag: NameTag> MemoryBank<N, Tag> {
    /// Read a byte at `address`. The memory bank's address space starts at `BASE`.
    pub(crate) fn read_byte<const BASE: u16>(&self, address: u16) -> Result<u8, ReadByteError> {
        let translated_address = address.checked_sub(BASE).unwrap() as usize;
        self.data
            .get(translated_address)
            .ok_or(ReadByteError::InvalidAddressForPeripheral {
                name: Tag::name(),
                address,
            })
            .copied()
    }

    /// Write a byte to `address`. The memory bank's address space starts at `BASE`.
    pub(crate) fn write_byte<const BASE: u16>(
        &mut self,
        address: u16,
        value: u8,
    ) -> Result<(), WriteByteError> {
        let translated_address = address.checked_sub(BASE).unwrap() as usize;
        let byte = self.data.get_mut(translated_address).ok_or(
            WriteByteError::InvalidAddressForPeripheral {
                name: Tag::name(),
                address,
            },
        )?;
        *byte = value;
        Ok(())
    }
}
