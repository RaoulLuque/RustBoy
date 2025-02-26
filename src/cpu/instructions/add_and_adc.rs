use crate::cpu::instructions::{ArithmeticSource, Register};
use crate::cpu::CPU;

impl CPU {
    /// Handles the add instruction for the given [Register].
    pub fn handle_add_instruction(&mut self, source: ArithmeticSource) -> u16 {
        let value = source.get_value(&self.registers, &self.bus, self.pc);
        let new_value = self.add(value, false);
        self.registers.a = new_value;
        self.pc.wrapping_add(1)
    }

    /// Adds a value to the A register and sets the corresponding flags in the flags register
    /// [super::registers::FlagsRegister].
    fn add(&mut self, value: u8, carry_flag: bool) -> u8 {
        let new_value = self
            .registers
            .a
            .wrapping_add(value)
            .wrapping_add(carry_flag as u8);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        // The carry flag is set if there is an overflow from the 8th bit to the 9th bit.
        // This is the case if the sum of the A register and the value are greater than 0xFF = 0b 1111 1111 (binary).
        self.registers.f.carry = self.registers.a as u16 + value as u16 + carry_flag as u16 > 0xFF;
        // The half carry flag is set if there is an overflow from the lower 4 bits to the fifth bit.
        // This is the case if the sum of the lower 4 bits of the A register and the value are greater
        // than 0xF = 0b 0000 1111 (binary).
        self.registers.f.half_carry =
            ((self.registers.a & 0xF) + (value & 0xF) + carry_flag as u8) > 0xF;
        new_value
    }

    /// Handles the adc instruction for the given [Register].
    /// Does the same as [handle_add_instruction] but adds the carry flag to the value.
    pub fn handle_adc_instruction(&mut self, source: ArithmeticSource) -> u16 {
        let value = source.get_value(&self.registers, &self.bus, self.pc);
        let new_value = self.add(value, self.registers.f.carry);
        self.registers.a = new_value;
        self.pc.wrapping_add(1)
    }
}
