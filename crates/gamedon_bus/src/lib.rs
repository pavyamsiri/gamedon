use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadByteError {}
#[derive(Debug, Error)]
pub enum WriteByteError {}

#[derive(Debug, Clone)]
pub struct MemoryBus {
    rom: [u8; 65536],
}

impl MemoryBus {
    #[inline]
    pub const fn read_byte(&self, address: u16) -> Result<u8, ReadByteError> {
        let value = self.rom[address as usize];
        Ok(value)
    }

    #[inline]
    pub const fn write_byte(&mut self, address: u16, value: u8) -> Result<(), WriteByteError> {
        self.rom[address as usize] = value;
        Ok(())
    }

    #[inline]
    pub const fn write_word(&mut self, address: u16, value: u16) -> Result<(), WriteByteError> {
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

    // TODO(pavyamsiri): Implement
    #[inline]
    pub const fn pending_interrupts(&self) -> bool {
        false
    }
}
