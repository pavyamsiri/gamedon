use crate::Cpu;

/// Fill the missing bit in a rotate operation using the carry flag bit or not.
#[derive(Debug, Clone, Copy)]
pub(crate) enum FillWith {
    /// Fill the missing bit with the carry flag bit.
    Carry,
    /// Fill the missing bit using the bit that was rotated off.
    OldBit,
}

/// Fill the missing bit in a shift operation using the carry flag bit or not.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ShiftOption {
    /// Leave the value of the missing bit unchanged.
    Unchanged,
    /// Reset the missing bit.
    Reset,
}

impl Cpu {
    /// Rotate an 8-bit `value` to the left and fill with the carry flag or the old bit.
    pub(crate) const fn alu_rotate_left(&mut self, value: u8, fill_with: FillWith) -> u8 {
        let old_bit = value >> 7;
        let fill_bit = match fill_with {
            FillWith::Carry => self.registers.get_carry_flag() as u8,
            FillWith::OldBit => old_bit,
        };
        let new_value = (value << 1) | fill_bit;

        // Store old bit 7 in carry flag
        self.registers.set_carry_flag(old_bit != 0x0);

        // Set zero flag if result is zero else reset
        self.registers.set_zero_flag(new_value == 0);

        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);
        // Reset half-carry flag
        self.registers.set_half_carry_flag(false);

        new_value
    }

    /// Rotate an 8-bit `value` to the right and fill with the carry flag or the old bit.
    pub(crate) const fn alu_rotate_right(&mut self, value: u8, fill_with: FillWith) -> u8 {
        let old_bit = value << 7;
        let fill_bit = match fill_with {
            FillWith::Carry => (self.registers.get_carry_flag() as u8) << 7,
            FillWith::OldBit => old_bit,
        };
        let new_value = (value >> 1) | fill_bit;

        // Store old bit 7 in carry flag
        self.registers.set_carry_flag(old_bit != 0x0);

        // Set zero flag if result is zero else reset
        self.registers.set_zero_flag(new_value == 0);

        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);
        // Reset half-carry flag
        self.registers.set_half_carry_flag(false);

        new_value
    }

    /// Shift an 8-bit `value` to the left.
    pub(crate) const fn alu_shift_left(&mut self, value: u8) -> u8 {
        let old_bit = value >> 7;
        let new_value = (value << 1) & 0xFE;

        // Store old bit 7 in carry flag
        self.registers.set_carry_flag(old_bit != 0x0);

        // Set zero flag if result is zero else reset
        self.registers.set_zero_flag(new_value == 0);

        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);
        // Reset half-carry flag
        self.registers.set_half_carry_flag(false);

        new_value
    }

    /// Shift an 8-bit `value` to the right and either leave the missing bit unchanged or reset it.
    pub(crate) const fn alu_shift_right(&mut self, value: u8, leave: ShiftOption) -> u8 {
        let old_bit = value & 0x1;
        let fill_bit = match leave {
            ShiftOption::Unchanged => value & 0b1000_0000,
            ShiftOption::Reset => 0,
        };
        let new_value = (value >> 1) | fill_bit;

        // Store old bit 7 in carry flag
        self.registers.set_carry_flag(old_bit != 0x0);

        // Set zero flag if result is zero else reset
        self.registers.set_zero_flag(new_value == 0);

        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);
        // Reset half-carry flag
        self.registers.set_half_carry_flag(false);

        new_value
    }

    /// Swap the upper and lower nibbles of an 8-bit `value`.
    pub(crate) const fn alu_swap(&mut self, value: u8) -> u8 {
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
