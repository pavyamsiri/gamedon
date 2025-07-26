use alu::WithCarry;
use gamedon_bus::{BusReader, BusWriter, MemoryBus, ReadByteError, WriteByteError};
use gamedon_opcode::{Decoder, MicroOp, Registers};
use rotate::{FillWith, ShiftOption};
use std::collections::{BTreeMap, VecDeque};
use thiserror::Error;

pub use boot::BootRom;
pub use gamedon_opcode::Instruction;

mod alu;
mod bit;
mod boot;
mod jump;
mod rotate;

type Map<K, V> = BTreeMap<K, V>;

/// Helper function to reinterpret `u8` as `i8` without using `as` keyword.
const fn cast_u8_to_i8(byte: u8) -> i8 {
    i8::from_ne_bytes([byte])
}

/// Errors that can occur during CPU execution.
#[derive(Debug, Error)]
pub enum ExecuteError {
    /// Errors from reading from the memory bus.
    #[error(transparent)]
    BusRead(#[from] ReadByteError),
    /// Errors from writing to the memory bus.
    #[error(transparent)]
    BusWrite(#[from] WriteByteError),
    /// The CPU encountered an invalid opcode.
    #[error("Attempted to run an invalid opcode.")]
    InvalidOpcode,
}

/// Information about an instruction.
#[derive(Debug, Default, Clone, Copy)]
pub struct InstructionInfo {
    /// The address of the instruction.
    pub address: u16,
    /// The opcode.
    pub opcode: u8,
    /// Whether the opcode is a prefixed opcode.
    pub is_prefix: bool,
}

impl InstructionInfo {
    /// Encode opcode as `u16` meaning the prefix byte and its opcode byte.
    #[inline]
    pub const fn to_u16(&self) -> u16 {
        if self.is_prefix {
            u16::from_be_bytes([0xCB, self.opcode])
        } else {
            u16::from_be_bytes([0x00, self.opcode])
        }
    }
}

/// Result of a CPU M-cycle.
pub struct ExecutionStep {
    /// The status of the instruction pipeline.
    pub status: ExecutionStatus,
    /// Whether the CPU has hit a breakpoint or not.
    pub hit_breakpoint: bool,
}

/// The status of the instruction pipeline.
pub enum ExecutionStatus {
    /// The CPU has completed an instruction.
    Completed(InstructionInfo),
    /// The CPU is halted.
    Halted,
    /// The CPU is in the process of executing an instruction.
    Incomplete,
    /// The CPU is not executing anything.
    Nop,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum BreakPointCondition {
    #[default]
    RunInto,
    AfterRun,
    WhenOp(u16),
    ReadBus,
    WriteBus {
        value: Option<u8>,
    },
}

/// The micro op pipeline.
#[derive(Debug, Default)]
struct OpPipeline {
    /// The queue of micro ops.
    queue: VecDeque<MicroOp>,
    /// The instruction being executed.
    info: InstructionInfo,
}

impl OpPipeline {
    /// Encode opcode as `u16` meaning the prefix byte and its opcode byte.
    const fn to_u16(&self) -> u16 {
        if self.info.is_prefix {
            u16::from_be_bytes([0xCB, self.info.opcode])
        } else {
            u16::from_be_bytes([0x00, self.info.opcode])
        }
    }
}

/// The master interrupt enable (IME) state.
#[derive(Debug, Default, Clone, Copy)]
enum ImeState {
    /// Interrupts are disable.
    #[default]
    Off,
    /// Interrupts are enabled.
    On,
    /// Interrupts are scheduled to be enabled after the completion of the next instruction.
    /// This state would be entered after an `EI` instruction and then be updated upon
    /// the decoding of the next instruction to `Updating`.
    Scheduled,
    /// Interrupts are in the process of being enabled at the completion of the current instruction.
    /// This state is entered after the completion of the `EI` instruction and will transition to the
    /// `On` state at the next fetch.
    Updating,
}

impl ImeState {
    const fn on(self) -> bool {
        matches!(self, Self::On)
    }
}

/// The CPU.
#[derive(Debug, Default)]
pub struct Cpu {
    /// The CPU registers.
    pub registers: Registers,
    /// An internal working register for 8-bit numbers.
    pub tmp1: u8,
    /// An internal working register for addresses and 16-bit numbers.
    pub addr: u16,
    /// The active breakpoints.
    breakpoints: Map<u16, BreakPointCondition>,
    /// Whether the CPU is in conditional mode and the execution condition.
    conditional_mode: Option<bool>,
    /// The master interrupt enable.
    ime: ImeState,
    /// Whether the CPU is halted.
    is_halted: bool,
    /// The opcode decoder.
    decoder: Decoder,
    /// The micro op pipeline.
    pipeline: OpPipeline,
}

// Debug
impl Cpu {
    /// Add a breakpoint at `address` with `condition`.
    #[inline]
    pub fn add_breakpoint(&mut self, address: u16, condition: BreakPointCondition) {
        self.breakpoints.insert(address, condition);
    }

    /// Return the last instruction's info.
    #[inline]
    pub const fn get_last_op(&self) -> InstructionInfo {
        self.pipeline.info
    }
}

// Handlers
impl Cpu {
    /// Check breakpoints before decoding.
    fn handle_breakpoints_before_decode(&self) -> bool {
        let current_address = self.registers.get_pc();
        let Some(condition) = self.breakpoints.get(&current_address) else {
            return false;
        };

        if matches!(condition, BreakPointCondition::RunInto) {
            tracing::debug!("Hit run into breakpoint.");
            true
        } else {
            false
        }
    }

    /// Check breakpoints after decoding.
    fn handle_breakpoints_after_decode(&self) -> bool {
        let current_address = self.registers.get_pc();
        let Some(condition) = self.breakpoints.get(&current_address) else {
            return false;
        };

        match condition {
            BreakPointCondition::WhenOp(opcode) => {
                if self.pipeline.to_u16() == *opcode {
                    tracing::debug!("Hit run into break point when opcode is {opcode:#06X}.");
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Check breakpoints after execution.
    fn handle_breakpoints_after_execution(&self) -> bool {
        let current_address = self.registers.get_pc();
        let Some(condition) = self.breakpoints.get(&current_address) else {
            return false;
        };
        if matches!(condition, BreakPointCondition::AfterRun) {
            tracing::debug!("Hit after execution break point.");
            true
        } else {
            false
        }
    }

    /// Check breakpoints before bus read.
    fn handle_breakpoints_on_bus_read(&mut self, address: u16) -> bool {
        let Some(condition) = self.breakpoints.get(&address) else {
            return false;
        };
        if matches!(condition, BreakPointCondition::ReadBus) {
            tracing::debug!("Hit read bus breakpoint.");
            true
        } else {
            false
        }
    }

    /// Check breakpoints before bus write.
    fn handle_breakpoints_on_bus_write(&mut self, address: u16, value: u8) -> bool {
        let _ = value;
        let Some(condition) = self.breakpoints.get(&address) else {
            return false;
        };
        match condition {
            BreakPointCondition::WriteBus { value: None } => {
                tracing::debug!("Hit unconditional write bus breakpoint.");
                true
            }
            BreakPointCondition::WriteBus {
                value: Some(target),
            } => value == *target,
            _ => false,
        }
    }
}

// Diagnostics
impl Cpu {
    pub fn print_doctor_status(&self, bus: &impl BusReader) -> Result<String, ExecuteError> {
        let byte1 = bus.read_byte(self.registers.get_pc())?;
        let byte2 = bus.read_byte(self.registers.get_pc() + 1)?;
        let byte3 = bus.read_byte(self.registers.get_pc() + 2)?;
        let byte4 = bus.read_byte(self.registers.get_pc() + 3)?;
        let tima = bus.read_byte(0xFF04)?;
        Ok(format!(
            "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X} |TIMER={:02X}|",
            self.registers.get_a(),
            self.registers.get_f(),
            self.registers.get_b(),
            self.registers.get_c(),
            self.registers.get_d(),
            self.registers.get_e(),
            self.registers.get_h(),
            self.registers.get_l(),
            self.registers.get_sp(),
            self.registers.get_pc(),
            byte1,
            byte2,
            byte3,
            byte4,
            tima,
        ))
    }
}

// Executor
impl Cpu {
    /// Execute an M-cycle.
    pub fn step(
        &mut self,
        bus: &mut MemoryBus,
        skip_breakpoints: bool,
    ) -> Result<ExecutionStep, ExecuteError> {
        macro_rules! read_byte {
            ($bus:expr, $address:expr, $should_break:expr) => {{
                let address = $address;
                let value = $bus.read_byte(address)?;
                let should_break = self.handle_breakpoints_on_bus_read(address);
                *($should_break) |= should_break;
                value
            }};
        }

        macro_rules! write_byte {
            ($bus:expr, $address:expr, $value:expr, $should_break:expr) => {{
                let address = $address;
                let value = $value;
                $bus.write_byte(address, value)?;
                let should_break = self.handle_breakpoints_on_bus_write(address, value);
                *($should_break) |= should_break;
            }};
        }

        // Fetch instruction
        let current_address = self.registers.get_pc();

        if !skip_breakpoints && self.handle_breakpoints_before_decode() {
            return Ok(ExecutionStep {
                status: ExecutionStatus::Nop,
                hit_breakpoint: true,
            });
        }

        let mut should_pause = false;

        // Not currently executing an instruction
        if self.pipeline.queue.is_empty() {
            // CPU is halted, only wake up when there are pending interrupts.
            if self.is_halted {
                self.is_halted = !bus.is_pending_interrupts();
                return Ok(ExecutionStep {
                    status: ExecutionStatus::Halted,
                    hit_breakpoint: false,
                });
            }

            self.ime = match self.ime {
                flag @ (ImeState::Off | ImeState::On) => flag,
                ImeState::Scheduled => ImeState::Updating,
                ImeState::Updating => ImeState::On,
            };

            // If the Interrupt Master Enable is set
            if let Some(interrupt) = bus.get_pending_interrupt()
                && self.ime.on()
            {
                bus.reset_interrupt_request(interrupt);
                self.ime = ImeState::Off;
                Instruction::encode_interrupt_request(
                    &mut self.pipeline.queue,
                    interrupt.to_address(),
                );
            } else {
                let opcode = read_byte!(bus, current_address, &mut should_pause);
                let inst = self.decoder.decode(opcode);

                let mut it = bus.iter_from(current_address + 1);
                let mut display_str = format!("{current_address:#06X}: ");
                inst.format(&mut display_str, &mut it, opcode).unwrap();
                tracing::trace!("{display_str}");

                inst.decompose(&mut self.pipeline.queue);
                self.pipeline.info = InstructionInfo {
                    address: current_address,
                    opcode,
                    is_prefix: inst.is_prefix(),
                };

                if !skip_breakpoints && self.handle_breakpoints_after_decode() {
                    return Ok(ExecutionStep {
                        status: ExecutionStatus::Nop,
                        hit_breakpoint: true,
                    });
                }
            }
        }

        // Clear out ops until yield
        while let Some(op) = self.pipeline.queue.pop_front() {
            if let Some(false) = self.conditional_mode {
                if let MicroOp::EndCondition = op {
                    self.conditional_mode = None;
                    continue;
                }
            } else {
                tracing::trace!(
                    "\t\tExecuting uop ({:#06X}): {op:?} |Flags = {:08b}|",
                    self.pipeline.info.address,
                    self.registers.get_f()
                );
                match op {
                    MicroOp::IncPC => {
                        self.registers
                            .set_pc(self.registers.get_pc().wrapping_add(1));
                    }
                    MicroOp::Yield => break,
                    MicroOp::Halt => self.is_halted = true,
                    MicroOp::Stop => todo!(),
                    MicroOp::SetIME => self.ime = ImeState::On,
                    MicroOp::Ei => self.ime = ImeState::Scheduled,
                    MicroOp::Di => self.ime = ImeState::Off,
                    MicroOp::Daa => {
                        self.daa();
                    }
                    MicroOp::Invalid => {
                        return Err(ExecuteError::InvalidOpcode);
                    }
                    MicroOp::Scf => {
                        self.scf();
                    }
                    MicroOp::Ccf => {
                        self.ccf();
                    }
                    MicroOp::Cpl => {
                        self.cpl();
                    }
                    MicroOp::FetchPrefix => {}
                    MicroOp::ReadReg8(reg) => {
                        self.tmp1 = self.registers.get_reg8(reg);
                    }
                    MicroOp::WriteReg8(reg) => {
                        self.registers.set_reg8(reg, self.tmp1);
                    }
                    MicroOp::RotateLeftCarry => {
                        self.tmp1 = self.alu_rotate_left(self.tmp1, FillWith::Carry);
                    }
                    MicroOp::RotateLeft => {
                        self.tmp1 = self.alu_rotate_left(self.tmp1, FillWith::OldBit);
                    }
                    MicroOp::RotateRightCarry => {
                        self.tmp1 = self.alu_rotate_right(self.tmp1, FillWith::Carry);
                    }
                    MicroOp::RotateRight => {
                        self.tmp1 = self.alu_rotate_right(self.tmp1, FillWith::OldBit);
                    }
                    MicroOp::ReadReg16(reg) => {
                        self.addr = self.registers.get_reg16(reg);
                    }
                    MicroOp::WriteReg16(reg) => {
                        self.registers.set_reg16(reg, self.addr);
                    }
                    MicroOp::ReadPC => {
                        self.addr = self.registers.get_pc();
                    }
                    MicroOp::WritePC => {
                        should_pause |= self.addr == self.pipeline.info.address;
                        self.registers.set_pc(self.addr);
                    }
                    MicroOp::ReadByteFromMem => {
                        self.tmp1 = read_byte!(bus, self.addr, &mut should_pause);
                    }
                    MicroOp::ReadByteFromImm => {
                        self.tmp1 = read_byte!(bus, self.registers.get_pc(), &mut should_pause);
                        self.registers
                            .set_pc(self.registers.get_pc().wrapping_add(1));
                    }
                    MicroOp::WriteByteIntoMem => {
                        write_byte!(bus, self.addr, self.tmp1, &mut should_pause);
                    }
                    MicroOp::WriteLoByteIntoHighAddr => {
                        self.addr = 0xFF00 | u16::from(self.tmp1);
                    }
                    MicroOp::WriteLoByteIntoAddr => {
                        self.addr = (self.addr & 0xFF00) | u16::from(self.tmp1);
                    }
                    MicroOp::WriteHiByteIntoAddr => {
                        self.addr = (self.addr & 0x00FF) | u16::from(self.tmp1) << 8;
                    }
                    MicroOp::IncSP => {
                        self.registers
                            .set_sp(self.registers.get_sp().wrapping_add(1));
                    }
                    MicroOp::DecSP => {
                        self.registers
                            .set_sp(self.registers.get_sp().wrapping_sub(1));
                    }
                    MicroOp::IncAddr => {
                        self.addr = self.addr.wrapping_add(1);
                    }
                    MicroOp::DecAddr => {
                        self.addr = self.addr.wrapping_sub(1);
                    }
                    MicroOp::LoadLoAddr => {
                        self.tmp1 = self.addr.to_le_bytes()[0];
                    }
                    MicroOp::LoadHiAddr => {
                        self.tmp1 = self.addr.to_be_bytes()[0];
                    }
                    MicroOp::LoadAddr(value) => {
                        self.addr = value;
                    }
                    MicroOp::BeginCondition { condition } => {
                        self.conditional_mode = Some(self.evaluate_condition(condition));
                    }
                    MicroOp::EndCondition => {
                        assert!(
                            self.conditional_mode.is_some(),
                            "Shouldn't be ending a condition if not in conditional mode."
                        );
                        self.conditional_mode = None;
                    }
                    MicroOp::AluAddU16I8 => {
                        self.addr = self.alu_add16_signed(self.addr, cast_u8_to_i8(self.tmp1));
                    }
                    MicroOp::AluAddU16I8NoFlags => {
                        self.addr = self
                            .addr
                            .wrapping_add_signed(i16::from(cast_u8_to_i8(self.tmp1)));
                    }
                    MicroOp::AluInc8 => {
                        self.tmp1 = self.alu_inc8(self.tmp1);
                    }
                    MicroOp::AluDec8 => {
                        self.tmp1 = self.alu_dec8(self.tmp1);
                    }
                    MicroOp::AluAddU16Reg16(reg) => {
                        self.addr = self.alu_add16(self.addr, self.registers.get_reg16(reg));
                    }
                    MicroOp::AluAdd8 => {
                        let value = self.alu_add8(self.registers.get_a(), self.tmp1, WithCarry::No);
                        self.registers.set_a(value);
                    }
                    MicroOp::AluRlc8 => {
                        self.tmp1 = self.alu_rotate_left(self.tmp1, FillWith::OldBit);
                    }
                    MicroOp::AluRrc8 => {
                        self.tmp1 = self.alu_rotate_right(self.tmp1, FillWith::OldBit);
                    }
                    MicroOp::AluRl8 => {
                        self.tmp1 = self.alu_rotate_left(self.tmp1, FillWith::Carry);
                    }
                    MicroOp::AluRr8 => {
                        self.tmp1 = self.alu_rotate_right(self.tmp1, FillWith::Carry);
                    }
                    MicroOp::AluSla8 => {
                        self.tmp1 = self.alu_shift_left(self.tmp1);
                    }
                    MicroOp::AluSra8 => {
                        self.tmp1 = self.alu_shift_right(self.tmp1, ShiftOption::Unchanged);
                    }
                    MicroOp::AluSrl8 => {
                        self.tmp1 = self.alu_shift_right(self.tmp1, ShiftOption::Reset);
                    }
                    MicroOp::AluSwap8 => {
                        self.tmp1 = self.alu_swap(self.tmp1);
                    }
                    MicroOp::AluBit { bit } => {
                        self.alu_bit(self.tmp1, bit);
                    }
                    MicroOp::AluRes { bit } => {
                        self.tmp1 = Self::alu_reset(self.tmp1, bit);
                    }
                    MicroOp::AluSet { bit } => {
                        self.tmp1 = Self::alu_set(self.tmp1, bit);
                    }
                    MicroOp::AluAdc8 => {
                        let value =
                            self.alu_add8(self.registers.get_a(), self.tmp1, WithCarry::Yes);
                        self.registers.set_a(value);
                    }
                    MicroOp::AluSub8 => {
                        let value = self.alu_sub8(self.registers.get_a(), self.tmp1, WithCarry::No);
                        self.registers.set_a(value);
                    }
                    MicroOp::AluSbc8 => {
                        let value =
                            self.alu_sub8(self.registers.get_a(), self.tmp1, WithCarry::Yes);
                        self.registers.set_a(value);
                    }
                    MicroOp::AluAnd8 => {
                        let value = self.alu_and8(self.registers.get_a(), self.tmp1);
                        self.registers.set_a(value);
                    }
                    MicroOp::AluXor8 => {
                        let value = self.alu_xor8(self.registers.get_a(), self.tmp1);
                        self.registers.set_a(value);
                    }
                    MicroOp::AluOr8 => {
                        let value = self.alu_or8(self.registers.get_a(), self.tmp1);
                        self.registers.set_a(value);
                    }
                    MicroOp::AluCp8 => {
                        let _ = self.alu_sub8(self.registers.get_a(), self.tmp1, WithCarry::No);
                    }
                    MicroOp::ReadLoByteFromPC => {
                        self.tmp1 = (self.registers.get_pc() & 0x00FF) as u8;
                    }
                    MicroOp::ReadHiByteFromPC => {
                        self.tmp1 = ((self.registers.get_pc() & 0xFF00) >> 8) as u8;
                    }
                    MicroOp::WriteByteIntoSP => {
                        write_byte!(bus, self.registers.get_sp(), self.tmp1, &mut should_pause);
                    }
                    MicroOp::ReadByteFromSP => {
                        self.tmp1 = read_byte!(bus, self.registers.get_sp(), &mut should_pause);
                    }
                    MicroOp::SetZeroFlag(value) => self.registers.set_zero_flag(value),
                    MicroOp::SetSubtractionFlag(value) => {
                        self.registers.set_subtraction_flag(value);
                    }
                    MicroOp::SetHalfCarryFlag(value) => self.registers.set_half_carry_flag(value),
                    MicroOp::SetCarryFlag(value) => self.registers.set_carry_flag(value),
                }
            }
        }

        if self.pipeline.queue.is_empty() {
            should_pause |= self.handle_breakpoints_after_execution();
            let should_pause = should_pause && !skip_breakpoints;
            Ok(ExecutionStep {
                status: ExecutionStatus::Completed(self.pipeline.info),
                hit_breakpoint: should_pause,
            })
        } else {
            let should_pause = should_pause && !skip_breakpoints;
            Ok(ExecutionStep {
                status: ExecutionStatus::Incomplete,
                hit_breakpoint: should_pause,
            })
        }
    }
}

// miscellaneous
impl Cpu {
    /// Perform the `SCF` instruction or "set carry flag".
    const fn scf(&mut self) {
        self.registers.set_carry_flag(true);
        self.registers.set_half_carry_flag(false);
        self.registers.set_subtraction_flag(false);
    }

    /// Perform the `CCF` instruction or "complement carry flag".
    const fn ccf(&mut self) {
        self.registers
            .set_carry_flag(!self.registers.get_carry_flag());
        self.registers.set_half_carry_flag(false);
        self.registers.set_subtraction_flag(false);
    }

    /// Perform the `CPL` instruction or "complement".
    const fn cpl(&mut self) {
        // Flip bits of register A
        let current_value = self.registers.get_a();
        let new_value = current_value ^ 0xFF;

        // Set subtraction flag
        self.registers.set_subtraction_flag(true);
        // Set half-carry flag
        self.registers.set_half_carry_flag(true);

        self.registers.set_a(new_value);
    }

    /// Perform the `DAA` instruction or "decimal adjust accumulator".
    const fn daa(&mut self) {
        if self.registers.get_subtraction_flag() {
            if self.registers.get_carry_flag() {
                self.registers
                    .set_a(self.registers.get_a().wrapping_sub(0x60));
            }
            if self.registers.get_half_carry_flag() {
                self.registers
                    .set_a(self.registers.get_a().wrapping_sub(0x06));
            }
        } else {
            if self.registers.get_carry_flag() || self.registers.get_a() > 0x99 {
                self.registers
                    .set_a(self.registers.get_a().wrapping_add(0x60));
                self.registers.set_carry_flag(true);
            }
            if self.registers.get_half_carry_flag() || (self.registers.get_a() & 0x0f) > 0x09 {
                self.registers
                    .set_a(self.registers.get_a().wrapping_add(0x06));
            }
        }

        self.registers.set_zero_flag(self.registers.get_a() == 0);
        self.registers.set_half_carry_flag(false);
    }
}
