use crate::{
    TaggedMbc,
    mbc::{Mbc1, MbcCreationError, NoMbc},
};
use thiserror::Error;

pub(crate) use size::{RamSize, RomSize};

/// Types to represent ROM and RAM sizes;
mod size;

/// The size of the cartridge header plus the leading 0x100 bytes.
const HEADER_PLUS_PREFIX_SIZE: usize = 0x150;

/// Errors that can occur when loading the cartridge header.
#[derive(Error, Debug)]
pub enum HeaderLoadError {
    /// The header is too small.
    #[error(
        "The cartridge header must at least be {HEADER_PLUS_PREFIX_SIZE} but it is {actual} bytes instead."
    )]
    TooSmall { actual: usize },
    /// The header's specified ROM size is not valid.
    #[error("The ROM size byte is an invalid value {byte:#04X}.")]
    InvalidRomSize { byte: u8 },
    /// The header's specified RAM size is not valid.
    #[error("The RAM size byte is an invalid value {byte:#04X}.")]
    InvalidRamSize { byte: u8 },
    /// The header's cartridge type is not valid.
    #[error("The cartridge type byte is an invalid value {byte:#04X}.")]
    InvalidCartridgeType { byte: u8 },
}

/// The color mode.
#[derive(Debug)]
pub(crate) enum ColorMode {
    /// The cartridge has no CGB compatibility.
    Off,
    /// The cartridge has optional CGB features.
    Optional,
    /// The cartridge has required CGB features.
    Required,
}

/// The intended vendor destination of the cartridge.
#[derive(Debug)]
pub(crate) enum VendorDestination {
    /// Japan or overseas.
    Japan,
    /// Overseas only.
    OverseasOnly,
    /// Unknown.
    Unknown,
}

/// The cartridge header.
#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct CartridgeHeader {
    /// The cartridge title.
    title: String,
    /// The color mode.
    color_mode: ColorMode,
    /// The licensee name.
    licensee: &'static str,
    /// Whether the cartridge supports SGB functions.
    sgb_supported: bool,
    /// The cartridge type.
    cartridge_type: CartridgeType,
    /// The size of the ROM.
    rom_size: RomSize,
    /// The size of the RAM.
    ram_size: RamSize,
    /// The vendor destination.
    destination: VendorDestination,
    /// The version.
    version: u8,
}

impl CartridgeHeader {
    /// Parse `bytes` into the cartridge header.
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, HeaderLoadError> {
        if bytes.len() < HEADER_PLUS_PREFIX_SIZE {
            return Err(HeaderLoadError::TooSmall {
                actual: bytes.len(),
            });
        }

        // Title: $0134-$0143
        let title = String::from_utf8_lossy(&bytes[0x0134..=0x0143])
            .trim_end_matches('\0')
            .to_owned();

        // CGB flag: $0143
        let color_mode = match bytes[0x0143] {
            0x80 => ColorMode::Optional,
            0xC0 => ColorMode::Required,
            _ => ColorMode::Off,
        };

        // Old licensee code: $014B
        let old_licensee = bytes[0x014B];
        // New licensee code: $0144-$0145
        let new_licensee = u16::from_le_bytes([bytes[0x0144], bytes[0x0145]]);
        let licensee = Self::get_licensee(old_licensee, new_licensee);
        // SGB flag: $0146
        let sgb_supported = bytes[0x0146] == 0x03;
        // Cartridge type: $0147
        let cartridge_type = CartridgeType::from_byte(bytes[0x0147]).ok_or_else(|| {
            HeaderLoadError::InvalidCartridgeType {
                byte: bytes[0x0147],
            }
        })?;

        // ROM size: $0148
        let rom_size =
            RomSize::from_byte(bytes[0x0148]).ok_or_else(|| HeaderLoadError::InvalidRomSize {
                byte: bytes[0x0148],
            })?;

        // RAM size: $0149
        let ram_size =
            RamSize::from_byte(bytes[0x0149]).ok_or_else(|| HeaderLoadError::InvalidRamSize {
                byte: bytes[0x0149],
            })?;
        // Vendor destination: $014A
        let destination = match bytes[0x014A] {
            0x00 => VendorDestination::Japan,
            0x01 => VendorDestination::OverseasOnly,
            _ => VendorDestination::Unknown,
        };
        // Version: $014C
        let version = bytes[0x014C];

        Ok(Self {
            title,
            color_mode,
            licensee,
            sgb_supported,
            cartridge_type,
            rom_size,
            ram_size,
            destination,
            version,
        })
    }

