use super::ArithmeticOrLogicalSource;
use crate::{CPU, MemoryBus};

impl CPU {
    /// Handles the sub instruction for the given [ArithmeticSource](super::ArithmeticOrLogicalSource).
    ///
    /// The SUB instruction takes 1 cycle if the source is a register and 2 otherwise.
    pub fn handle_sub_instruction(
        &mut self,
        memory_bus: &MemoryBus,
        source: ArithmeticOrLogicalSource,
    ) -> u16 {
        let new_pc = source.increment_pc_and_cycle(self);
        let value = source.get_value(memory_bus, &self.registers, self.pc);
        let new_value = self.sub(value, false);
        self.registers.a = new_value;
        new_pc
    }

    /// Subtracts a value from the A register and sets the corresponding flags in the flags register
    /// [super::registers::FlagsRegister].
    pub fn sub(&mut self, value: u8, carry_flag: bool) -> u8 {
        let new_value = self
            .registers
            .a
            .wrapping_sub(value)
            .wrapping_sub(carry_flag as u8);
        self.registers.f.set_zero_flag(new_value == 0);
        self.registers.f.set_subtract_flag(true);
        self.registers
            .f
            .set_carry_flag((self.registers.a as u16) < ((value as u16) + (carry_flag as u16)));
        // The half carry flag is set if there is an overflow from the lower 4 bits to the fifth bit.
        // This is the case if the subtraction of the lower 4 bits of the A register and the value is less
        // than 0. That is, if there is a wrap around and the new_value is greater than 0xF.
        self.registers.f.set_half_carry_flag(
            (self.registers.a & 0xF)
                .wrapping_sub(value & 0xF)
                .wrapping_sub(carry_flag as u8)
                > 0xF,
        );
        new_value
    }

    /// Handles the sbc instruction for the given [ArithmeticSource](super::ArithmeticOrLogicalSource).
    ///
    /// The SBC instruction takes 1 cycle if the source is a register and 2 otherwise.
    pub fn handle_sbc_instruction(
        &mut self,
        memory_bus: &MemoryBus,
        source: ArithmeticOrLogicalSource,
    ) -> u16 {
        let new_pc = source.increment_pc_and_cycle(self);
        let value = source.get_value(memory_bus, &self.registers, self.pc);
        let new_value = self.sub(value, self.registers.f.get_carry_flag());
        self.registers.a = new_value;
        new_pc
    }
}
