use crate::register::{Reg8, Reg16};
use core::fmt::Write;

#[derive(Debug, Clone, Copy)]
pub enum Condition {
    Z,
    NZ,
    C,
    NC,
}

impl core::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Condition::Z => write!(f, "Z"),
            Condition::NZ => write!(f, "NZ"),
            Condition::C => write!(f, "C"),
            Condition::NC => write!(f, "NC"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RstAddress {
    RST00,
    RST08,
    RST10,
    RST18,
    RST20,
    RST28,
    RST30,
    RST38,
}

impl core::fmt::Display for RstAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RstAddress::RST00 => write!(f, "00"),
            RstAddress::RST08 => write!(f, "08"),
            RstAddress::RST10 => write!(f, "10"),
            RstAddress::RST18 => write!(f, "18"),
            RstAddress::RST20 => write!(f, "20"),
            RstAddress::RST28 => write!(f, "28"),
            RstAddress::RST30 => write!(f, "30"),
            RstAddress::RST38 => write!(f, "38"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    /// No operation.
    Nop,
    /// Halt the system clock and wait for interrupts.
    Halt,
    /// Stop the system clock until a reset occurs or there is joypad input.
    Stop,
    /// Enable interrupts.
    Ei,
    /// Disable interrupts.
    Di,
    /// Decimal adjust the accumulator to turn it into a binary coded decimal (BCD).
    Daa,
    /// Set the carry flag.
    Scf,
    /// Flip the carry flag.
    Ccf,
    /// Flip bits of the accumulator.
    Cpl,
    /// Rotate accumulator left.
    /// The overflow bit (bit 7) is stored in both the carry flag and new bit 0.
    Rlca,
    /// Rotate accumulator left.
    /// The overflow bit (bit 7) is stored in the carry flag and the new bit 0 is set
    /// to the previous carry flag value.
    Rla,
    /// Rotate accumulator right.
    /// The overflow bit (bit 7) is stored in both the carry flag and new bit 0.
    Rrca,
    /// Rotate accumulator right.
    /// The overflow bit (bit 7) is stored in the carry flag and the new bit 0 is set
    /// to the previous carry flag value.
    Rra,
    /// Invalid.
    Invalid,
    /// Prefix.
    Prefix,

    // 8-bit loads
    /// Load byte from 8-bit register into another 8-bit register.
    LdReg8Reg8 { dst: Reg8, src: Reg8 },
    /// Load byte from address given by 16-bit register into a 8-bit register.
    LdReg8Mem16 { dst: Reg8, src: Reg16 },
    /// Load immediate byte into 8-bit register.
    LdReg8Imm8 { dst: Reg8 },
    /// Load byte from 8-bit register into address given by 16-bit register.
    LdMem16Reg8 { dst: Reg16, src: Reg8 },
    /// Load immediate byte into address given by 16-bit register.
    LdMem16Imm8 { dst: Reg16 },
    /// Load byte from 8-bit register into address given by immediate byte,
    /// High byte of address is 0xFF.
    LdAdr8Reg8 { src: Reg8 },
    /// Load byte from address given by immediate byte into an 8-bit register.
    /// High byte of address is 0xFF.
    LdReg8Adr8 { dst: Reg8 },
    /// Load byte from 8-bit register into address given by an 8-bit register,
    /// High byte of address is 0xFF.
    LdMem8Reg8 { dst: Reg8, src: Reg8 },
    /// Load byte from address given by 8-bit register into an 8-bit register.
    /// High byte of address is 0xFF.
    LdReg8Mem8 { dst: Reg8, src: Reg8 },
    /// Load byte from 8-bit register into address given by immediate word.
    LdAdr16Reg8 { src: Reg8 },
    /// Load byte from address given by immediate word into an 8-bit register.
    LdReg8Adr16 { dst: Reg8 },
    /// Load byte from 8-bit register into address given by 16-bit register,
    /// then increment the 16-bit register.
    LdIncMem16Reg8 { dst: Reg16, src: Reg8 },
    /// Load byte from 8-bit register into address given by 16-bit register,
    /// then decrement the 16-bit register.
    LdDecMem16Reg8 { dst: Reg16, src: Reg8 },
    /// Load byte from address given by 16-bit register into 8-bit register,
    /// then increment the 16-bit register.
    LdIncReg8MemR16 { dst: Reg8, src: Reg16 },
    /// Load byte from address given by 16-bit register into 8-bit register,
    /// then decrement the 16-bit register.
    LdDecReg8MemR16 { dst: Reg8, src: Reg16 },

    // 16-bit loads
    /// Load immediate word into 16-bit register.
    LdReg16Imm16 { dst: Reg16 },
    /// Load word from 16-bit register into address given by immediate word.
    LdAdr16Reg16 { src: Reg16 },
    /// Load word from 16-bit register into another 16-bit register.
    LdReg16Reg16 { dst: Reg16, src: Reg16 },
    /// Load word from address given by stack pointer offset by signed immediate byte
    /// into a 16-bit register.
    LdReg16Off8 { dst: Reg16 },

    // 8-bit increment and decrements
    /// Increment 8-bit register.
    IncReg8 { reg: Reg8 },
    /// Decrement 8-bit register.
    DecReg8 { reg: Reg8 },
    /// Increment byte at address given by 16-bit register.
    IncMem16 { reg: Reg16 },
    /// Increment byte at address given by 16-bit register.
    DecMem16 { reg: Reg16 },

    // 16-bit increment and decrements
    /// Increment 16-bit register.
    IncReg16 { reg: Reg16 },
    /// Decrement 16-bit register.
    DecReg16 { reg: Reg16 },

    // 16-bit adds
    /// Add a 16-bit register to another 16-bit register.
    AddReg16Reg16 { dst: Reg16, src: Reg16 },
    /// Add to a 16-bit register, a signed 8-bit immediate byte.
    AddReg16Off8 { dst: Reg16 },

    // Stack push and pop
    /// Push a 16-bit register value onto the stack.
    PushReg16 { src: Reg16 },
    /// Pop a 16-bit value from the stack, storing the result in a 16-bit register.
    PopReg16 { dst: Reg16 },

    // Relative jumps
    /// Jump relative to the current program counter by a signed immediate value.
    Jr,
    /// Jump relative to the current program counter by a signed immediate value if
    /// a condition is met.
    Jrc { condition: Condition },

    // Absolute jumps
    /// Jump to address given by immediate word.
    Jp,
    /// Jump to address given by immediate word if a condition is met.
    Jpc { condition: Condition },
    /// Jump to address given by 16-bit register.
    JpReg16 { reg: Reg16 },

    // Returns
    /// Pop a 16-bit value from the stack, treat it as an address and jump to it.
    Ret,
    /// Pop a 16-bit value from the stack, treat it as an address and jump to it if
    /// a condition is met.
    Retc { condition: Condition },
    /// Pop a 16-bit value from the stack, treat it as an address and jump to it,
    /// and also enable interrupts.
    Reti,

    // Calls
    /// Push the current program counter onto the stack and jump to the given address.
    Call,
    /// Push the current program counter onto the stack and jump to the given address if
    /// a condition is met.
    Callc { condition: Condition },

    // Resets
    /// Push the current program counter onto the stack and jump to the given reset
    /// vector address.
    Rst { target: RstAddress },

    // Arithmetic adds
    /// Add an 8-bit register to the accumulator.
    AddReg8 { src: Reg8 },
    /// Add a byte from an address given by a 16-bit register to the accumulator.
    AddMem16 { src: Reg16 },
    /// Add an immediate byte to the accumulator.
    AddImm8,

    // Arithmetic adds with carry
    /// Add an 8-bit register to the accumulator along with the carry flag.
    AdcReg8 { src: Reg8 },
    /// Add a byte from an address given by a 16-bit register to the accumulator along with
    /// the carry flag.
    AdcMem16 { src: Reg16 },
    /// Add an immediate byte to the accumulator along with the carry flag.
    AdcImm8,

    // Arithmetic subtractions
    /// Subtract an 8-bit register from the accumulator.
    SubReg8 { src: Reg8 },
    /// Subtract a byte at an address given by a 16-bit register from the accumulator.
    SubMem16 { src: Reg16 },
    /// Subtract an immediate byte from the accumulator.
    SubImm8,

    // Arithmetic subtractions with carry
    /// Subtract an 8-bit register from the accumulator with the carry flag.
    SbcReg8 { src: Reg8 },
    /// Subtract a byte at an address given by a 16-bit register from the accumulator with
    /// the carry flag.
    SbcMem16 { src: Reg16 },
    /// Subtract an immediate byte from the accumulator with the carry flag.
    SbcImm8,

    // Bitwise ands
    /// Bitwise and the accumulator and an 8-bit register.
    AndReg8 { src: Reg8 },
    /// Bitwise and the accumulator and a byte at the address given by a 16-bit register.
    AndMem16 { src: Reg16 },
    /// Bitwise and the accumulator and an immediate byte.
    AndImm8,

    // Bitwise xors
    /// Bitwise xors the accumulator and an 8-bit register.
    XorReg8 { src: Reg8 },
    /// Bitwise xors the accumulator and a byte at the address given by a 16-bit register.
    XorMem16 { src: Reg16 },
    /// Bitwise xors the accumulator and an immediate byte.
    XorImm8,

    // Bitwise ors
    /// Bitwise ors the accumulator and an 8-bit register.
    OrReg8 { src: Reg8 },
    /// Bitwise ors the accumulator and a byte at the address given by a 16-bit register.
    OrMem16 { src: Reg16 },
    /// Bitwise ors the accumulator and an immediate byte.
    OrImm8,

    // Comparisons
    /// Compare the accumulator and an 8-bit register.
    CpReg8 { src: Reg8 },
    /// Compare the accumulator and a byte at the address given by a 16-bit register.
    CpMem16 { src: Reg16 },
    /// Compare the accumulator and an immediate byte.
    CpImm8,
}

impl core::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Nop => write!(f, "nop"),
            Instruction::Halt => write!(f, "halt"),
            Instruction::Stop => write!(f, "stop"),
            Instruction::Ei => write!(f, "ei"),
            Instruction::Di => write!(f, "di"),
            Instruction::Daa => write!(f, "daa"),
            Instruction::Scf => write!(f, "scf"),
            Instruction::Ccf => write!(f, "ccf"),
            Instruction::Cpl => write!(f, "cpl"),
            Instruction::Rlca => write!(f, "rlca"),
            Instruction::Rla => write!(f, "rla"),
            Instruction::Rrca => write!(f, "rrca"),
            Instruction::Rra => write!(f, "rra"),
            Instruction::Invalid => write!(f, "invalid"),
            Instruction::Prefix => write!(f, "prefix"),
            Instruction::LdReg8Reg8 { dst, src } => write!(f, "ld {dst}, {src}"),
            Instruction::LdReg8Mem16 { dst, src } => write!(f, "ld {dst}, ({src})"),
            Instruction::LdReg8Imm8 { dst } => write!(f, "ld {dst}, imm8"),
            Instruction::LdMem16Reg8 { dst, src } => write!(f, "ld ({dst}), {src}"),
            Instruction::LdMem16Imm8 { dst } => write!(f, "ld ({dst}), imm8"),
            Instruction::LdAdr8Reg8 { src } => write!(f, "ld (imm8), {src}"),
            Instruction::LdReg8Adr8 { dst } => write!(f, "ld {dst}, (imm8)"),
            Instruction::LdMem8Reg8 { dst, src } => write!(f, "ld ({dst}), {src}"),
            Instruction::LdReg8Mem8 { dst, src } => write!(f, "ld {dst}, ({src})"),
            Instruction::LdAdr16Reg8 { src } => write!(f, "ld (imm16), {src}"),
            Instruction::LdReg8Adr16 { dst } => write!(f, "ld {dst}, (imm16)"),
            Instruction::LdIncMem16Reg8 { dst, src } => write!(f, "ldi ({dst}) {src}"),
            Instruction::LdDecMem16Reg8 { dst, src } => write!(f, "ldd ({dst}) {src}"),
            Instruction::LdIncReg8MemR16 { dst, src } => write!(f, "ldi {dst} ({src})"),
            Instruction::LdDecReg8MemR16 { dst, src } => write!(f, "ldd {dst} ({src})"),
            Instruction::LdReg16Imm16 { dst } => write!(f, "ld {dst}, imm16"),
            Instruction::LdAdr16Reg16 { src } => write!(f, "ld (imm16), {src}"),
            Instruction::LdReg16Reg16 { dst, src } => write!(f, "ld {dst}, {src}"),
            Instruction::LdReg16Off8 { dst } => write!(f, "ld {dst}, s8"),
            Instruction::IncReg8 { reg } => write!(f, "inc {reg}"),
            Instruction::DecReg8 { reg } => write!(f, "dec {reg}"),
            Instruction::IncMem16 { reg } => write!(f, "inc ({reg})"),
            Instruction::DecMem16 { reg } => write!(f, "dec ({reg})"),
            Instruction::IncReg16 { reg } => write!(f, "inc {reg}"),
            Instruction::DecReg16 { reg } => write!(f, "dec {reg}"),
            Instruction::AddReg16Reg16 { dst, src } => write!(f, "add {dst} {src}"),
            Instruction::AddReg16Off8 { dst } => write!(f, "add {dst} e8"),
            Instruction::PushReg16 { src } => write!(f, "push {src}"),
            Instruction::PopReg16 { dst } => write!(f, "pop {dst}"),
            Instruction::Jr => write!(f, "jr imm16"),
            Instruction::Jrc { condition } => write!(f, "jr {condition} imm16"),
            Instruction::Jp => write!(f, "jp imm16"),
            Instruction::Jpc { condition } => write!(f, "jp {condition} imm16"),
            Instruction::JpReg16 { reg } => write!(f, "jp {reg}"),
            Instruction::Ret => write!(f, "ret"),
            Instruction::Retc { condition } => write!(f, "ret {condition}"),
            Instruction::Reti => write!(f, "reti"),
            Instruction::Call => write!(f, "call imm16"),
            Instruction::Callc { condition } => write!(f, "call {condition} imm16"),
            Instruction::Rst { target } => write!(f, "rst {target}"),
            Instruction::AddReg8 { src } => write!(f, "add A, {src}"),
            Instruction::AddMem16 { src } => write!(f, "add A, ({src})"),
            Instruction::AddImm8 => write!(f, "add A, imm8"),
            Instruction::AdcReg8 { src } => write!(f, "adc A, {src}"),
            Instruction::AdcMem16 { src } => write!(f, "adc A, ({src})"),
            Instruction::AdcImm8 => write!(f, "adc A, imm8"),
            Instruction::SubReg8 { src } => write!(f, "sub A, {src}"),
            Instruction::SubMem16 { src } => write!(f, "sub A, ({src})"),
            Instruction::SubImm8 => write!(f, "sub A, imm8"),
            Instruction::SbcReg8 { src } => write!(f, "sbc {src}"),
            Instruction::SbcMem16 { src } => write!(f, "sbc ({src})"),
            Instruction::SbcImm8 => write!(f, "sbc imm8"),
            Instruction::AndReg8 { src } => write!(f, "and A, {src}"),
            Instruction::AndMem16 { src } => write!(f, "and A, ({src})"),
            Instruction::AndImm8 => write!(f, "and A, imm8"),
            Instruction::XorReg8 { src } => write!(f, "xor A, {src}"),
            Instruction::XorMem16 { src } => write!(f, "xor A, ({src})"),
            Instruction::XorImm8 => write!(f, "xor A, imm8"),
            Instruction::OrReg8 { src } => write!(f, "or A, {src}"),
            Instruction::OrMem16 { src } => write!(f, "or A, ({src})"),
            Instruction::OrImm8 => write!(f, "or A, imm8"),
            Instruction::CpReg8 { src } => write!(f, "cp A, {src}"),
            Instruction::CpMem16 { src } => write!(f, "cp A, ({src})"),
            Instruction::CpImm8 => write!(f, "cp A, imm8"),
        }
    }
}

