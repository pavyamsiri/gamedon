use super::{Condition, Instruction};
use crate::{Reg8, Reg16};
use gamedon_bits::BitShift8;
use std::collections::VecDeque;

macro_rules! enqueue {
    ($queue:expr, $($op:expr),* $(,)?) => {
        {
            $(
                ($queue).push_back($op);
            )*
        }
    };
}

/// This micro op instruction set assumes there exists some hidden registers called:
/// - TMP1: The primary temporary byte register.
/// - TMP2: The secondary temporary byte register.
/// - ADDR: The primary address register.
#[derive(Debug, Clone, Copy)]
pub enum MicroOp {
    /// Signifies the end of an M-cycle allowing the CPU to yield to other devices.
    Yield,
    /// Execute the halt procedure.
    Halt,
    /// Execute the stop procedure.
    Stop,
    /// Enable interrupts.
    SetIME,
    /// Enable interrupts.
    Ei,
    /// Disable interrupts.
    Di,
    /// Decimal adjust the accumulator.
    Daa,
    /// Execute the invalid instruction procedure.
    Invalid,
    /// Set the carry flag.
    Scf,
    /// Flip the carry flag.
    Ccf,
    /// Flip the bits of the accumulator.
    Cpl,
    /// Switch the decoder to prefix mode.
    FetchPrefix,
    /// Read a byte from an 8-bit register into TMP1.
    ReadReg8(Reg8),
    /// Write a byte from TMP1 into an 8-bit register.
    WriteReg8(Reg8),
    /// Rotate TMP1 left.
    /// The overflow bit (bit 7) is stored in both the carry flag and new bit 0.
    RotateLeftCarry,
    /// Rotate TMP1 left.
    /// The overflow bit (bit 7) is stored in the carry flag and the new bit 0 is set
    /// to the previous carry flag value.
    RotateLeft,
    /// Rotate TMP1 right.
    /// The overflow bit (bit 7) is stored in both the carry flag and new bit 0.
    RotateRightCarry,
    /// Rotate TMP1 right.
    /// The overflow bit (bit 7) is stored in the carry flag and the new bit 0 is set
    /// to the previous carry flag value.
    RotateRight,
    /// Read a 16-bit register into ADDR.
    ReadReg16(Reg16),
    /// Write ADDR into a 16-bit register.
    WriteReg16(Reg16),
    /// Read P(C) into TMP1.
    ReadLoByteFromPC,
    /// Read (P)C into TMP1.
    ReadHiByteFromPC,
    /// Read PC into ADDR.
    ReadPC,
    /// Write ADDR into PC.
    WritePC,
    /// Read a byte at $ADDR and store into TMP1.
    ReadByteFromMem,
    /// Read an immediate byte and store into TMP1.
    ReadByteFromImm,
    /// Read a byte from $SP and store into TMP1.
    ReadByteFromSP,
    /// Write a byte from TMP1 to $ADDR.
    WriteByteIntoMem,
    /// Write a byte from TMP1 to $SP.
    WriteByteIntoSP,
    /// Write TMP1 into low byte of $ADDR and set high byte to 0xFF.
    WriteLoByteIntoHighAddr,
    /// Write TMP1 into low byte of $ADDR.
    WriteLoByteIntoAddr,
    /// Write TMP1 into high byte of $ADDR.
    WriteHiByteIntoAddr,
    /// Increment SP.
    IncSP,
    /// Decrement SP.
    DecSP,
    /// Increment ADDR.
    IncAddr,
    /// Decrement ADDR.
    DecAddr,
    /// Load low byte of ADDR into TMP1.
    LoadLoAddr,
    /// Load high byte of ADDR into TMP1.
    LoadHiAddr,
    /// Load u16 address.
    LoadAddr(u16),
    // Set register flags.
    /// Set zero flag to given value.
    SetZeroFlag(bool),
    /// Set subtraction flag to given value.
    SetSubtractionFlag(bool),
    /// Set half carry flag to given value.
    SetHalfCarryFlag(bool),
    /// Set carry flag to given value.
    SetCarryFlag(bool),

