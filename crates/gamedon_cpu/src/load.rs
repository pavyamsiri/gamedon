use crate::{
    Cpu, ExecuteError, FIVE_M_STATE, FOUR_M_STATE, IncDirection, NextPc, ONE_M_STATE,
    THREE_M_STATE, TWO_M_STATE,
};
use gamedon_bus::{BusReader, BusWriter, MemoryBus};
use gamedon_opcode::{Reg8, Reg16};

// 8-bit loads.
impl Cpu {
    pub(crate) fn ld_reg8_reg8(&mut self, dst: Reg8, src: Reg8) -> (NextPc, usize) {
        let value = self.registers.get_reg8(src);
        self.registers.set_reg8(dst, value);

        (NextPc::Relative(1), ONE_M_STATE)
    }

    pub(crate) fn ld_reg8_mem16(
        &mut self,
        dst: Reg8,
        src: Reg16,
        bus: &MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.read_byte_mem16(src, bus)?;
        self.registers.set_reg8(dst, value);

        Ok((NextPc::Relative(1), TWO_M_STATE))
    }

    pub(crate) fn ld_reg8_imm8(
        &mut self,
        dst: Reg8,
        bus: &MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.read_imm8(bus)?;
        self.registers.set_reg8(dst, value);
        Ok((NextPc::Relative(2), TWO_M_STATE))
    }

    pub(crate) fn ld_mem16_reg8(
        &mut self,
        dst: Reg16,
        src: Reg8,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.registers.get_reg8(src);
        self.write_byte_mem16(dst, value, bus)?;

        Ok((NextPc::Relative(1), TWO_M_STATE))
    }

    pub(crate) fn ld_mem16_imm8(
        &mut self,
        dst: Reg16,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.read_imm8(bus)?;
        self.write_byte_mem16(dst, value, bus)?;

        Ok((NextPc::Relative(2), TWO_M_STATE))
    }

    pub(crate) fn ld_adr8_reg8(
        &mut self,
        src: Reg8,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.registers.get_reg8(src);
        let address = self.read_adr8(bus)?;
        bus.write_byte(address, value)?;

        Ok((NextPc::Relative(2), THREE_M_STATE))
    }

    pub(crate) fn ld_reg8_adr8(
        &mut self,
        dst: Reg8,
        bus: &MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let address = self.read_adr8(bus)?;
        let value = bus.read_byte(address)?;
        self.registers.set_reg8(dst, value);

        Ok((NextPc::Relative(2), THREE_M_STATE))
    }

    pub(crate) fn ld_mem8_reg8(
        &mut self,
        dst: Reg8,
        src: Reg8,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.registers.get_reg8(src);
        self.write_byte_mem8(dst, value, bus)?;

        Ok((NextPc::Relative(1), TWO_M_STATE))
    }

    pub(crate) fn ld_reg8_mem8(
        &mut self,
        dst: Reg8,
        src: Reg8,
        bus: &MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.read_byte_mem8(src, bus)?;
        self.registers.set_reg8(dst, value);

        Ok((NextPc::Relative(1), TWO_M_STATE))
    }

    pub(crate) fn ld_adr16_reg8(
        &mut self,
        src: Reg8,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.registers.get_reg8(src);
        let address = self.read_imm16(bus)?;
        bus.write_byte(address, value)?;

        Ok((NextPc::Relative(3), FOUR_M_STATE))
    }

    pub(crate) fn ld_reg8_adr16(
        &mut self,
        dst: Reg8,
        bus: &MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let address = self.read_imm16(bus)?;
        let value = bus.read_byte(address)?;
        self.registers.set_reg8(dst, value);

        Ok((NextPc::Relative(3), FOUR_M_STATE))
    }

    pub(crate) fn ldi_mem16_reg8(
        &mut self,
        dst: Reg16,
        src: Reg8,
        direction: IncDirection,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.registers.get_reg8(src);
        self.write_byte_mem16(dst, value, bus)?;

        let new_reg16 = self
            .registers
            .get_reg16(dst)
            .wrapping_add_signed(direction.offset_i16());
        self.registers.set_reg16(dst, new_reg16);

        Ok((NextPc::Relative(1), TWO_M_STATE))
    }

    pub(crate) fn ldi_reg8_mem16(
        &mut self,
        dst: Reg8,
        src: Reg16,
        direction: IncDirection,
        bus: &MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.read_byte_mem16(src, bus)?;
        self.registers.set_reg8(dst, value);

        let new_reg16 = self
            .registers
            .get_reg16(src)
            .wrapping_add_signed(direction.offset_i16());

        self.registers.set_reg16(src, new_reg16);

        Ok((NextPc::Relative(1), TWO_M_STATE))
    }
}

// 16-bit loads.
impl Cpu {
    pub(crate) fn ld_reg16_imm16(
        &mut self,
        dst: Reg16,
        bus: &MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let value = self.read_imm16(bus)?;
        self.registers.set_reg16(dst, value);

        Ok((NextPc::Relative(3), THREE_M_STATE))
    }

    pub(crate) fn ld_adr16_reg16(
        &mut self,
        src: Reg16,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let address = self.read_imm16(bus)?;
        let value = self.registers.get_reg16(src);
        bus.write_word(address, value)?;

        Ok((NextPc::Relative(3), FIVE_M_STATE))
    }

    pub(crate) fn ld_reg16_reg16(&mut self, dst: Reg16, src: Reg16) -> (NextPc, usize) {
        let value = self.registers.get_reg16(src);
        self.registers.set_reg16(dst, value);

        (NextPc::Relative(1), TWO_M_STATE)
    }

    #[expect(
        clippy::cast_sign_loss,
        reason = "this op requires casting to be concise."
    )]
    pub(crate) fn ld_reg16_off8(
        &mut self,
        dst: Reg16,
        bus: &MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let offset = self.read_i8(bus)?;
        let sp = self.registers.get_sp();
        let value = sp.wrapping_add_signed(i16::from(offset));

        // Reset zero flag
        self.registers.set_zero_flag(false);
        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);

        // Set half-carry flag if lower four bits of lower byte overflowed
        let half_carry = ((sp & 0xF) + (u16::from(offset as u8) & 0xF)) > 0xF;
        // Set carry flag if upper four bits of lower byte overflowed
        let carry = ((sp & 0xFF) + u16::from(offset as u8)) > 0xFF;

        self.registers.set_carry_flag(carry);
        self.registers.set_half_carry_flag(half_carry);

        self.registers.set_reg16(dst, value);

        Ok((NextPc::Relative(2), THREE_M_STATE))
    }
}
