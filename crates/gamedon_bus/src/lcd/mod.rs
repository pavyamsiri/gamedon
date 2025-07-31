use crate::{BusReader, BusWriter, ReadByteError, WriteByteError};
use gamedon_bits::HwReg8;

/// Addresses mapping to the LCD registers.
#[macro_export]
macro_rules! lcd_addresses {
    () => {
        0xFF40..=0xFF45 | 0xFF47..=0xFF4B | 0xFF68..=0xFF6B
    };
}

/// The LCD registers.
#[derive(Debug, Clone, Default)]
pub(crate) struct Lcd {
    /// $FF40: LCD control or `LCDC`.
    control: HwReg8,
    /// $FF41: LCD status or `STATUS`.
    status: HwReg8,
    /// $FF42: Background viewport y position.
    viewport_y: u8,
    /// $FF43: Background viewport x position.
    viewport_x: u8,
    /// $FF44: LCD y coordinate or `LY`.
    ly: u8,
    /// $FF45: LY compare or `LYC`.
    lyc: u8,
    /// $FF47: Background palette data.
    background_palette: u8,
    /// $FF48: First object palette or `OBP0`.
    first_obj_palette: u8,
    /// $FF49: Second object palette or `OBP1`.
    second_obj_palette: u8,
    /// $FF4A: The y coordinate of the window's anchor (top left most pixel).
    window_y: u8,
    /// $FF4B: The x coordinate + 7 of the window's anchor (top left most pixel).
    /// The constant `+ 7` allows the window to be slightly off screen to the left.
    window_x: u8,
    /// $FF68: Background colour palette specification or background palette index.
    /// Only meaningful in CGB mode. Also known as `BCPS` or `BGPI`.
    bgpi: u8,
    /// $FF69: Background colour palette data or background palette data.
    /// Only meaningful in CGB mode. Also known as `BCPD` or `BGPD`.
    bgpd: u8,
    /// $FF6A: Object colour palette specification or object palette index.
    /// Only meaningful in CGB mode. Also known as `OCPS` or `OBPI`.
    obpi: u8,
    /// $FF6B: Object colour palette data or object palette data.
    /// Only meaningful in CGB mode. Also known as `OCPD` or `OBPD`.
    obpd: u8,
}

impl BusReader for Lcd {
    #[inline]
    fn read_byte(&self, address: u16) -> Result<u8, ReadByteError> {
        let value = match address {
            0xFF40 => self.control.0,
            0xFF41 => self.status.0,
            0xFF42 => self.viewport_y,
            0xFF43 => self.viewport_x,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.background_palette,
            0xFF48 => self.first_obj_palette,
            0xFF49 => self.second_obj_palette,
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            0xFF68 => self.bgpi,
            0xFF69 => self.bgpd,
            0xFF6A => self.obpi,
            0xFF6B => self.obpd,
            _ => {
                return Err(ReadByteError::InvalidAddressForPeripheral {
                    name: "LCD",
                    address,
                });
            }
        };
        Ok(value)
    }
}

impl BusWriter for Lcd {
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), WriteByteError> {
        match address {
            0xFF40 => {
                self.control.0 = value;
            }
            0xFF41 => {
                self.status.0 = value;
            }
            0xFF42 => {
                self.viewport_y = value;
            }
            0xFF43 => {
                self.viewport_x = value;
            }
            0xFF44 => {
                self.ly = value;
            }
            0xFF45 => {
                self.lyc = value;
            }
            0xFF47 => {
                self.background_palette = value;
            }
            0xFF48 => {
                self.first_obj_palette = value;
            }
            0xFF49 => {
                self.second_obj_palette = value;
            }
            0xFF4A => {
                self.window_y = value;
            }
            0xFF4B => {
                self.window_x = value;
            }
            0xFF68 => {
                self.bgpi = value;
            }
            0xFF69 => {
                self.bgpd = value;
            }
            0xFF6A => {
                self.obpi = value;
            }
            0xFF6B => {
                self.obpd = value;
            }
            _ => {
                return Err(WriteByteError::InvalidAddressForPeripheral {
                    name: "LCD",
                    address,
                });
            }
        }
        Ok(())
    }
}
