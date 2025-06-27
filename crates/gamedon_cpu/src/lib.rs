mod load;

use gamedon_bus::{MemoryBus, ReadByteError, WriteByteError};
use gamedon_opcode::{Instruction, Reg8, Reg16, Registers};
use thiserror::Error;

/// The number of clock cycles in a machine cycle.
const ONE_M_STATE: usize = 4;
const TWO_M_STATE: usize = 8;
const THREE_M_STATE: usize = 12;
const FOUR_M_STATE: usize = 16;
const FIVE_M_STATE: usize = 20;

#[derive(Debug, Error)]
pub enum ExecuteError {
    #[error("Failed to read byte from bus: {0}")]
    BusRead(#[from] ReadByteError),
    #[error("Failed to write byte into bus: {0}")]
    BusWrite(#[from] WriteByteError),
    #[error("The next PC is not a valid 16-bit address: base = {base} + {offset} > 2^16 - 1")]
    OutOfBoundsPc { base: u16, offset: u16 },
}

enum NextPc {
    Relative(u16),
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

pub struct Cpu {
    registers: Registers,
    is_halted: bool,
    ime: bool,
    set_ime: bool,
}

// Executor
impl Cpu {
    const fn calculate_offset_pc(&self, offset: u16) -> Result<u16, ExecuteError> {
        let pc = self.registers.get_pc();
        match pc.checked_add(pc) {
            Some(next_pc) => Ok(next_pc),
            None => Err(ExecuteError::OutOfBoundsPc { base: pc, offset }),
        }
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
            self.is_halted = !bus.pending_interrupts();
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
            let (next_interrupt_pc, interrupt_cycles) = self.handle_interrupts(next_pc, bus);
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
            Instruction::Nop | Instruction::Prefix => Ok((NextPc::Relative(1), ONE_M_STATE)),
            Instruction::Halt => Ok(self.halt()),
            Instruction::Stop => {
                // TODO(pavyamsiri): Stop is not halt but it is more complicated so leave it until later
                Ok(self.halt())
            }
            Instruction::Ei => Ok(self.ei()),
            Instruction::Di => Ok(self.di()),
            Instruction::Daa => todo!(),
            Instruction::Scf => Ok(self.scf()),
            Instruction::Ccf => Ok(self.ccf()),
            Instruction::Cpl => Ok(self.cpl()),
            Instruction::Rlca => todo!(),
            Instruction::Rla => todo!(),
            Instruction::Rrca => todo!(),
            Instruction::Rra => todo!(),
            Instruction::Invalid => todo!("invalid instruction has specific behaviour"),
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
            Instruction::IncReg8 { reg } => todo!(),
            Instruction::DecReg8 { reg } => todo!(),
            Instruction::IncMem16 { reg } => todo!(),
            Instruction::DecMem16 { reg } => todo!(),
            Instruction::IncReg16 { reg } => todo!(),
            Instruction::DecReg16 { reg } => todo!(),
            Instruction::AddReg16Reg16 { dst, src } => todo!(),
            Instruction::AddReg16Off8 { dst } => todo!(),
            Instruction::PushReg16 { src } => todo!(),
            Instruction::PopReg16 { dst } => todo!(),
            Instruction::Jr => todo!(),
            Instruction::Jrc { condition } => todo!(),
            Instruction::Jp => todo!(),
            Instruction::Jpc { condition } => todo!(),
            Instruction::JpReg16 { reg } => todo!(),
            Instruction::Ret => todo!(),
            Instruction::Retc { condition } => todo!(),
            Instruction::Reti => todo!(),
            Instruction::Call => todo!(),
            Instruction::Callc { condition } => todo!(),
            Instruction::Rst { target } => todo!(),
            Instruction::AddReg8 { src } => todo!(),
            Instruction::AddMem16 { src } => todo!(),
            Instruction::AddImm8 => todo!(),
            Instruction::AdcReg8 { src } => todo!(),
            Instruction::AdcMem16 { src } => todo!(),
            Instruction::AdcImm8 => todo!(),
            Instruction::SubReg8 { src } => todo!(),
            Instruction::SubMem16 { src } => todo!(),
            Instruction::SubImm8 => todo!(),
            Instruction::SbcReg8 { src } => todo!(),
            Instruction::SbcMem16 { src } => todo!(),
            Instruction::SbcImm8 => todo!(),
            Instruction::AndReg8 { src } => todo!(),
            Instruction::AndMem16 { src } => todo!(),
            Instruction::AndImm8 => todo!(),
            Instruction::XorReg8 { src } => todo!(),
            Instruction::XorMem16 { src } => todo!(),
            Instruction::XorImm8 => todo!(),
            Instruction::OrReg8 { src } => todo!(),
            Instruction::OrMem16 { src } => todo!(),
            Instruction::OrImm8 => todo!(),
            Instruction::CpReg8 { src } => todo!(),
            Instruction::CpMem16 { src } => todo!(),
            Instruction::CpImm8 => todo!(),
            Instruction::RlcReg8 { reg } => todo!(),
            Instruction::RlcMem16 { reg } => todo!(),
            Instruction::RrcReg8 { reg } => todo!(),
            Instruction::RrcMem16 { reg } => todo!(),
            Instruction::RlReg8 { reg } => todo!(),
            Instruction::RlMem16 { reg } => todo!(),
            Instruction::RrReg8 { reg } => todo!(),
            Instruction::RrMem16 { reg } => todo!(),
            Instruction::SlaReg8 { reg } => todo!(),
            Instruction::SlaMem16 { reg } => todo!(),
            Instruction::SraReg8 { reg } => todo!(),
            Instruction::SraMem16 { reg } => todo!(),
            Instruction::SwapReg8 { reg } => todo!(),
            Instruction::SwapMem16 { reg } => todo!(),
            Instruction::SrlReg8 { reg } => todo!(),
            Instruction::SrlMem16 { reg } => todo!(),
            Instruction::BitReg8 { reg, bit } => todo!(),
            Instruction::BitMem16 { reg, bit } => todo!(),
            Instruction::ResReg8 { reg, bit } => todo!(),
            Instruction::ResMem16 { reg, bit } => todo!(),
            Instruction::SetReg8 { reg, bit } => todo!(),
            Instruction::SetMem16 { reg, bit } => todo!(),
        }
    }
}

// memory helpers
impl Cpu {
    const fn read_byte_mem16(&self, reg: Reg16, bus: &MemoryBus) -> Result<u8, ReadByteError> {
        let address = self.registers.get_reg16(reg);
        bus.read_byte(address)
    }

    const fn write_byte_mem16(
        &self,
        reg: Reg16,
        value: u8,
        bus: &mut MemoryBus,
    ) -> Result<(), WriteByteError> {
        let address = self.registers.get_reg16(reg);
        bus.write_byte(address, value)
    }

    const fn read_byte_mem8(&self, reg: Reg8, bus: &MemoryBus) -> Result<u8, ReadByteError> {
        let address = self.read_mem8(reg);
        bus.read_byte(address)
    }

    const fn write_byte_mem8(
        &self,
        reg: Reg8,
        value: u8,
        bus: &mut MemoryBus,
    ) -> Result<(), WriteByteError> {
        let address = self.read_mem8(reg);
        bus.write_byte(address, value)
    }

    const fn read_imm8(&self, bus: &MemoryBus) -> Result<u8, ExecuteError> {
        let address = match self.calculate_offset_pc(1) {
            Ok(address) => address,
            Err(err) => return Err(err),
        };
        match bus.read_byte(address) {
            Ok(byte) => Ok(byte),
            Err(err) => Err(ExecuteError::BusRead(err)),
        }
    }

    const fn read_imm16(&self, bus: &MemoryBus) -> Result<u16, ExecuteError> {
        let lo_address = match self.calculate_offset_pc(1) {
            Ok(lo) => lo,
            Err(err) => return Err(err),
        };
        let hi_address = match self.calculate_offset_pc(2) {
            Ok(hi) => hi,
            Err(err) => return Err(err),
        };

        let lo = match bus.read_byte(lo_address) {
            Ok(lo) => lo,
            Err(err) => return Err(ExecuteError::BusRead(err)),
        };
        let hi = match bus.read_byte(hi_address) {
            Ok(hi) => hi,
            Err(err) => return Err(ExecuteError::BusRead(err)),
        };
        Ok(((hi as u16) << 8) | (lo as u16))
    }

    const fn read_adr8(&self, bus: &MemoryBus) -> Result<u16, ExecuteError> {
        let offset = match self.read_imm8(bus) {
            Ok(offset) => offset,
            Err(err) => return Err(err),
        };
        let address = 0xFF00 | (offset as u16);
        Ok(address)
    }

    const fn read_mem8(&self, reg: Reg8) -> u16 {
        let offset = self.registers.get_reg8(reg);
        0xFF00 | (offset as u16)
    }

    #[expect(clippy::cast_possible_wrap, reason = "this behaviour is expected.")]
    const fn read_i8(&self, bus: &MemoryBus) -> Result<i8, ExecuteError> {
        let value = match self.read_imm8(bus) {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        Ok(value as i8)
    }
}

// interrupts
impl Cpu {
    fn handle_interrupts(&mut self, next_pc: u16, bus: &mut MemoryBus) -> (u16, usize) {
        // if bus.pending_vblank_interrupts() {
        //     bus.set_vblank_interrupt_request(false);
        //     self.ime = false;
        //     self.push(bus, next_pc);
        //     return (0x40, 5 * M_STATE);
        // } else if bus.pending_lcd_stat_interrupts() {
        //     tracing::trace!("Servicing LCD STAT interrupt!");
        //     bus.set_lcd_stat_interrupt_request(false);
        //     self.ime = false;
        //     self.push(bus, next_pc);
        //     return (0x48, 5 * M_STATE);
        // } else if bus.pending_timer_interrupts() {
        //     bus.set_timer_interrupt_request(false);
        //     self.ime = false;
        //     self.push(bus, next_pc);
        //     return (0x50, 5 * M_STATE);
        // } else if bus.pending_serial_interrupts() {
        //     bus.set_serial_interrupt_request(false);
        //     self.ime = false;
        //     self.push(bus, next_pc);
        //     return (0x58, 5 * M_STATE);
        // } else if bus.pending_joypad_interrupts() {
        //     bus.set_joypad_interrupt_request(false);
        //     self.ime = false;
        //     self.push(bus, next_pc);
        //     return (0x60, 5 * M_STATE);
        // }
        (next_pc, 0)
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
}
