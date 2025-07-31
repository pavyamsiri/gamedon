use crate::{
    BusReader, BusWriter, Interrupt, InterruptSource, Peripheral, ReadByteError, WriteByteError,
};
use gamedon_bits::{BitShift8, FlagLatch, HwReg8};

/// Addresses mapping to the LCD registers.
#[macro_export]
macro_rules! lcd_addresses {
    () => {
        0xFF40..=0xFF45 | 0xFF47..=0xFF4B | 0xFF68..=0xFF6B
    };
}

/// Bit 6: LYC interrupt select.
const LYC_INT_SELECT: BitShift8 = BitShift8::Bit6;
/// Bit 5: OAM scan mode interrupt select.
const OAM_SCAN_INT_SELECT: BitShift8 = BitShift8::Bit5;
/// Bit 4: V blank interrupt select.
const VBLANK_INT_SELECT: BitShift8 = BitShift8::Bit4;
/// Bit 3: H blank interrupt select.
const HBLANK_INT_SELECT: BitShift8 = BitShift8::Bit3;
/// Bit 2: LYC = LY coincidence flag.
const LYC_COINCIDENCE: BitShift8 = BitShift8::Bit2;

/// Bit 0: Enable the background and window.
const BG_WINDOW_ENABLE: BitShift8 = BitShift8::Bit0;

#[derive(Debug, Clone, Copy, Default)]
pub enum PpuState {
    /// Mode 0: Horizontal blank.
    HBlank,
    /// Mode 1: Vertical blank.
    VBlank,
    /// Mode 2: OAM scan.
    #[default]
    OamScan,
    /// Mode 3: Horizontal draw.
    HDraw,
}

impl PpuState {
    /// Return the bit representation of the mode.
    const fn to_bits(self) -> u8 {
        match self {
            PpuState::HBlank => 0b00,
            PpuState::VBlank => 0b01,
            PpuState::OamScan => 0b10,
            PpuState::HDraw => 0b11,
        }
    }
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

    // Interrupt specific fields.
    interrupt_line: FlagLatch,
}

impl BusReader for Lcd {
    #[inline]
    fn read_byte(&self, address: u16) -> Result<u8, ReadByteError> {
        let value = match address {
            0xFF40 => self.control.0,
            // TODO(pavyamsiri): This is wrong. Bits 1 and 0 representing PPU state should report 0b00
            // if PPU is disabled.
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
                self.status.0 = (value & 0b0111_1000) | (self.status.0 & 0b0000_0111);
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

impl InterruptSource for Lcd {
    fn get_interrupt_request(&mut self) -> Option<Interrupt> {
        if self.interrupt_line.rising() {
            Some(Interrupt::Lcd)
        } else {
            None
        }
    }
}

impl Peripheral for Lcd {
    fn tick(&mut self) {
        self.interrupt_line.tick();
    }
}

impl Lcd {
    /// Return the current LCD/PPU mode.
    pub(crate) fn get_lcd_mode(&self) -> PpuState {
        match self.status.0 & 0x3 {
            0 => PpuState::HBlank,
            1 => PpuState::VBlank,
            2 => PpuState::OamScan,
            3 => PpuState::HDraw,
            _ => unreachable!("impossible due to bit mask"),
        }
    }

    /// Set the LCD/PPU mode.
    #[inline]
    pub(crate) const fn set_lcd_mode(&mut self, mode: PpuState) {
        self.status.0 = (self.status.0 & 0b1111_1100) | mode.to_bits();

        // Update interrupt source line
        if self.status.bit(OAM_SCAN_INT_SELECT) {
            self.interrupt_line.or(self.status.bit(OAM_SCAN_INT_SELECT));
        }
        if self.status.bit(VBLANK_INT_SELECT) {
            self.interrupt_line.or(self.status.bit(VBLANK_INT_SELECT));
        }
        if self.status.bit(HBLANK_INT_SELECT) {
            self.interrupt_line.or(self.status.bit(HBLANK_INT_SELECT));
        }
    }

    /// Whether the LCD/PPU is enabled.
    pub(crate) const fn is_lcd_enabled(&self) -> bool {
        self.control.bit(BitShift8::Bit7)
    }

    /// Return the scanline index.
    #[inline]
    pub(crate) const fn get_scanline(&self) -> u8 {
        self.ly
    }

    /// Increment `LY`/current scanline.
    pub(crate) const fn increment_scanline(&mut self) {
        self.ly = self.ly.wrapping_add(1);
        // Check LYC
        self.status.set_value(LYC_COINCIDENCE, self.ly == self.lyc);

        // Update interrupt source line
        if self.status.bit(LYC_INT_SELECT) {
            self.interrupt_line.or(self.status.bit(LYC_COINCIDENCE));
        }
    }

    /// Return the x coordindate of the viewport's top left pixel.
    #[inline]
    pub(crate) const fn get_viewport_x(&self) -> u8 {
        self.viewport_x
    }

    /// Return the y coordindate of the viewport's top left pixel.
    #[inline]
    pub(crate) const fn get_viewport_y(&self) -> u8 {
        self.viewport_y
    }

    /// Whether the background and window are enabled.
    #[inline]
    pub(crate) const fn get_background_and_window_enable(&self) -> bool {
        self.control.bit(BG_WINDOW_ENABLE)
    }
}