    /// Evaluate condition and set conditional mode to true.
    /// If condition is true then subsequent micro ops are to be executed otherwise it is skipped.
    BeginCondition { condition: Condition },
    /// End conditional mode.
    EndCondition,
    /// Increment PC.
    IncPC,
    // ALU
    /// Add signed byte to ADDR.
    AluAddU16I8,
    /// Add signed byte to ADDR. No effect on flags.
    AluAddU16I8NoFlags,
    /// Increment TMP1.
    AluInc8,
    /// Decrement TMP1.
    AluDec8,
    /// Add a 16-bit register value to ADDR.
    AluAddU16Reg16(Reg16),
    /// Add 8-bit value to accumulator.
    AluAdd8,
    /// Rotate Left Circular
    AluRlc8,
    /// Rotate Right Circular
    AluRrc8,
    /// Rotate Left through Carry
    AluRl8,
    /// Rotate Right through Carry
    AluRr8,
    /// Shift Left Arithmetic
    AluSla8,
    /// Shift Right Arithmetic
    AluSra8,
    /// Shift Right Logical
    AluSrl8,
    /// Swap upper and lower nibbles
    AluSwap8,
    // Bit Instructions
    /// Test bit
    AluBit { bit: BitShift8 },
    /// Reset bit
    AluRes { bit: BitShift8 },
    /// Set bit.
    AluSet { bit: BitShift8 },
    /// Add value + carry flag to A (A = A + tmp8 + C), set flags Z, N=0, H, C
    AluAdc8,
    /// Subtract value from A (A = A - tmp8), set flags Z, N=1, H, C
    AluSub8,
    /// Subtract value + carry flag from A (A = A - tmp8 - C), set flags Z, N=1, H, C
    AluSbc8,
    /// Bitwise AND with A (A = A & tmp8), set flags Z, N=0, H=1, C=0
    AluAnd8,
    /// Bitwise XOR with A (A = A ^ tmp8), set flags Z, N=0, H=0, C=0
    AluXor8,
    /// Bitwise OR with A (A = A | tmp8), set flags Z, N=0, H=0, C=0
    AluOr8,
    /// Compare A with value (sets flags as if A - tmp8, but does not store result), set flags Z, N=1, H, C
    AluCp8,
}

