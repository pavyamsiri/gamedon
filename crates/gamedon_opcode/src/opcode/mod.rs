use crate::register::{Reg8, Reg16};
use core::fmt::Write;
use gamedon_bits::BitShift8;
use owo_colors::colors::*;
use owo_colors::{OwoColorize, Stream, Style};

const INSTRUCTION_STYLE: Style = Style::new().yellow();
const ERROR_STYLE: Style = Style::new().red();
const IMMEDIATE_STYLE: Style = Style::new().magenta();
const ADDRESS_STYLE: Style = Style::new().bright_magenta();
const CONDITION_STYLE: Style = Style::new().green();

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
            Condition::Z => write!(
                f,
                "{}",
                "z".if_supports_color(Stream::Stdout, |text| text.style(CONDITION_STYLE))
            ),
            Condition::NZ => write!(
                f,
                "{}",
                "nz".if_supports_color(Stream::Stdout, |text| text.style(CONDITION_STYLE))
            ),
            Condition::C => write!(
                f,
                "{}",
                "c".if_supports_color(Stream::Stdout, |text| text.style(CONDITION_STYLE))
            ),
            Condition::NC => write!(
                f,
                "{}",
                "nc".if_supports_color(Stream::Stdout, |text| text.style(CONDITION_STYLE))
            ),
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

impl RstAddress {
    #[inline]
    pub const fn to_address(self) -> u16 {
        match self {
            RstAddress::RST00 => 0x0000,
            RstAddress::RST08 => 0x0008,
            RstAddress::RST10 => 0x0010,
            RstAddress::RST18 => 0x0018,
            RstAddress::RST20 => 0x0020,
            RstAddress::RST28 => 0x0028,
            RstAddress::RST30 => 0x0030,
            RstAddress::RST38 => 0x0038,
        }
    }
}

impl core::fmt::Display for RstAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RstAddress::RST00 => write!(
                f,
                "{}",
                "$0000".if_supports_color(Stream::Stdout, |text| text.style(ADDRESS_STYLE))
            ),
            RstAddress::RST08 => write!(
                f,
                "{}",
                "$0008".if_supports_color(Stream::Stdout, |text| text.style(ADDRESS_STYLE))
            ),
            RstAddress::RST10 => write!(
                f,
                "{}",
                "$0010".if_supports_color(Stream::Stdout, |text| text.style(ADDRESS_STYLE))
            ),
            RstAddress::RST18 => write!(
                f,
                "{}",
                "$0018".if_supports_color(Stream::Stdout, |text| text.style(ADDRESS_STYLE))
            ),
            RstAddress::RST20 => write!(
                f,
                "{}",
                "$0020".if_supports_color(Stream::Stdout, |text| text.style(ADDRESS_STYLE))
            ),
            RstAddress::RST28 => write!(
                f,
                "{}",
                "$0028".if_supports_color(Stream::Stdout, |text| text.style(ADDRESS_STYLE))
            ),
            RstAddress::RST30 => write!(
                f,
                "{}",
                "$0030".if_supports_color(Stream::Stdout, |text| text.style(ADDRESS_STYLE))
            ),
            RstAddress::RST38 => write!(
                f,
                "{}",
                "$0038".if_supports_color(Stream::Stdout, |text| text.style(ADDRESS_STYLE))
            ),
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
    LdIncReg8Mem16 { dst: Reg8, src: Reg16 },
    /// Load byte from address given by 16-bit register into 8-bit register,
    /// then decrement the 16-bit register.
    LdDecReg8Mem16 { dst: Reg8, src: Reg16 },

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

    // Prefix instructions
    /// Rotate an 8-bit register left. Old bit 7 is stored in the carry flag and the new bit 0.
    RlcReg8 { reg: Reg8 },
    /// Rotate a byte at the address given by a 16-bit register left.
    /// Old bit 7 is stored in the carry flag and the new bit 0.
    RlcMem16 { reg: Reg16 },
    /// Rotate an 8-bit register right. Old bit 0 is stored in the carry flag and the new bit 7.
    RrcReg8 { reg: Reg8 },
    /// Rotate a byte at the address given by a 16-bit register right.
    /// Old bit 0 is stored in the carry flag and the new bit 7.
    RrcMem16 { reg: Reg16 },

    /// Rotate an 8-bit register left. Old bit 7 is stored in the carry flag with the new bit 0 set to the previous carry flag value.
    RlReg8 { reg: Reg8 },
    /// Rotate a byte at the address given by a 16-bit register left.
    /// Old bit 7 is stored in the carry flag with the new bit 0 set to the previous carry flag value.
    RlMem16 { reg: Reg16 },
    /// Rotate an 8-bit register right.
    /// Old bit 0 is stored in the carry flag with the new bit 7 set to the previous carry flag value.
    RrReg8 { reg: Reg8 },
    /// Rotate a byte at the address given by a 16-bit register right.
    /// Old bit 0 is stored in the carry flag with the new bit 7 set to the previous carry flag value.
    RrMem16 { reg: Reg16 },

    /// Shift an 8-bit register left. Old bit 7 is stored in the carry flag with bit 0 reset.
    SlaReg8 { reg: Reg8 },
    /// Shift a byte at the address given by a 16-bit register left.
    /// Old bit 7 is stored in the carry flag with bit 0 reset.
    SlaMem16 { reg: Reg16 },
    /// Shift an 8-bit register right. Old bit 0 is stored in the carry flag with bit 7 unchanged.
    SraReg8 { reg: Reg8 },
    /// Shift a byte at the address given by a 16-bit register right.
    /// Old bit 0 is stored in the carry flag with bit 7 unchanged.
    SraMem16 { reg: Reg16 },

    /// Swap the upper four bits and the lower four bits of an 8-bit register.
    SwapReg8 { reg: Reg8 },
    /// Swap the upper four bits and the lower four bits of a byte at the address given by a 16-bit register.
    SwapMem16 { reg: Reg16 },

    /// Shift an 8-bit register right. Old bit 0 is stored in the carry flag with bit 7 reset.
    SrlReg8 { reg: Reg8 },
    /// Shift a byte at the address given by a 16-bit register right.
    /// Old bit 0 is stored in the carry flag with bit 7 reset.
    SrlMem16 { reg: Reg16 },

    // Bit operations
    /// Test if bit n is set in an 8-bit register.
    BitReg8 { reg: Reg8, bit: BitShift8 },
    /// Test if bit n is set in the byte at the address given by a 16-bit register.
    BitMem16 { reg: Reg16, bit: BitShift8 },
    /// Reset bit n is set in an 8-bit register.
    ResReg8 { reg: Reg8, bit: BitShift8 },
    /// Reset bit n is set in the byte at the address given by a 16-bit register.
    ResMem16 { reg: Reg16, bit: BitShift8 },
    /// Set bit n is set in an 8-bit register.
    SetReg8 { reg: Reg8, bit: BitShift8 },
    /// Set bit n is set in the byte at the address given by a 16-bit register.
    SetMem16 { reg: Reg16, bit: BitShift8 },
}