impl Instruction {
    pub fn format(
        &self,
        buffer: &mut String,
        stream: &mut impl Iterator<Item = u8>,
    ) -> Result<(), core::fmt::Error> {
        let mut bytes = Vec::new();
        let mut write_byte = |byte: Option<u8>| {
            if let Some(b) = byte {
                bytes.push(b);
                Ok(b)
            } else {
                Err(core::fmt::Error)
            }
        };

        macro_rules! next_u8 {
            () => {
                write_byte(stream.next())
            };
        }

        macro_rules! next_u16 {
            () => {{
                let lo = write_byte(stream.next())?;
                let hi = write_byte(stream.next())?;
                Ok(((hi as u16) << 8) | lo as u16)
            }};
        }

        match self {
            Instruction::LdReg8Imm8 { dst } => {
                let value = next_u8!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "ld {dst}, ")?;
                Self::format_u8(buffer, Some(value))?;
            }
            Instruction::LdMem16Imm8 { dst } => {
                let value = next_u8!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "ld ({dst}), ")?;
                Self::format_u8(buffer, Some(value))?;
            }
            Instruction::LdAdr8Reg8 { src } => {
                let addr = next_u8!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "ld (")?;
                Self::format_u8(buffer, Some(addr))?;
                write!(buffer, "), {src}")?;
            }
            Instruction::LdReg8Adr8 { dst } => {
                let addr = next_u8!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "ld {dst}, (")?;
                Self::format_u8(buffer, Some(addr))?;
                write!(buffer, ")")?;
            }
            Instruction::LdAdr16Reg8 { src } => {
                let value = next_u16!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "ld (0x{value:04X}), {src}")?;
            }
            Instruction::LdReg8Adr16 { dst } => {
                let value = next_u16!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "ld {dst}, (0x{value:04X})")?;
            }
            Instruction::LdReg16Imm16 { dst } => {
                let value = next_u16!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "ld {dst}, 0x{value:04X}")?;
            }
            Instruction::LdAdr16Reg16 { src } => {
                let value = next_u16!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "ld (0x{value:04X}), {src}")?;
            }
            Instruction::LdReg16Off8 { dst } => {
                let offset = next_u8!()? as i8;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "ld {dst}, {offset:+#04X}")?;
            }
            Instruction::AddReg16Off8 { dst } => {
                let offset = next_u8!()? as i8;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "add {dst}, {offset:+#04X}")?;
            }
            Instruction::Jr => {
                let offset = next_u8!()? as i8;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "jr {offset:+#04X}")?;
            }
            Instruction::Jrc { condition } => {
                let offset = next_u8!()? as i8;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "jr {condition}, {offset:+#04X}")?;
            }
            Instruction::Jp => {
                let addr = next_u16!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "jp 0x{addr:04X}")?;
            }
            Instruction::Jpc { condition } => {
                let addr = next_u16!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "jp {condition}, 0x{addr:04X}")?;
            }
            Instruction::JpReg16 { reg } => {
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "jp {reg}")?;
            }
            Instruction::Call => {
                let addr = next_u16!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "call 0x{addr:04X}")?;
            }
            Instruction::Callc { condition } => {
                let addr = next_u16!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "call {condition}, 0x{addr:04X}")?;
            }
            Instruction::AddImm8 => {
                let value = next_u8!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "add A, ")?;
                Self::format_u8(buffer, Some(value))?;
            }
            Instruction::AdcImm8 => {
                let value = next_u8!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "adc A, ")?;
                Self::format_u8(buffer, Some(value))?;
            }
            Instruction::SubImm8 => {
                let value = next_u8!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "sub A, ")?;
                Self::format_u8(buffer, Some(value))?;
            }
            Instruction::SbcImm8 => {
                let value = next_u8!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "sbc A, ")?;
                Self::format_u8(buffer, Some(value))?;
            }
            Instruction::AndImm8 => {
                let value = next_u8!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "and A, ")?;
                Self::format_u8(buffer, Some(value))?;
            }
            Instruction::XorImm8 => {
                let value = next_u8!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "xor A, ")?;
                Self::format_u8(buffer, Some(value))?;
            }
            Instruction::OrImm8 => {
                let value = next_u8!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "or A, ")?;
                Self::format_u8(buffer, Some(value))?;
            }
            Instruction::CpImm8 => {
                let value = next_u8!()?;
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "cp A, ")?;
                Self::format_u8(buffer, Some(value))?;
            }
            _ => {
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(buffer, "{self}")?;
            }
        }
        Ok(())
    }

    fn write_bytes_prefix(buffer: &mut String, bytes: &[u8]) -> Result<(), core::fmt::Error> {
        for b in bytes.iter() {
            write!(buffer, " ")?;
            write!(buffer, "{b:02X}")?;
        }
        let leftover = (2usize).saturating_sub(bytes.len());
        for _ in 0..leftover {
            write!(buffer, " __")?;
        }
        write!(buffer, " ")?;
        Ok(())
    }

    fn format_u8(buffer: &mut String, value: Option<u8>) -> Result<(), core::fmt::Error> {
        if let Some(byte) = value {
            write!(buffer, "{byte:#04X}")
        } else {
            write!(buffer, "0x??")
        }
    }
}

