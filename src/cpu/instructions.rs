use super::CPU;
use crate::cpu::registers::Registers;

/// Represents a CPU instruction. The instruction can be either a prefix or non-prefix instruction.
/// For details please refer to [Pan Docs](https://gbdev.io/pandocs/CPU_Instruction_Set.html) or
/// the [interactive CPU instruction set guide](https://meganesu.github.io/generate-gb-opcodes/).
#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    ADD(Register),
    JP(JumpCondition),
    LD(LoadType),
}

/// Enum to represent the Registers of the CPU (except for the f register) as target or sources of operations.
#[derive(Clone, Copy, Debug)]
enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

/// Represents the possible conditions for a jump instruction.
#[derive(Clone, Copy, Debug)]
enum JumpCondition {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

/// Represents the possible targets for a byte load instruction.
#[derive(Clone, Copy, Debug)]
enum LoadByteTarget {
    REGISTER(Register),
    HLI,
}

/// Represents the possible sources for a byte load instruction.
#[derive(Clone, Copy, Debug)]
enum LoadByteSource {
    REGISTER(Register),
    D8,
    HLI,
}

/// Represents the possible types of load instructions.
#[derive(Clone, Copy, Debug)]
enum LoadType {
    Byte(LoadByteTarget, LoadByteSource),
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
            0x87 => Some(Instruction::ADD(Register::A)),
            0x80 => Some(Instruction::ADD(Register::B)),
            0x81 => Some(Instruction::ADD(Register::C)),
            0x82 => Some(Instruction::ADD(Register::D)),
            0x83 => Some(Instruction::ADD(Register::E)),
            0x84 => Some(Instruction::ADD(Register::H)),
            0x85 => Some(Instruction::ADD(Register::L)),
            _ => None,
        }
    }
}

impl Register {
    /// Returns the value of the register corresponding to the enum variant.
    fn get_register(&self, registers: &Registers) -> u8 {
        match &self {
            Register::A => registers.a,
            Register::B => registers.b,
            Register::C => registers.c,
            Register::D => registers.d,
            Register::E => registers.e,
            Register::H => registers.h,
            Register::L => registers.l,
        }
    }

    /// Sets the value of the register corresponding to the enum variant.
    fn set_register(&self, registers: &mut Registers, value: u8) {
        match &self {
            Register::A => registers.a = value,
            Register::B => registers.b = value,
            Register::C => registers.c = value,
            Register::D => registers.d = value,
            Register::E => registers.e = value,
            Register::H => registers.h = value,
            Register::L => registers.l = value,
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
            Instruction::JP(condition) => {
                let should_jump = match condition {
                    JumpCondition::NotZero => !self.registers.f.zero,
                    JumpCondition::Zero => self.registers.f.zero,
                    JumpCondition::NotCarry => !self.registers.f.carry,
                    JumpCondition::Carry => self.registers.f.carry,
                    JumpCondition::Always => true,
                };
                self.jump(should_jump)
            }
            Instruction::LD(type_of_load) => match type_of_load {
                LoadType::Byte(target, source) => {
                    let value = match source {
                        LoadByteSource::REGISTER(register) => {
                            register.get_register(&self.registers)
                        }
                        LoadByteSource::D8 => self.bus.read_byte(self.pc + 1),
                        LoadByteSource::HLI => self.bus.read_byte(self.registers.get_hl()),
                    };
                    match target {
                        LoadByteTarget::REGISTER(register) => {
                            register.set_register(&mut self.registers, value);
                        }
                        LoadByteTarget::HLI => {
                            self.bus.write_byte(self.registers.get_hl(), value);
                        }
                    }
                    match source {
                        LoadByteSource::D8 => self.pc.wrapping_add(2),
                        _ => self.pc.wrapping_add(1),
                    }
                }
            },
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

    fn jump(&self, should_jump: bool) -> u16 {
        if should_jump {
            // The Gameboy is little endian so the least significant byte is stored first. However,
            // in the correct order, so we can just patch them together.
            let low_byte = self.bus.read_byte(self.pc + 1) as u16;
            let high_byte = self.bus.read_byte(self.pc + 2) as u16;
            (high_byte << 8) | low_byte
        } else {
            // If we don't jump we just move to the next instruction.
            // The jump instruction is 3 bytes long (1 byte for the instruction and 2 bytes for the address).
            self.pc.wrapping_add(3)
        }
    }
}