    /// Return the licensee name given the old licensee code and the new licensee code.
    const fn get_licensee(old: u8, new: u16) -> &'static str {
        if old == 0x33 {
            Self::get_new_licensee(new)
        } else {
            Self::get_old_licensee(old)
        }
    }

    /// Return the licensee name given the old licensee `code`
    const fn get_old_licensee(code: u8) -> &'static str {
        match code {
            0x00 => "None",
            0x01 => "Nintendo",
            0x08 => "Capcom",
            0x09 => "Hot-B",
            0x0A => "Jaleco",
            0x0B => "Coconuts Japan",
            0x0C => "Elite Systems",
            0x13 => "EA (Electronic Arts)",
            0x18 => "Hudsonsoft",
            0x19 => "ITC Entertainment",
            0x1A => "Yanoman",
            0x1D => "Japan Clary",
            0x1F => "Virgin Interactive",
            0x24 => "PCM Complete",
            0x25 => "San-X",
            0x28 => "Kotobuki Systems",
            0x29 => "Seta",
            0x30 => "Infogrames",
            0x31 => "Nintendo",
            0x32 => "Bandai",
            0x33 => "Indicates that the New licensee code should be used instead.",
            0x34 => "Konami",
            0x35 => "HectorSoft",
            0x38 => "Capcom",
            0x39 => "Banpresto",
            0x3C => ".Entertainment i",
            0x3E => "Gremlin",
            0x41 => "Ubisoft",
            0x42 => "Atlus",
            0x44 => "Malibu",
            0x46 => "Angel",
            0x47 => "Spectrum Holoby",
            0x49 => "Irem",
            0x4A => "Virgin Interactive",
            0x4D => "Malibu",
            0x4F => "U.S. Gold",
            0x50 => "Absolute",
            0x51 => "Acclaim",
            0x52 => "Activision",
            0x53 => "American Sammy",
            0x54 => "GameTek",
            0x55 => "Park Place",
            0x56 => "LJN",
            0x57 => "Matchbox",
            0x59 => "Milton Bradley",
            0x5A => "Mindscape",
            0x5B => "Romstar",
            0x5C => "Naxat Soft",
            0x5D => "Tradewest",
            0x60 => "Titus",
            0x61 => "Virgin Interactive",
            0x67 => "Ocean Interactive",
            0x69 => "EA (Electronic Arts)",
            0x6E => "Elite Systems",
            0x6F => "Electro Brain",
            0x70 => "Infogrames",
            0x71 => "Interplay",
            0x72 => "Broderbund",
            0x73 => "Sculptered Soft",
            0x75 => "The Sales Curve",
            0x78 => "t.hq",
            0x79 => "Accolade",
            0x7A => "Triffix Entertainment",
            0x7C => "Microprose",
            0x7F => "Kemco",
            0x80 => "Misawa Entertainment",
            0x83 => "Lozc",
            0x86 => "Tokuma Shoten Intermedia",
            0x8B => "Bullet-Proof Software",
            0x8C => "Vic Tokai",
            0x8E => "Ape",
            0x8F => "I'Max",
            0x91 => "Chunsoft Co.",
            0x92 => "Video System",
            0x93 => "Tsubaraya Productions Co.",
            0x95 => "Varie Corporation",
            0x96 => "Yonezawa/S'Pal",
            0x97 => "Kaneko",
            0x99 => "Arc",
            0x9A => "Nihon Bussan",
            0x9B => "Tecmo",
            0x9C => "Imagineer",
            0x9D => "Banpresto",
            0x9F => "Nova",
            0xA1 => "Hori Electric",
            0xA2 => "Bandai",
            0xA4 => "Konami",
            0xA6 => "Kawada",
            0xA7 => "Takara",
            0xA9 => "Technos Japan",
            0xAA => "Broderbund",
            0xAC => "Toei Animation",
            0xAD => "Toho",
            0xAF => "Namco",
            0xB0 => "acclaim",
            0xB1 => "ASCII or Nexsoft",
            0xB2 => "Bandai",
            0xB4 => "Square Enix",
            0xB6 => "HAL Laboratory",
            0xB7 => "SNK",
            0xB9 => "Pony Canyon",
            0xBA => "Culture Brain",
            0xBB => "Sunsoft",
            0xBD => "Sony Imagesoft",
            0xBF => "Sammy",
            0xC0 => "Taito",
            0xC2 => "Kemco",
            0xC3 => "Squaresoft",
            0xC4 => "Tokuma Shoten Intermedia",
            0xC5 => "Data East",
            0xC6 => "Tonkinhouse",
            0xC8 => "Koei",
            0xC9 => "UFL",
            0xCA => "Ultra",
            0xCB => "Vap",
            0xCC => "Use Corporation",
            0xCD => "Meldac",
            0xCE => ".Pony Canyon or",
            0xCF => "Angel",
            0xD0 => "Taito",
            0xD1 => "Sofel",
            0xD2 => "Quest",
            0xD3 => "Sigma Enterprises",
            0xD4 => "ASK Kodansha Co.",
            0xD6 => "Naxat Soft",
            0xD7 => "Copya System",
            0xD9 => "Banpresto",
            0xDA => "Tomy",
            0xDB => "LJN",
            0xDD => "NCS",
            0xDE => "Human",
            0xDF => "Altron",
            0xE0 => "Jaleco",
            0xE1 => "Towa Chiki",
            0xE2 => "Yutaka",
            0xE3 => "Varie",
            0xE5 => "Epcoh",
            0xE7 => "Athena",
            0xE8 => "Asmik ACE Entertainment",
            0xE9 => "Natsume",
            0xEA => "King Records",
            0xEB => "Atlus",
            0xEC => "Epic/Sony Records",
            0xEE => "IGS",
            0xF0 => "A Wave",
            0xF3 => "Extreme Entertainment",
            0xFF => "LJN",
            _ => "Unknown",
        }
    }

    /// Return the licensee name given the new licensee `code`
    const fn get_new_licensee(code: u16) -> &'static str {
        match code {
            0x3030 => "None",
            0x3031 => "Nintendo R&D1",
            0x3038 => "Capcom",
            0x3133 => "Electronic Arts",
            0x3138 => "Hudson Soft",
            0x3139 => "b-ai",
            0x3230 => "kss",
            0x3232 => "pow",
            0x3234 => "PCM Complete",
            0x3235 => "san-x",
            0x3238 => "Kemco Japan",
            0x3239 => "seta",
            0x3330 => "Viacom",
            0x3331 => "Nintendo",
            0x3332 => "Bandai",
            0x3333 => "Ocean/Acclaim",
            0x3334 => "Konami",
            0x3335 => "Hector",
            0x3337 => "Taito",
            0x3338 => "Hudson",
            0x3339 => "Banpresto",
            0x3431 => "Ubi Soft",
            0x3432 => "Atlus",
            0x3434 => "Malibu",
            0x3436 => "angel",
            0x3437 => "Bullet-Proof",
            0x3439 => "irem",
            0x3530 => "Absolute",
            0x3531 => "Acclaim",
            0x3532 => "Activision",
            0x3533 => "American sammy",
            0x3534 => "Konami",
            0x3535 => "Hi tech entertainment",
            0x3536 => "LJN",
            0x3537 => "Matchbox",
            0x3538 => "Mattel",
            0x3539 => "Milton Bradley",
            0x3630 => "Titus",
            0x3631 => "Virgin",
            0x3634 => "LucasArts",
            0x3637 => "Ocean",
            0x3639 => "Electronic Arts",
            0x3730 => "Infogrames",
            0x3731 => "Interplay",
            0x3732 => "Broderbund",
            0x3733 => "sculptured",
            0x3735 => "sci",
            0x3738 => "THQ",
            0x3739 => "Accolade",
            0x3830 => "misawa",
            0x3833 => "lozc",
            0x3836 => "Tokuma Shoten Intermedia",
            0x3837 => "Tsukuda Original",
            0x3931 => "Chunsoft",
            0x3932 => "Video system",
            0x3933 => "Ocean/Acclaim",
            0x3935 => "Varie",
            0x3936 => "Yonezawa/s'pal",
            0x3937 => "Kaneko",
            0x3939 => "Pack in soft",
            0x4134 => "Konami (Yu-Gi-Oh!)",
            _ => "Unknown",
        }
    }

    /// Return the memory bank controller of the cartridge.
    pub(crate) fn get_mbc(&self) -> Result<TaggedMbc, MbcCreationError> {
        match self.cartridge_type {
            CartridgeType::RomOnly { .. } => Ok(TaggedMbc::RomOnly(NoMbc::new(
                self.rom_size,
                self.ram_size,
            )?)),
            CartridgeType::Mbc1 { .. } => {
                Ok(TaggedMbc::Mbc1(Mbc1::new(self.rom_size, self.ram_size)?))
            }
            CartridgeType::Mbc2 { .. } => unimplemented!(),
            CartridgeType::Mbc3 { .. } => unimplemented!(),
            CartridgeType::Mbc5 { .. } => unimplemented!(),
            CartridgeType::Mbc6 => unimplemented!(),
            CartridgeType::Mbc7 => unimplemented!(),
            CartridgeType::Camera => unimplemented!(),
            CartridgeType::BandaiTama5 => unimplemented!(),
            CartridgeType::Mmm01 { .. } => unimplemented!(),
            CartridgeType::M161 => unimplemented!(),
            CartridgeType::HuC1 => unimplemented!(),
            CartridgeType::HuCDash3 => unimplemented!(),
        }
    }
}

