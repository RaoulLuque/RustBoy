use crate::cpu::CPU;

/// Represents the possible conditions for a jump instruction.
#[derive(Clone, Copy, Debug)]
pub(super) enum JumpCondition {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

impl CPU {
    /// Handles the jump instruction for the given [JumpCondition].
    pub fn handle_jump_instruction(&mut self, condition: JumpCondition) -> u16 {
        let should_jump = match condition {
            JumpCondition::NotZero => !self.registers.f.zero,
            JumpCondition::Zero => self.registers.f.zero,
            JumpCondition::NotCarry => !self.registers.f.carry,
            JumpCondition::Carry => self.registers.f.carry,
            JumpCondition::Always => true,
        };
        self.jump(should_jump)
    }

    /// Jumps (the program counter) to the given address if should_jump is true. Otherwise, it just
    /// moves to the next instruction.
    fn jump(&self, should_jump: bool) -> u16 {
        if should_jump {
            // The Gameboy is little endian so the least significant byte is stored first. However,
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