#[macro_use]
mod _opcode_macros {
    macro_rules! ld8 {
        (reg $dst:ident, reg $src:ident) => {
            Instruction::LdReg8Reg8 {
                dst: Reg8::$dst,
                src: Reg8::$src,
            }
        };
        (reg $dst:ident, addr8) => {
            Instruction::LdReg8Adr8 { dst: Reg8::$dst }
        };
        (reg $dst:ident, addr16) => {
            Instruction::LdReg8Adr16 { dst: Reg8::$dst }
        };
        (reg $dst:ident, mem8 $src:ident) => {
            Instruction::LdReg8Mem8 {
                dst: Reg8::$dst,
                src: Reg8::$src,
            }
        };
        (reg $dst:ident, mem16 $src:ident) => {
            Instruction::LdReg8Mem16 {
                dst: Reg8::$dst,
                src: Reg16::$src,
            }
        };
        (reg $dst:ident, imm) => {
            Instruction::LdReg8Imm8 { dst: Reg8::$dst }
        };
        (addr8, reg $src:ident) => {
            Instruction::LdAdr8Reg8 { src: Reg8::$src }
        };
        (addr16, reg $src:ident) => {
            Instruction::LdAdr16Reg8 { src: Reg8::$src }
        };
        (mem8 $dst:ident, reg $src:ident) => {
            Instruction::LdMem8Reg8 {
                dst: Reg8::$dst,
                src: Reg8::$src,
            }
        };
        (mem16 $dst:ident, reg $src:ident) => {
            Instruction::LdMem16Reg8 {
                dst: Reg16::$dst,
                src: Reg8::$src,
            }
        };
        (mem16 $dst:ident, imm) => {
            Instruction::LdMem16Imm8 { dst: Reg16::$dst }
        };
        (mem16 $dst:ident+, reg $src:ident) => {
            Instruction::LdIncMem16Reg8 {
                dst: Reg16::$dst,
                src: Reg8::$src,
            }
        };
        (mem16 $dst:ident-, reg $src:ident) => {
            Instruction::LdDecMem16Reg8 {
                dst: Reg16::$dst,
                src: Reg8::$src,
            }
        };
        (reg $dst:ident, mem16 $src:ident+) => {
            Instruction::LdIncReg8MemR16 {
                dst: Reg8::$dst,
                src: Reg16::$src,
            }
        };
        (reg $dst:ident, mem16 $src:ident-) => {
            Instruction::LdDecReg8MemR16 {
                dst: Reg8::$dst,
                src: Reg16::$src,
            }
        };
    }

