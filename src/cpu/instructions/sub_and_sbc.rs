use super::ArithmeticSource;
use crate::cpu::CPU;

impl CPU {
    pub fn handle_sub_instruction(&mut self, source: ArithmeticSource) -> u16 {
        let value = source.get_value(&self.registers, &self.bus, self.pc);
        let new_value = self.sub(value, false);
        self.registers.a = new_value;
        self.pc.wrapping_add(1)
    }

    fn sub(&mut self, value: u8, carry_flag: bool) -> u8 {
        let new_value = self.registers.a.wrapping_sub(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = (self.registers.a as u16) < ((value as u16) + (carry_flag as u16));
        // The half carry flag is set if there is an overflow from the lower 4 bits to the fifth bit.
        // This is the case if the subtraction of the lower 4 bits of the A register and the value is less
        // than 0. That is, if the lower 4 bits of the A register are less than the lower 4 bits of the value.
        self.registers.f.half_carry = (self.registers.a & 0xF)
            .wrapping_sub(value & 0xF)
            .wrapping_sub(carry_flag as u8)
            > 0xF;
        new_value
    }

    pub fn handle_sbc_instruction(&mut self, source: ArithmeticSource) -> u16 {
        let value = source.get_value(&self.registers, &self.bus, self.pc);
        let new_value = self.sub(value, self.registers.f.carry);
        self.registers.a = new_value;
        self.pc.wrapping_add(1)
    }
}
