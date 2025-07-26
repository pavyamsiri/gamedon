/// The number of bytes in a kibibyte.
const KIB_IN_BYTES: usize = 1024;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RomSize {
    Rom32KiB,
    Rom64KiB,
    Rom128KiB,
    Rom256KiB,
    Rom512KiB,
    Rom1024KiB,
    Rom2048KiB,
    Rom4096KiB,
    Rom8192KiB,
}

impl RomSize {
    pub(crate) const fn get_size_in_bytes(self) -> usize {
        match self {
            Self::Rom32KiB => 32 * KIB_IN_BYTES,
            Self::Rom64KiB => 64 * KIB_IN_BYTES,
            Self::Rom128KiB => 128 * KIB_IN_BYTES,
            Self::Rom256KiB => 256 * KIB_IN_BYTES,
            Self::Rom512KiB => 512 * KIB_IN_BYTES,
            Self::Rom1024KiB => 1024 * KIB_IN_BYTES,
            Self::Rom2048KiB => 2048 * KIB_IN_BYTES,
            Self::Rom4096KiB => 4096 * KIB_IN_BYTES,
            Self::Rom8192KiB => 8192 * KIB_IN_BYTES,
        }
    }

    pub(crate) const fn get_number_of_banks(self) -> usize {
        match self {
            Self::Rom32KiB => 2,
            Self::Rom64KiB => 4,
            Self::Rom128KiB => 8,
            Self::Rom256KiB => 16,
            Self::Rom512KiB => 32,
            Self::Rom1024KiB => 64,
            Self::Rom2048KiB => 128,
            Self::Rom4096KiB => 256,
            Self::Rom8192KiB => 512,
        }
    }

    pub(crate) const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::Rom32KiB),
            0x01 => Some(Self::Rom64KiB),
            0x02 => Some(Self::Rom128KiB),
            0x03 => Some(Self::Rom256KiB),
            0x04 => Some(Self::Rom512KiB),
            0x05 => Some(Self::Rom1024KiB),
            0x06 => Some(Self::Rom2048KiB),
            0x07 => Some(Self::Rom4096KiB),
            0x08 => Some(Self::Rom8192KiB),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RamSize {
    Ram0KiB,
    Ram8KiB,
    Ram32KiB,
    Ram64KiB,
    Ram128KiB,
}

impl RamSize {
    pub(crate) const fn get_size_in_bytes(self) -> usize {
        match self {
            RamSize::Ram0KiB => 0,
            RamSize::Ram8KiB => 8 * KIB_IN_BYTES,
            RamSize::Ram32KiB => 32 * KIB_IN_BYTES,
            RamSize::Ram64KiB => 64 * KIB_IN_BYTES,
            RamSize::Ram128KiB => 128 * KIB_IN_BYTES,
        }
    }

    pub(crate) const fn get_number_of_banks(self) -> usize {
        match self {
            RamSize::Ram0KiB => 0,
            RamSize::Ram8KiB => 1,
            RamSize::Ram32KiB => 4,
            RamSize::Ram64KiB => 8,
            RamSize::Ram128KiB => 16,
        }
    }

    pub(crate) const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::Ram0KiB),
            0x01 => Some(Self::Ram8KiB),
            0x02 => Some(Self::Ram8KiB),
            0x03 => Some(Self::Ram32KiB),
            0x04 => Some(Self::Ram128KiB),
            0x05 => Some(Self::Ram64KiB),
            _ => None,
        }
    }
}
