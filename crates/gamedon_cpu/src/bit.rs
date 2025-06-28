use crate::{Cpu, ExecuteError, NextPc, ONE_M_STATE, THREE_M_STATE, TWO_M_STATE};
use gamedon_bus::MemoryBus;
use gamedon_opcode::{BitShift8, Reg8, Reg16};

// Raw ALU ops
impl Cpu {
    pub(crate) fn alu_bit(&mut self, value: u8, bit: BitShift8) {
        let bit_shift_amount = bit.get_shift_amount();

        let test_bit = (value >> bit_shift_amount) & 0x1;

        // Set zero flag if bit is not set.
        self.registers.set_zero_flag(test_bit == 0);

        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);
        // Set half-carry flag
        self.registers.set_half_carry_flag(true);
    }

    pub(crate) fn alu_reset(value: u8, bit: BitShift8) -> u8 {
        let bit_shift_amount = bit.get_shift_amount();

        let reset_bit_mask = !(0x1u8 << bit_shift_amount);

        value & reset_bit_mask
    }

    pub(crate) fn alu_set(value: u8, bit: BitShift8) -> u8 {
        let bit_shift_amount = bit.get_shift_amount();

        let set_bit_mask = 0x1u8 << bit_shift_amount;

        value | set_bit_mask
    }
}

impl Cpu {
    pub(crate) fn bit_reg8(&mut self, reg: Reg8, bit: BitShift8) -> (NextPc, usize) {
        let value = self.registers.get_reg8(reg);
        self.alu_bit(value, bit);

        (NextPc::Relative(1), ONE_M_STATE)
    }

    pub(crate) fn bit_mem16(
        &mut self,
        reg: Reg16,
        bit: BitShift8,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.read_byte_mem16(reg, bus)?;
        self.alu_bit(value, bit);

        Ok((NextPc::Relative(1), TWO_M_STATE))
    }

    pub(crate) fn reset_reg8(&mut self, reg: Reg8, bit: BitShift8) -> (NextPc, usize) {
        let value = self.registers.get_reg8(reg);
        let new_value = Self::alu_reset(value, bit);
        self.registers.set_reg8(reg, new_value);

        (NextPc::Relative(1), ONE_M_STATE)
    }

    pub(crate) fn reset_mem16(
        &mut self,
        reg: Reg16,
        bit: BitShift8,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.read_byte_mem16(reg, bus)?;
        let new_value = Self::alu_reset(value, bit);
        self.write_byte_mem16(reg, new_value, bus)?;

        Ok((NextPc::Relative(1), THREE_M_STATE))
    }

    pub(crate) fn set_reg8(&mut self, reg: Reg8, bit: BitShift8) -> (NextPc, usize) {
        let value = self.registers.get_reg8(reg);
        let new_value = Self::alu_set(value, bit);
        self.registers.set_reg8(reg, new_value);

        (NextPc::Relative(1), ONE_M_STATE)
    }

    pub(crate) fn set_mem16(
        &mut self,
        reg: Reg16,
        bit: BitShift8,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.read_byte_mem16(reg, bus)?;
        let new_value = Self::alu_set(value, bit);
        self.write_byte_mem16(reg, new_value, bus)?;

        Ok((NextPc::Relative(1), THREE_M_STATE))
    }
}