/// The memory controller type.
#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum CartridgeType {
    /// For games that are 32KiB or less, the ROM gets directly mapped
    /// to memory at $0000-$7FFF.
    RomOnly {
        /// Whether the RAM is battery backed.
        has_battery: bool,
    },
    /// MBC1 can support up to 2MiB ROMs and 32KiB RAM.
    Mbc1 {
        /// Whether the RAM is battery backed.
        has_battery: bool,
    },
    /// MBC2 can support up to 256KiB ROMs and 512x4 bits RAM.
    Mbc2 {
        /// Whether the RAM is battery backed.
        has_battery: bool,
    },
    /// MBC3 can support up to 2MiB ROMs and 32KiB RAM and also an RTC timer.
    Mbc3 {
        /// Whether there is an RTC timer.
        has_timer: bool,
        /// Whether the RAM is battery backed.
        has_battery: bool,
    },
    /// MBC5 can support up to 8MiB ROMs and 128KiB RAM.
    /// Is guaranteed to work with CGB double speed mode.
    Mbc5 {
        /// Whether the RAM is battery backed.
        has_battery: bool,
        /// Whether the rumble pack is included.
        has_rumble: bool,
    },
    /// MBC6 controller.
    Mbc6,
    /// MBC7 controller.
    Mbc7,
    /// A pocket camera.
    Camera,
    /// Unimplemented MBC.
    BandaiTama5,
    /// Unimplemented MBC.
    Mmm01 {
        /// Whether the RAM is battery backed.
        has_battery: bool,
    },
    /// Unimplemented MBC.
    M161,
    /// Unimplemented MBC.
    HuC1,
    /// Unimplemented MBC.
    HuCDash3,
}

