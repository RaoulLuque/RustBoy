use super::{check_instruction_condition, InstructionCondition};
use crate::cpu::CPU;

/// Represents the possible targets for the jump instruction.
///
#[derive(Clone, Copy, Debug)]
pub(super) enum JumpType {
    JumpToImmediateOperand(InstructionCondition),
    JumpToHL,
}

impl CPU {
    /// Handles the jump instruction for the given [InstructionCondition].
    pub fn handle_jump_instruction(&mut self, jump_type: JumpType) -> u16 {
        match jump_type {
            JumpType::JumpToImmediateOperand(condition) => {
                let should_jump = check_instruction_condition(condition, &self.registers.f);
                self.jump(should_jump)
            }
            JumpType::JumpToHL => {
                self.pc = self.registers.get_hl();
                self.pc
            }
        }
    }

    /// Jumps (the program counter) to the given address if should_jump is true. Otherwise, it just
    /// moves to the next instruction.
    fn jump(&self, should_jump: bool) -> u16 {
        if should_jump {
            // The Game Boy is little endian so the least significant byte is stored first. However,
            // in the correct order, so we can just patch them together.
            let low_byte = self.bus.read_byte(self.pc + 1) as u16;
            let high_byte = self.bus.read_byte(self.pc + 2) as u16;
            (high_byte << 8) | low_byte
        } else {
            // If we don't jump we just move to the next instruction.
            // The jump instruction is 3 bytes long (1 byte for the instruction and 2 bytes for the address).
            self.pc.wrapping_add(3)
        }
    }
}
