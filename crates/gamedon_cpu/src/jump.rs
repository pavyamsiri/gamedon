use gamedon_opcode::Condition;

use crate::Cpu;

impl Cpu {
    pub(crate) const fn evaluate_condition(&self, condition: Condition) -> bool {
        match condition {
            Condition::Z => self.registers.get_zero_flag(),
            Condition::NZ => !self.registers.get_zero_flag(),
            Condition::C => self.registers.get_carry_flag(),
            Condition::NC => !self.registers.get_carry_flag(),
        }
    }
}
