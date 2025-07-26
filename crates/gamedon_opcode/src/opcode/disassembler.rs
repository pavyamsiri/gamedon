use super::Instruction;
use super::{ADDRESS_STYLE, IMMEDIATE_STYLE, INSTRUCTION_STYLE};
use crate::{Decoder, Reg8};
use core::fmt::{self, Write};
use owo_colors::{OwoColorize, Stream};

macro_rules! apply_colour {
    ($token:expr, $style:expr) => {
        $token.if_supports_color(Stream::Stdout, |text| text.style($style))
    };
}

impl Instruction {
    /// Format an instruction into a nice display.
    pub fn format(
        &self,
        buffer: &mut String,
        stream: &mut impl Iterator<Item = (u16, u8)>,
        byte: u8,
    ) -> Result<(), fmt::Error> {
        let acc = Reg8::A;
        let mut bytes = vec![byte];
        let mut write_byte = |byte: Option<u8>| {
            if let Some(b) = byte {
                bytes.push(b);
                Some(b)
            } else {
                None
            }
        };

        macro_rules! next_u8 {
            () => {
                write_byte(stream.next().map(|(_, byte)| byte))
            };
        }

        macro_rules! next_u8_and_address {
            () => {
                stream
                    .next()
                    .map(|(address, byte)| (address, write_byte(Some(byte)).unwrap()))
            };
        }

        match self {
            Self::LdReg8Imm8 { dst } => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "{} {dst}, ", apply_colour!("ld", INSTRUCTION_STYLE))?;
                Self::format_u8(buffer, value)?;
            }
            Self::LdMem16Imm8 { dst } => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} ({dst}), ",
                    apply_colour!("ld", INSTRUCTION_STYLE),
                )?;
                Self::format_u8(buffer, value)?;
            }
            Self::LdAdr8Reg8 { src } => {
                let addr = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "{} (", apply_colour!("ld", INSTRUCTION_STYLE))?;
                Self::format_u8(buffer, addr)?;
                write!(buffer, "), {src}")?;
            }
            Self::LdReg8Adr8 { dst } => {
                let addr = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {dst}, (",
                    apply_colour!("ld", INSTRUCTION_STYLE)
                )?;
                Self::format_u8(buffer, addr)?;
                write!(buffer, ")")?;
            }
            Self::LdAdr16Reg8 { src } => {
                let lo = next_u8!();
                let hi = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "{} ", apply_colour!("ld", INSTRUCTION_STYLE))?;
                Self::format_u16_address(buffer, lo, hi)?;
                write!(buffer, ", {src}")?;
            }
            Self::LdReg8Adr16 { dst } => {
                let lo = next_u8!();
                let hi = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {dst}, (",
                    apply_colour!("ld", INSTRUCTION_STYLE)
                )?;
                Self::format_u16_address(buffer, lo, hi)?;
                write!(buffer, ")")?;
            }
            Self::LdReg16Imm16 { dst } => {
                let lo = next_u8!();
                let hi = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "{} {dst}, ", apply_colour!("ld", INSTRUCTION_STYLE))?;
                Self::format_u16(buffer, lo, hi)?;
            }
            Self::LdAdr16Reg16 { src } => {
                let lo = next_u8!();
                let hi = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "{} (", apply_colour!("ld", INSTRUCTION_STYLE))?;
                Self::format_u16_address(buffer, lo, hi)?;
                write!(buffer, "), {src}")?;
            }
            Self::LdReg16Off8 { dst } => {
                let offset = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "{} {dst}, ", apply_colour!("ld", INSTRUCTION_STYLE))?;
                Self::format_i8(buffer, offset)?;
            }
            Self::AddReg16Off8 { dst } => {
                let offset = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {dst}, ",
                    apply_colour!("add", INSTRUCTION_STYLE)
                )?;
                Self::format_i8(buffer, offset)?;
            }
            Self::Jr => {
                let offset = next_u8_and_address!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "{} ", apply_colour!("jr", INSTRUCTION_STYLE))?;
                Self::format_i8_address(buffer, offset)?;
            }
            Self::Jrc { condition } => {
                let offset = next_u8_and_address!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {condition}, ",
                    apply_colour!("jr", INSTRUCTION_STYLE)
                )?;
                Self::format_i8_address(buffer, offset)?;
            }
            Self::Jp => {
                let lo = next_u8!();
                let hi = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "{} ", apply_colour!("jp", INSTRUCTION_STYLE))?;
                Self::format_u16_address(buffer, lo, hi)?;
            }
            Self::Jpc { condition } => {
                let lo = next_u8!();
                let hi = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {condition}, ",
                    apply_colour!("jp", INSTRUCTION_STYLE)
                )?;
                Self::format_u16_address(buffer, lo, hi)?;
            }
            Self::JpReg16 { reg } => {
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "{} {reg}", apply_colour!("jp", INSTRUCTION_STYLE))?;
            }
            Self::Call => {
                let lo = next_u8!();
                let hi = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "{} ", apply_colour!("call", INSTRUCTION_STYLE))?;
                Self::format_u16_address(buffer, lo, hi)?;
            }
            Self::Callc { condition } => {
                let lo = next_u8!();
                let hi = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {condition:02}, ",
                    apply_colour!("call", INSTRUCTION_STYLE)
                )?;
                Self::format_u16_address(buffer, lo, hi)?;
            }
            Self::AddImm8 => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {acc}, ",
                    apply_colour!("add", INSTRUCTION_STYLE)
                )?;
                Self::format_u8(buffer, value)?;
            }
            Self::AdcImm8 => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {acc}, ",
                    apply_colour!("adc", INSTRUCTION_STYLE)
                )?;
                Self::format_u8(buffer, value)?;
            }
            Self::SubImm8 => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {acc}, ",
                    apply_colour!("sub", INSTRUCTION_STYLE)
                )?;
                Self::format_u8(buffer, value)?;
            }
            Self::SbcImm8 => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {acc}, ",
                    apply_colour!("sbc", INSTRUCTION_STYLE)
                )?;
                Self::format_u8(buffer, value)?;
            }
            Self::AndImm8 => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {acc}, ",
                    apply_colour!("and", INSTRUCTION_STYLE)
                )?;
                Self::format_u8(buffer, value)?;
            }
            Self::XorImm8 => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {acc}, ",
                    apply_colour!("xor", INSTRUCTION_STYLE)
                )?;
                Self::format_u8(buffer, value)?;
            }
            Self::OrImm8 => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "{} {acc}, ", apply_colour!("or", INSTRUCTION_STYLE))?;
                Self::format_u8(buffer, value)?;
            }
            Self::CpImm8 => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "{} {acc}, ", apply_colour!("cp", INSTRUCTION_STYLE))?;
                Self::format_u8(buffer, value)?;
            }
            _ => {
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "{}", apply_colour!(self, INSTRUCTION_STYLE))?;
            }
        }
        Ok(())
    }

    fn write_bytes_prefix(buffer: &mut String, bytes: &[u8]) -> Result<(), fmt::Error> {
        for b in bytes.iter() {
            write!(buffer, " ")?;
            write!(buffer, "{b:02X}")?;
        }
        let leftover = (3usize).saturating_sub(bytes.len());
        for _ in 0..leftover {
            write!(buffer, " __")?;
        }
        write!(buffer, " ")?;
        Ok(())
    }

    fn format_u8(buffer: &mut String, value: Option<u8>) -> Result<(), fmt::Error> {
        if let Some(byte) = value {
            write!(buffer, "{:#04X}", apply_colour!(byte, IMMEDIATE_STYLE))
        } else {
            write!(buffer, "0x??")
        }
    }

    fn format_i8(buffer: &mut String, value: Option<u8>) -> Result<(), fmt::Error> {
        if let Some(byte) = value {
            let value = byte as i8;
            write!(buffer, "+{:#04X}", apply_colour!(value, IMMEDIATE_STYLE),)
        } else {
            write!(buffer, "??")
        }
    }

    fn format_i8_address(
        buffer: &mut String,
        value: Option<(u16, u8)>,
    ) -> Result<(), core::fmt::Error> {
        if let Some((address, byte)) = value {
            let base_pc = address.wrapping_sub(1);
            let target = usize::from(base_pc).saturating_add_signed((byte as i8) as isize);
            write!(
                buffer,
                "{}{:04x}",
                apply_colour!("$", IMMEDIATE_STYLE),
                apply_colour!(target, IMMEDIATE_STYLE)
            )
        } else {
            write!(buffer, "??")
        }
    }

    fn format_u16(
        buffer: &mut String,
        lo: Option<u8>,
        hi: Option<u8>,
    ) -> Result<(), core::fmt::Error> {
        let value = match (lo, hi) {
            (None, None) => "????".to_owned(),
            (None, Some(hi)) => format!("{hi:02X}??"),
            (Some(lo), None) => format!("??{lo:02X}"),
            (Some(lo), Some(hi)) => {
                let value = ((hi as u16) << 8) | (lo as u16);
                format!("0x{value:04X}")
            }
        };

        write!(buffer, "{}", apply_colour!(value, IMMEDIATE_STYLE))
    }

    fn format_u16_address(
        buffer: &mut String,
        lo: Option<u8>,
        hi: Option<u8>,
    ) -> Result<(), core::fmt::Error> {
        let value = match (lo, hi) {
            (None, None) => "$????".to_owned(),
            (None, Some(hi)) => format!("${hi:02x}??"),
            (Some(lo), None) => format!("$??{lo:02x}"),
            (Some(lo), Some(hi)) => {
                let value = ((hi as u16) << 8) | (lo as u16);
                format!("${value:04x}")
            }
        };

        write!(buffer, "{}", apply_colour!(value, ADDRESS_STYLE))
    }
}
/// Diassemble a byte stream of addressses and opcodes and store output as a string.
pub fn disassemble(byte_stream: &mut impl Iterator<Item = (u16, u8)>) -> String {
    let mut buffer = String::new();

    let mut decoder = Decoder::default();

    let mut first_nop = true;
    loop {
        let Some((address, byte)) = byte_stream.next() else {
            break;
        };

        let inst = decoder.decode(byte);

        if inst.is_prefix() {
            continue;
        }

        if inst.is_nop() {
            if !first_nop {
                continue;
            }
            first_nop = false;
        } else {
            first_nop = true;
        }

        write!(buffer, "{address:#06X}: ").unwrap();
        inst.format(&mut buffer, byte_stream, byte).unwrap();
        buffer.push('\n');
    }

    buffer
}
