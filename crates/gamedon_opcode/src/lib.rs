mod opcode;
mod register;

pub use opcode::{Condition, Instruction, RstAddress, disassemble};
pub use register::{Reg8, Reg16, Registers};
