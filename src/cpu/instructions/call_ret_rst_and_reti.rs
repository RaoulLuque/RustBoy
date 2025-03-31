use super::{InstructionCondition, check_instruction_condition};
use crate::{CPU, MemoryBus};

impl CPU {
    /// Handles the call instruction for the given [InstructionCondition].
    ///
    /// The CALL instruction takes 6 cycles if the call is taken and 3 cycles if it is not.
    pub fn handle_call_instruction(
        &mut self,
        memory_bus: &mut MemoryBus,
        condition: InstructionCondition,
    ) -> u16 {
        let should_call = check_instruction_condition(condition, &self.registers.f);
        if should_call {
            self.increment_cycle_counter(6)
        } else {
            self.increment_cycle_counter(3)
        };
        self.call(memory_bus, should_call, None, false)
    }

    /// Calls a subroutine at the address following the call instruction if should_call is true.
    /// The call instruction is 3 bytes long (1 byte for the instruction and 2 bytes for the address).
    /// The regular next program counter is pushed onto the stack.
    ///
    /// If an address is provided, it is used instead of the address following the call instruction.
    /// This option is only used for RST instructions which provide a fixed address.
    fn call(
        &mut self,
        memory_bus: &mut MemoryBus,
        should_call: bool,
        address_provided: Option<u16>,
        called_from_rst: bool,
    ) -> u16 {
        let next_pc = if called_from_rst {
            self.pc.wrapping_add(1)
        } else {
            self.pc.wrapping_add(3)
        };
        if should_call {
            self.push(memory_bus, next_pc);
            if let Some(address) = address_provided {
                // If we are executing an RST instruction, we use the fixed address it provides
                address
            } else {
                // If we are executing a CALL instruction, we use the address following the instruction
                memory_bus.read_next_word_little_endian(self.pc)
            }
        } else {
            next_pc
        }
    }

    /// Handles the RET instruction for the given [InstructionCondition].
    ///
    /// The RET instruction takes 5 cycles if the return is taken and 2 cycles if it is not.
    /// Except for the RETI and RET::Always instruction which take 4 cycles.
    pub fn handle_ret_instruction(
        &mut self,
        memory_bus: &MemoryBus,
        condition: InstructionCondition,
    ) -> u16 {
        let should_return = check_instruction_condition(condition, &self.registers.f);
        if condition == InstructionCondition::Always {
            self.increment_cycle_counter(4)
        } else {
            if should_return {
                self.increment_cycle_counter(5)
            } else {
                self.increment_cycle_counter(2)
            }
        };
        self.ret(memory_bus, should_return)
    }

    /// Returns from a subroutine if should_return is true. The next program counter is popped from the stack.
    fn ret(&mut self, memory_bus: &MemoryBus, should_return: bool) -> u16 {
        if should_return {
            self.pop(memory_bus)
        } else {
            self.pc.wrapping_add(1)
        }
    }

    /// Handles the RST instruction for the given address.
    /// This instruction is just a special case of the CALL instruction where the address is fixed.
    ///
    /// The RST instruction takes 4 cycles.
    pub fn handle_rst_instruction(&mut self, memory_bus: &mut MemoryBus, address: u16) -> u16 {
        self.increment_cycle_counter(4);
        self.call(memory_bus, true, Some(address), true)
    }

    /// Handles the RETI instruction.
    /// This instruction is just a special case of the RET instruction where the IME flag is set to true.
    ///
    /// The RETI instruction takes 4 cycles.
    pub fn handle_reti_instruction(&mut self, memory_bus: &MemoryBus) -> u16 {
        self.ime = true;
        self.increment_cycle_counter(4);
        self.ret(memory_bus, true)
    }
}
