use crate::Cpu;

#[derive(Debug, Clone, Copy)]
pub(crate) enum WithCarry {
    Yes,
    No,
}

// Raw ALU ops
impl Cpu {
    pub(crate) const fn alu_inc8(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_add(1);

        // Set zero flag to true if result is zero
        self.registers.set_zero_flag(new_value == 0);
        // Set subtraction flag
        self.registers.set_subtraction_flag(false);
        // If the lower four bits of register A and the target added together overflow into the upper four bits, we set
        // the half-carry flag
        self.registers
            .set_half_carry_flag(new_value.trailing_zeros() >= 4);

        new_value
    }

    pub(crate) const fn alu_dec8(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_sub(1);

        // Set zero flag to true if result is zero
        self.registers.set_zero_flag(new_value == 0);
        // Set subtraction flag
        self.registers.set_subtraction_flag(true);
        // If decrementing led to an underflow in the lower four bits, then set the half-carry flag otherwise reset.
        self.registers.set_half_carry_flag((new_value & 0xF) == 0xF);

        new_value
    }

    pub(crate) const fn alu_add8(&mut self, lhs: u8, rhs: u8, with_carry: WithCarry) -> u8 {
        let carry_value = match with_carry {
            WithCarry::Yes => self.registers.get_carry_flag() as u8,
            WithCarry::No => 0,
        };
        let (new_value, first_did_overflow) = lhs.overflowing_add(rhs);
        let (new_value, second_did_overflow) = new_value.overflowing_add(carry_value);

        // Set zero flag to true if result is zero
        self.registers.set_zero_flag(new_value == 0);
        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);
        // If overflowed, carry flag is set to true
        self.registers
            .set_carry_flag(first_did_overflow || second_did_overflow);
        // If the lower four bits of register A and the target added together overflow into the upper four bits, we set
        // the half-carry flag
        self.registers
            .set_half_carry_flag(((lhs & 0xF) + (rhs & 0xF) + carry_value) > 0xF);

        new_value
    }

    pub(crate) const fn alu_sub8(&mut self, lhs: u8, rhs: u8, with_carry: WithCarry) -> u8 {
        let carry_value = match with_carry {
            WithCarry::Yes => self.registers.get_carry_flag() as u8,
            WithCarry::No => 0,
        };
        // Subtract value + carry flag
        let (new_value, did_first_overflow) = lhs.overflowing_sub(rhs);
        let (new_value, did_second_overflow) = new_value.overflowing_sub(carry_value);

        // Set zero flag to true if result is zero
        self.registers.set_zero_flag(new_value == 0);
        // Set subtraction flag
        self.registers.set_subtraction_flag(true);
        // If the subtraction will cause the value to wrap, then set the carry flag
        self.registers
            .set_carry_flag(did_first_overflow || did_second_overflow);
        // If the subtraction between the lower four bits of register A and the target cause underflow, then set
        // the half-carry flag
        self.registers
            .set_half_carry_flag((lhs & 0xF) < (rhs & 0xF) + (carry_value));

        new_value
    }

    pub(crate) const fn alu_and8(&mut self, lhs: u8, rhs: u8) -> u8 {
        let result = rhs & lhs;

        // Set zero flag to true if result is false
        self.registers.set_zero_flag(result == 0);
        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);
        // Set half-carry flag
        self.registers.set_half_carry_flag(true);
        // Reset carry flag
        self.registers.set_carry_flag(false);

        result
    }

    pub(crate) const fn alu_xor8(&mut self, lhs: u8, rhs: u8) -> u8 {
        let result = rhs ^ lhs;

        // Set zero flag to true if result is false
        self.registers.set_zero_flag(result == 0);
        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);
        // Reset half-carry flag
        self.registers.set_half_carry_flag(false);
        // Reset carry flag
        self.registers.set_carry_flag(false);

        result
    }

    pub(crate) const fn alu_or8(&mut self, lhs: u8, rhs: u8) -> u8 {
        let result = rhs | lhs;

        // Set zero flag to true if result is false
        self.registers.set_zero_flag(result == 0);
        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);
        // Reset half-carry flag
        self.registers.set_half_carry_flag(false);
        // Reset carry flag
        self.registers.set_carry_flag(false);

        result
    }

    pub(crate) const fn alu_add16(&mut self, lhs: u16, rhs: u16) -> u16 {
        let (new_value, did_overflow) = lhs.overflowing_add(rhs);

        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);
        // If overflowed, carry flag is set to true
        self.registers.set_carry_flag(did_overflow);
        // If the lower twelve bits of register HL and the target added together overflow into the upper four bits, we set
        // the half-carry flag
        let (half_carry_result, half_carry_overflow) = (lhs & 0x0FFF).overflowing_add(rhs & 0x0FFF);
        self.registers
            .set_half_carry_flag((half_carry_result > 0x0FFF) || half_carry_overflow);

        new_value
    }

    #[expect(clippy::cast_sign_loss, reason = "this behaviour is expected.")]
    pub(crate) const fn alu_add16_signed(&mut self, base: u16, offset: i8) -> u16 {
        let (new_value, _) = base.overflowing_add(offset as u16);

        // Reset zero flag
        self.registers.set_zero_flag(false);
        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);
        // If the lower byte overflowed, carry flag is set to true
        let (carry_result, _) = (base & 0x00FF).overflowing_add((offset as u16) & 0x00FF);
        self.registers.set_carry_flag(carry_result > 0xFF);
        // If the lower four bits of register SP and the target added together overflow into the upper four bits, we set
        // the half-carry flag
        let (half_carry_result, _) = (base & 0x000F).overflowing_add((offset as u16) & 0x000F);
        self.registers.set_half_carry_flag(half_carry_result > 0xF);

        new_value
    }
}
