use super::ArithmeticOrLogicalSource;
use crate::{CPU, MemoryBus};

impl CPU {
    /// Handles the AND instruction for the given [ArithmeticSource](super::ArithmeticOrLogicalSource).
    ///
    /// The AND instruction takes 1 cycle if the source is a register and 2 otherwise.
    pub fn handle_and_instruction(
        &mut self,
        memory_bus: &MemoryBus,
        source: ArithmeticOrLogicalSource,
    ) -> u16 {
        let new_pc = source.increment_pc_and_cycle(self);
        let value = source.get_value(memory_bus, &self.registers, self.pc);
        let new_value = self.and(value);
        self.registers.a = new_value;
        new_pc
    }

    /// Performs a bitwise AND operation on the A register and the given value and sets the
    /// corresponding flags in the flags register [super::registers::FlagsRegister].
    fn and(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a & value;
        self.registers.f.set_zero_flag(new_value == 0);
        self.registers.f.set_subtract_flag(false);
        self.registers.f.set_half_carry_flag(true);
        self.registers.f.set_carry_flag(false);
        new_value
    }

    /// Handles the XOR instruction for the given [ArithmeticSource](super::ArithmeticOrLogicalSource).
    ///
    /// The XOR instruction takes 1 cycle if the source is a register and 2 otherwise.
    pub fn handle_xor_instruction(
        &mut self,
        memory_bus: &MemoryBus,
        source: ArithmeticOrLogicalSource,
    ) -> u16 {
        let new_pc = source.increment_pc_and_cycle(self);
        let value = source.get_value(memory_bus, &self.registers, self.pc);
        let new_value = self.xor(value);
        self.registers.a = new_value;
        new_pc
    }

    /// Performs a bitwise XOR operation on the A register and the given value and sets the
    /// corresponding flags in the flags register [super::registers::FlagsRegister].
    fn xor(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a ^ value;
        self.registers.f.set_zero_flag(new_value == 0);
        self.registers.f.set_subtract_flag(false);
        self.registers.f.set_half_carry_flag(false);
        self.registers.f.set_carry_flag(false);
        new_value
    }

    /// Handles the OR instruction for the given [ArithmeticSource](super::ArithmeticOrLogicalSource).
    ///
    /// The OR instruction takes 1 cycle if the source is a register and 2 otherwise.
    pub fn handle_or_instruction(
        &mut self,
        memory_bus: &MemoryBus,
        source: ArithmeticOrLogicalSource,
    ) -> u16 {
        let new_pc = source.increment_pc_and_cycle(self);
        let value = source.get_value(memory_bus, &self.registers, self.pc);
        let new_value = self.or(value);
        self.registers.a = new_value;
        new_pc
    }

    /// Performs a bitwise OR operation on the A register and the given value and sets the
    /// corresponding flags in the flags register [super::registers::FlagsRegister].
    fn or(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a | value;
        self.registers.f.set_zero_flag(new_value == 0);
        self.registers.f.set_subtract_flag(false);
        self.registers.f.set_half_carry_flag(false);
        self.registers.f.set_carry_flag(false);
        new_value
    }

    /// Handles the CP instruction for the given [ArithmeticSource](super::ArithmeticOrLogicalSource).
    ///
    /// The CP instruction takes 1 cycle if the source is a register and 2 otherwise.
    pub fn handle_cp_instruction(
        &mut self,
        memory_bus: &MemoryBus,
        source: ArithmeticOrLogicalSource,
    ) -> u16 {
        let new_pc = source.increment_pc_and_cycle(self);
        let value = source.get_value(memory_bus, &self.registers, self.pc);
        self.sub(value, false);
        new_pc
    }
}
