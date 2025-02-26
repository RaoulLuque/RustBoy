use crate::cpu::instructions::Register;
use crate::cpu::CPU;

impl CPU {
    /// Handles the add instruction for the given [Register].
    pub fn handle_add_instruction(&mut self, target: Register) -> u16 {
        let value = target.get_register(&mut self.registers);
        let new_value = self.add(value);
        self.registers.a = new_value;
        self.pc.wrapping_add(1)
    }

    /// Adds a value to the A register and sets the corresponding flags in the flags register
    /// [super::registers::FlagsRegister].
    fn add(&mut self, value: u8) -> u8 {
        let (new_value, overflow_flag) = self.registers.a.overflowing_add(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = overflow_flag;
        // The half carry flag is set if there is an overflow from the lower 4 bits to the fifth bit.
        // This is the case if the sum of the lower 4 bits of the A register and the value are greater
        // than 0xF = 0b 0000 1111 (binary).
        self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;
        new_value
    }

    /// Handles the adc instruction for the given [Register].
    /// Does the same as [handle_add_instruction] but adds the carry flag to the value.
    pub fn handle_adc_instruction(&mut self, target: Register) -> u16 {
        let value = target.get_register(&mut self.registers);
        let carry = if self.registers.f.carry { 1 } else { 0 };
        let new_value = self.add(value.wrapping_add(carry));
        self.registers.a = new_value;
        self.pc.wrapping_add(1)
    }
}