impl core::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! write_reg_op {
            ($name:literal, $reg:expr) => {
                write!(
                    f,
                    "{} {}",
                    $name.if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE),),
                    $reg
                )
            };
            ($name:literal, $reg:expr, $bit:expr) => {
                write!(
                    f,
                    "{} {} {}",
                    $name.if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE),),
                    $bit,
                    $reg
                )
            };
        }

        let acc = Reg8::A;
        match self {
            Instruction::Nop => write!(
                f,
                "{}",
                "nop".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Ret => write!(
                f,
                "{}",
                "ret".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Reti => write!(
                f,
                "{}",
                "reti".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Halt => write!(
                f,
                "{}",
                "halt".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Stop => write!(
                f,
                "{}",
                "stop".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Ei => write!(
                f,
                "{}",
                "ei".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Di => write!(
                f,
                "{}",
                "di".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Daa => write!(
                f,
                "{}",
                "daa".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Scf => write!(
                f,
                "{}",
                "scf".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Ccf => write!(
                f,
                "{}",
                "ccf".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Cpl => write!(
                f,
                "{}",
                "cpl".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Rlca => write!(
                f,
                "{}",
                "rlca".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Rla => write!(
                f,
                "{}",
                "rla".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Rrca => write!(
                f,
                "{}",
                "rrca".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Rra => write!(
                f,
                "{}",
                "rra".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Invalid => write!(
                f,
                "{}",
                "invalid".if_supports_color(Stream::Stdout, |text| text.style(ERROR_STYLE))
            ),
            Instruction::Prefix => write!(
                f,
                "{}",
                "prefix".if_supports_color(Stream::Stdout, |text| text.style(ERROR_STYLE))
            ),
            Instruction::LdReg8Reg8 { dst, src } => write!(
                f,
                "{} {dst}, {src}",
                "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdReg8Mem16 { dst, src } => write!(
                f,
                "{} {dst}, ({src})",
                "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdReg8Imm8 { dst } => write!(
                f,
                "{} {dst}, imm8",
                "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdMem16Reg8 { dst, src } => write!(
                f,
                "{} ({dst}), {src}",
                "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdMem16Imm8 { dst } => write!(
                f,
                "{} ({dst}), imm8",
                "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdAdr8Reg8 { src } => write!(
                f,
                "{} (imm8), {src}",
                "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdReg8Adr8 { dst } => write!(
                f,
                "{} {dst}, (imm8)",
                "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdMem8Reg8 { dst, src } => write!(
                f,
                "{} ({dst}), {src}",
                "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdReg8Mem8 { dst, src } => write!(
                f,
                "{} {dst}, ({src})",
                "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdAdr16Reg8 { src } => write!(
                f,
                "{} (imm16), {src}",
                "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdReg8Adr16 { dst } => write!(
                f,
                "{} {dst}, (imm16)",
                "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdIncMem16Reg8 { dst, src } => write!(
                f,
                "{} ({dst}) {src}",
                "ldi".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdDecMem16Reg8 { dst, src } => write!(
                f,
                "{} ({dst}) {src}",
                "ldd".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdIncReg8Mem16 { dst, src } => write!(
                f,
                "{} {dst} ({src})",
                "ldi".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdDecReg8Mem16 { dst, src } => write!(
                f,
                "{} {dst} ({src})",
                "ldd".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdReg16Imm16 { dst } => write!(
                f,
                "{} {dst}, imm16",
                "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdAdr16Reg16 { src } => write!(
                f,
                "{} (imm16), {src}",
                "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdReg16Reg16 { dst, src } => write!(
                f,
                "{} {dst}, {src}",
                "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::LdReg16Off8 { dst } => write!(
                f,
                "{} {dst}, s8",
                "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::IncReg8 { reg } => write!(
                f,
                "{} {reg}",
                "inc".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::DecReg8 { reg } => write!(
                f,
                "{} {reg}",
                "dec".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::IncMem16 { reg } => write!(
                f,
                "{} ({reg})",
                "inc".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::DecMem16 { reg } => write!(
                f,
                "{} ({reg})",
                "dec".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::IncReg16 { reg } => write!(
                f,
                "{} {reg}",
                "inc".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::DecReg16 { reg } => write!(
                f,
                "{} {reg}",
                "dec".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::AddReg16Reg16 { dst, src } => write!(
                f,
                "{} {dst} {src}",
                "add".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::AddReg16Off8 { dst } => write!(
                f,
                "{} {dst} e8",
                "add".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::PushReg16 { src } => write!(
                f,
                "{} {src}",
                "push".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::PopReg16 { dst } => write!(
                f,
                "{} {dst}",
                "pop".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Jr => write!(
                f,
                "{} imm16",
                "jr".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Jrc { condition } => write!(
                f,
                "{} {condition} imm16",
                "jr".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Jp => write!(
                f,
                "{} imm16",
                "jp".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Jpc { condition } => write!(
                f,
                "{} {condition} imm16",
                "jp".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::JpReg16 { reg } => write!(
                f,
                "{} {reg}",
                "jp".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Retc { condition } => write!(
                f,
                "{} {condition}",
                "ret".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Call => write!(
                f,
                "{} imm16",
                "call".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Callc { condition } => write!(
                f,
                "{} {condition} imm16",
                "call".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::Rst { target } => write!(
                f,
                "{} {target}",
                "rst".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::AddReg8 { src } => write!(
                f,
                "{} {acc}, {src}",
                "add".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::AddMem16 { src } => write!(
                f,
                "{} {acc}, ({src})",
                "add".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::AddImm8 => write!(
                f,
                "{} {acc}, imm8",
                "add".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::AdcReg8 { src } => write!(
                f,
                "{} {acc}, {src}",
                "adc".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::AdcMem16 { src } => write!(
                f,
                "{} {acc}, ({src})",
                "adc".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::AdcImm8 => write!(
                f,
                "{} {acc}, imm8",
                "adc".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::SubReg8 { src } => write!(
                f,
                "{} {acc}, {src}",
                "sub".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::SubMem16 { src } => write!(
                f,
                "{} {acc}, ({src})",
                "sub".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::SubImm8 => write!(
                f,
                "{} {acc}, imm8",
                "sub".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::SbcReg8 { src } => write!(
                f,
                "{} {src}",
                "sbc".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::SbcMem16 { src } => write!(
                f,
                "{} ({src})",
                "sbc".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::SbcImm8 => write!(
                f,
                "{} imm8",
                "sbc".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::AndReg8 { src } => write!(
                f,
                "{} {acc}, {src}",
                "and".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::AndMem16 { src } => write!(
                f,
                "{} {acc}, ({src})",
                "and".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::AndImm8 => write!(
                f,
                "{} {acc}, imm8",
                "and".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::XorReg8 { src } => write!(
                f,
                "{} {acc}, {src}",
                "xor".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::XorMem16 { src } => write!(
                f,
                "{} {acc}, ({src})",
                "xor".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::XorImm8 => write!(
                f,
                "{} {acc}, imm8",
                "xor".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::OrReg8 { src } => write!(
                f,
                "{} {acc}, {src}",
                "or".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::OrMem16 { src } => write!(
                f,
                "{} {acc}, ({src})",
                "or".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::OrImm8 => write!(
                f,
                "{} {acc}, imm8",
                "or".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::CpReg8 { src } => write!(
                f,
                "{} {acc}, {src}",
                "cp".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::CpMem16 { src } => write!(
                f,
                "{} {acc}, ({src})",
                "cp".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::CpImm8 => write!(
                f,
                "{} {acc}, imm8",
                "cp".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
            ),
            Instruction::RlcReg8 { reg } => write_reg_op!("rlc", reg),
            Instruction::RlcMem16 { reg } => write_reg_op!("rlc", reg),
            Instruction::RrcReg8 { reg } => write_reg_op!("rrc", reg),
            Instruction::RrcMem16 { reg } => write_reg_op!("rrc", reg),
            Instruction::RlReg8 { reg } => write_reg_op!("rl", reg),
            Instruction::RlMem16 { reg } => write_reg_op!("rl", reg),
            Instruction::RrReg8 { reg } => write_reg_op!("rr", reg),
            Instruction::RrMem16 { reg } => write_reg_op!("rr", reg),
            Instruction::SlaReg8 { reg } => write_reg_op!("sla", reg),
            Instruction::SlaMem16 { reg } => write_reg_op!("sla", reg),
            Instruction::SraReg8 { reg } => write_reg_op!("sra", reg),
            Instruction::SraMem16 { reg } => write_reg_op!("sra", reg),
            Instruction::SwapReg8 { reg } => write_reg_op!("swap", reg),
            Instruction::SwapMem16 { reg } => write_reg_op!("swap", reg),
            Instruction::SrlReg8 { reg } => write_reg_op!("srl", reg),
            Instruction::SrlMem16 { reg } => write_reg_op!("srl", reg),
            Instruction::BitReg8 { reg, bit } => write_reg_op!("bit", reg, bit),
            Instruction::BitMem16 { reg, bit } => write_reg_op!("bit", reg, bit),
            Instruction::ResReg8 { reg, bit } => write_reg_op!("res", reg, bit),
            Instruction::ResMem16 { reg, bit } => write_reg_op!("res", reg, bit),
            Instruction::SetReg8 { reg, bit } => write_reg_op!("set", reg, bit),
            Instruction::SetMem16 { reg, bit } => write_reg_op!("set", reg, bit),
        }
    }
}

impl Instruction {
    pub fn format(
        &self,
        buffer: &mut String,
        stream: &mut impl Iterator<Item = (usize, u8)>,
        byte: u8,
    ) -> Result<(), core::fmt::Error> {
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
            Instruction::LdReg8Imm8 { dst } => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {dst}, ",
                    "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u8(buffer, value)?;
            }
            Instruction::LdMem16Imm8 { dst } => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} ({dst}), ",
                    "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u8(buffer, value)?;
            }
            Instruction::LdAdr8Reg8 { src } => {
                let addr = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} (",
                    "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u8(buffer, addr)?;
                write!(buffer, "), {src}")?;
            }
            Instruction::LdReg8Adr8 { dst } => {
                let addr = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {dst}, (",
                    "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u8(buffer, addr)?;
                write!(buffer, ")")?;
            }
            Instruction::LdAdr16Reg8 { src } => {
                let lo = next_u8!();
                let hi = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} ",
                    "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u16_address(buffer, lo, hi)?;
                write!(buffer, ", {src}")?;
            }
            Instruction::LdReg8Adr16 { dst } => {
                let lo = next_u8!();
                let hi = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {dst}, (",
                    "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u16_address(buffer, lo, hi)?;
                write!(buffer, ")")?;
            }
            Instruction::LdReg16Imm16 { dst } => {
                let lo = next_u8!();
                let hi = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {dst}, ",
                    "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u16(buffer, lo, hi)?;
            }
            Instruction::LdAdr16Reg16 { src } => {
                let lo = next_u8!();
                let hi = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} (",
                    "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u16_address(buffer, lo, hi)?;
                write!(buffer, "), {src}")?;
            }
            Instruction::LdReg16Off8 { dst } => {
                let offset = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {dst}, ",
                    "ld".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_i8(buffer, offset)?;
            }
            Instruction::AddReg16Off8 { dst } => {
                let offset = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {dst}, ",
                    "add".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_i8(buffer, offset)?;
            }
            Instruction::Jr => {
                let offset = next_u8_and_address!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} ",
                    "jr".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_i8_address(buffer, offset)?;
            }
            Instruction::Jrc { condition } => {
                let offset = next_u8_and_address!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {condition}, ",
                    "jr".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_i8_address(buffer, offset)?;
            }
            Instruction::Jp => {
                let lo = next_u8!();
                let hi = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} ",
                    "jp".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u16_address(buffer, lo, hi)?;
            }
            Instruction::Jpc { condition } => {
                let lo = next_u8!();
                let hi = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {condition}, ",
                    "jp".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u16_address(buffer, lo, hi)?;
            }
            Instruction::JpReg16 { reg } => {
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {reg}",
                    "jp".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
            }
            Instruction::Call => {
                let lo = next_u8!();
                let hi = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} ",
                    "call".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u16_address(buffer, lo, hi)?;
            }
            Instruction::Callc { condition } => {
                let lo = next_u8!();
                let hi = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {condition:02}, ",
                    "call".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u16_address(buffer, lo, hi)?;
            }
            Instruction::AddImm8 => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {acc}, ",
                    "add".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u8(buffer, value)?;
            }
            Instruction::AdcImm8 => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {acc}, ",
                    "adc".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u8(buffer, value)?;
            }
            Instruction::SubImm8 => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {acc}, ",
                    "sub".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u8(buffer, value)?;
            }
            Instruction::SbcImm8 => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {acc}, ",
                    "sbc".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u8(buffer, value)?;
            }
            Instruction::AndImm8 => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {acc}, ",
                    "and".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u8(buffer, value)?;
            }
            Instruction::XorImm8 => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {acc}, ",
                    "xor".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u8(buffer, value)?;
            }
            Instruction::OrImm8 => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {acc}, ",
                    "or".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u8(buffer, value)?;
            }
            Instruction::CpImm8 => {
                let value = next_u8!();
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{} {acc}, ",
                    "cp".if_supports_color(Stream::Stdout, |text| text.style(INSTRUCTION_STYLE))
                )?;
                Self::format_u8(buffer, value)?;
            }
            _ => {
                Self::write_bytes_prefix(buffer, &bytes)?;
                write!(
                    buffer,
                    "{}",
                    self.if_supports_color(Stream::Stdout, |text| text.fg::<Yellow>())
                )?;
            }
        }
        Ok(())
    }

    fn write_bytes_prefix(buffer: &mut String, bytes: &[u8]) -> Result<(), core::fmt::Error> {
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

    fn format_u8(buffer: &mut String, value: Option<u8>) -> Result<(), core::fmt::Error> {
        if let Some(byte) = value {
            write!(
                buffer,
                "{:#04X}",
                byte.if_supports_color(Stream::Stdout, |text| text.style(IMMEDIATE_STYLE))
            )
        } else {
            write!(buffer, "0x??")
        }
    }

    fn format_i8(buffer: &mut String, value: Option<u8>) -> Result<(), core::fmt::Error> {
        if let Some(byte) = value {
            let value = byte as i8;
            write!(
                buffer,
                "+{:#04X}",
                value.if_supports_color(Stream::Stdout, |text| text.style(IMMEDIATE_STYLE))
            )
        } else {
            write!(buffer, "??")
        }
    }

    fn format_i8_address(
        buffer: &mut String,
        value: Option<(usize, u8)>,
    ) -> Result<(), core::fmt::Error> {
        if let Some((address, byte)) = value {
            let base_pc = address.wrapping_sub(1);
            let target = base_pc.saturating_add_signed((byte as i8) as isize);
            write!(
                buffer,
                "{}{:04x}",
                "$".if_supports_color(Stream::Stdout, |text| text.style(IMMEDIATE_STYLE)),
                target.if_supports_color(Stream::Stdout, |text| text.style(IMMEDIATE_STYLE)),
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

        write!(
            buffer,
            "{}",
            value.if_supports_color(Stream::Stdout, |text| text.style(IMMEDIATE_STYLE))
        )
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

        write!(
            buffer,
            "{}",
            value.if_supports_color(Stream::Stdout, |text| text.style(ADDRESS_STYLE))
        )
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
            Instruction::LdIncReg8Mem16 {
                dst: Reg8::$dst,
                src: Reg16::$src,
            }
        };
        (reg $dst:ident, mem16 $src:ident-) => {
            Instruction::LdDecReg8Mem16 {
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

    macro_rules! prefix_reg_op {
        ($op:ident, reg $reg:ident) => {
            Instruction::$op { reg: Reg8::$reg }
        };
        ($op:ident, mem $reg:ident) => {
            Instruction::$op { reg: Reg16::$reg }
        };
    }

    macro_rules! prefix_bit_op {
        ($op:ident, reg $reg:ident, pos $pos:ident) => {
            Instruction::$op {
                reg: Reg8::$reg,
                bit: BitShift8::$pos,
            }
        };
        ($op:ident, mem $reg:ident, pos $pos:ident) => {
            Instruction::$op {
                reg: Reg16::$reg,
                bit: BitShift8::$pos,
            }
        };
    }
}

impl Instruction {
    pub const fn decode_no_prefix(value: u8) -> Instruction {
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

    pub const fn decode_prefix(value: u8) -> Instruction {
        match value {
            // rlc
            0x00 => prefix_reg_op!(RlcReg8, reg B),
            0x01 => prefix_reg_op!(RlcReg8, reg C),
            0x02 => prefix_reg_op!(RlcReg8, reg D),
            0x03 => prefix_reg_op!(RlcReg8, reg E),
            0x04 => prefix_reg_op!(RlcReg8, reg H),
            0x05 => prefix_reg_op!(RlcReg8, reg L),
            0x06 => prefix_reg_op!(RlcMem16, mem HL),
            0x07 => prefix_reg_op!(RlcReg8, reg A),
            // rrc
            0x08 => prefix_reg_op!(RrcReg8, reg B),
            0x09 => prefix_reg_op!(RrcReg8, reg C),
            0x0A => prefix_reg_op!(RrcReg8, reg D),
            0x0B => prefix_reg_op!(RrcReg8, reg E),
            0x0C => prefix_reg_op!(RrcReg8, reg H),
            0x0D => prefix_reg_op!(RrcReg8, reg L),
            0x0E => prefix_reg_op!(RrcMem16, mem HL),
            0x0F => prefix_reg_op!(RrcReg8, reg A),
            // rl
            0x10 => prefix_reg_op!(RlReg8, reg B),
            0x11 => prefix_reg_op!(RlReg8, reg C),
            0x12 => prefix_reg_op!(RlReg8, reg D),
            0x13 => prefix_reg_op!(RlReg8, reg E),
            0x14 => prefix_reg_op!(RlReg8, reg H),
            0x15 => prefix_reg_op!(RlReg8, reg L),
            0x16 => prefix_reg_op!(RlMem16, mem HL),
            0x17 => prefix_reg_op!(RlReg8, reg A),
            // rr
            0x18 => prefix_reg_op!(RrReg8, reg B),
            0x19 => prefix_reg_op!(RrReg8, reg C),
            0x1A => prefix_reg_op!(RrReg8, reg D),
            0x1B => prefix_reg_op!(RrReg8, reg E),
            0x1C => prefix_reg_op!(RrReg8, reg H),
            0x1D => prefix_reg_op!(RrReg8, reg L),
            0x1E => prefix_reg_op!(RrMem16, mem HL),
            0x1F => prefix_reg_op!(RrReg8, reg A),
            // sla
            0x20 => prefix_reg_op!(SlaReg8, reg B),
            0x21 => prefix_reg_op!(SlaReg8, reg C),
            0x22 => prefix_reg_op!(SlaReg8, reg D),
            0x23 => prefix_reg_op!(SlaReg8, reg E),
            0x24 => prefix_reg_op!(SlaReg8, reg H),
            0x25 => prefix_reg_op!(SlaReg8, reg L),
            0x26 => prefix_reg_op!(SlaMem16, mem HL),
            0x27 => prefix_reg_op!(SlaReg8, reg A),
            // sra
            0x28 => prefix_reg_op!(SraReg8, reg B),
            0x29 => prefix_reg_op!(SraReg8, reg C),
            0x2A => prefix_reg_op!(SraReg8, reg D),
            0x2B => prefix_reg_op!(SraReg8, reg E),
            0x2C => prefix_reg_op!(SraReg8, reg H),
            0x2D => prefix_reg_op!(SraReg8, reg L),
            0x2E => prefix_reg_op!(SraMem16, mem HL),
            0x2F => prefix_reg_op!(SraReg8, reg A),
            // swap
            0x30 => prefix_reg_op!(SwapReg8, reg B),
            0x31 => prefix_reg_op!(SwapReg8, reg C),
            0x32 => prefix_reg_op!(SwapReg8, reg D),
            0x33 => prefix_reg_op!(SwapReg8, reg E),
            0x34 => prefix_reg_op!(SwapReg8, reg H),
            0x35 => prefix_reg_op!(SwapReg8, reg L),
            0x36 => prefix_reg_op!(SwapMem16, mem HL),
            0x37 => prefix_reg_op!(SwapReg8, reg A),
            // srl
            0x38 => prefix_reg_op!(SrlReg8, reg B),
            0x39 => prefix_reg_op!(SrlReg8, reg C),
            0x3A => prefix_reg_op!(SrlReg8, reg D),
            0x3B => prefix_reg_op!(SrlReg8, reg E),
            0x3C => prefix_reg_op!(SrlReg8, reg H),
            0x3D => prefix_reg_op!(SrlReg8, reg L),
            0x3E => prefix_reg_op!(SrlMem16, mem HL),
            0x3F => prefix_reg_op!(SrlReg8, reg A),
            // bit 0 and 1
            0x40 => prefix_bit_op!(BitReg8, reg B, pos Bit0),
            0x41 => prefix_bit_op!(BitReg8, reg C, pos Bit0),
            0x42 => prefix_bit_op!(BitReg8, reg D, pos Bit0),
            0x43 => prefix_bit_op!(BitReg8, reg E, pos Bit0),
            0x44 => prefix_bit_op!(BitReg8, reg H, pos Bit0),
            0x45 => prefix_bit_op!(BitReg8, reg L, pos Bit0),
            0x46 => prefix_bit_op!(BitMem16, mem HL, pos Bit0),
            0x47 => prefix_bit_op!(BitReg8, reg A, pos Bit0),
            0x48 => prefix_bit_op!(BitReg8, reg B, pos Bit1),
            0x49 => prefix_bit_op!(BitReg8, reg C, pos Bit1),
            0x4A => prefix_bit_op!(BitReg8, reg D, pos Bit1),
            0x4B => prefix_bit_op!(BitReg8, reg E, pos Bit1),
            0x4C => prefix_bit_op!(BitReg8, reg H, pos Bit1),
            0x4D => prefix_bit_op!(BitReg8, reg L, pos Bit1),
            0x4E => prefix_bit_op!(BitMem16, mem HL, pos Bit1),
            0x4F => prefix_bit_op!(BitReg8, reg A, pos Bit1),
            // bit 2 and 3
            0x50 => prefix_bit_op!(BitReg8, reg B, pos Bit2),
            0x51 => prefix_bit_op!(BitReg8, reg C, pos Bit2),
            0x52 => prefix_bit_op!(BitReg8, reg D, pos Bit2),
            0x53 => prefix_bit_op!(BitReg8, reg E, pos Bit2),
            0x54 => prefix_bit_op!(BitReg8, reg H, pos Bit2),
            0x55 => prefix_bit_op!(BitReg8, reg L, pos Bit2),
            0x56 => prefix_bit_op!(BitMem16, mem HL, pos Bit2),
            0x57 => prefix_bit_op!(BitReg8, reg A, pos Bit2),
            0x58 => prefix_bit_op!(BitReg8, reg B, pos Bit3),
            0x59 => prefix_bit_op!(BitReg8, reg C, pos Bit3),
            0x5A => prefix_bit_op!(BitReg8, reg D, pos Bit3),
            0x5B => prefix_bit_op!(BitReg8, reg E, pos Bit3),
            0x5C => prefix_bit_op!(BitReg8, reg H, pos Bit3),
            0x5D => prefix_bit_op!(BitReg8, reg L, pos Bit3),
            0x5E => prefix_bit_op!(BitMem16, mem HL, pos Bit3),
            0x5F => prefix_bit_op!(BitReg8, reg A, pos Bit3),
            // bit 4 and 5
            0x60 => prefix_bit_op!(BitReg8, reg B, pos Bit4),
            0x61 => prefix_bit_op!(BitReg8, reg C, pos Bit4),
            0x62 => prefix_bit_op!(BitReg8, reg D, pos Bit4),
            0x63 => prefix_bit_op!(BitReg8, reg E, pos Bit4),
            0x64 => prefix_bit_op!(BitReg8, reg H, pos Bit4),
            0x65 => prefix_bit_op!(BitReg8, reg L, pos Bit4),
            0x66 => prefix_bit_op!(BitMem16, mem HL, pos Bit4),
            0x67 => prefix_bit_op!(BitReg8, reg A, pos Bit4),
            0x68 => prefix_bit_op!(BitReg8, reg B, pos Bit5),
            0x69 => prefix_bit_op!(BitReg8, reg C, pos Bit5),
            0x6A => prefix_bit_op!(BitReg8, reg D, pos Bit5),
            0x6B => prefix_bit_op!(BitReg8, reg E, pos Bit5),
            0x6C => prefix_bit_op!(BitReg8, reg H, pos Bit5),
            0x6D => prefix_bit_op!(BitReg8, reg L, pos Bit5),
            0x6E => prefix_bit_op!(BitMem16, mem HL, pos Bit5),
            0x6F => prefix_bit_op!(BitReg8, reg A, pos Bit5),
            // bit 6 and 7
            0x70 => prefix_bit_op!(BitReg8, reg B, pos Bit6),
            0x71 => prefix_bit_op!(BitReg8, reg C, pos Bit6),
            0x72 => prefix_bit_op!(BitReg8, reg D, pos Bit6),
            0x73 => prefix_bit_op!(BitReg8, reg E, pos Bit6),
            0x74 => prefix_bit_op!(BitReg8, reg H, pos Bit6),
            0x75 => prefix_bit_op!(BitReg8, reg L, pos Bit6),
            0x76 => prefix_bit_op!(BitMem16, mem HL, pos Bit6),
            0x77 => prefix_bit_op!(BitReg8, reg A, pos Bit6),
            0x78 => prefix_bit_op!(BitReg8, reg B, pos Bit7),
            0x79 => prefix_bit_op!(BitReg8, reg C, pos Bit7),
            0x7A => prefix_bit_op!(BitReg8, reg D, pos Bit7),
            0x7B => prefix_bit_op!(BitReg8, reg E, pos Bit7),
            0x7C => prefix_bit_op!(BitReg8, reg H, pos Bit7),
            0x7D => prefix_bit_op!(BitReg8, reg L, pos Bit7),
            0x7E => prefix_bit_op!(BitMem16, mem HL, pos Bit7),
            0x7F => prefix_bit_op!(BitReg8, reg A, pos Bit7),
            // reset 0 and 1
            0x80 => prefix_bit_op!(ResReg8, reg B, pos Bit0),
            0x81 => prefix_bit_op!(ResReg8, reg C, pos Bit0),
            0x82 => prefix_bit_op!(ResReg8, reg D, pos Bit0),
            0x83 => prefix_bit_op!(ResReg8, reg E, pos Bit0),
            0x84 => prefix_bit_op!(ResReg8, reg H, pos Bit0),
            0x85 => prefix_bit_op!(ResReg8, reg L, pos Bit0),
            0x86 => prefix_bit_op!(ResMem16, mem HL, pos Bit0),
            0x87 => prefix_bit_op!(ResReg8, reg A, pos Bit0),
            0x88 => prefix_bit_op!(ResReg8, reg B, pos Bit1),
            0x89 => prefix_bit_op!(ResReg8, reg C, pos Bit1),
            0x8A => prefix_bit_op!(ResReg8, reg D, pos Bit1),
            0x8B => prefix_bit_op!(ResReg8, reg E, pos Bit1),
            0x8C => prefix_bit_op!(ResReg8, reg H, pos Bit1),
            0x8D => prefix_bit_op!(ResReg8, reg L, pos Bit1),
            0x8E => prefix_bit_op!(ResMem16, mem HL, pos Bit1),
            0x8F => prefix_bit_op!(ResReg8, reg A, pos Bit1),
            // reset 2 and 3
            0x90 => prefix_bit_op!(ResReg8, reg B, pos Bit2),
            0x91 => prefix_bit_op!(ResReg8, reg C, pos Bit2),
            0x92 => prefix_bit_op!(ResReg8, reg D, pos Bit2),
            0x93 => prefix_bit_op!(ResReg8, reg E, pos Bit2),
            0x94 => prefix_bit_op!(ResReg8, reg H, pos Bit2),
            0x95 => prefix_bit_op!(ResReg8, reg L, pos Bit2),
            0x96 => prefix_bit_op!(ResMem16, mem HL, pos Bit2),
            0x97 => prefix_bit_op!(ResReg8, reg A, pos Bit2),
            0x98 => prefix_bit_op!(ResReg8, reg B, pos Bit3),
            0x99 => prefix_bit_op!(ResReg8, reg C, pos Bit3),
            0x9A => prefix_bit_op!(ResReg8, reg D, pos Bit3),
            0x9B => prefix_bit_op!(ResReg8, reg E, pos Bit3),
            0x9C => prefix_bit_op!(ResReg8, reg H, pos Bit3),
            0x9D => prefix_bit_op!(ResReg8, reg L, pos Bit3),
            0x9E => prefix_bit_op!(ResMem16, mem HL, pos Bit3),
            0x9F => prefix_bit_op!(ResReg8, reg A, pos Bit3),
            // reset 4 and 5
            0xA0 => prefix_bit_op!(ResReg8, reg B, pos Bit4),
            0xA1 => prefix_bit_op!(ResReg8, reg C, pos Bit4),
            0xA2 => prefix_bit_op!(ResReg8, reg D, pos Bit4),
            0xA3 => prefix_bit_op!(ResReg8, reg E, pos Bit4),
            0xA4 => prefix_bit_op!(ResReg8, reg H, pos Bit4),
            0xA5 => prefix_bit_op!(ResReg8, reg L, pos Bit4),
            0xA6 => prefix_bit_op!(ResMem16, mem HL, pos Bit4),
            0xA7 => prefix_bit_op!(ResReg8, reg A, pos Bit4),
            0xA8 => prefix_bit_op!(ResReg8, reg B, pos Bit5),
            0xA9 => prefix_bit_op!(ResReg8, reg C, pos Bit5),
            0xAA => prefix_bit_op!(ResReg8, reg D, pos Bit5),
            0xAB => prefix_bit_op!(ResReg8, reg E, pos Bit5),
            0xAC => prefix_bit_op!(ResReg8, reg H, pos Bit5),
            0xAD => prefix_bit_op!(ResReg8, reg L, pos Bit5),
            0xAE => prefix_bit_op!(ResMem16, mem HL, pos Bit5),
            0xAF => prefix_bit_op!(ResReg8, reg A, pos Bit5),
            // reset 6 and 7
            0xB0 => prefix_bit_op!(ResReg8, reg B, pos Bit6),
            0xB1 => prefix_bit_op!(ResReg8, reg C, pos Bit6),
            0xB2 => prefix_bit_op!(ResReg8, reg D, pos Bit6),
            0xB3 => prefix_bit_op!(ResReg8, reg E, pos Bit6),
            0xB4 => prefix_bit_op!(ResReg8, reg H, pos Bit6),
            0xB5 => prefix_bit_op!(ResReg8, reg L, pos Bit6),
            0xB6 => prefix_bit_op!(ResMem16, mem HL, pos Bit6),
            0xB7 => prefix_bit_op!(ResReg8, reg A, pos Bit6),
            0xB8 => prefix_bit_op!(ResReg8, reg B, pos Bit7),
            0xB9 => prefix_bit_op!(ResReg8, reg C, pos Bit7),
            0xBA => prefix_bit_op!(ResReg8, reg D, pos Bit7),
            0xBB => prefix_bit_op!(ResReg8, reg E, pos Bit7),
            0xBC => prefix_bit_op!(ResReg8, reg H, pos Bit7),
            0xBD => prefix_bit_op!(ResReg8, reg L, pos Bit7),
            0xBE => prefix_bit_op!(ResMem16, mem HL, pos Bit7),
            0xBF => prefix_bit_op!(ResReg8, reg A, pos Bit7),
            // set 0 and 1
            0xC0 => prefix_bit_op!(SetReg8, reg B, pos Bit0),
            0xC1 => prefix_bit_op!(SetReg8, reg C, pos Bit0),
            0xC2 => prefix_bit_op!(SetReg8, reg D, pos Bit0),
            0xC3 => prefix_bit_op!(SetReg8, reg E, pos Bit0),
            0xC4 => prefix_bit_op!(SetReg8, reg H, pos Bit0),
            0xC5 => prefix_bit_op!(SetReg8, reg L, pos Bit0),
            0xC6 => prefix_bit_op!(SetMem16, mem HL, pos Bit0),
            0xC7 => prefix_bit_op!(SetReg8, reg A, pos Bit0),
            0xC8 => prefix_bit_op!(SetReg8, reg B, pos Bit1),
            0xC9 => prefix_bit_op!(SetReg8, reg C, pos Bit1),
            0xCA => prefix_bit_op!(SetReg8, reg D, pos Bit1),
            0xCB => prefix_bit_op!(SetReg8, reg E, pos Bit1),
            0xCC => prefix_bit_op!(SetReg8, reg H, pos Bit1),
            0xCD => prefix_bit_op!(SetReg8, reg L, pos Bit1),
            0xCE => prefix_bit_op!(SetMem16, mem HL, pos Bit1),
            0xCF => prefix_bit_op!(SetReg8, reg A, pos Bit1),
            // set 2 and 3
            0xD0 => prefix_bit_op!(SetReg8, reg B, pos Bit2),
            0xD1 => prefix_bit_op!(SetReg8, reg C, pos Bit2),
            0xD2 => prefix_bit_op!(SetReg8, reg D, pos Bit2),
            0xD3 => prefix_bit_op!(SetReg8, reg E, pos Bit2),
            0xD4 => prefix_bit_op!(SetReg8, reg H, pos Bit2),
            0xD5 => prefix_bit_op!(SetReg8, reg L, pos Bit2),
            0xD6 => prefix_bit_op!(SetMem16, mem HL, pos Bit2),
            0xD7 => prefix_bit_op!(SetReg8, reg A, pos Bit2),
            0xD8 => prefix_bit_op!(SetReg8, reg B, pos Bit3),
            0xD9 => prefix_bit_op!(SetReg8, reg C, pos Bit3),
            0xDA => prefix_bit_op!(SetReg8, reg D, pos Bit3),
            0xDB => prefix_bit_op!(SetReg8, reg E, pos Bit3),
            0xDC => prefix_bit_op!(SetReg8, reg H, pos Bit3),
            0xDD => prefix_bit_op!(SetReg8, reg L, pos Bit3),
            0xDE => prefix_bit_op!(SetMem16, mem HL, pos Bit3),
            0xDF => prefix_bit_op!(SetReg8, reg A, pos Bit3),
            // set 4 and 5
            0xE0 => prefix_bit_op!(SetReg8, reg B, pos Bit4),
            0xE1 => prefix_bit_op!(SetReg8, reg C, pos Bit4),
            0xE2 => prefix_bit_op!(SetReg8, reg D, pos Bit4),
            0xE3 => prefix_bit_op!(SetReg8, reg E, pos Bit4),
            0xE4 => prefix_bit_op!(SetReg8, reg H, pos Bit4),
            0xE5 => prefix_bit_op!(SetReg8, reg L, pos Bit4),
            0xE6 => prefix_bit_op!(SetMem16, mem HL, pos Bit4),
            0xE7 => prefix_bit_op!(SetReg8, reg A, pos Bit4),
            0xE8 => prefix_bit_op!(SetReg8, reg B, pos Bit5),
            0xE9 => prefix_bit_op!(SetReg8, reg C, pos Bit5),
            0xEA => prefix_bit_op!(SetReg8, reg D, pos Bit5),
            0xEB => prefix_bit_op!(SetReg8, reg E, pos Bit5),
            0xEC => prefix_bit_op!(SetReg8, reg H, pos Bit5),
            0xED => prefix_bit_op!(SetReg8, reg L, pos Bit5),
            0xEE => prefix_bit_op!(SetMem16, mem HL, pos Bit5),
            0xEF => prefix_bit_op!(SetReg8, reg A, pos Bit5),
            // set 6 and 7
            0xF0 => prefix_bit_op!(SetReg8, reg B, pos Bit6),
            0xF1 => prefix_bit_op!(SetReg8, reg C, pos Bit6),
            0xF2 => prefix_bit_op!(SetReg8, reg D, pos Bit6),
            0xF3 => prefix_bit_op!(SetReg8, reg E, pos Bit6),
            0xF4 => prefix_bit_op!(SetReg8, reg H, pos Bit6),
            0xF5 => prefix_bit_op!(SetReg8, reg L, pos Bit6),
            0xF6 => prefix_bit_op!(SetMem16, mem HL, pos Bit6),
            0xF7 => prefix_bit_op!(SetReg8, reg A, pos Bit6),
            0xF8 => prefix_bit_op!(SetReg8, reg B, pos Bit7),
            0xF9 => prefix_bit_op!(SetReg8, reg C, pos Bit7),
            0xFA => prefix_bit_op!(SetReg8, reg D, pos Bit7),
            0xFB => prefix_bit_op!(SetReg8, reg E, pos Bit7),
            0xFC => prefix_bit_op!(SetReg8, reg H, pos Bit7),
            0xFD => prefix_bit_op!(SetReg8, reg L, pos Bit7),
            0xFE => prefix_bit_op!(SetMem16, mem HL, pos Bit7),
            0xFF => prefix_bit_op!(SetReg8, reg A, pos Bit7),
        }
    }
}

pub fn disassemble(byte_stream: &[u8]) -> String {
    let mut buffer = String::new();

    let mut it = byte_stream.iter().copied().enumerate();

    let mut is_prefix = false;
    let mut first_nop = true;
    loop {
        let Some((address, byte)) = it.next() else {
            break;
        };
        assert!(address < u16::MAX as usize + 1, "{address}");

        let inst = if is_prefix {
            Instruction::decode_prefix(byte)
        } else {
            Instruction::decode_no_prefix(byte)
        };

        is_prefix = matches!(inst, Instruction::Prefix);
        if is_prefix {
            continue;
        }

        if matches!(inst, Instruction::Nop) {
            if first_nop {
                first_nop = false;
            } else {
                continue;
            }
        } else {
            first_nop = true;
        }

        write!(buffer, "{address:#06X}: ").unwrap();
        inst.format(&mut buffer, &mut it, byte).unwrap();
        buffer.push('\n');
    }

    buffer
}
