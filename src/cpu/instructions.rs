//! This module contains the CPU instructions. The instructions are represented as an enum and
//! the CPU struct has a method to execute the instruction.
//!
//! The instructions are divided into two categories: prefixed and non-prefixed instructions.
//! For details please refer to [Pan Docs - CPU Instruction set](https://gbdev.io/pandocs/CPU_Instruction_Set.html),
//! the [interactive CPU instruction set guide](https://meganesu.github.io/generate-gb-opcodes/) or the
//! [CPU opcode reference](https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7).
//!
//! The instructions are implemented in separate modules for better organization and readability.
//! In the cases where instructions share the same target or source for their operations
//! (e.g. ADD and ADC who share [ArithmeticOrLogicalSource]), they use a common type to represent the target or source which is then
//! implemented in this module.

pub(crate) mod add_and_adc;
mod bit;
mod call_ret_rst_and_reti;
mod daa_scf_cpl_and_ccf;
mod di_and_ei;
mod halt;
mod inc_and_dec;
mod jr;
mod jump;
pub(crate) mod ldh;
pub(crate) mod load;
mod logical_operators;
mod parsing;
mod push_and_pop;
mod res_and_set;
mod rlc_rrc_rl_and_rr;
mod rlca_rrca_rla_and_rra;
mod sla_sra_and_srl;
mod sub_and_sbc;
mod swap;

use crate::cpu::registers::{CPURegisters, FlagsRegister};
use crate::{CPU, MemoryBus};
use add_and_adc::{AddWordSource, AddWordTarget};
use bit::BitInstructionType;
use inc_and_dec::IncDecTarget;
use jump::JumpType;
use ldh::LDHType;
use load::LoadType;
use push_and_pop::{PopTarget, PushSource};
use res_and_set::ResAndSetInstructionType;
use std::cmp::PartialEq;

/// Represents a CPU instruction. The instruction can be either a prefix or non-prefix instruction.
/// For details please refer to [Pan Docs - CPU Instruction set](https://gbdev.io/pandocs/CPU_Instruction_Set.html),
/// the [interactive CPU instruction set guide](https://meganesu.github.io/generate-gb-opcodes/) or
/// the [CPU opcode reference](https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instruction {
    // 8 Bit Opcodes
    NOP,
    ADDByte(ArithmeticOrLogicalSource),
    ADDWord(AddWordTarget, AddWordSource),
    ADC(ArithmeticOrLogicalSource),
    SUB(ArithmeticOrLogicalSource),
    SBC(ArithmeticOrLogicalSource),
    AND(ArithmeticOrLogicalSource),
    OR(ArithmeticOrLogicalSource),
    XOR(ArithmeticOrLogicalSource),
    CP(ArithmeticOrLogicalSource),
    INC(IncDecTarget),
    DEC(IncDecTarget),
    JP(JumpType),
    LD(LoadType),
    LDH(LDHType),
    PUSH(PushSource),
    POP(PopTarget),
    CALL(InstructionCondition),
    RET(InstructionCondition),
    RST(u16),
    JR(InstructionCondition),
    DAA,
    SCF,
    CPL,
    CCF,
    DI,
    EI,
    RETI,
    HALT,

    // 16 bit Opcodes
    RLC(SixteenBitInstructionTarget),
    RRC(SixteenBitInstructionTarget),
    RL(SixteenBitInstructionTarget),
    RR(SixteenBitInstructionTarget),
    SLA(SixteenBitInstructionTarget),
    SRA(SixteenBitInstructionTarget),
    SWAP(SixteenBitInstructionTarget),
    SRL(SixteenBitInstructionTarget),
    RLCA,
    RRCA,
    RLA,
    RRA,
    BIT(BitInstructionType),
    RES(ResAndSetInstructionType),
    SET(ResAndSetInstructionType),
}

/// Enum to represent the Registers of the CPU (except for the f register) as target or sources of operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Register {
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArithmeticOrLogicalSource {
    Register(Register),
    D8,
    HLRef,
}

