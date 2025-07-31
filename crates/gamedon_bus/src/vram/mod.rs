use crate::{BusReader, BusWriter, MemoryBank, NameTag, ReadByteError, WriteByteError};

/// Addresses mapping to OAM.
#[macro_export]
macro_rules! oam_addresses {
    () => {
        0xFE00..=0xFE97
    };
}

/// Addresses mapping to the VRAM.
#[macro_export]
macro_rules! vram_addresses {
    () => {
        0x8000..=0x9FFF | oam_addresses!()
    };
}

/// The tag for tile data.
struct TileDataTag;
impl NameTag for TileDataTag {
    fn name() -> &'static str {
        "Tile Data"
    }
}

/// The tag for tile map.
struct TileMapTag;
impl NameTag for TileMapTag {
    fn name() -> &'static str {
        "Tile Map"
    }
}

/// The tag for OAM.
struct OamTag;
impl NameTag for OamTag {
    fn name() -> &'static str {
        "OAM"
    }
}

/// The video RAM containing data for rendering tiles and sprites.
#[derive(Debug, Clone, Default)]
pub(crate) struct VideoRam {
    /// $8000-$97FF: Tile data.
    tiles: MemoryBank<0x1800, TileDataTag>,
    /// $9800-$9FFF: Two 32x32 tile maps each 1024 bytes.
    tile_maps: MemoryBank<0x0800, TileMapTag>,
    /// $FE00-$FE9F: Object attribute memory or OAM.
    oam: MemoryBank<0x00A0, OamTag>,
}

impl BusReader for VideoRam {
    #[inline]
    fn read_byte(&self, address: u16) -> Result<u8, ReadByteError> {
        match address {
            0x8000..=0x97FF => self.tiles.read_byte::<0x8000>(address),
            0x9800..=0x9FFF => self.tile_maps.read_byte::<0x9800>(address),
            oam_addresses!() => self.oam.read_byte::<0xFE00>(address),
            _ => Err(ReadByteError::InvalidAddressForPeripheral {
                name: "Video RAM",
                address,
            }),
        }
    }
}

impl BusWriter for VideoRam {
    fn write_byte(&mut self, address: u16, value: u8) -> Result<(), WriteByteError> {
        match address {
            0x8000..=0x97FF => self.tiles.write_byte::<0x8000>(address, value),
            0x9800..=0x9FFF => self.tile_maps.write_byte::<0x9800>(address, value),
            oam_addresses!() => self.oam.write_byte::<0xFE00>(address, value),
            _ => Err(WriteByteError::InvalidAddressForPeripheral {
                name: "Video RAM",
                address,
            }),
        }
    }
}

impl VideoRam {
    /// Write `value` to $FE00 + `address`.
    /// The `address` must be within `0x00..=0x9F` otherwise this function will fail.
    pub(crate) fn write_to_oam(&mut self, address: u8, value: u8) -> Result<(), WriteByteError> {
        self.oam.write_byte::<0x0000>(u16::from(address), value)
    }
}
