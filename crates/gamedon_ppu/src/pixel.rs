#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum ColourId {
    #[default]
    Zero,
    One,
    Two,
    Three,
}

impl ColourId {
    pub(crate) const fn to_bits(self) -> u8 {
        match self {
            Self::Zero => 0,
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum PaletteIndex {
    #[default]
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Pixel {
    /// The index into the four colour palette.
    pub(crate) colour: ColourId,
    /// The index of the palette.
    pub(crate) palette: PaletteIndex,
    #[allow(dead_code)]
    /// In CGB mode this contains the OAM index.
    pub(crate) obj_priority: u8,
    /// If true this then backgrounds and windows will take priority over sprites when rendering,
    /// and if false then sprites take priority over others.
    pub(crate) bg_priority: bool,
}
