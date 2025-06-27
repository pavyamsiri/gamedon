use gamedon_bus::MemoryBus;
use gamedon_opcode::{Condition, Reg16, RstAddress};

use crate::{
    Cpu, ExecuteError, FIVE_M_STATE, FOUR_M_STATE, NextPc, ONE_M_STATE, SIX_M_STATE, THREE_M_STATE,
    TWO_M_STATE,
};

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

// Absolute jumps
impl Cpu {
    pub(crate) const fn jump_absolute(
        &mut self,
        condition: Option<Condition>,
        bus: &MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let should_jump = match condition {
            Some(condition) => self.evaluate_condition(condition),
            None => true,
        };

        if should_jump {
            let address = match self.read_imm16(bus) {
                Ok(address) => address,
                Err(err) => return Err(err),
            };
            Ok((NextPc::Absolute(address), FOUR_M_STATE))
        } else {
            Ok((NextPc::Relative(3), THREE_M_STATE))
        }
    }

    pub(crate) const fn jump_absolute_reg16(&mut self, reg: Reg16) -> (NextPc, usize) {
        let address = self.registers.get_reg16(reg);
        (NextPc::Absolute(address), ONE_M_STATE)
    }
}

// Relative jumps
impl Cpu {
    pub(crate) const fn jump_relative(
        &mut self,
        condition: Option<Condition>,
        bus: &MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let should_jump = match condition {
            Some(condition) => self.evaluate_condition(condition),
            None => true,
        };

        if should_jump {
            let offset = match self.read_i8(bus) {
                Ok(offset) => offset,
                Err(err) => return Err(err),
            };
            Ok((NextPc::Relative(offset as i16), THREE_M_STATE))
        } else {
            Ok((NextPc::Relative(2), TWO_M_STATE))
        }
    }
}

// Call, return and misc
impl Cpu {
    pub(crate) const fn rst(
        &mut self,
        address: RstAddress,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let next_pc = match self.calculate_offset_pc(1) {
            Ok(next_pc) => next_pc,
            Err(err) => return Err(err),
        };
        match self.push_raw(next_pc, bus) {
            Ok(()) => {}
            Err(err) => return Err(err),
        }
        let address = address.to_address();
        Ok((NextPc::Absolute(address), FOUR_M_STATE))
    }

    pub(crate) const fn call(
        &mut self,
        condition: Option<Condition>,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let should_jump = match condition {
            Some(condition) => self.evaluate_condition(condition),
            None => true,
        };
        if should_jump {
            let next_pc = match self.calculate_offset_pc(1) {
                Ok(next_pc) => next_pc,
                Err(err) => return Err(err),
            };
            let address = match self.read_imm16(bus) {
                Ok(address) => address,
                Err(err) => return Err(err),
            };

            match self.push_raw(next_pc, bus) {
                Ok(()) => {}
                Err(err) => return Err(err),
            }
            Ok((NextPc::Absolute(address), SIX_M_STATE))
        } else {
            Ok((NextPc::Relative(3), THREE_M_STATE))
        }
    }

    pub(crate) const fn ret(
        &mut self,
        condition: Option<Condition>,
        enable_interrupts: bool,
        bus: &mut MemoryBus,
    ) -> Result<(NextPc, usize), ExecuteError> {
        let (should_jump, num_cycles_if_jump) = match condition {
            Some(condition) => (self.evaluate_condition(condition), FIVE_M_STATE),
            None => (true, FOUR_M_STATE),
        };

        if enable_interrupts {
            self.ime = true;
        }

        if should_jump {
            let address = match self.pop_raw(bus) {
                Ok(address) => address,
                Err(err) => return Err(err),
            };

            Ok((NextPc::Absolute(address), num_cycles_if_jump))
        } else {
            Ok((NextPc::Relative(1), TWO_M_STATE))
        }
    }
}