    macro_rules! ld16 {
        (reg $dst:ident, imm) => {
            Instruction::LdReg16Imm16 { dst: Reg16::$dst }
        };
        (addr, reg $src:ident) => {
            Instruction::LdAdr16Reg16 { src: Reg16::$src }
        };
        (reg $dst:ident, reg $src:ident) => {
            Instruction::LdReg16Reg16 {
                dst: Reg16::$dst,
                src: Reg16::$src,
            }
        };
        (reg $dst:ident, sp+-) => {
            Instruction::LdReg16Off8 { dst: Reg16::$dst }
        };
    }

    macro_rules! inc8 {
        (reg $reg:ident) => {
            Instruction::IncReg8 { reg: Reg8::$reg }
        };
        (mem $reg:ident) => {
            Instruction::IncMem16 { reg: Reg16::$reg }
        };
    }

    macro_rules! dec8 {
        (reg $reg:ident) => {
            Instruction::DecReg8 { reg: Reg8::$reg }
        };
        (mem $reg:ident) => {
            Instruction::DecMem16 { reg: Reg16::$reg }
        };
    }

    macro_rules! inc16 {
        (reg $reg:ident) => {
            Instruction::IncReg16 { reg: Reg16::$reg }
        };
    }

    macro_rules! dec16 {
        (reg $reg:ident) => {
            Instruction::DecReg16 { reg: Reg16::$reg }
        };
    }

