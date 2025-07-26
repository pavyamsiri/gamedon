use crate::Cpu;
use gamedon_opcode::BitShift8;

// Raw ALU ops
impl Cpu {
    /// Test a `bit` in an 8-bit `value`.
    pub(crate) const fn alu_bit(&mut self, value: u8, bit: BitShift8) {
        let bit_shift_amount = bit.get_shift_amount();
        let test_bit = (value >> bit_shift_amount) & 0x1;

        // Set zero flag if bit is not set.
        self.registers.set_zero_flag(test_bit == 0);

        // Reset subtraction flag
        self.registers.set_subtraction_flag(false);
        // Set half-carry flag
        self.registers.set_half_carry_flag(true);
    }

    /// Reset a `bit` in an 8-bit `value`.
    pub(crate) const fn alu_reset(value: u8, bit: BitShift8) -> u8 {
        let bit_shift_amount = bit.get_shift_amount();
        let reset_bit_mask = !(0x1u8 << bit_shift_amount);

        value & reset_bit_mask
    }

    /// Set a `bit` in an 8-bit `value`.
    pub(crate) const fn alu_set(value: u8, bit: BitShift8) -> u8 {
        let bit_shift_amount = bit.get_shift_amount();
        let set_bit_mask = 0x1u8 << bit_shift_amount;

        value | set_bit_mask
    }
}
