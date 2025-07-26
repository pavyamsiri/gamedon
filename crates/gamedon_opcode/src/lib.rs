mod opcode;
mod register;

pub use gamedon_bits::BitShift8;
pub use opcode::{Condition, Instruction, MicroOp, RstAddress, disassemble};
pub use register::{Reg8, Reg16, RegFlags, Registers};

/// The decoder state.
#[derive(Debug, Default)]
enum State {
    /// Decoder is in normal state.
    #[default]
    Normal,
    /// Decoder is in prefix mode.
    Prefix,
}

/// An instruction decoder.
#[derive(Debug, Default)]
pub struct Decoder {
    /// The deocder state.
    state: State,
}

impl Decoder {
    /// Decode an opcode.
    #[inline]
    pub const fn decode(&mut self, opcode: u8) -> Instruction {
        if matches!(self.state, State::Prefix) {
            self.state = State::Normal;
            Instruction::decode_prefix(opcode)
        } else {
            let inst = Instruction::decode_no_prefix(opcode);
            if inst.is_prefix() {
                self.state = State::Prefix;
            }
            inst
        }
    }
}
