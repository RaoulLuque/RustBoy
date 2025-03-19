use super::{InstructionCondition, check_instruction_condition};
use crate::RustBoy;

/// Represents the possible targets for the jump instruction.
///
/// The JP instruction can either jump to an immediate operand or to the HL register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JumpType {
    JumpToImmediateOperand(InstructionCondition),
    JumpToHL,
}

impl RustBoy {
    /// Handles the jump instruction for the given [InstructionCondition].
    ///
    /// The JP instruction takes 4 cycles if the jump is taken and 3 cycles if it is not if the
    /// target is an immediate operand. If the target is HL, it takes 1 cycle.
    pub fn handle_jump_instruction(&mut self, jump_type: JumpType) -> u16 {
        match jump_type {
            JumpType::JumpToImmediateOperand(condition) => {
                let should_jump = check_instruction_condition(condition, &self.registers.f);
                if should_jump {
                    self.increment_cycle_counter(4)
                } else {
                    self.increment_cycle_counter(3)
                };
                self.jump(should_jump)
            }
            JumpType::JumpToHL => {
                self.pc = self.registers.get_hl();
                self.increment_cycle_counter(1);
                self.pc
            }
        }
    }

    /// Jumps (the program counter) to the given address if should_jump is true. Otherwise, it just
    /// moves to the next instruction.
    fn jump(&self, should_jump: bool) -> u16 {
        if should_jump {
            // The Rust Boy is little endian so the least significant byte is stored first. However,
            // in the correct order, so we can just patch them together.
            let low_byte = self.read_byte(self.pc.wrapping_add(1)) as u16;
            let high_byte = self.read_byte(self.pc.wrapping_add(2)) as u16;
            (high_byte << 8) | low_byte
        } else {
            // If we don't jump we just move to the next instruction.
            // The jump instruction is 3 bytes long (1 byte for the instruction and 2 bytes for the address).
            self.pc.wrapping_add(3)
        }
    }
}