    macro_rules! add8 {
        (reg $src:ident) => {
            Instruction::AddReg8 { src: Reg8::$src }
        };
        (mem $src:ident) => {
            Instruction::AddMem16 { src: Reg16::$src }
        };
        (imm) => {
            Instruction::AddImm8
        };
    }

    macro_rules! add16 {
        (reg $dst:ident, reg $src:ident) => {
            Instruction::AddReg16Reg16 {
                dst: Reg16::$dst,
                src: Reg16::$src,
            }
        };
        (reg $dst:ident, imm+-) => {
            Instruction::AddReg16Off8 { dst: Reg16::$dst }
        };
    }

    macro_rules! adc8 {
        (reg $src:ident) => {
            Instruction::AdcReg8 { src: Reg8::$src }
        };
        (mem $src:ident) => {
            Instruction::AdcMem16 { src: Reg16::$src }
        };
        (imm) => {
            Instruction::AdcImm8
        };
    }

    macro_rules! sub8 {
        (reg $src:ident) => {
            Instruction::SubReg8 { src: Reg8::$src }
        };
        (mem $src:ident) => {
            Instruction::SubMem16 { src: Reg16::$src }
        };
        (imm) => {
            Instruction::SubImm8
        };
    }

    macro_rules! sbc8 {
        (reg $src:ident) => {
            Instruction::SbcReg8 { src: Reg8::$src }
        };
        (mem $src:ident) => {
            Instruction::SbcMem16 { src: Reg16::$src }
        };
        (imm) => {
            Instruction::SbcImm8
        };
    }

