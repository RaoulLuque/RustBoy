//! This module contains the CPU instructions. The instructions are represented as an enum and
//! the CPU struct has a method to execute the instruction.
//!
//! The instructions are divided into two categories: prefix and non-prefix instructions.
//! For details please refer to [Pan Docs](https://gbdev.io/pandocs/CPU_Instruction_Set.html),
//! the [interactive CPU instruction set guide](https://meganesu.github.io/generate-gb-opcodes/) or the
//! [CPU opcode reference](https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7).
//!
//! The instructions are implemented in separate modules for better organization and readability.
//! In the cases where instructions share the same target or source for their operations
//! (e.g. ADD and ADC, [ArithmeticOrLogicalSource]), they use a common type to represent the target or source which is then
//! implemented in this module.

mod add_and_adc;
mod call_and_ret;
mod inc_and_dec;
mod jump;
mod load;
mod logical_operators;
mod parsing;
mod push_and_pop;
mod sub_and_sbc;

use super::{MemoryBus, CPU};
use crate::cpu::registers::{FlagsRegister, Registers};
use inc_and_dec::IncDecTarget;
use load::LoadType;
use push_and_pop::{PopTarget, PushSource};

/// Represents a CPU instruction. The instruction can be either a prefix or non-prefix instruction.
/// For details please refer to [Pan Docs](https://gbdev.io/pandocs/CPU_Instruction_Set.html),
/// the [interactive CPU instruction set guide](https://meganesu.github.io/generate-gb-opcodes/) or
/// the [CPU opcode reference](https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7).
#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    NOP,
    ADDToA(ArithmeticOrLogicalSource),
    ADC(ArithmeticOrLogicalSource),
    SUB(ArithmeticOrLogicalSource),
    SBC(ArithmeticOrLogicalSource),
    AND(ArithmeticOrLogicalSource),
    OR(ArithmeticOrLogicalSource),
    XOR(ArithmeticOrLogicalSource),
    CP(ArithmeticOrLogicalSource),
    INC(IncDecTarget),
    DEC(IncDecTarget),
    JP(InstructionCondition),
    LD(LoadType),
    PUSH(PushSource),
    POP(PopTarget),
    CALL(InstructionCondition),
    RET(InstructionCondition),
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

/// Represents the possible targets for arithmetic or logical instructions such as
/// ADD, ADC, SUB, SBC, AND, OR, XOR or CP.
#[derive(Clone, Copy, Debug)]
pub(in crate::cpu::instructions) enum ArithmeticOrLogicalSource {
    Register(Register),
    D8,
    HL,
}

/// Represents the possible conditions for an instruction to execute or not (e.g. JP or CALL).
#[derive(Clone, Copy, Debug)]
enum InstructionCondition {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

impl Instruction {
    /// Returns the (prefix or non-prefix) instruction corresponding to the given byte. See
    /// [Interactive CPU Instructions](https://meganesu.github.io/generate-gb-opcodes/) or
    /// [CPU opcode reference](https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7) for details.
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
    /// or [CPU opcode reference](https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7) for details.
    pub fn from_byte_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            // TODO: Add more instructions
            _ => None,
        }
    }

    /// Returns the non-prefix instruction corresponding to the given byte. See
    /// [Interactive CPU Instructions](https://meganesu.github.io/generate-gb-opcodes/)
    /// or [CPU opcode reference](https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7) for details.
    ///
    /// - Group 0 are miscellaneous instructions.
    /// - Group 1 are load instructions and the HALT instruction.
    /// - Group 2 are arithmetic instructions.
    /// - Group 3 are control flow and miscellaneous instructions.
    pub fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
        let higher_nibble = (byte & 0xF0) >> 4;
        match higher_nibble {
            0x0 | 0x1 | 0x2 | 0x3 => Self::from_byte_not_prefixed_group_0(byte),
            0x4 | 0x5 | 0x6 | 0x7 => Self::from_byte_not_prefixed_group_1(byte),
            0x8 | 0x9 | 0xA | 0xB => Self::from_byte_not_prefixed_group_2(byte),
            0xC | 0xD | 0xE | 0xF => Self::from_byte_not_prefixed_group_3(byte),
            _ => None,
        }
    }
}

impl CPU {
    /// Executes the instruction on the CPU.
    pub fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::NOP => self.pc.wrapping_add(1),
            Instruction::ADDToA(target) => self.handle_add_instruction(target),
            Instruction::ADC(target) => self.handle_adc_instruction(target),
            Instruction::SUB(target) => self.handle_sub_instruction(target),
            Instruction::SBC(target) => self.handle_sbc_instruction(target),
            Instruction::AND(target) => self.handle_and_instruction(target),
            Instruction::OR(target) => self.handle_or_instruction(target),
            Instruction::XOR(target) => self.handle_xor_instruction(target),
            Instruction::CP(target) => self.handle_cp_instruction(target),
            Instruction::JP(condition) => self.handle_jump_instruction(condition),
            Instruction::LD(type_of_load) => self.handle_load_instruction(type_of_load),
            Instruction::INC(target) => self.handle_inc_instruction(target),
            Instruction::DEC(target) => self.handle_dec_instruction(target),
            Instruction::CALL(condition) => self.handle_call_instruction(condition),
            Instruction::RET(condition) => self.handle_ret_instruction(condition),
            Instruction::PUSH(source) => self.handle_push_instruction(source),
            Instruction::POP(target) => self.handle_pop_instruction(target),
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

impl ArithmeticOrLogicalSource {
    /// Returns the value of the source corresponding to the enum variant.
    fn get_value(&self, registers: &Registers, bus: &MemoryBus, pc: u16) -> u8 {
        match &self {
            ArithmeticOrLogicalSource::Register(register) => register.get_register(registers),
            ArithmeticOrLogicalSource::D8 => bus.read_byte(pc + 1),
            ArithmeticOrLogicalSource::HL => bus.read_byte(registers.get_hl()),
        }
    }
}

/// Checks the condition of the instruction using the registers and returns true if the instruction should
/// execute, false otherwise.
fn check_instruction_condition(
    condition: InstructionCondition,
    flags_register: &FlagsRegister,
) -> bool {
    match condition {
        InstructionCondition::NotZero => !flags_register.zero,
        InstructionCondition::Zero => flags_register.zero,
        InstructionCondition::NotCarry => !flags_register.carry,
        InstructionCondition::Carry => flags_register.carry,
        InstructionCondition::Always => true,
    }
}
