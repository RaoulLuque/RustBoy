use super::CPU;
use crate::cpu::registers::Registers;

pub enum Instruction {
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

impl Instruction {
    /// Returns the (prefix or non-prefix) instruction corresponding to the given byte. See
    /// [Interactive CPU Instructions](https://meganesu.github.io/generate-gb-opcodes/)
    /// for details.
    ///
    /// Checks the prefix bools to determine if a prefix instruction should be returned or not.
    /// That is, the u8 byte should only contain the instruction byte and not include the prefix byte.
    ///
    /// Calls [Instruction::from_byte_not_prefixed] or [Instruction::from_byte_prefixed] depending on the prefix bool.
    pub fn from_byte(byte: u8, prefixed: bool) -> Option<Instruction> {
        if prefixed {
            Self::from_byte_prefixed(byte)
        } else {
            Self::from_byte_not_prefixed(byte)
        }
    }

    /// Returns the prefix instruction corresponding to the given byte. See
    /// [Interactive CPU Instructions](https://meganesu.github.io/generate-gb-opcodes/)
    /// for details.
    pub fn from_byte_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            // TODO: Add more instructions
            _ => None,
        }
    }

    /// Returns the non-prefix instruction corresponding to the given byte. See
    /// [Interactive CPU Instructions](https://meganesu.github.io/generate-gb-opcodes/)
    /// for details.
    pub fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            // TODO: Add more instructions
            0x87 => Some(Instruction::ADD(ArithmeticTarget::A)),
            0x80 => Some(Instruction::ADD(ArithmeticTarget::B)),
            0x81 => Some(Instruction::ADD(ArithmeticTarget::C)),
            0x82 => Some(Instruction::ADD(ArithmeticTarget::D)),
            0x83 => Some(Instruction::ADD(ArithmeticTarget::E)),
            0x84 => Some(Instruction::ADD(ArithmeticTarget::H)),
            0x85 => Some(Instruction::ADD(ArithmeticTarget::L)),
            _ => None,
        }
    }
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
    /// Executes the instruction on the CPU.
    pub fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::ADD(target) => {
                let value = target.get_register(&mut self.registers);
                let new_value = self.add(value);
                self.registers.a = new_value;
                self.pc.wrapping_add(1)
            }
            _ => {
                /* TODO: Support more instructions */
                self.pc
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
