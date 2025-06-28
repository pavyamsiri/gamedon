use crate::{BitShift8, BitShift16};
use owo_colors::{OwoColorize, Stream, Style};

const BIT_POSITION_STYLE: Style = Style::new().green();

macro_rules! impl_display_for_bitshift {
    ($type:ident, $($variant:ident => $num:expr),+) => {
        impl core::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let bit_num = match self {
                    $(
                        $type::$variant => $num,
                    )+
                };
                write!(
                    f,
                    "{}",
                    bit_num.to_string().if_supports_color(Stream::Stdout, |text| text.style(BIT_POSITION_STYLE))
                )
            }
        }
    };
}

impl_display_for_bitshift!(BitShift8,
    Bit0 => 0,
    Bit1 => 1,
    Bit2 => 2,
    Bit3 => 3,
    Bit4 => 4,
    Bit5 => 5,
    Bit6 => 6,
    Bit7 => 7
);

impl_display_for_bitshift!(BitShift16,
    Bit00 => 0,
    Bit01 => 1,
    Bit02 => 2,
    Bit03 => 3,
    Bit04 => 4,
    Bit05 => 5,
    Bit06 => 6,
    Bit07 => 7,
    Bit08 => 8,
    Bit09 => 9,
    Bit10 => 10,
    Bit11 => 11,
    Bit12 => 12,
    Bit13 => 13,
    Bit14 => 14,
    Bit15 => 15
);