impl CartridgeType {
    /// Parse the cartridge type byte.
    pub(crate) const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 | 0x08 => Some(Self::RomOnly { has_battery: false }),
            0x01 | 0x02 => Some(Self::Mbc1 { has_battery: false }),
            0x03 => Some(Self::Mbc1 { has_battery: true }),
            0x05 => Some(Self::Mbc2 { has_battery: false }),
            0x06 => Some(Self::Mbc2 { has_battery: true }),
            0x09 => Some(Self::RomOnly { has_battery: true }),
            0x0B | 0x0C => Some(Self::Mmm01 { has_battery: false }),
            0x0D => Some(Self::Mmm01 { has_battery: true }),
            0x0F | 0x10 => Some(Self::Mbc3 {
                has_timer: true,
                has_battery: true,
            }),
            0x11 | 0x12 => Some(Self::Mbc3 {
                has_timer: false,
                has_battery: false,
            }),
            0x13 => Some(Self::Mbc3 {
                has_timer: false,
                has_battery: true,
            }),
            0x19 | 0x1A => Some(Self::Mbc5 {
                has_battery: false,
                has_rumble: false,
            }),
            0x1B => Some(Self::Mbc5 {
                has_battery: true,
                has_rumble: false,
            }),
            0x1C | 0x1D => Some(Self::Mbc5 {
                has_battery: false,
                has_rumble: true,
            }),
            0x1E => Some(Self::Mbc5 {
                has_battery: true,
                has_rumble: true,
            }),
            0x20 => Some(Self::Mbc6),
            0x22 => Some(Self::Mbc7),
            0xFC => Some(Self::Camera),
            0xFD => Some(Self::BandaiTama5),
            0xFE => Some(Self::HuCDash3),
            0xFF => Some(Self::HuC1),
            _ => None,
        }
    }
}
