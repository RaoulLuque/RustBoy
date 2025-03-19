use super::{InstructionCondition, check_instruction_condition};
use crate::RustBoy;

impl RustBoy {
    /// Handles the JR instruction for the given [InstructionCondition].
    ///
    /// The JR instruction takes 3 cycles if the jump is taken and 2 cycle if it is not.
    pub fn handle_jr_instruction(&mut self, condition: InstructionCondition) -> u16 {
        let should_jump = check_instruction_condition(condition, &self.registers.f);
        if should_jump {
            self.increment_cycle_counter(3)
        } else {
            self.increment_cycle_counter(2)
        };
        self.jr(should_jump)
    }

    /// Jumps (the program counter) a relative distance if should_jump is true. Otherwise, it just
    /// moves to the next instruction. The JR instruction is 2 bytes long (1 byte for the instruction
    /// and 1 byte for relative jump which is encoded as a signed integer. This means that the jump
    /// can be forward or backward. The jump is computed from the address of the following instruction
    /// which is the address of the JR instruction plus 2.
    fn jr(&mut self, should_jump: bool) -> u16 {
        if should_jump {
            // The relative jump is encoded as a signed integer.  Therefore, we add it using
            // wrapping_add_signed. Note that the offset is calculated from the address of the
            // instruction following the JR instruction.
            let relative_jump = (self.read_byte(self.pc.wrapping_add(1)) as i8) as i16;
            let new_pc = self.pc.wrapping_add(2).wrapping_add_signed(relative_jump);
            new_pc
        } else {
            // If we don't jump we just move to the next instruction.
            self.pc.wrapping_add(2)
        }
    }
}