impl Instruction {
    /// Decompose an instruction into its micro ops and pipe them into a queue.
    pub fn decompose(self, queue: &mut VecDeque<MicroOp>) {
        match self {
            // Single cycle.
            Self::Nop => enqueue!(queue, MicroOp::IncPC, MicroOp::Yield),
            Self::Halt => enqueue!(queue, MicroOp::IncPC, MicroOp::Halt, MicroOp::Yield),
            Self::Stop => enqueue!(queue, MicroOp::IncPC, MicroOp::Stop, MicroOp::Yield),
            Self::Ei => enqueue!(queue, MicroOp::IncPC, MicroOp::Ei, MicroOp::Yield),
            Self::Di => enqueue!(queue, MicroOp::IncPC, MicroOp::Di, MicroOp::Yield),
            Self::Daa => enqueue!(queue, MicroOp::IncPC, MicroOp::Daa, MicroOp::Yield),
            Self::Scf => enqueue!(queue, MicroOp::IncPC, MicroOp::Scf, MicroOp::Yield),
            Self::Ccf => enqueue!(queue, MicroOp::IncPC, MicroOp::Ccf, MicroOp::Yield),
            Self::Cpl => enqueue!(queue, MicroOp::IncPC, MicroOp::Cpl, MicroOp::Yield),
            Self::Rlca => enqueue!(
                queue,
                MicroOp::ReadReg8(Reg8::A),
                MicroOp::RotateLeft,
                MicroOp::WriteReg8(Reg8::A),
                MicroOp::SetZeroFlag(false),
                MicroOp::SetSubtractionFlag(false),
                MicroOp::SetHalfCarryFlag(false),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::Rla => enqueue!(
                queue,
                MicroOp::ReadReg8(Reg8::A),
                MicroOp::RotateLeftCarry,
                MicroOp::WriteReg8(Reg8::A),
                MicroOp::SetZeroFlag(false),
                MicroOp::SetSubtractionFlag(false),
                MicroOp::SetHalfCarryFlag(false),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::Rrca => enqueue!(
                queue,
                MicroOp::ReadReg8(Reg8::A),
                MicroOp::RotateRight,
                MicroOp::WriteReg8(Reg8::A),
                MicroOp::SetZeroFlag(false),
                MicroOp::SetSubtractionFlag(false),
                MicroOp::SetHalfCarryFlag(false),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::Rra => enqueue!(
                queue,
                MicroOp::ReadReg8(Reg8::A),
                MicroOp::RotateRightCarry,
                MicroOp::WriteReg8(Reg8::A),
                MicroOp::SetZeroFlag(false),
                MicroOp::SetSubtractionFlag(false),
                MicroOp::SetHalfCarryFlag(false),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::Invalid => {
                enqueue!(queue, MicroOp::Invalid, MicroOp::IncPC, MicroOp::Yield)
            }
            Self::Prefix => {
                enqueue!(queue, MicroOp::FetchPrefix, MicroOp::IncPC, MicroOp::Yield)
            }
            Self::LdReg8Reg8 { dst, src } => enqueue!(
                queue,
                MicroOp::ReadReg8(src),
                MicroOp::WriteReg8(dst),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::LdReg8Mem16 { dst, src } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(src),
                MicroOp::ReadByteFromMem,
                MicroOp::WriteReg8(dst),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::LdReg8Imm8 { dst } => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::WriteReg8(dst),
                MicroOp::Yield,
            ),
            Self::LdMem16Reg8 { dst, src } => enqueue!(
                queue,
                // Takes 2 M-cycles.
                MicroOp::Yield,
                // Read byte into TMP1.
                MicroOp::ReadReg8(src),
                // Read address into ADDR.
                MicroOp::ReadReg16(dst),
                // Write TMP1 into $ADDR.
                MicroOp::WriteByteIntoMem,
                // End.
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::LdMem16Imm8 { dst } => enqueue!(
                queue,
                // Takes 3 M-cycles.
                MicroOp::IncPC,
                MicroOp::Yield,
                // Read immediate byte into TMP1.
                MicroOp::ReadByteFromImm,
                MicroOp::Yield,
                // Read address into ADDR.
                MicroOp::ReadReg16(dst),
                // Write TMP1 into $ADDR.
                MicroOp::WriteByteIntoMem,
                // End.
                MicroOp::Yield,
            ),
            Self::LdMem8Reg8 { dst, src } => enqueue!(
                queue,
                MicroOp::Yield,
                // Read reg into ADDR as 0xFF00 + (reg).
                MicroOp::ReadReg8(dst),
                MicroOp::WriteLoByteIntoHighAddr,
                // Read byte into TMP1.
                MicroOp::ReadReg8(src),
                // Write TMP1 into $ADDR.
                MicroOp::WriteByteIntoMem,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::LdReg8Mem8 { dst, src } => enqueue!(
                queue,
                MicroOp::Yield,
                // Read reg into ADDR as 0xFF00 + (reg).
                MicroOp::ReadReg8(src),
                MicroOp::WriteLoByteIntoHighAddr,
                // Read byte from $ADDR into TMP1.
                MicroOp::ReadByteFromMem,
                // Write byte to register.
                MicroOp::WriteReg8(dst),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::LdAdr8Reg8 { src } => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                // Read immediate byte into TMP1.
                MicroOp::ReadByteFromImm,
                MicroOp::Yield,
                // Write byte into low byte of $ADDR = 0xFF00.
                MicroOp::WriteLoByteIntoHighAddr,
                // Read byte from register into TMP1.
                MicroOp::ReadReg8(src),
                // Write byte from TMP1 into $ADDR.
                MicroOp::WriteByteIntoMem,
                MicroOp::Yield,
            ),
            Self::LdReg8Adr8 { dst } => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                // Read immediate byte into TMP1.
                MicroOp::ReadByteFromImm,
                MicroOp::Yield,
                // Write byte into low byte of $ADDR = 0xFF00.
                MicroOp::WriteLoByteIntoHighAddr,
                // Read byte from $ADDR.
                MicroOp::ReadByteFromMem,
                // Write byte into register.
                MicroOp::WriteReg8(dst),
                MicroOp::Yield,
            ),
            Self::LdAdr16Reg8 { src } => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                // Read immediate into low byte of ADDR.
                MicroOp::ReadByteFromImm,
                MicroOp::WriteLoByteIntoAddr,
                MicroOp::Yield,
                // Read immediate into high byte of ADDR.
                MicroOp::ReadByteFromImm,
                MicroOp::WriteHiByteIntoAddr,
                MicroOp::Yield,
                // Write byte from register into $ADDR.
                MicroOp::ReadReg8(src),
                MicroOp::WriteByteIntoMem,
                MicroOp::Yield,
            ),
            Self::LdReg8Adr16 { dst } => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                // Read immediate into low byte of ADDR.
                MicroOp::ReadByteFromImm,
                MicroOp::WriteLoByteIntoAddr,
                MicroOp::Yield,
                // Read immediate into high byte of ADDR.
                MicroOp::ReadByteFromImm,
                MicroOp::WriteHiByteIntoAddr,
                MicroOp::Yield,
                // Read byte from $ADDR.
                MicroOp::ReadByteFromMem,
                // Write byte into register.
                MicroOp::WriteReg8(dst),
                MicroOp::Yield,
            ),
            Self::LdIncMem16Reg8 { dst, src } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(dst),
                MicroOp::ReadReg8(src),
                MicroOp::WriteByteIntoMem,
                MicroOp::IncAddr,
                MicroOp::WriteReg16(dst),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::LdDecMem16Reg8 { dst, src } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(dst),
                MicroOp::ReadReg8(src),
                MicroOp::WriteByteIntoMem,
                MicroOp::DecAddr,
                MicroOp::WriteReg16(dst),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::LdIncReg8Mem16 { dst, src } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(src),
                MicroOp::ReadByteFromMem,
                MicroOp::WriteReg8(dst),
                MicroOp::IncAddr,
                MicroOp::WriteReg16(src),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::LdDecReg8Mem16 { dst, src } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(src),
                MicroOp::ReadByteFromMem,
                MicroOp::WriteReg8(dst),
                MicroOp::DecAddr,
                MicroOp::WriteReg16(src),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::LdReg16Imm16 { dst } => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::WriteReg8(dst.lo()),
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::WriteReg8(dst.hi()),
                MicroOp::Yield,
            ),
            Self::LdAdr16Reg16 { src } => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::WriteLoByteIntoAddr,
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::WriteHiByteIntoAddr,
                MicroOp::Yield,
                MicroOp::ReadReg8(src.lo()),
                MicroOp::WriteByteIntoMem,
                MicroOp::Yield,
                MicroOp::ReadReg8(src.hi()),
                MicroOp::IncAddr,
                MicroOp::WriteByteIntoMem,
                MicroOp::Yield,
            ),
            Self::LdReg16Reg16 { dst, src } => enqueue!(
                queue,
                MicroOp::ReadReg16(src),
                MicroOp::WriteReg16(dst),
                MicroOp::Yield,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::LdReg16Off8 { dst } => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                // Load SP into ADDR.
                MicroOp::ReadReg16(Reg16::SP),
                // Read i8.
                MicroOp::ReadByteFromImm,
                // Add i8 to SP.
                MicroOp::AluAddU16I8,
                // Write ADDR to 16-bit register.
                MicroOp::WriteReg16(dst),
                MicroOp::Yield,
                MicroOp::Yield,
            ),
            Self::IncReg8 { reg } => enqueue!(
                queue,
                MicroOp::ReadReg8(reg),
                MicroOp::AluInc8,
                MicroOp::WriteReg8(reg),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::DecReg8 { reg } => enqueue!(
                queue,
                MicroOp::ReadReg8(reg),
                MicroOp::AluDec8,
                MicroOp::WriteReg8(reg),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::IncMem16 { reg } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(reg),
                MicroOp::ReadByteFromMem,
                MicroOp::Yield,
                MicroOp::AluInc8,
                MicroOp::WriteByteIntoMem,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::DecMem16 { reg } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(reg),
                MicroOp::ReadByteFromMem,
                MicroOp::Yield,
                MicroOp::AluDec8,
                MicroOp::WriteByteIntoMem,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::IncReg16 { reg } => enqueue!(
                queue,
                MicroOp::ReadReg16(reg),
                MicroOp::IncAddr,
                MicroOp::LoadLoAddr,
                MicroOp::WriteReg8(reg.lo()),
                MicroOp::Yield,
                MicroOp::LoadHiAddr,
                MicroOp::WriteReg8(reg.hi()),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::DecReg16 { reg } => enqueue!(
                queue,
                MicroOp::ReadReg16(reg),
                MicroOp::DecAddr,
                MicroOp::LoadLoAddr,
                MicroOp::WriteReg8(reg.lo()),
                MicroOp::Yield,
                MicroOp::LoadHiAddr,
                MicroOp::WriteReg8(reg.hi()),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::AddReg16Reg16 { dst, src } => enqueue!(
                queue,
                MicroOp::ReadReg16(dst),
                MicroOp::AluAddU16Reg16(src),
                MicroOp::LoadLoAddr,
                MicroOp::WriteReg8(dst.lo()),
                MicroOp::Yield,
                MicroOp::LoadHiAddr,
                MicroOp::WriteReg8(dst.hi()),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::AddReg16Off8 { dst } => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                MicroOp::ReadReg16(Reg16::SP),
                MicroOp::ReadByteFromImm,
                MicroOp::AluAddU16I8,
                MicroOp::Yield,
                MicroOp::LoadLoAddr,
                MicroOp::WriteReg8(dst.lo()),
                MicroOp::Yield,
                MicroOp::LoadHiAddr,
                MicroOp::WriteReg8(dst.hi()),
                MicroOp::Yield,
            ),
            Self::PushReg16 { src } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::Yield,
                // Decrement stack pointer
                MicroOp::DecSP,
                // Push high byte
                MicroOp::ReadReg8(src.hi()),
                MicroOp::WriteByteIntoSP,
                MicroOp::Yield,
                // Decrement stack pointer
                MicroOp::DecSP,
                // Push low byte
                MicroOp::ReadReg8(src.lo()),
                MicroOp::WriteByteIntoSP,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::PopReg16 { dst } => enqueue!(
                queue,
                MicroOp::Yield,
                // Write low byte
                MicroOp::ReadReg16(Reg16::SP),
                MicroOp::ReadByteFromMem,
                MicroOp::WriteReg8(dst.lo()),
                // Increment stack pointer
                MicroOp::IncSP,
                MicroOp::Yield,
                // Write high byte
                MicroOp::ReadReg16(Reg16::SP),
                MicroOp::ReadByteFromMem,
                MicroOp::WriteReg8(dst.hi()),
                // Increment stack pointer
                MicroOp::IncSP,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::Jr => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::ReadPC,
                MicroOp::AluAddU16I8NoFlags,
                MicroOp::Yield,
                // Unconditional jump.
                MicroOp::WritePC,
                MicroOp::Yield,
            ),
            Self::Jrc { condition } => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::ReadPC,
                MicroOp::AluAddU16I8NoFlags,
                MicroOp::Yield,
                // Conditional jump.
                MicroOp::BeginCondition { condition },
                MicroOp::WritePC,
                MicroOp::Yield,
                MicroOp::EndCondition,
            ),
            Self::Jp => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::WriteLoByteIntoAddr,
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::WriteHiByteIntoAddr,
                MicroOp::Yield,
                // Unconditional jump.
                MicroOp::WritePC,
                MicroOp::Yield,
            ),
            Self::Jpc { condition } => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::WriteLoByteIntoAddr,
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::WriteHiByteIntoAddr,
                MicroOp::Yield,
                // Conditional jump.
                MicroOp::BeginCondition { condition },
                MicroOp::WritePC,
                MicroOp::Yield,
                MicroOp::EndCondition,
            ),
            Self::JpReg16 { reg } => enqueue!(
                queue,
                MicroOp::ReadReg16(reg),
                MicroOp::WritePC,
                MicroOp::Yield,
            ),
            Self::Ret => enqueue!(
                queue,
                MicroOp::Yield,
                // Write low byte
                MicroOp::ReadByteFromSP,
                MicroOp::WriteLoByteIntoAddr,
                // Increment stack pointer
                MicroOp::IncSP,
                MicroOp::Yield,
                // Write high byte
                MicroOp::ReadByteFromSP,
                MicroOp::WriteHiByteIntoAddr,
                // Increment stack pointer
                MicroOp::IncSP,
                MicroOp::Yield,
                // Set PC
                MicroOp::WritePC,
                MicroOp::Yield,
            ),
            Self::Retc { condition } => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                MicroOp::BeginCondition { condition },
                MicroOp::Yield,
                // Write low byte
                MicroOp::ReadByteFromSP,
                MicroOp::WriteLoByteIntoAddr,
                // Increment stack pointer
                MicroOp::IncSP,
                MicroOp::Yield,
                // Write high byte
                MicroOp::ReadByteFromSP,
                MicroOp::WriteHiByteIntoAddr,
                // Increment stack pointer
                MicroOp::IncSP,
                MicroOp::Yield,
                // Set PC
                MicroOp::WritePC,
                MicroOp::EndCondition,
                MicroOp::Yield,
            ),
            Self::Reti => enqueue!(
                queue,
                MicroOp::Yield,
                // Write low byte
                MicroOp::ReadByteFromSP,
                MicroOp::WriteLoByteIntoAddr,
                // Increment stack pointer
                MicroOp::IncSP,
                MicroOp::Yield,
                // Write high byte
                MicroOp::ReadByteFromSP,
                MicroOp::WriteHiByteIntoAddr,
                // Increment stack pointer
                MicroOp::IncSP,
                MicroOp::Yield,
                // Set PC
                MicroOp::WritePC,
                // Set IME
                MicroOp::SetIME,
                MicroOp::Yield,
            ),
            Self::Call => enqueue!(
                queue,
                // 1. Fetch
                MicroOp::IncPC,
                MicroOp::Yield,
                // 2. Read low byte
                MicroOp::ReadByteFromImm,
                MicroOp::WriteLoByteIntoAddr,
                MicroOp::Yield,
                // 3. Read high byte
                MicroOp::ReadByteFromImm,
                MicroOp::WriteHiByteIntoAddr,
                MicroOp::Yield,
                // 4. Internal branch decision?
                MicroOp::Yield,
                // 5. Write P(C) to --SP
                MicroOp::DecSP,
                MicroOp::ReadHiByteFromPC,
                MicroOp::WriteByteIntoSP,
                MicroOp::Yield,
                // 6. Write (P)C to --SP
                MicroOp::DecSP,
                MicroOp::ReadLoByteFromPC,
                MicroOp::WriteByteIntoSP,
                MicroOp::WritePC,
                MicroOp::Yield,
            ),
            Self::Callc { condition } => enqueue!(
                queue,
                // 1. Fetch
                MicroOp::IncPC,
                MicroOp::Yield,
                // 2. Read low byte
                MicroOp::ReadByteFromImm,
                MicroOp::WriteLoByteIntoAddr,
                MicroOp::Yield,
                // 3. Read high byte
                MicroOp::ReadByteFromImm,
                MicroOp::WriteHiByteIntoAddr,
                // 4a. Internal branch decision
                MicroOp::BeginCondition { condition },
                MicroOp::Yield,
                // 5a. Write high byte
                MicroOp::DecSP,
                MicroOp::ReadHiByteFromPC,
                MicroOp::WriteByteIntoSP,
                MicroOp::Yield,
                // 6a. Write low byte
                MicroOp::DecSP,
                MicroOp::ReadLoByteFromPC,
                MicroOp::WriteByteIntoSP,
                MicroOp::WritePC,
                MicroOp::EndCondition,
                // 4b. Final yield
                MicroOp::Yield,
            ),
            Self::Rst { target } => enqueue!(
                queue,
                // 1. Fetch
                MicroOp::Yield,
                // 2. Internal
                MicroOp::Yield,
                // 3. Write P to --SP
                MicroOp::DecSP,
                MicroOp::IncPC,
                MicroOp::ReadPC,
                MicroOp::LoadHiAddr,
                MicroOp::ReadReg16(Reg16::SP),
                MicroOp::WriteByteIntoMem,
                MicroOp::Yield,
                // 3. Write C to --SP
                MicroOp::DecSP,
                MicroOp::ReadPC,
                MicroOp::LoadLoAddr,
                MicroOp::ReadReg16(Reg16::SP),
                MicroOp::WriteByteIntoMem,
                MicroOp::Yield,
                // 3a. Set PC to RST address
                MicroOp::LoadAddr(target.to_address()),
                MicroOp::WritePC,
                MicroOp::Yield,
            ),
            Self::AddReg8 { src } => enqueue!(
                queue,
                MicroOp::ReadReg8(src),
                MicroOp::AluAdd8,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::AddMem16 { src } => enqueue!(
                queue,
                // 1. Fetch
                MicroOp::Yield,
                // 2. Add
                MicroOp::ReadReg16(src),
                MicroOp::ReadByteFromMem,
                MicroOp::AluAdd8,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::AddImm8 => enqueue!(
                queue,
                // 1. Fetch
                MicroOp::IncPC,
                MicroOp::Yield,
                // 2. Add
                MicroOp::ReadByteFromImm,
                MicroOp::AluAdd8,
                MicroOp::Yield,
            ),
            Self::AdcReg8 { src } => enqueue!(
                queue,
                MicroOp::ReadReg8(src),
                MicroOp::AluAdc8,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::AdcMem16 { src } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(src),
                MicroOp::ReadByteFromMem,
                MicroOp::AluAdc8,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::AdcImm8 => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::AluAdc8,
                MicroOp::Yield,
            ),

            Self::SubReg8 { src } => enqueue!(
                queue,
                MicroOp::ReadReg8(src),
                MicroOp::AluSub8,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::SubMem16 { src } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(src),
                MicroOp::ReadByteFromMem,
                MicroOp::AluSub8,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::SubImm8 => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::AluSub8,
                MicroOp::Yield,
            ),

            Self::SbcReg8 { src } => enqueue!(
                queue,
                MicroOp::ReadReg8(src),
                MicroOp::AluSbc8,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::SbcMem16 { src } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(src),
                MicroOp::ReadByteFromMem,
                MicroOp::AluSbc8,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::SbcImm8 => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::AluSbc8,
                MicroOp::Yield,
            ),

            Self::AndReg8 { src } => enqueue!(
                queue,
                MicroOp::ReadReg8(src),
                MicroOp::AluAnd8,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::AndMem16 { src } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(src),
                MicroOp::ReadByteFromMem,
                MicroOp::AluAnd8,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::AndImm8 => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::AluAnd8,
                MicroOp::Yield,
            ),

            Self::XorReg8 { src } => enqueue!(
                queue,
                MicroOp::ReadReg8(src),
                MicroOp::AluXor8,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::XorMem16 { src } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(src),
                MicroOp::ReadByteFromMem,
                MicroOp::AluXor8,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::XorImm8 => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::AluXor8,
                MicroOp::Yield,
            ),
            Self::OrReg8 { src } => enqueue!(
                queue,
                MicroOp::ReadReg8(src),
                MicroOp::AluOr8,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::OrMem16 { src } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(src),
                MicroOp::ReadByteFromMem,
                MicroOp::AluOr8,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::OrImm8 => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::AluOr8,
                MicroOp::Yield,
            ),
            Self::CpReg8 { src } => enqueue!(
                queue,
                MicroOp::ReadReg8(src),
                MicroOp::AluCp8,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::CpMem16 { src } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(src),
                MicroOp::ReadByteFromMem,
                MicroOp::AluCp8,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::CpImm8 => enqueue!(
                queue,
                MicroOp::IncPC,
                MicroOp::Yield,
                MicroOp::ReadByteFromImm,
                MicroOp::AluCp8,
                MicroOp::Yield,
            ),
            Self::RlcReg8 { reg } => enqueue!(
                queue,
                MicroOp::ReadReg8(reg),
                MicroOp::AluRlc8,
                MicroOp::WriteReg8(reg),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::RlcMem16 { reg } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(reg),
                MicroOp::ReadByteFromMem,
                MicroOp::AluRlc8,
                MicroOp::WriteByteIntoMem,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),

            Self::RrcReg8 { reg } => enqueue!(
                queue,
                MicroOp::ReadReg8(reg),
                MicroOp::AluRrc8,
                MicroOp::WriteReg8(reg),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::RrcMem16 { reg } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(reg),
                MicroOp::ReadByteFromMem,
                MicroOp::AluRrc8,
                MicroOp::WriteByteIntoMem,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),

            Self::RlReg8 { reg } => enqueue!(
                queue,
                MicroOp::ReadReg8(reg),
                MicroOp::AluRl8,
                MicroOp::WriteReg8(reg),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::RlMem16 { reg } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(reg),
                MicroOp::ReadByteFromMem,
                MicroOp::AluRl8,
                MicroOp::WriteByteIntoMem,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),

            Self::RrReg8 { reg } => enqueue!(
                queue,
                MicroOp::ReadReg8(reg),
                MicroOp::AluRr8,
                MicroOp::WriteReg8(reg),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::RrMem16 { reg } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(reg),
                MicroOp::ReadByteFromMem,
                MicroOp::AluRr8,
                MicroOp::WriteByteIntoMem,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),

            Self::SlaReg8 { reg } => enqueue!(
                queue,
                MicroOp::ReadReg8(reg),
                MicroOp::AluSla8,
                MicroOp::WriteReg8(reg),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::SlaMem16 { reg } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(reg),
                MicroOp::ReadByteFromMem,
                MicroOp::AluSla8,
                MicroOp::WriteByteIntoMem,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),

            Self::SraReg8 { reg } => enqueue!(
                queue,
                MicroOp::ReadReg8(reg),
                MicroOp::AluSra8,
                MicroOp::WriteReg8(reg),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::SraMem16 { reg } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(reg),
                MicroOp::ReadByteFromMem,
                MicroOp::AluSra8,
                MicroOp::WriteByteIntoMem,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),

            Self::SwapReg8 { reg } => enqueue!(
                queue,
                MicroOp::ReadReg8(reg),
                MicroOp::AluSwap8,
                MicroOp::WriteReg8(reg),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::SwapMem16 { reg } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(reg),
                MicroOp::ReadByteFromMem,
                MicroOp::AluSwap8,
                MicroOp::WriteByteIntoMem,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),

            Self::SrlReg8 { reg } => enqueue!(
                queue,
                MicroOp::ReadReg8(reg),
                MicroOp::AluSrl8,
                MicroOp::WriteReg8(reg),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::SrlMem16 { reg } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(reg),
                MicroOp::ReadByteFromMem,
                MicroOp::AluSrl8,
                MicroOp::WriteByteIntoMem,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),

            Self::BitReg8 { reg, bit } => enqueue!(
                queue,
                MicroOp::ReadReg8(reg),
                MicroOp::AluBit { bit },
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::BitMem16 { reg, bit } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(reg),
                MicroOp::ReadByteFromMem,
                MicroOp::AluBit { bit },
                MicroOp::IncPC,
                MicroOp::Yield,
            ),

            Self::ResReg8 { reg, bit } => enqueue!(
                queue,
                MicroOp::ReadReg8(reg),
                MicroOp::AluRes { bit },
                MicroOp::WriteReg8(reg),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::ResMem16 { reg, bit } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(reg),
                MicroOp::ReadByteFromMem,
                MicroOp::AluRes { bit },
                MicroOp::WriteByteIntoMem,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),

            Self::SetReg8 { reg, bit } => enqueue!(
                queue,
                MicroOp::ReadReg8(reg),
                MicroOp::AluSet { bit },
                MicroOp::WriteReg8(reg),
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
            Self::SetMem16 { reg, bit } => enqueue!(
                queue,
                MicroOp::Yield,
                MicroOp::ReadReg16(reg),
                MicroOp::ReadByteFromMem,
                MicroOp::AluSet { bit },
                MicroOp::WriteByteIntoMem,
                MicroOp::IncPC,
                MicroOp::Yield,
            ),
        }
    }

    /// Encode an interrupt request to an `address` as a series of micro ops and pump into queue.
    pub fn encode_interrupt_request(queue: &mut VecDeque<MicroOp>, address: u16) {
        enqueue!(
            queue,
            // 1. Wait
            MicroOp::Yield,
            // 2. Wait
            MicroOp::Yield,
            // 3. Write P to --SP
            MicroOp::DecSP,
            MicroOp::ReadPC,
            MicroOp::LoadHiAddr,
            MicroOp::WriteByteIntoSP,
            MicroOp::Yield,
            // 4. Write C to --SP
            MicroOp::DecSP,
            MicroOp::ReadPC,
            MicroOp::LoadLoAddr,
            MicroOp::WriteByteIntoSP,
            MicroOp::LoadAddr(address),
            MicroOp::WritePC,
            MicroOp::Yield,
            MicroOp::Yield,
        );
    }
}
