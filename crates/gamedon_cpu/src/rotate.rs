use crate::{Cpu, WithCarry};

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
        self.registers.set_carry_flag(old_bit != 0x0);

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
        self.registers.set_carry_flag(old_bit != 0x0);

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
        self.registers.set_carry_flag(old_bit != 0x0);

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
        self.registers.set_carry_flag(old_bit != 0x0);

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