    macro_rules! and8 {
        (reg $src:ident) => {
            Instruction::AndReg8 { src: Reg8::$src }
        };
        (mem $src:ident) => {
            Instruction::AndMem16 { src: Reg16::$src }
        };
        (imm) => {
            Instruction::AndImm8
        };
    }

    macro_rules! xor8 {
        (reg $src:ident) => {
            Instruction::XorReg8 { src: Reg8::$src }
        };
        (mem $src:ident) => {
            Instruction::XorMem16 { src: Reg16::$src }
        };
        (imm) => {
            Instruction::XorImm8
        };
    }

    macro_rules! or8 {
        (reg $src:ident) => {
            Instruction::OrReg8 { src: Reg8::$src }
        };
        (mem $src:ident) => {
            Instruction::OrMem16 { src: Reg16::$src }
        };
        (imm) => {
            Instruction::OrImm8
        };
    }

    macro_rules! cp8 {
        (reg $src:ident) => {
            Instruction::CpReg8 { src: Reg8::$src }
        };
        (mem $src:ident) => {
            Instruction::CpMem16 { src: Reg16::$src }
        };
        (imm) => {
            Instruction::CpImm8
        };
    }

    macro_rules! push {
        (reg $src:ident) => {
            Instruction::PushReg16 { src: Reg16::$src }
        };
    }

    macro_rules! pop {
        (reg $dst:ident) => {
            Instruction::PopReg16 { dst: Reg16::$dst }
        };
    }

    macro_rules! jump {
        (rel) => {
            Instruction::Jr
        };
        (rel $cond:ident) => {
            Instruction::Jrc {
                condition: Condition::$cond,
            }
        };
        (adr) => {
            Instruction::Jp
        };
        (adr $cond:ident) => {
            Instruction::Jpc {
                condition: Condition::$cond,
            }
        };
        (reg $reg:ident) => {
            Instruction::JpReg16 { reg: Reg16::$reg }
        };
    }

    macro_rules! ret {
        () => {
            Instruction::Ret
        };
        (enable) => {
            Instruction::Reti
        };
        (cond $cond:ident) => {
            Instruction::Retc {
                condition: Condition::$cond,
            }
        };
    }

    macro_rules! call {
        () => {
            Instruction::Call
        };
        (cond $cond:ident) => {
            Instruction::Callc {
                condition: Condition::$cond,
            }
        };
    }

    macro_rules! rst {
        (adr $adr:ident) => {
            Instruction::Rst {
                target: RstAddress::$adr,
            }
        };
    }
}