/// Represents the possible conditions for an instruction to execute or not (e.g. JP or CALL).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstructionCondition {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

/// Represents the possible targets for the 16-bit opcodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SixteenBitInstructionTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLRef,
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
        let higher_nibble = (byte & 0xF0) >> 4;
        match higher_nibble {
            0x0 | 0x1 | 0x2 | 0x3 => Self::from_byte_prefixed_group_0(byte),
            0x4 | 0x5 | 0x6 | 0x7 => Self::from_byte_prefixed_group_1(byte),
            0x8 | 0x9 | 0xA | 0xB => Self::from_byte_prefixed_group_2(byte),
            0xC | 0xD | 0xE | 0xF => Self::from_byte_prefixed_group_3(byte),
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
    /// Executes the provided instruction on the CPU by matching the instruction and calling the
    /// corresponding handler function to execute the instruction.
    pub fn execute(&mut self, memory_bus: &mut MemoryBus, instruction: Instruction) -> u16 {
        use Instruction::*;
        let next_pc = match instruction {
            // 8 Bit Opcodes
            NOP => {
                self.increment_cycle_counter(1);
                self.pc.wrapping_add(1)
            }
            ADDByte(source) => self.handle_add_byte_instruction(memory_bus, source),
            ADDWord(target, source) => {
                self.handle_add_word_instruction(memory_bus, (target, source))
            }
            ADC(source) => self.handle_adc_instruction(memory_bus, source),
            SUB(source) => self.handle_sub_instruction(memory_bus, source),
            SBC(source) => self.handle_sbc_instruction(memory_bus, source),
            AND(source) => self.handle_and_instruction(memory_bus, source),
            OR(source) => self.handle_or_instruction(memory_bus, source),
            XOR(source) => self.handle_xor_instruction(memory_bus, source),
            CP(source) => self.handle_cp_instruction(memory_bus, source),
            JP(type_of_jump) => self.handle_jump_instruction(memory_bus, type_of_jump),
            LD(type_of_load) => self.handle_load_instruction(memory_bus, type_of_load),
            LDH(type_of_ldh) => self.handle_ldh_instruction(memory_bus, type_of_ldh),
            INC(target) => self.handle_inc_instruction(memory_bus, target),
            DEC(target) => self.handle_dec_instruction(memory_bus, target),
            CALL(condition) => self.handle_call_instruction(memory_bus, condition),
            RET(condition) => self.handle_ret_instruction(memory_bus, condition),
            PUSH(source) => self.handle_push_instruction(memory_bus, source),
            POP(target) => self.handle_pop_instruction(memory_bus, target),
            RST(address) => self.handle_rst_instruction(memory_bus, address),
            JR(condition) => self.handle_jr_instruction(memory_bus, condition),
            DAA => self.handle_daa_instruction(),
            SCF => self.handle_scf_instruction(),
            CPL => self.handle_cpl_instruction(),
            CCF => self.handle_ccf_instruction(),
            DI => self.handle_di_instruction(),
            EI => self.handle_ei_instruction(),
            RETI => self.handle_reti_instruction(memory_bus),
            RLCA => self.handle_rlca_instruction(),
            RRCA => self.handle_rrca_instruction(),
            RLA => self.handle_rla_instruction(),
            RRA => self.handle_rra_instruction(),
            HALT => self.handle_halt_instruction(),

            // 16-bit Opcodes
            RLC(target) => self.handle_rlc_instruction(memory_bus, target),
            RRC(target) => self.handle_rrc_instruction(memory_bus, target),
            RL(target) => self.handle_rl_instruction(memory_bus, target),
            RR(target) => self.handle_rr_instruction(memory_bus, target),
            SLA(target) => self.handle_sla_instruction(memory_bus, target),
            SRA(target) => self.handle_sra_instruction(memory_bus, target),
            SWAP(target) => self.handle_swap_instruction(memory_bus, target),
            SRL(target) => self.handle_srl_instruction(memory_bus, target),
            BIT(type_of_bit_instruction) => {
                self.handle_bit_instruction(memory_bus, type_of_bit_instruction)
            }
            RES(type_of_res) => self.handle_res_instruction(memory_bus, type_of_res),
            SET(type_of_set) => self.handle_set_instruction(memory_bus, type_of_set),
        };

        if instruction != EI && self.ime_to_be_set {
            self.ime = true;
            self.ime_to_be_set = false;
        }

        next_pc
    }
}

