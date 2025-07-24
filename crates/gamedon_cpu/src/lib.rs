mod alu;
mod bit;
mod boot;
mod jump;
mod rotate;

use std::collections::{BTreeMap, VecDeque};

use alu::WithCarry;
use gamedon_bus::{BusReader, BusWriter, MemoryBus, ReadByteError, WriteByteError};
pub use gamedon_opcode::Instruction;
use gamedon_opcode::{MicroOp, Registers};
use rotate::ShiftOption;
use thiserror::Error;

pub use boot::BootRom;

type Map<K, V> = BTreeMap<K, V>;

#[derive(Debug, Error)]
pub enum ExecuteError {
    #[error("{0}")]
    BusRead(#[from] ReadByteError),
    #[error("{0}")]
    BusWrite(#[from] WriteByteError),
    #[error("The next PC is not a valid 16-bit address: base = {base} + {offset} > 2^16 - 1")]
    OutOfBoundsPc { base: u16, offset: i16 },
    #[error("Attempted to run an invalid opcode.")]
    InvalidOpcode,
}

pub struct StepResult {
    pub kind: StepResultKind,
    pub should_pause: bool,
}

pub enum StepResultKind {
    Completed {
        address: u16,
        opcode: u16,
        inst: Instruction,
    },
    Incomplete,
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

#[derive(Debug, Default)]
struct OpPipeline {
    queue: VecDeque<MicroOp>,
    address: u16,
    opcode: u16,
    instruction: Instruction,
}

#[derive(Debug, Default)]
pub struct Cpu {
    breakpoints: Map<u16, BreakPointCondition>,
    conditional_mode: Option<bool>,
    ime: bool,

    is_halted: bool,

    is_prefix: bool,
    op_pipeline: OpPipeline,

    pub registers: Registers,

    set_ime: bool,

    // Internal registers
    pub tmp1: u8,
    pub addr: u16,
}

// Debug
impl Cpu {
    pub fn add_breakpoint(&mut self, address: u16, condition: BreakPointCondition) {
        self.breakpoints.insert(address, condition);
    }

    pub const fn get_last_op(&self) -> (u16, u16, Instruction) {
        (
            self.op_pipeline.address,
            self.op_pipeline.opcode,
            self.op_pipeline.instruction,
        )
    }
}

// Executor
impl Cpu {
    fn handle_breakpoints_before_execution(&self) -> bool {
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

    fn handle_breakpoints_after_decode(&self) -> bool {
        let current_address = self.registers.get_pc();
        let Some(condition) = self.breakpoints.get(&current_address) else {
            return false;
        };

        match condition {
            BreakPointCondition::WhenOp(opcode) => {
                if self.op_pipeline.opcode == *opcode {
                    tracing::debug!("Hit run into break point when opcode is {opcode:#06X}.");
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

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

    pub fn step(
        &mut self,
        bus: &mut MemoryBus,
        skip_breakpoints: bool,
    ) -> Result<StepResult, ExecuteError> {
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

        if !skip_breakpoints && self.handle_breakpoints_before_execution() {
            return Ok(StepResult {
                kind: StepResultKind::Nop,
                should_pause: true,
            });
        }

        let mut should_pause = false;

        // Not currently executing an instruction
        if self.op_pipeline.queue.is_empty() {
            // CPU is halted, only wake up when there are pending interrupts.
            if self.is_halted {
                self.is_halted = !bus.is_pending_interrupts();
                return Ok(StepResult {
                    kind: StepResultKind::Incomplete,
                    should_pause: false,
                });
            }

            if self.set_ime {
                self.ime = true;
                self.set_ime = false;
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
                let opcode = read_byte!(bus, current_address, &mut should_pause);
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
                self.op_pipeline.instruction = inst;

                if !skip_breakpoints && self.handle_breakpoints_after_decode() {
                    return Ok(StepResult {
                        kind: StepResultKind::Nop,
                        should_pause: true,
                    });
                }
            }
        }

        // Clear out ops until yield
        while let Some(op) = self.op_pipeline.queue.pop_front() {
            if let Some(false) = self.conditional_mode {
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
                        should_pause |= self.addr == self.op_pipeline.address;
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
                        self.tmp1 = self.alu_rotate_left(self.tmp1, WithCarry::No);
                    }
                    MicroOp::AluRrc8 => {
                        self.tmp1 = self.alu_rotate_right(self.tmp1, WithCarry::No);
                    }
                    MicroOp::AluRl8 => {
                        self.tmp1 = self.alu_rotate_left(self.tmp1, WithCarry::Yes);
                    }
                    MicroOp::AluRr8 => {
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

        if self.op_pipeline.queue.is_empty() {
            should_pause |= self.handle_breakpoints_after_execution();
            let should_pause = should_pause && !skip_breakpoints;
            Ok(StepResult {
                kind: StepResultKind::Completed {
                    address: self.op_pipeline.address,
                    opcode: self.op_pipeline.opcode,
                    inst: self.op_pipeline.instruction,
                },
                should_pause,
            })
        } else {
            let should_pause = should_pause && !skip_breakpoints;
            Ok(StepResult {
                kind: StepResultKind::Incomplete,
                should_pause,
            })
        }
    }
}

// miscellaneous
impl Cpu {
    const fn scf(&mut self) {
        self.registers.set_carry_flag(true);
        self.registers.set_half_carry_flag(false);
        self.registers.set_subtraction_flag(false);
    }

    const fn ccf(&mut self) {
        self.registers
            .set_carry_flag(!self.registers.get_carry_flag());
        self.registers.set_half_carry_flag(false);
        self.registers.set_subtraction_flag(false);
    }

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

#[cfg(test)]
mod tests {
    use gamedon_bus::{BusReader, BusWriter, MemoryBus};

    use crate::{BootRom, Cpu, StepResultKind};

    #[test]
    fn ei_delay_behavior() {
        // Initialize CPU and bus
        let mut cpu = Cpu::default();
        cpu.boot(BootRom::Dmg);
        let mut bus = MemoryBus::default();

        // Test program: EI -> NOP (IME should enable AFTER NOP)
        bus.write_byte(0xC000, 0xFB).unwrap(); // EI
        bus.write_byte(0xC001, 0x00).unwrap(); // NOP
        cpu.registers.set_pc(0xC000);

        // Step 1: Execute EI (IME should NOT be enabled yet)
        let result = cpu.step(&mut bus, true).unwrap();
        assert!(matches!(result.kind, StepResultKind::Completed { .. }));
        assert!(!cpu.ime, "IME should NOT be enabled immediately after EI");

        // Step 2: Execute NOP (IME should now be enabled)
        let result = cpu.step(&mut bus, true).unwrap();
        assert!(matches!(result.kind, StepResultKind::Completed { .. }));
        assert!(
            cpu.ime,
            "IME should be enabled after the next instruction (NOP)"
        );

        // Verify PC advanced correctly
        assert_eq!(0xC002, cpu.registers.get_pc());
    }

    #[test]
    fn ei_delay_with_interrupt() {
        // Initialize CPU and bus
        let mut cpu = Cpu::default();
        cpu.boot(BootRom::Dmg);
        let mut bus = MemoryBus::default();

        // Test program: EI -> HALT
        bus.write_byte(0xC000, 0xFB).unwrap(); // EI
        bus.write_byte(0xC001, 0x76).unwrap(); // HALT
        cpu.registers.set_pc(0xC000);
        cpu.registers.set_sp(0xFFFE);

        // Set up VBlank interrupt
        bus.write_byte(0xFF0F, 0x01).unwrap(); // IF: VBlank requested
        bus.write_byte(0xFFFF, 0x01).unwrap(); // IE: VBlank enabled

        // Execute EI
        let _ = cpu.step(&mut bus, true).unwrap();

        // Execute HALT (should increment PC before halting)
        let _ = cpu.step(&mut bus, true).unwrap();

        // Process interrupt (may take multiple steps)
        while cpu.registers.get_pc() != 0x0040 {
            let _ = cpu.step(&mut bus, true).unwrap();
        }

        // ---- Verify Correct Game Boy Behavior ----
        assert_eq!(0x0040, cpu.registers.get_pc(), "Should jump to handler");
        assert_eq!(0xFFFC, cpu.registers.get_sp(), "SP should decrement by 2");

        // This is the crucial fix - we EXPECT 0xC002 to be pushed:
        assert_eq!(
            0xC0,
            bus.read_byte(0xFFFD).unwrap(),
            "High byte of next instruction"
        );
        assert_eq!(
            0x02,
            bus.read_byte(0xFFFC).unwrap(),
            "Low byte of next instruction"
        );

        assert_eq!(0x00, bus.read_byte(0xFF0F).unwrap(), "IF should be cleared");
        assert!(!cpu.ime, "IME should be disabled during handler");
    }

    #[test]
    fn daa_add_test() {
        // 0x27
        let mut cpu = Cpu::default();
        cpu.boot(BootRom::Dmg);
        let mut bus = MemoryBus::default();
        cpu.registers.set_a(0x10);
        cpu.registers.set_b(0x25);
        cpu.registers.set_pc(0xC000);
        bus.write_byte(0xC000, 0x80).unwrap();
        bus.write_byte(0xC001, 0x27).unwrap();
        cpu.step(&mut bus, true).unwrap();
        cpu.step(&mut bus, true).unwrap();

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
        bus.write_byte(0xC000, 0x88).unwrap();
        bus.write_byte(0xC001, 0x27).unwrap();
        cpu.step(&mut bus, true).unwrap();
        cpu.step(&mut bus, true).unwrap();

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
        bus.write_byte(0xC000, 0x90).unwrap();
        bus.write_byte(0xC001, 0x27).unwrap();
        cpu.step(&mut bus, true).unwrap();
        cpu.step(&mut bus, true).unwrap();

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
        bus.write_byte(0xC000, 0x98).unwrap();
        bus.write_byte(0xC001, 0x27).unwrap();
        cpu.step(&mut bus, true).unwrap();
        cpu.step(&mut bus, true).unwrap();

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