impl Instruction {
    pub const fn decode(value: u8) -> Instruction {
        match value {
            // loads
            // 0xX1: 16 bit loads
            0x01 => ld16!(reg BC, imm),
            0x11 => ld16!(reg DE, imm),
            0x21 => ld16!(reg HL, imm),
            0x31 => ld16!(reg SP, imm),
            // 0x08: 16 bit load into immediate address
            0x08 => ld16!(addr, reg SP),
            // 0xX2: 8 bit loads into mem
            0x02 => ld8!(mem16 BC, reg A),
            0x12 => ld8!(mem16 DE, reg A),
            0x22 => ld8!(mem16 HL+, reg A),
            0x32 => ld8!(mem16 HL-, reg A),
            // 0xX6: 8 bit immediate loads
            0x06 => ld8!(reg B, imm),
            0x16 => ld8!(reg D, imm),
            0x26 => ld8!(reg H, imm),
            0x36 => ld8!(mem16 HL, imm),
            // 0xXE: 8 bit immediate loads
            0x0E => ld8!(reg C, imm),
            0x1E => ld8!(reg E, imm),
            0x2E => ld8!(reg L, imm),
            0x3E => ld8!(reg A, imm),
            // 0xXA: 8-bit loads from mem
            0x0A => ld8!(reg A, mem16 BC),
            0x1A => ld8!(reg A, mem16 DE),
            0x2A => ld8!(reg A, mem16 HL+),
            0x3A => ld8!(reg A, mem16 HL-),
            // 0x4X: loads into reg B and C
            0x40 => ld8!(reg B, reg B),
            0x41 => ld8!(reg B, reg C),
            0x42 => ld8!(reg B, reg D),
            0x43 => ld8!(reg B, reg E),
            0x44 => ld8!(reg B, reg H),
            0x45 => ld8!(reg B, reg L),
            0x46 => ld8!(reg B, mem16 HL),
            0x47 => ld8!(reg B, reg A),
            0x48 => ld8!(reg C, reg B),
            0x49 => ld8!(reg C, reg C),
            0x4A => ld8!(reg C, reg D),
            0x4B => ld8!(reg C, reg E),
            0x4C => ld8!(reg C, reg H),
            0x4D => ld8!(reg C, reg L),
            0x4E => ld8!(reg C, mem16 HL),
            0x4F => ld8!(reg C, reg A),
            // 0x5X: loads into reg D and E
            0x50 => ld8!(reg D, reg B),
            0x51 => ld8!(reg D, reg C),
            0x52 => ld8!(reg D, reg D),
            0x53 => ld8!(reg D, reg E),
            0x54 => ld8!(reg D, reg H),
            0x55 => ld8!(reg D, reg L),
            0x56 => ld8!(reg D, mem16 HL),
            0x57 => ld8!(reg D, reg A),
            0x58 => ld8!(reg E, reg B),
            0x59 => ld8!(reg E, reg C),
            0x5A => ld8!(reg E, reg D),
            0x5B => ld8!(reg E, reg E),
            0x5C => ld8!(reg E, reg H),
            0x5D => ld8!(reg E, reg L),
            0x5E => ld8!(reg E, mem16 HL),
            0x5F => ld8!(reg E, reg A),
            // 0x6X: loads into reg H and L
            0x60 => ld8!(reg H, reg B),
            0x61 => ld8!(reg H, reg C),
            0x62 => ld8!(reg H, reg D),
            0x63 => ld8!(reg H, reg E),
            0x64 => ld8!(reg H, reg H),
            0x65 => ld8!(reg H, reg L),
            0x66 => ld8!(reg H, mem16 HL),
            0x67 => ld8!(reg H, reg A),
            0x68 => ld8!(reg L, reg B),
            0x69 => ld8!(reg L, reg C),
            0x6A => ld8!(reg L, reg D),
            0x6B => ld8!(reg L, reg E),
            0x6C => ld8!(reg L, reg H),
            0x6D => ld8!(reg L, reg L),
            0x6E => ld8!(reg L, mem16 HL),
            0x6F => ld8!(reg L, reg A),
            // 0x7X: loads into mem16 (HL) and reg A
            0x70 => ld8!(mem16 HL, reg B),
            0x71 => ld8!(mem16 HL, reg C),
            0x72 => ld8!(mem16 HL, reg D),
            0x73 => ld8!(mem16 HL, reg E),
            0x74 => ld8!(mem16 HL, reg H),
            0x75 => ld8!(mem16 HL, reg L),
            0x77 => ld8!(mem16 HL, reg A),
            0x78 => ld8!(reg A, reg B),
            0x79 => ld8!(reg A, reg C),
            0x7A => ld8!(reg A, reg D),
            0x7B => ld8!(reg A, reg E),
            0x7C => ld8!(reg A, reg H),
            0x7D => ld8!(reg A, reg L),
            0x7E => ld8!(reg A, mem16 HL),
            0x7F => ld8!(reg A, reg A),
            // 0xE0: 8-bit load into immediate 8-bit address from register
            0xE0 => ld8!(addr8, reg A),
            // 0xF0: 8-bit load into immediate 8-bit address from register
            0xF0 => ld8!(reg A, addr8),
            // 0xE2: 8-bit load into 8-bit register address from register
            0xE2 => ld8!(mem8 C, reg A),
            // 0xF2: 8-bit load into 8-bit register address from register
            0xF2 => ld8!(reg A, mem8 C),
            // 0xEA: 8-bit load into immediate 16-bit address from register
            0xEA => ld8!(addr16, reg A),
            // 0xFA: 8-bit load from immediate 16-bit address into register
            0xFA => ld8!(reg A, addr16),
            // 0xF8: 16-bit register load from sp with signed offset
            0xF8 => ld16!(reg SP, sp+-),
            // 0xF9: 16-bit register load
            0xF9 => ld16!(reg SP, reg HL),
            // increments and decrements
            // 0xX3: 16-bit increments
            0x03 => inc16!(reg BC),
            0x13 => inc16!(reg DE),
            0x23 => inc16!(reg HL),
            0x33 => inc16!(reg SP),
            // 0xXB: 16-bit decrements
            0x0B => dec16!(reg BC),
            0x1B => dec16!(reg DE),
            0x2B => dec16!(reg HL),
            0x3B => dec16!(reg SP),
            // 0xX4: 8-bit increments
            0x04 => inc8!(reg B),
            0x14 => inc8!(reg D),
            0x24 => inc8!(reg H),
            0x34 => inc8!(mem HL),
            // 0xXC: 8-bit increments
            0x0C => inc8!(reg C),
            0x1C => inc8!(reg E),
            0x2C => inc8!(reg L),
            0x3C => inc8!(reg A),
            // 0xX5: 8-bit decrements
            0x05 => dec8!(reg B),
            0x15 => dec8!(reg D),
            0x25 => dec8!(reg H),
            0x35 => dec8!(mem HL),
            // 0xXD: 8-bit decrements
            0x0D => dec8!(reg C),
            0x1D => dec8!(reg E),
            0x2D => dec8!(reg L),
            0x3D => dec8!(reg A),
            // 16-bit adds
            0x09 => add16!(reg HL, reg BC),
            0x19 => add16!(reg HL, reg DE),
            0x29 => add16!(reg HL, reg HL),
            0x39 => add16!(reg HL, reg SP),
            0xE8 => add16!(reg SP, imm+-),
            // 0xX1: pop
            0xC1 => pop!(reg BC),
            0xD1 => pop!(reg DE),
            0xE1 => pop!(reg HL),
            0xF1 => pop!(reg AF),
            // 0xX5: push
            0xC5 => push!(reg BC),
            0xD5 => push!(reg DE),
            0xE5 => push!(reg HL),
            0xF5 => push!(reg AF),
            // relative jumps
            0x20 => jump!(rel NZ),
            0x30 => jump!(rel NC),
            0x28 => jump!(rel Z),
            0x38 => jump!(rel C),
            0x18 => jump!(rel),
            // absolute jumps
            0xC2 => jump!(adr NZ),
            0xD2 => jump!(adr NC),
            0xCA => jump!(adr Z),
            0xDA => jump!(adr C),
            0xE9 => jump!(reg HL),
            0xC3 => jump!(adr),
            // returns
            0xC0 => ret!(cond NZ),
            0xD0 => ret!(cond NC),
            0xC8 => ret!(cond Z),
            0xD8 => ret!(cond C),
            0xC9 => ret!(),
            0xD9 => ret!(enable),
            // calls
            0xC4 => call!(cond NZ),
            0xD4 => call!(cond NC),
            0xCC => call!(cond Z),
            0xDC => call!(cond C),
            0xCD => call!(),
            // 0xX7: rsts
            0xC7 => rst!(adr RST00),
            0xD7 => rst!(adr RST10),
            0xE7 => rst!(adr RST20),
            0xF7 => rst!(adr RST30),
            // 0xXF: rsts
            0xCF => rst!(adr RST08),
            0xDF => rst!(adr RST18),
            0xEF => rst!(adr RST28),
            0xFF => rst!(adr RST38),
            // adds
            0x80 => add8!(reg B),
            0x81 => add8!(reg C),
            0x82 => add8!(reg D),
            0x83 => add8!(reg E),
            0x84 => add8!(reg H),
            0x85 => add8!(reg L),
            0x86 => add8!(mem HL),
            0x87 => add8!(reg A),
            0xC6 => add8!(imm),
            // adcs
            0x88 => adc8!(reg B),
            0x89 => adc8!(reg C),
            0x8A => adc8!(reg D),
            0x8B => adc8!(reg E),
            0x8C => adc8!(reg H),
            0x8D => adc8!(reg L),
            0x8E => adc8!(mem HL),
            0x8F => adc8!(reg A),
            0xCE => adc8!(imm),
            // subtractions
            0x90 => sub8!(reg B),
            0x91 => sub8!(reg C),
            0x92 => sub8!(reg D),
            0x93 => sub8!(reg E),
            0x94 => sub8!(reg H),
            0x95 => sub8!(reg L),
            0x96 => sub8!(mem HL),
            0x97 => sub8!(reg A),
            0xD6 => sub8!(imm),
            // subtractions with carry
            0x98 => sbc8!(reg B),
            0x99 => sbc8!(reg C),
            0x9A => sbc8!(reg D),
            0x9B => sbc8!(reg E),
            0x9C => sbc8!(reg H),
            0x9D => sbc8!(reg L),
            0x9E => sbc8!(mem HL),
            0x9F => sbc8!(reg A),
            0xDE => sbc8!(imm),
            // bitwise and
            0xA0 => and8!(reg B),
            0xA1 => and8!(reg C),
            0xA2 => and8!(reg D),
            0xA3 => and8!(reg E),
            0xA4 => and8!(reg H),
            0xA5 => and8!(reg L),
            0xA6 => and8!(mem HL),
            0xA7 => and8!(reg A),
            0xE6 => and8!(imm),
            // bitwise xor
            0xA8 => xor8!(reg B),
            0xA9 => xor8!(reg C),
            0xAA => xor8!(reg D),
            0xAB => xor8!(reg E),
            0xAC => xor8!(reg H),
            0xAD => xor8!(reg L),
            0xAE => xor8!(mem HL),
            0xAF => xor8!(reg A),
            0xEE => xor8!(imm),
            // bitwise or
            0xB0 => or8!(reg B),
            0xB1 => or8!(reg C),
            0xB2 => or8!(reg D),
            0xB3 => or8!(reg E),
            0xB4 => or8!(reg H),
            0xB5 => or8!(reg L),
            0xB6 => or8!(mem HL),
            0xB7 => or8!(reg A),
            0xF6 => or8!(imm),
            // comparisons
            0xB8 => cp8!(reg B),
            0xB9 => cp8!(reg C),
            0xBA => cp8!(reg D),
            0xBB => cp8!(reg E),
            0xBC => cp8!(reg H),
            0xBD => cp8!(reg L),
            0xBE => cp8!(mem HL),
            0xBF => cp8!(reg A),
            0xFE => cp8!(imm),
            // rotate accumulator
            0x07 => Instruction::Rlca,
            0x17 => Instruction::Rla,
            0x0F => Instruction::Rrca,
            0x1F => Instruction::Rra,
            // misc
            0x00 => Instruction::Nop,
            0x10 => Instruction::Stop,
            0xCB => Instruction::Prefix,
            0x76 => Instruction::Halt,
            0xF3 => Instruction::Di,
            0xFB => Instruction::Ei,
            0x27 => Instruction::Daa,
            0x2F => Instruction::Cpl,
            0x37 => Instruction::Scf,
            0x3F => Instruction::Ccf,
            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
                Instruction::Invalid
            }
        }
    }
}

pub fn disassemble(byte_stream: &[u8]) -> String {
    use core::fmt::Write;
    let mut buffer = String::new();

    let mut it = byte_stream.iter().enumerate();

    loop {
        let Some((address, byte)) = it.next() else {
            break;
        };
        assert!(address < u16::MAX as usize + 1, "{address}");

        let inst = Instruction::decode(*byte);

        write!(buffer, "{address:#06X}: {byte:02X}").unwrap();
        inst.format(&mut buffer, &mut it.clone().map(|(_, v)| *v))
            .unwrap();
        buffer.push('\n');
    }

    buffer
}