impl Register {
    /// Returns the value of the register corresponding to the enum variant.
    fn get_register(&self, registers: &CPURegisters) -> u8 {
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
    fn set_register(&self, registers: &mut CPURegisters, value: u8) {
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
    fn get_value(&self, memory_bus: &MemoryBus, registers: &CPURegisters, pc: u16) -> u8 {
        match &self {
            ArithmeticOrLogicalSource::Register(register) => register.get_register(registers),
            ArithmeticOrLogicalSource::D8 => memory_bus.read_byte(pc + 1),
            ArithmeticOrLogicalSource::HLRef => memory_bus.read_byte(registers.get_hl()),
        }
    }

    /// Returns the next program counter value and increments the cycle counter according to the source
    fn increment_pc_and_cycle(self, cpu: &mut CPU) -> u16 {
        match self {
            ArithmeticOrLogicalSource::D8 => {
                cpu.increment_cycle_counter(2);
                cpu.pc.wrapping_add(2)
            }
            ArithmeticOrLogicalSource::HLRef => {
                cpu.increment_cycle_counter(2);
                cpu.pc.wrapping_add(1)
            }
            _ => {
                cpu.increment_cycle_counter(1);
                cpu.pc.wrapping_add(1)
            }
        }
    }
}

impl SixteenBitInstructionTarget {
    /// Returns the value of the target register.
    pub fn get_value(&self, memory_bus: &MemoryBus, cpu: &CPU) -> u8 {
        match self {
            SixteenBitInstructionTarget::A => cpu.registers.a,
            SixteenBitInstructionTarget::B => cpu.registers.b,
            SixteenBitInstructionTarget::C => cpu.registers.c,
            SixteenBitInstructionTarget::D => cpu.registers.d,
            SixteenBitInstructionTarget::E => cpu.registers.e,
            SixteenBitInstructionTarget::H => cpu.registers.h,
            SixteenBitInstructionTarget::L => cpu.registers.l,
            SixteenBitInstructionTarget::HLRef => memory_bus.read_byte(cpu.registers.get_hl()),
        }
    }

    /// Sets the value of the target register.
    pub fn set_value(&self, memory_bus: &mut MemoryBus, cpu: &mut CPU, value: u8) {
        match self {
            SixteenBitInstructionTarget::A => cpu.registers.a = value,
            SixteenBitInstructionTarget::B => cpu.registers.b = value,
            SixteenBitInstructionTarget::C => cpu.registers.c = value,
            SixteenBitInstructionTarget::D => cpu.registers.d = value,
            SixteenBitInstructionTarget::E => cpu.registers.e = value,
            SixteenBitInstructionTarget::H => cpu.registers.h = value,
            SixteenBitInstructionTarget::L => cpu.registers.l = value,
            SixteenBitInstructionTarget::HLRef => {
                memory_bus.write_byte(cpu.registers.get_hl(), value)
            }
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
        InstructionCondition::NotZero => !flags_register.get_zero_flag(),
        InstructionCondition::Zero => flags_register.get_zero_flag(),
        InstructionCondition::NotCarry => !flags_register.get_carry_flag(),
        InstructionCondition::Carry => flags_register.get_carry_flag(),
        InstructionCondition::Always => true,
    }
}

/// Represents the possible bits to check in the BIT instruction or RES(et)/SET in the respective
/// instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitTarget {
    Bit0,
    Bit1,
    Bit2,
    Bit3,
    Bit4,
    Bit5,
    Bit6,
    Bit7,
}
