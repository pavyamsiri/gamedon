use crate::{
    Cpu, ExecuteError, FOUR_M_STATE, NextPc, ONE_M_STATE, THREE_M_STATE, TWO_M_STATE, WithCarry,
};
use gamedon_bus::MemoryBus;
use gamedon_opcode::{Reg8, Reg16};

#[derive(Debug, Clone, Copy)]
pub(crate) enum ShiftOption {
    Unchanged,
    Reset,
}

impl Cpu {
    pub(crate) fn alu_rotate_left(&mut self, value: u8, with_carry: WithCarry) -> u8 {
        let old_bit = value >> 7;
        let fill_bit = match with_carry {
            WithCarry::Yes => self.registers.get_carry_flag() as u8,
            WithCarry::No => old_bit,
        };
        let new_value = (value << 1) | fill_bit;

        // Store old bit 7 in carry flag
        self.registers.set_carry_flag(old_bit == 0x1);

        // Set zero flag if result is zero else reset
        self.registers.set_zero_flag(new_value == 0);

        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);
        // Reset half-carry flag
        self.registers.set_half_carry_flag(false);

        new_value
    }

    pub(crate) fn alu_rotate_right(&mut self, value: u8, with_carry: WithCarry) -> u8 {
        let old_bit = value << 7;
        let fill_bit = match with_carry {
            WithCarry::Yes => (self.registers.get_carry_flag() as u8) << 7,
            WithCarry::No => old_bit,
        };
        let new_value = (value >> 1) | fill_bit;

        // Store old bit 7 in carry flag
        self.registers.set_carry_flag(old_bit == 0x1);

        // Set zero flag if result is zero else reset
        self.registers.set_zero_flag(new_value == 0);

        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);
        // Reset half-carry flag
        self.registers.set_half_carry_flag(false);

        new_value
    }

    pub(crate) fn alu_shift_left(&mut self, value: u8) -> u8 {
        let old_bit = value >> 7;
        let new_value = (value << 1) & 0xFE;

        // Store old bit 7 in carry flag
        self.registers.set_carry_flag(old_bit == 0x1);

        // Set zero flag if result is zero else reset
        self.registers.set_zero_flag(new_value == 0);

        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);
        // Reset half-carry flag
        self.registers.set_half_carry_flag(false);

        new_value
    }

    pub(crate) fn alu_shift_right(&mut self, value: u8, leave: ShiftOption) -> u8 {
        let old_bit = value & 0x1;
        let fill_bit = match leave {
            ShiftOption::Unchanged => value & 0b1000_0000,
            ShiftOption::Reset => 0,
        };
        let new_value = (value >> 1) | fill_bit;

        // Store old bit 7 in carry flag
        self.registers.set_carry_flag(old_bit == 0x1);

        // Set zero flag if result is zero else reset
        self.registers.set_zero_flag(new_value == 0);

        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);
        // Reset half-carry flag
        self.registers.set_half_carry_flag(false);

        new_value
    }

    pub(crate) fn alu_swap(&mut self, value: u8) -> u8 {
        let new_value = ((value & 0x0F) << 4) | ((value & 0xF0) >> 4);

        // Set zero flag if result is zero else reset
        self.registers.set_zero_flag(new_value == 0);

        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);
        // Reset half-carry flag
        self.registers.set_half_carry_flag(false);
        // Reset carry flag
        self.registers.set_carry_flag(false);

        new_value
    }
}

impl Cpu {
    pub(crate) fn rotate_left_reg8(&mut self, reg: Reg8, with_carry: WithCarry) -> (NextPc, usize) {
        let value = self.registers.get_reg8(reg);
        let new_value = self.alu_rotate_left(value, with_carry);
        self.registers.set_reg8(reg, new_value);

        (NextPc::Relative(1), ONE_M_STATE)
    }

    pub(crate) fn rotate_left_mem16(
        &mut self,
        reg: Reg16,
        with_carry: WithCarry,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.read_byte_mem16(reg, bus)?;
        let new_value = self.alu_rotate_left(value, with_carry);
        self.write_byte_mem16(reg, new_value, bus)?;

        Ok((NextPc::Relative(1), THREE_M_STATE))
    }

    pub(crate) fn rotate_right_reg8(
        &mut self,
        reg: Reg8,
        with_carry: WithCarry,
    ) -> (NextPc, usize) {
        let value = self.registers.get_reg8(reg);
        let new_value = self.alu_rotate_right(value, with_carry);
        self.registers.set_reg8(reg, new_value);

        (NextPc::Relative(1), ONE_M_STATE)
    }

    pub(crate) fn rotate_right_mem16(
        &mut self,
        reg: Reg16,
        with_carry: WithCarry,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.read_byte_mem16(reg, bus)?;
        let new_value = self.alu_rotate_right(value, with_carry);
        self.write_byte_mem16(reg, new_value, bus)?;

        Ok((NextPc::Relative(1), THREE_M_STATE))
    }

    pub(crate) fn shift_left_reg8(&mut self, reg: Reg8) -> (NextPc, usize) {
        let value = self.registers.get_reg8(reg);
        let new_value = self.alu_shift_left(value);
        self.registers.set_reg8(reg, new_value);

        (NextPc::Relative(1), ONE_M_STATE)
    }

    pub(crate) fn shift_left_mem16(
        &mut self,
        reg: Reg16,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.read_byte_mem16(reg, bus)?;
        let new_value = self.alu_shift_left(value);
        self.write_byte_mem16(reg, new_value, bus)?;

        Ok((NextPc::Relative(1), THREE_M_STATE))
    }

    pub(crate) fn shift_right_reg8(&mut self, reg: Reg8, leave: ShiftOption) -> (NextPc, usize) {
        let value = self.registers.get_reg8(reg);
        let new_value = self.alu_shift_right(value, leave);
        self.registers.set_reg8(reg, new_value);

        (NextPc::Relative(1), ONE_M_STATE)
    }

    pub(crate) fn shift_right_mem16(
        &mut self,
        reg: Reg16,
        leave: ShiftOption,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.read_byte_mem16(reg, bus)?;
        let new_value = self.alu_shift_right(value, leave);
        self.write_byte_mem16(reg, new_value, bus)?;

        Ok((NextPc::Relative(1), THREE_M_STATE))
    }

    pub(crate) fn swap_reg8(&mut self, reg: Reg8) -> (NextPc, usize) {
        let value = self.registers.get_reg8(reg);
        let new_value = self.alu_swap(value);
        self.registers.set_reg8(reg, new_value);

        (NextPc::Relative(1), ONE_M_STATE)
    }

    pub(crate) fn swap_mem16(
        &mut self,
        reg: Reg16,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.read_byte_mem16(reg, bus)?;
        let new_value = self.alu_swap(value);
        self.write_byte_mem16(reg, new_value, bus)?;

        Ok((NextPc::Relative(1), THREE_M_STATE))
    }
}
