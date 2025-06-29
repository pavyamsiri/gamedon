mod alu;
mod bit;
mod boot;
mod jump;
mod load;
mod rotate;

use std::collections::{BTreeMap, VecDeque};

use alu::WithCarry;
use gamedon_bus::{BusReader, BusWriter, MemoryBus, ReadByteError, WriteByteError};
use gamedon_opcode::{Instruction, MicroOp, Reg8, Reg16, Registers};
use rotate::ShiftOption;
use thiserror::Error;

pub use boot::BootRom;

type Map<K, V> = BTreeMap<K, V>;

/// The number of clock cycles in a machine cycle.
const ONE_M_STATE: usize = 1;
const TWO_M_STATE: usize = 2;
const THREE_M_STATE: usize = 3;
const FOUR_M_STATE: usize = 4;
const FIVE_M_STATE: usize = 5;
const SIX_M_STATE: usize = 6;

#[derive(Debug, Error)]
pub enum ExecuteError {
    #[error("Failed to read byte from bus: {0}")]
    BusRead(#[from] ReadByteError),
    #[error("Failed to write byte into bus: {0}")]
    BusWrite(#[from] WriteByteError),
    #[error("The next PC is not a valid 16-bit address: base = {base} + {offset} > 2^16 - 1")]
    OutOfBoundsPc { base: u16, offset: i16 },
    #[error("Attempted to run an invalid opcode.")]
    InvalidOpcode,
}

#[derive(Debug, Clone, Copy)]
enum NextPc {
    Relative(i16),
    Absolute(u16),
}

enum IncDirection {
    Inc,
    Dec,
}

impl IncDirection {
    const fn offset_i16(self) -> i16 {
        match self {
            IncDirection::Inc => 1,
            IncDirection::Dec => -1,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum BreakPointCondition {
    #[default]
    RunInto,
    AfterRun,
    WhenOp(u16),
}

#[derive(Debug, Default)]
struct OpPipeline {
    queue: VecDeque<MicroOp>,
    address: u16,
    opcode: u16,
}

impl OpPipeline {
    const fn is_prefix_op(&self) -> bool {
        (self.opcode & 0xFF00) == 0xCB00
    }
}

#[derive(Debug, Default)]
pub struct Cpu {
    addr: u16,
    breakpoints: Map<u16, BreakPointCondition>,
    conditional_mode: Option<bool>,
    pub debug_flag: bool,
    ime: bool,

    is_halted: bool,

    is_prefix: bool,
    op_pipeline: OpPipeline,
    // Debug
    paused: bool,

    registers: Registers,

    set_ime: bool,
    step_count: usize,

    // Internal registers
    tmp1: u8,
}

// Debug
impl Cpu {
    pub fn add_breakpoint(&mut self, address: u16, condition: BreakPointCondition) {
        self.breakpoints.insert(address, condition);
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }
}

// Executor
impl Cpu {
    fn handle_breakpoints_before_execution(&mut self) {
        let current_address = self.registers.get_pc();
        let Some(condition) = self.breakpoints.get(&current_address) else {
            return;
        };
        match condition {
            BreakPointCondition::RunInto => self.paused = true,
            BreakPointCondition::AfterRun => {}
            BreakPointCondition::WhenOp(_) => {}
        }
    }

    fn handle_breakpoints_after_execution(&mut self) {
        let current_address = self.registers.get_pc();
        let Some(condition) = self.breakpoints.get(&current_address) else {
            return;
        };
        match condition {
            BreakPointCondition::RunInto => {}
            BreakPointCondition::AfterRun => self.paused = true,
            BreakPointCondition::WhenOp(opcode) => {
                if self.op_pipeline.opcode == *opcode {
                    self.paused = true;
                }
            }
        }
    }

    fn print_doctor_status(&self, bus: &impl BusReader) -> Result<String, ExecuteError> {
        let byte1 = bus.read_byte(self.registers.get_pc())?;
        let byte2 = bus.read_byte(self.registers.get_pc() + 1)?;
        let byte3 = bus.read_byte(self.registers.get_pc() + 2)?;
        let byte4 = bus.read_byte(self.registers.get_pc() + 3)?;
        Ok(format!(
            "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
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
        ))
    }

    const fn calculate_offset_pc(&self, offset: i16) -> Result<u16, ExecuteError> {
        let pc = self.registers.get_pc();
        match pc.checked_add_signed(offset) {
            Some(next_pc) => Ok(next_pc),
            None => Err(ExecuteError::OutOfBoundsPc { base: pc, offset }),
        }
    }

    pub fn step(&mut self, bus: &mut MemoryBus) -> Result<usize, ExecuteError> {
        if self.paused {
            return Ok(0);
        }

        // Fetch instruction
        let current_address = self.registers.get_pc();

        self.handle_breakpoints_before_execution();

        if self.paused {
            return Ok(0);
        }

        // Not currently executing an instruction
        if self.op_pipeline.queue.is_empty() {
            // CPU is halted, only wake up when there are pending interrupts.
            if self.is_halted {
                self.is_halted = !bus.is_pending_interrupts();
                return Ok(1);
            }

            // If the Interrupt Master Enable is set
            if let Some(interrupt) = bus.get_pending_interrupt()
                && self.ime
            {
                bus.reset_interrupt_request(interrupt);
                self.ime = false;
                Instruction::encode_interrupt_request(
                    &mut self.op_pipeline.queue,
                    interrupt.to_address(),
                );
            } else {
                let opcode = bus.read_byte(current_address)?;
                let inst = if self.is_prefix {
                    self.op_pipeline.opcode = 0xCB00 | u16::from(opcode);
                    self.is_prefix = false;
                    Instruction::decode_prefix(opcode)
                } else {
                    self.op_pipeline.opcode = u16::from(opcode);
                    Instruction::decode_no_prefix(opcode)
                };
                let mut it = bus
                    .iter_from(current_address + 1)
                    .enumerate()
                    .map(|(i, val)| (i + current_address as usize + 1 + i, val));
                let mut display_str = format!("{current_address:#06X}: ");
                inst.format(&mut display_str, &mut it, opcode).unwrap();
                tracing::trace!("{display_str}");

                inst.decompose(&mut self.op_pipeline.queue);
                self.op_pipeline.address = current_address;
            }
        }

        // Clear out ops until yield
        while let Some(op) = self.op_pipeline.queue.pop_front() {
            if self.debug_flag {
                tracing::trace!("\t\tBEFORE");
                tracing::trace!(
                    "\t\tAF: {:#06X} |Z={},N={},H={},C={}|, BC: {:#06X}, DE: {:#06X}, HL: {:#06X}, SP: {:#06X}, PC: {:#06X} |TMP1 = {:#04X}, ADDR = {:#06X}| (conditional_mode = {:?})\n",
                    self.registers.get_af(),
                    self.registers.get_zero_flag(),
                    self.registers.get_subtraction_flag(),
                    self.registers.get_half_carry_flag(),
                    self.registers.get_carry_flag(),
                    self.registers.get_bc(),
                    self.registers.get_de(),
                    self.registers.get_hl(),
                    self.registers.get_sp(),
                    self.registers.get_pc(),
                    self.tmp1,
                    self.addr,
                    self.conditional_mode,
                );
            }
            if let Some(false) = self.conditional_mode {
                tracing::trace!(
                    "\t\tSkipping uop ({:#06X}): {op:?} |Flags = {:08b}|...",
                    self.op_pipeline.address,
                    self.registers.get_f()
                );
                if let MicroOp::EndCondition = op {
                    self.conditional_mode = None;
                    continue;
                }
            } else {
                tracing::trace!(
                    "\t\tExecuting uop ({:#06X}): {op:?} |Flags = {:08b}|",
                    self.op_pipeline.address,
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
                    MicroOp::SetIME => self.ime = true,
                    MicroOp::Ei => self.set_ime = true,
                    MicroOp::Di => self.ime = false,
                    MicroOp::Daa => {
                        let _ = self.daa();
                    }
                    MicroOp::Invalid => {
                        return Err(ExecuteError::InvalidOpcode);
                    }
                    MicroOp::Scf => {
                        let _ = self.scf();
                    }
                    MicroOp::Ccf => {
                        let _ = self.ccf();
                    }
                    MicroOp::Cpl => {
                        let _ = self.cpl();
                    }
                    MicroOp::FetchPrefix => {
                        self.is_prefix = true;
                    }
                    MicroOp::ReadReg8(reg) => {
                        self.tmp1 = self.registers.get_reg8(reg);
                    }
                    MicroOp::WriteReg8(reg) => {
                        self.registers.set_reg8(reg, self.tmp1);
                    }
                    MicroOp::RotateLeftCarry => {
                        self.tmp1 = self.alu_rotate_left(self.tmp1, WithCarry::Yes);
                    }
                    MicroOp::RotateLeft => {
                        self.tmp1 = self.alu_rotate_left(self.tmp1, WithCarry::No);
                    }
                    MicroOp::RotateRightCarry => {
                        self.tmp1 = self.alu_rotate_right(self.tmp1, WithCarry::Yes);
                    }
                    MicroOp::RotateRight => {
                        self.tmp1 = self.alu_rotate_right(self.tmp1, WithCarry::No);
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
                        self.registers.set_pc(self.addr);
                    }
                    MicroOp::ReadByteFromMem => {
                        self.tmp1 = bus.read_byte(self.addr)?;
                    }
                    MicroOp::ReadByteFromImm => {
                        self.tmp1 = bus.read_byte(self.registers.get_pc())?;
                        self.registers
                            .set_pc(self.registers.get_pc().wrapping_add(1));
                    }
                    MicroOp::WriteByteIntoMem => {
                        bus.write_byte(self.addr, self.tmp1)?;
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
                        self.tmp1 = (self.addr & 0x00FF) as u8;
                    }
                    MicroOp::LoadHiAddr => {
                        self.tmp1 = ((self.addr & 0xFF00) >> 8) as u8;
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
                        self.addr = self.alu_add16_signed(self.addr, self.tmp1 as i8);
                    }
                    MicroOp::AluAddU16I8NoFlags => {
                        self.addr = self.addr.wrapping_add_signed(i16::from(self.tmp1 as i8));
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
                        self.tmp1 = self.alu_rotate_left(self.tmp1, WithCarry::Yes);
                    }
                    MicroOp::AluRrc8 => {
                        self.tmp1 = self.alu_rotate_right(self.tmp1, WithCarry::Yes);
                    }
                    MicroOp::AluRl8 => {
                        self.tmp1 = self.alu_rotate_left(self.tmp1, WithCarry::No);
                    }
                    MicroOp::AluRr8 => {
                        let old_value = self.tmp1;
                        self.tmp1 = self.alu_rotate_right(self.tmp1, WithCarry::Yes);
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
                        bus.write_byte(self.registers.get_sp(), self.tmp1)?;
                    }
                    MicroOp::ReadByteFromSP => {
                        self.tmp1 = bus.read_byte(self.registers.get_sp())?;
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

        // Enable IME
        if self.op_pipeline.queue.is_empty() && self.set_ime {
            self.ime = true;
            self.set_ime = false;
        }

        if self.debug_flag {
            tracing::trace!(
                "AF: {:#06X}, BC: {:#06X}, DE: {:#06X}, HL: {:#06X}, SP: {:#06X}, PC: {:#06X} |TMP1 = {:#04X}, ADDR = {:#06X}| (conditional_mode = {:?})\n",
                self.registers.get_af(),
                self.registers.get_bc(),
                self.registers.get_de(),
                self.registers.get_hl(),
                self.registers.get_sp(),
                self.registers.get_pc(),
                self.tmp1,
                self.addr,
                self.conditional_mode,
            );
        }

        #[expect(
            clippy::print_stdout,
            reason = "Need to print to stdout for certain debugging uses."
        )]
        if !self.debug_flag && self.op_pipeline.queue.is_empty() && !self.op_pipeline.is_prefix_op()
        {
            println!("{}", self.print_doctor_status(bus)?);
        }

        if self.op_pipeline.queue.is_empty() {
            self.handle_breakpoints_after_execution();
        }

        Ok(1)
    }

    pub fn execute(
        &mut self,
        inst: Instruction,
        bus: &mut MemoryBus,
    ) -> Result<(u16, usize), ExecuteError> {
        let (mut next_pc, mut num_cycles) = if self.is_halted {
            // Stay halted but increment cycle count by an M state
            let (next_pc, num_cycles) = (self.registers.get_pc(), ONE_M_STATE);

            // CPU is halted and waiting for interrupts
            self.is_halted = !bus.is_pending_interrupts();
            (next_pc, num_cycles)
        } else {
            let (next_pc, num_cycles) = self.execute_raw(inst, bus)?;
            let next_pc = match next_pc {
                NextPc::Relative(offset) => self.calculate_offset_pc(offset)?,
                NextPc::Absolute(next_pc) => next_pc,
            };
            (next_pc, num_cycles)
        };
        // If the Interrupt Master Enable is set
        if self.ime {
            // // Handle interrupts
            let (next_interrupt_pc, interrupt_cycles) = self.handle_interrupts(next_pc, bus)?;
            next_pc = next_interrupt_pc;
            num_cycles += interrupt_cycles;
            self.set_ime = false;
        }

        // Enable IME
        if self.set_ime {
            self.ime = true;
        }

        Ok((next_pc, num_cycles))
    }

    fn execute_raw(
        &mut self,
        inst: Instruction,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        match inst {
            Instruction::Nop => Ok((NextPc::Relative(1), ONE_M_STATE)),
            Instruction::Prefix => {
                self.is_prefix = true;
                Ok((NextPc::Relative(1), ONE_M_STATE))
            }
            Instruction::Halt => Ok(self.halt()),
            Instruction::Stop => {
                // TODO(pavyamsiri): Stop is not halt but it is more complicated so leave it until later
                Ok(self.halt())
            }
            Instruction::Invalid => Err(ExecuteError::InvalidOpcode),
            Instruction::Ei => Ok(self.ei()),
            Instruction::Di => Ok(self.di()),
            Instruction::Daa => Ok(self.daa()),
            Instruction::Scf => Ok(self.scf()),
            Instruction::Ccf => Ok(self.ccf()),
            Instruction::Cpl => Ok(self.cpl()),
            Instruction::Rlca => {
                self.rotate_left_reg8(Reg8::A, WithCarry::No);
                self.registers.set_zero_flag(false);
                self.registers.set_subtraction_flag(false);
                self.registers.set_half_carry_flag(false);
                Ok((NextPc::Relative(1), ONE_M_STATE))
            }
            Instruction::Rla => {
                self.rotate_left_reg8(Reg8::A, WithCarry::Yes);
                self.registers.set_zero_flag(false);
                self.registers.set_subtraction_flag(false);
                self.registers.set_half_carry_flag(false);
                Ok((NextPc::Relative(1), ONE_M_STATE))
            }
            Instruction::Rrca => {
                self.rotate_right_reg8(Reg8::A, WithCarry::No);
                self.registers.set_zero_flag(false);
                self.registers.set_subtraction_flag(false);
                self.registers.set_half_carry_flag(false);
                Ok((NextPc::Relative(1), ONE_M_STATE))
            }
            Instruction::Rra => {
                self.rotate_right_reg8(Reg8::A, WithCarry::Yes);
                self.registers.set_zero_flag(false);
                self.registers.set_subtraction_flag(false);
                self.registers.set_half_carry_flag(false);
                Ok((NextPc::Relative(1), ONE_M_STATE))
            }
            Instruction::LdReg8Reg8 { dst, src } => Ok(self.ld_reg8_reg8(dst, src)),
            Instruction::LdReg8Mem16 { dst, src } => self.ld_reg8_mem16(dst, src, bus),
            Instruction::LdReg8Imm8 { dst } => self.ld_reg8_imm8(dst, bus),
            Instruction::LdMem16Reg8 { dst, src } => self.ld_mem16_reg8(dst, src, bus),
            Instruction::LdMem16Imm8 { dst } => self.ld_mem16_imm8(dst, bus),
            Instruction::LdAdr8Reg8 { src } => self.ld_adr8_reg8(src, bus),
            Instruction::LdReg8Adr8 { dst } => self.ld_reg8_adr8(dst, bus),
            Instruction::LdMem8Reg8 { dst, src } => self.ld_mem8_reg8(dst, src, bus),
            Instruction::LdReg8Mem8 { dst, src } => self.ld_reg8_mem8(dst, src, bus),
            Instruction::LdAdr16Reg8 { src } => self.ld_adr16_reg8(src, bus),
            Instruction::LdReg8Adr16 { dst } => self.ld_reg8_adr16(dst, bus),
            Instruction::LdIncMem16Reg8 { dst, src } => {
                self.ldi_mem16_reg8(dst, src, IncDirection::Inc, bus)
            }
            Instruction::LdDecMem16Reg8 { dst, src } => {
                self.ldi_mem16_reg8(dst, src, IncDirection::Dec, bus)
            }
            Instruction::LdIncReg8Mem16 { dst, src } => {
                self.ldi_reg8_mem16(dst, src, IncDirection::Inc, bus)
            }
            Instruction::LdDecReg8Mem16 { dst, src } => {
                self.ldi_reg8_mem16(dst, src, IncDirection::Dec, bus)
            }
            Instruction::LdReg16Imm16 { dst } => self.ld_reg16_imm16(dst, bus),
            Instruction::LdAdr16Reg16 { src } => self.ld_adr16_reg16(src, bus),
            Instruction::LdReg16Reg16 { dst, src } => Ok(self.ld_reg16_reg16(dst, src)),
            Instruction::LdReg16Off8 { dst } => self.ld_reg16_off8(dst, bus),
            Instruction::IncReg8 { reg } => Ok(self.inc_reg8(reg)),
            Instruction::DecReg8 { reg } => Ok(self.dec_reg8(reg)),
            Instruction::IncMem16 { reg } => self.inc_mem16(reg, bus),
            Instruction::DecMem16 { reg } => self.dec_mem16(reg, bus),
            Instruction::IncReg16 { reg } => Ok(self.inc_reg16(reg)),
            Instruction::DecReg16 { reg } => Ok(self.dec_reg16(reg)),
            Instruction::AddReg16Reg16 { dst, src } => Ok(self.add_reg16_reg16(dst, src)),
            Instruction::AddReg16Off8 { dst } => self.add_reg16_off8(dst, bus),
            Instruction::PushReg16 { src } => self.push_reg16(src, bus),
            Instruction::PopReg16 { dst } => self.pop_reg16(dst, bus),
            Instruction::Jr => self.jump_relative(None, bus),
            Instruction::Jrc { condition } => self.jump_relative(Some(condition), bus),
            Instruction::Jp => self.jump_absolute(None, bus),
            Instruction::Jpc { condition } => self.jump_absolute(Some(condition), bus),
            Instruction::JpReg16 { reg } => Ok(self.jump_absolute_reg16(reg)),
            Instruction::Ret => self.ret(None, false, bus),
            Instruction::Retc { condition } => self.ret(Some(condition), false, bus),
            Instruction::Reti => self.ret(None, true, bus),
            Instruction::Call => self.call(None, bus),
            Instruction::Callc { condition } => self.call(Some(condition), bus),
            Instruction::Rst { target } => self.rst(target, bus),
            Instruction::AddReg8 { src } => Ok(self.add_reg8_reg8(Reg8::A, src, WithCarry::No)),
            Instruction::AddMem16 { src } => self.add_reg8_mem16(Reg8::A, src, WithCarry::No, bus),
            Instruction::AddImm8 => self.add_reg8_imm8(Reg8::A, WithCarry::No, bus),
            Instruction::AdcReg8 { src } => Ok(self.add_reg8_reg8(Reg8::A, src, WithCarry::Yes)),
            Instruction::AdcMem16 { src } => self.add_reg8_mem16(Reg8::A, src, WithCarry::Yes, bus),
            Instruction::AdcImm8 => self.add_reg8_imm8(Reg8::A, WithCarry::Yes, bus),
            Instruction::SubReg8 { src } => Ok(self.sub_reg8_reg8(Reg8::A, src, WithCarry::No)),
            Instruction::SubMem16 { src } => self.sub_reg8_mem16(Reg8::A, src, WithCarry::No, bus),
            Instruction::SubImm8 => self.sub_reg8_imm8(Reg8::A, WithCarry::No, bus),
            Instruction::SbcReg8 { src } => Ok(self.sub_reg8_reg8(Reg8::A, src, WithCarry::Yes)),
            Instruction::SbcMem16 { src } => self.sub_reg8_mem16(Reg8::A, src, WithCarry::Yes, bus),
            Instruction::SbcImm8 => self.sub_reg8_imm8(Reg8::A, WithCarry::Yes, bus),
            Instruction::AndReg8 { src } => Ok(self.and_reg8_reg8(Reg8::A, src)),
            Instruction::AndMem16 { src } => self.and_reg8_mem16(Reg8::A, src, bus),
            Instruction::AndImm8 => self.and_reg8_imm8(Reg8::A, bus),
            Instruction::XorReg8 { src } => Ok(self.xor_reg8_reg8(Reg8::A, src)),
            Instruction::XorMem16 { src } => self.xor_reg8_mem16(Reg8::A, src, bus),
            Instruction::XorImm8 => self.xor_reg8_imm8(Reg8::A, bus),
            Instruction::OrReg8 { src } => Ok(self.or_reg8_reg8(Reg8::A, src)),
            Instruction::OrMem16 { src } => self.or_reg8_mem16(Reg8::A, src, bus),
            Instruction::OrImm8 => self.or_reg8_imm8(Reg8::A, bus),
            Instruction::CpReg8 { src } => Ok(self.cp_reg8_reg8(Reg8::A, src)),
            Instruction::CpMem16 { src } => self.cp_reg8_mem16(Reg8::A, src, bus),
            Instruction::CpImm8 => self.cp_reg8_imm8(Reg8::A, bus),
            Instruction::RlcReg8 { reg } => Ok(self.rotate_left_reg8(reg, WithCarry::No)),
            Instruction::RlcMem16 { reg } => self.rotate_left_mem16(reg, WithCarry::No, bus),
            Instruction::RrcReg8 { reg } => Ok(self.rotate_right_reg8(reg, WithCarry::No)),
            Instruction::RrcMem16 { reg } => self.rotate_right_mem16(reg, WithCarry::No, bus),
            Instruction::RlReg8 { reg } => Ok(self.rotate_left_reg8(reg, WithCarry::Yes)),
            Instruction::RlMem16 { reg } => self.rotate_left_mem16(reg, WithCarry::Yes, bus),
            Instruction::RrReg8 { reg } => Ok(self.rotate_right_reg8(reg, WithCarry::Yes)),
            Instruction::RrMem16 { reg } => self.rotate_right_mem16(reg, WithCarry::Yes, bus),
            Instruction::SlaReg8 { reg } => Ok(self.shift_left_reg8(reg)),
            Instruction::SlaMem16 { reg } => self.shift_left_mem16(reg, bus),
            Instruction::SraReg8 { reg } => Ok(self.shift_right_reg8(reg, ShiftOption::Unchanged)),
            Instruction::SraMem16 { reg } => {
                self.shift_right_mem16(reg, ShiftOption::Unchanged, bus)
            }
            Instruction::SrlReg8 { reg } => Ok(self.shift_right_reg8(reg, ShiftOption::Reset)),
            Instruction::SrlMem16 { reg } => self.shift_right_mem16(reg, ShiftOption::Reset, bus),
            Instruction::SwapReg8 { reg } => Ok(self.swap_reg8(reg)),
            Instruction::SwapMem16 { reg } => self.swap_mem16(reg, bus),
            Instruction::BitReg8 { reg, bit } => Ok(self.bit_reg8(reg, bit)),
            Instruction::BitMem16 { reg, bit } => self.bit_mem16(reg, bit, bus),
            Instruction::ResReg8 { reg, bit } => Ok(self.reset_reg8(reg, bit)),
            Instruction::ResMem16 { reg, bit } => self.reset_mem16(reg, bit, bus),
            Instruction::SetReg8 { reg, bit } => Ok(self.set_reg8(reg, bit)),
            Instruction::SetMem16 { reg, bit } => self.set_mem16(reg, bit, bus),
        }
    }
}

// memory helpers
impl Cpu {
    fn read_byte_mem16(&self, reg: Reg16, bus: &MemoryBus) -> Result<u8, ReadByteError> {
        let address = self.registers.get_reg16(reg);
        bus.read_byte(address)
    }

    fn write_byte_mem16(
        &self,
        reg: Reg16,
        value: u8,
        bus: &mut MemoryBus,
    ) -> Result<(), WriteByteError> {
        let address = self.registers.get_reg16(reg);
        bus.write_byte(address, value)
    }

    fn read_byte_mem8(&self, reg: Reg8, bus: &MemoryBus) -> Result<u8, ReadByteError> {
        let address = self.read_mem8(reg);
        bus.read_byte(address)
    }

    fn write_byte_mem8(
        &self,
        reg: Reg8,
        value: u8,
        bus: &mut MemoryBus,
    ) -> Result<(), WriteByteError> {
        let address = self.read_mem8(reg);
        bus.write_byte(address, value)
    }

    fn read_imm8(&self, bus: &MemoryBus) -> Result<u8, ExecuteError> {
        let address = self.calculate_offset_pc(1)?;
        let value = bus.read_byte(address)?;
        Ok(value)
    }

    fn read_imm16(&self, bus: &MemoryBus) -> Result<u16, ExecuteError> {
        let lo_address = self.calculate_offset_pc(1)?;
        let hi_address = self.calculate_offset_pc(2)?;

        let lo = bus.read_byte(lo_address)?;
        let hi = bus.read_byte(hi_address)?;
        Ok((u16::from(hi) << 8) | u16::from(lo))
    }

    fn read_adr8(&self, bus: &MemoryBus) -> Result<u16, ExecuteError> {
        let offset = self.read_imm8(bus)?;
        let address = 0xFF00 | u16::from(offset);
        Ok(address)
    }

    const fn read_mem8(&self, reg: Reg8) -> u16 {
        let offset = self.registers.get_reg8(reg);
        0xFF00 | (offset as u16)
    }

    #[expect(clippy::cast_possible_wrap, reason = "this behaviour is expected.")]
    fn read_i8(&self, bus: &MemoryBus) -> Result<i8, ExecuteError> {
        let value = self.read_imm8(bus)?;
        Ok(value as i8)
    }
}

// interrupts
impl Cpu {
    fn handle_interrupts(
        &mut self,
        next_pc: u16,
        bus: &mut MemoryBus,
    ) -> Result<(u16, usize), ExecuteError> {
        if let Some(interrupt) = bus.get_pending_interrupt() {
            bus.reset_interrupt_request(interrupt);
            self.push_raw(next_pc, bus)?;
            Ok((interrupt.to_address(), FIVE_M_STATE))
        } else {
            Ok((next_pc, 0))
        }
    }
}

// miscellaneous
impl Cpu {
    const fn ei(&mut self) -> (NextPc, usize) {
        self.set_ime = true;
        (NextPc::Relative(1), ONE_M_STATE)
    }

    const fn di(&mut self) -> (NextPc, usize) {
        self.ime = false;
        (NextPc::Relative(1), ONE_M_STATE)
    }

    const fn scf(&mut self) -> (NextPc, usize) {
        self.registers.set_carry_flag(true);
        self.registers.set_half_carry_flag(false);
        self.registers.set_subtraction_flag(false);

        (NextPc::Relative(1), ONE_M_STATE)
    }

    const fn ccf(&mut self) -> (NextPc, usize) {
        self.registers
            .set_carry_flag(!self.registers.get_carry_flag());
        self.registers.set_half_carry_flag(false);
        self.registers.set_subtraction_flag(false);

        (NextPc::Relative(1), ONE_M_STATE)
    }

    const fn cpl(&mut self) -> (NextPc, usize) {
        // Flip bits of register A
        let current_value = self.registers.get_a();
        let new_value = current_value ^ 0xFF;

        // Set subtraction flag
        self.registers.set_subtraction_flag(true);
        // Set half-carry flag
        self.registers.set_half_carry_flag(true);

        self.registers.set_a(new_value);

        (NextPc::Relative(1), ONE_M_STATE)
    }

    const fn halt(&mut self) -> (NextPc, usize) {
        self.is_halted = true;

        (NextPc::Relative(1), ONE_M_STATE)
    }

    const fn daa(&mut self) -> (NextPc, usize) {
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
        (NextPc::Relative(1), ONE_M_STATE)
    }
}

// Stack operations
impl Cpu {
    fn pop_raw(&mut self, bus: &MemoryBus) -> Result<u16, ExecuteError> {
        // Read least significant byte from stack
        let lo = bus.read_byte(self.registers.get_sp())?;

        // Increment stack pointer
        self.inc_reg16(Reg16::SP);

        // Read least significant byte from stack
        let hi = bus.read_byte(self.registers.get_sp())?;

        // Increment stack pointer
        self.inc_reg16(Reg16::SP);

        // Compute and return popped value
        Ok((u16::from(hi) << 8) | u16::from(lo))
    }

    fn push_raw(&mut self, value: u16, bus: &mut MemoryBus) -> Result<(), ExecuteError> {
        let hi = ((value & 0xFF00) >> 8) as u8;
        let lo = (value & 0x00FF) as u8;
        // First decrement the stack pointer
        self.dec_reg16(Reg16::SP);

        // Then copy the most significant byte from the source to the stack
        match bus.write_byte(self.registers.get_sp(), hi) {
            Ok(()) => {}
            Err(err) => return Err(ExecuteError::BusWrite(err)),
        }

        // Decrement the stack pointer again
        self.dec_reg16(Reg16::SP);

        // And write the least significant byte to the stack
        match bus.write_byte(self.registers.get_sp(), lo) {
            Ok(()) => {}
            Err(err) => return Err(ExecuteError::BusWrite(err)),
        }

        Ok(())
    }

    fn pop_reg16(&mut self, dst: Reg16, bus: &MemoryBus) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.pop_raw(bus)?;
        self.registers.set_reg16(dst, value);

        Ok((NextPc::Relative(1), FOUR_M_STATE))
    }

    fn push_reg16(
        &mut self,
        src: Reg16,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.registers.get_reg16(src);
        self.push_raw(value, bus)?;

        Ok((NextPc::Relative(1), FOUR_M_STATE))
    }
}

#[cfg(test)]
mod tests {
    use gamedon_bus::MemoryBus;

    use crate::{BootRom, Cpu};

    #[test]
    fn daa_add_test() {
        // 0x27
        let mut cpu = Cpu::default();
        cpu.boot(BootRom::Dmg);
        let mut bus = MemoryBus::default();
        cpu.registers.set_a(0x10);
        cpu.registers.set_b(0x25);
        cpu.registers.set_pc(0xC000);
        bus.write_byte_raw(0xC000, 0x80).unwrap();
        bus.write_byte_raw(0xC001, 0x27).unwrap();
        cpu.step(&mut bus).unwrap();
        cpu.step(&mut bus).unwrap();

        assert_eq!(0xC002, cpu.registers.get_pc());

        // Result
        let byte = cpu.registers.get_a();
        assert_eq!(0x35, byte);

        // Check flags
        assert!(!cpu.registers.get_zero_flag());
        assert!(!cpu.registers.get_subtraction_flag());
        assert!(!cpu.registers.get_half_carry_flag());
        assert!(!cpu.registers.get_carry_flag());
    }

    #[test]
    fn daa_adc_test() {
        // 0x27
        let mut cpu = Cpu::default();
        cpu.boot(BootRom::Dmg);
        let mut bus = MemoryBus::default();
        cpu.registers.set_a(0x10);
        cpu.registers.set_b(0x25);
        cpu.registers.set_carry_flag(true);
        cpu.registers.set_pc(0xC000);
        bus.write_byte_raw(0xC000, 0x88).unwrap();
        bus.write_byte_raw(0xC001, 0x27).unwrap();
        cpu.step(&mut bus).unwrap();
        cpu.step(&mut bus).unwrap();

        assert_eq!(0xC002, cpu.registers.get_pc());

        // Result
        let byte = cpu.registers.get_a();
        assert_eq!(0x36, byte);

        // Check flags
        assert!(!cpu.registers.get_zero_flag());
        assert!(!cpu.registers.get_subtraction_flag());
        assert!(!cpu.registers.get_half_carry_flag());
        assert!(!cpu.registers.get_carry_flag());
    }

    #[test]
    fn daa_sub_test() {
        // 0x27
        let mut cpu = Cpu::default();
        cpu.boot(BootRom::Dmg);
        let mut bus = MemoryBus::default();
        cpu.registers.set_a(0x25);
        cpu.registers.set_b(0x10);
        cpu.registers.set_pc(0xC000);
        bus.write_byte_raw(0xC000, 0x90).unwrap();
        bus.write_byte_raw(0xC001, 0x27).unwrap();
        cpu.step(&mut bus).unwrap();
        cpu.step(&mut bus).unwrap();

        assert_eq!(0xC002, cpu.registers.get_pc());

        // Result
        let byte = cpu.registers.get_a();
        assert_eq!(0x15, byte);

        // Check flags
        assert!(!cpu.registers.get_zero_flag());
        assert!(cpu.registers.get_subtraction_flag());
        assert!(!cpu.registers.get_half_carry_flag());
        assert!(!cpu.registers.get_carry_flag());
    }

    #[test]
    fn daa_sbc_test() {
        // 0x27
        let mut cpu = Cpu::default();
        cpu.boot(BootRom::Dmg);
        let mut bus = MemoryBus::default();
        cpu.registers.set_a(0x25);
        cpu.registers.set_b(0x10);
        cpu.registers.set_carry_flag(true);
        cpu.registers.set_pc(0xC000);
        bus.write_byte_raw(0xC000, 0x98).unwrap();
        bus.write_byte_raw(0xC001, 0x27).unwrap();
        cpu.step(&mut bus).unwrap();
        cpu.step(&mut bus).unwrap();

        assert_eq!(0xC002, cpu.registers.get_pc());

        // Result
        let byte = cpu.registers.get_a();
        assert_eq!(0x14, byte);

        // Check flags
        assert!(!cpu.registers.get_zero_flag());
        assert!(cpu.registers.get_subtraction_flag());
        assert!(!cpu.registers.get_half_carry_flag());
        assert!(!cpu.registers.get_carry_flag());
    }
}
