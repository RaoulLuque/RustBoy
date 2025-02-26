use super::{check_instruction_condition, InstructionCondition};
use crate::cpu::CPU;

impl CPU {
    /// Handles the call instruction for the given [InstructionCondition].
    pub fn handle_call_instruction(&mut self, condition: InstructionCondition) -> u16 {
        let should_call = check_instruction_condition(condition, &self.registers.f);
        self.call(should_call)
    }

    /// Calls a subroutine at the address following the call instruction if should_call is true.
    /// The call instruction is 3 bytes long (1 byte for the instruction and 2 bytes for the address).
    /// The regular next program counter is pushed onto the stack.
    fn call(&mut self, should_call: bool) -> u16 {
        let next_pc = self.pc.wrapping_add(3);
        if should_call {
            self.push(next_pc);
            self.bus.read_next_word_little_endian(self.pc)
        } else {
            next_pc
        }
    }

    pub fn handle_ret_instruction(&mut self, condition: InstructionCondition) -> u16 {
        let should_return = check_instruction_condition(condition, &self.registers.f);
        self.ret(should_return)
    }

    /// Returns from a subroutine if should_return is true. The next program counter is popped from the stack.
    fn ret(&mut self, should_return: bool) -> u16 {
        if should_return {
            self.pop()
        } else {
            self.pc.wrapping_add(1)
        }
    }
}
