use super::CPU;
use crate::cpu::registers::Registers;

enum Instruction {
    ADD(ArithmeticTarget),
}

enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl ArithmeticTarget {
    /// Returns the value of the register corresponding to the target.
    fn get_register(&self, registers: &mut Registers) -> u8 {
        match &self {
            ArithmeticTarget::A => registers.a,
            ArithmeticTarget::B => registers.b,
            ArithmeticTarget::C => registers.c,
            ArithmeticTarget::D => registers.d,
            ArithmeticTarget::E => registers.e,
            ArithmeticTarget::H => registers.h,
            ArithmeticTarget::L => registers.l,
        }
    }
}

impl CPU {
    /// Executes a given instruction on the CPU.
    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::ADD(target) => {
                let value = target.get_register(&mut self.registers);
                let new_value = self.add(value);
                self.registers.a = new_value;
            }
        }
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
}
