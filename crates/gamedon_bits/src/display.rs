use crate::{BitShift8, BitShift16};
use core::fmt;
use owo_colors::{OwoColorize, Stream, Style};

/// The style to use when pretty printing a bit shift.
const BIT_POSITION_STYLE: Style = Style::new().green();

impl fmt::Display for BitShift8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.get_shift_amount()
                .if_supports_color(Stream::Stdout, |text| text.style(BIT_POSITION_STYLE))
        )
    }
}

impl fmt::Display for BitShift16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.get_shift_amount()
                .if_supports_color(Stream::Stdout, |text| text.style(BIT_POSITION_STYLE))
        )
    }
}
