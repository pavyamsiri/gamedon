use crate::Cpu;
use gamedon_opcode::Condition;

impl Cpu {
    /// Evaluate `condition` and return the result.
    pub(crate) const fn evaluate_condition(&self, condition: Condition) -> bool {
        match condition {
            Condition::Z => self.registers.get_zero_flag(),
            Condition::NZ => !self.registers.get_zero_flag(),
            Condition::C => self.registers.get_carry_flag(),
            Condition::NC => !self.registers.get_carry_flag(),
        }
    }
}
