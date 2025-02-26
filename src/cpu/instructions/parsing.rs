use super::load::{LoadByteSource, LoadByteTarget, LoadType, LoadWordSource, LoadWordTarget};
use super::{Instruction, Register};
use crate::cpu::instructions::ArithmeticSource;

impl Instruction {
    /// Returns the non-prefix instruction corresponding to the given byte in group 0.
    /// Group 0 consists of the non-prefixed instructions where the higher nibble is 0, 1, 2 or 3.
    /// See [Interactive CPU Instructions](https://meganesu.github.io/generate-gb-opcodes/)
    /// or [CPU opcode reference](https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7) for details.
    ///
    /// Group 0 consists of miscellaneous instructions.
    pub(super) fn from_byte_not_prefixed_group_0(byte: u8) -> Option<Instruction> {
        match byte {
            0x00 => Some(Instruction::NOP),
            0x01 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::BC,
                LoadWordSource::D16,
            ))),
            0x02 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::BCRef,
                LoadByteSource::REGISTER(Register::A),
            ))),
            // TODO: Add missing instructions
            0x06 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::B),
                LoadByteSource::D8,
            ))),
            // TODO: Add missing instructions
            0x08 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::A16Ref,
                LoadWordSource::SP,
            ))),
            // TODO: Add missing instructions
            0x0A => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::BCRef,
            ))),
            // TODO: Add missing instructions
            0x0E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::C),
                LoadByteSource::D8,
            ))),

            // TODO: Add missing instructions
            0x11 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::DE,
                LoadWordSource::D16,
            ))),
            0x12 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::DERef,
                LoadByteSource::REGISTER(Register::A),
            ))),
            // TODO: Add missing instructions
            0x16 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::D),
                LoadByteSource::D8,
            ))),
            // TODO: Add missing instructions
            0x1A => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::DERef,
            ))),
            // TODO: Add missing instructions
            0x1E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::E),
                LoadByteSource::D8,
            ))),
            // TODO: Add missing instructions

            // TODO: Add missing instructions
            0x21 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::HL,
                LoadWordSource::D16,
            ))),
            0x22 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLRefIncrement,
                LoadByteSource::REGISTER(Register::A),
            ))),
            // TODO: Add missing instructions
            0x26 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::H),
                LoadByteSource::D8,
            ))),
            // TODO: Add missing instructions
            0x2A => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::HLRefIncrement,
            ))),
            // TODO: Add missing instructions
            0x2E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::L),
                LoadByteSource::D8,
            ))),
            // TODO: Add missing instructions
            0x31 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::SP,
                LoadWordSource::D16,
            ))),
            0x32 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLRefDecrement,
                LoadByteSource::REGISTER(Register::A),
            ))),
            // TODO: Add missing instructions
            0x36 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLRef,
                LoadByteSource::D8,
            ))),
            // TODO: Add missing instructions
            0x3A => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::HLRefDecrement,
            ))),
            // TODO: Add missing instructions
            0x3E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::D8,
            ))),
            // TODO: Add missing instructions
            _ => None,
        }
    }

    /// Returns the non-prefix instruction corresponding to the given byte in group 1.
    /// Group 1 consists of the non-prefixed instructions where the higher nibble is 4, 5, 6 or 7.
    /// See [Interactive CPU Instructions](https://meganesu.github.io/generate-gb-opcodes/)
    /// or [CPU opcode reference](https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7) for details.
    ///
    /// Group 1 consists of LD instructions and the HALT instruction.
    pub(super) fn from_byte_not_prefixed_group_1(byte: u8) -> Option<Instruction> {
        match byte {
            0x40 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::B),
                LoadByteSource::REGISTER(Register::B),
            ))),
            0x41 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::B),
                LoadByteSource::REGISTER(Register::C),
            ))),
            0x42 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::B),
                LoadByteSource::REGISTER(Register::D),
            ))),
            0x43 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::B),
                LoadByteSource::REGISTER(Register::E),
            ))),
            0x44 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::B),
                LoadByteSource::REGISTER(Register::H),
            ))),
            0x45 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::B),
                LoadByteSource::REGISTER(Register::L),
            ))),
            0x46 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::B),
                LoadByteSource::HLRef,
            ))),
            0x47 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::B),
                LoadByteSource::REGISTER(Register::A),
            ))),
            0x48 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::C),
                LoadByteSource::REGISTER(Register::B),
            ))),
            0x49 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::C),
                LoadByteSource::REGISTER(Register::C),
            ))),
            0x4A => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::C),
                LoadByteSource::REGISTER(Register::D),
            ))),
            0x4B => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::C),
                LoadByteSource::REGISTER(Register::E),
            ))),
            0x4C => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::C),
                LoadByteSource::REGISTER(Register::H),
            ))),
            0x4D => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::C),
                LoadByteSource::REGISTER(Register::L),
            ))),
            0x4E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::C),
                LoadByteSource::HLRef,
            ))),
            0x4F => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::C),
                LoadByteSource::REGISTER(Register::A),
            ))),
            0x50 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::D),
                LoadByteSource::REGISTER(Register::B),
            ))),
            0x51 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::D),
                LoadByteSource::REGISTER(Register::C),
            ))),
            0x52 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::D),
                LoadByteSource::REGISTER(Register::D),
            ))),
            0x53 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::D),
                LoadByteSource::REGISTER(Register::E),
            ))),
            0x54 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::D),
                LoadByteSource::REGISTER(Register::H),
            ))),
            0x55 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::D),
                LoadByteSource::REGISTER(Register::L),
            ))),
            0x56 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::D),
                LoadByteSource::HLRef,
            ))),
            0x57 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::D),
                LoadByteSource::REGISTER(Register::A),
            ))),
            0x58 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::E),
                LoadByteSource::REGISTER(Register::B),
            ))),
            0x59 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::E),
                LoadByteSource::REGISTER(Register::C),
            ))),
            0x5A => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::E),
                LoadByteSource::REGISTER(Register::D),
            ))),
            0x5B => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::E),
                LoadByteSource::REGISTER(Register::E),
            ))),
            0x5C => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::E),
                LoadByteSource::REGISTER(Register::H),
            ))),
            0x5D => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::E),
                LoadByteSource::REGISTER(Register::L),
            ))),
            0x5E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::E),
                LoadByteSource::HLRef,
            ))),
            0x5F => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::E),
                LoadByteSource::REGISTER(Register::A),
            ))),
            0x60 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::H),
                LoadByteSource::REGISTER(Register::B),
            ))),
            0x61 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::H),
                LoadByteSource::REGISTER(Register::C),
            ))),
            0x62 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::H),
                LoadByteSource::REGISTER(Register::D),
            ))),
            0x63 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::H),
                LoadByteSource::REGISTER(Register::E),
            ))),
            0x64 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::H),
                LoadByteSource::REGISTER(Register::H),
            ))),
            0x65 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::H),
                LoadByteSource::REGISTER(Register::L),
            ))),
            0x66 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::H),
                LoadByteSource::HLRef,
            ))),
            0x67 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::H),
                LoadByteSource::REGISTER(Register::A),
            ))),
            0x68 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::L),
                LoadByteSource::REGISTER(Register::B),
            ))),
            0x69 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::L),
                LoadByteSource::REGISTER(Register::C),
            ))),
            0x6A => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::L),
                LoadByteSource::REGISTER(Register::D),
            ))),
            0x6B => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::L),
                LoadByteSource::REGISTER(Register::E),
            ))),
            0x6C => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::L),
                LoadByteSource::REGISTER(Register::H),
            ))),
            0x6D => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::L),
                LoadByteSource::REGISTER(Register::L),
            ))),
            0x6E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::L),
                LoadByteSource::HLRef,
            ))),
            0x6F => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::L),
                LoadByteSource::REGISTER(Register::A),
            ))),
            0x70 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLRef,
                LoadByteSource::REGISTER(Register::B),
            ))),
            0x71 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLRef,
                LoadByteSource::REGISTER(Register::C),
            ))),
            0x72 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLRef,
                LoadByteSource::REGISTER(Register::D),
            ))),
            0x73 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLRef,
                LoadByteSource::REGISTER(Register::E),
            ))),
            0x74 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLRef,
                LoadByteSource::REGISTER(Register::H),
            ))),
            0x75 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLRef,
                LoadByteSource::REGISTER(Register::L),
            ))),
            // TODO: Add HALT Instruction
            0x77 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLRef,
                LoadByteSource::REGISTER(Register::A),
            ))),
            0x78 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::REGISTER(Register::B),
            ))),
            0x79 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::REGISTER(Register::C),
            ))),
            0x7A => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::REGISTER(Register::D),
            ))),
            0x7B => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::REGISTER(Register::E),
            ))),
            0x7C => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::REGISTER(Register::H),
            ))),
            0x7D => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::REGISTER(Register::L),
            ))),
            0x7E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::HLRef,
            ))),
            0x7F => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::REGISTER(Register::A),
            ))),
            _ => None,
        }
    }

    /// Returns the non-prefix instruction corresponding to the given byte in group 2.
    /// Group 2 consists of the non-prefixed instructions where the higher nibble is 8, 9, A or B.
    /// See [Interactive CPU Instructions](https://meganesu.github.io/generate-gb-opcodes/) or [CPU opcode reference](https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7)  or [CPU opcode reference](https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7)
    /// for details.
    ///
    /// Group 2 consists of arithmetic instructions.
    pub(super) fn from_byte_not_prefixed_group_2(byte: u8) -> Option<Instruction> {
        match byte {
            0x80 => Some(Instruction::ADDToA(ArithmeticSource::Register(Register::B))),
            0x81 => Some(Instruction::ADDToA(ArithmeticSource::Register(Register::C))),
            0x82 => Some(Instruction::ADDToA(ArithmeticSource::Register(Register::D))),
            0x83 => Some(Instruction::ADDToA(ArithmeticSource::Register(Register::E))),
            0x84 => Some(Instruction::ADDToA(ArithmeticSource::Register(Register::H))),
            0x85 => Some(Instruction::ADDToA(ArithmeticSource::Register(Register::L))),
            0x86 => Some(Instruction::ADDToA(ArithmeticSource::HL)),
            0x87 => Some(Instruction::ADDToA(ArithmeticSource::Register(Register::A))),
            0x88 => Some(Instruction::ADC(ArithmeticSource::Register(Register::B))),
            0x89 => Some(Instruction::ADC(ArithmeticSource::Register(Register::C))),
            0x8A => Some(Instruction::ADC(ArithmeticSource::Register(Register::D))),
            0x8B => Some(Instruction::ADC(ArithmeticSource::Register(Register::E))),
            0x8C => Some(Instruction::ADC(ArithmeticSource::Register(Register::H))),
            0x8D => Some(Instruction::ADC(ArithmeticSource::Register(Register::L))),
            0x8E => Some(Instruction::ADC(ArithmeticSource::HL)),
            0x8F => Some(Instruction::ADC(ArithmeticSource::Register(Register::A))),

            0x90 => Some(Instruction::SUB(ArithmeticSource::Register(Register::B))),
            0x91 => Some(Instruction::SUB(ArithmeticSource::Register(Register::C))),
            0x92 => Some(Instruction::SUB(ArithmeticSource::Register(Register::D))),
            0x93 => Some(Instruction::SUB(ArithmeticSource::Register(Register::E))),
            0x94 => Some(Instruction::SUB(ArithmeticSource::Register(Register::H))),
            0x95 => Some(Instruction::SUB(ArithmeticSource::Register(Register::L))),
            0x96 => Some(Instruction::SUB(ArithmeticSource::HL)),
            0x97 => Some(Instruction::SUB(ArithmeticSource::Register(Register::A))),
            0x98 => Some(Instruction::SBC(ArithmeticSource::Register(Register::B))),
            0x99 => Some(Instruction::SBC(ArithmeticSource::Register(Register::C))),
            0x9A => Some(Instruction::SBC(ArithmeticSource::Register(Register::D))),
            0x9B => Some(Instruction::SBC(ArithmeticSource::Register(Register::E))),
            0x9C => Some(Instruction::SBC(ArithmeticSource::Register(Register::H))),
            0x9D => Some(Instruction::SBC(ArithmeticSource::Register(Register::L))),
            0x9E => Some(Instruction::SBC(ArithmeticSource::HL)),
            0x9F => Some(Instruction::SBC(ArithmeticSource::Register(Register::A))),

            0xA0 => Some(Instruction::AND(ArithmeticSource::Register(Register::B))),
            0xA1 => Some(Instruction::AND(ArithmeticSource::Register(Register::C))),
            0xA2 => Some(Instruction::AND(ArithmeticSource::Register(Register::D))),
            0xA3 => Some(Instruction::AND(ArithmeticSource::Register(Register::E))),
            0xA4 => Some(Instruction::AND(ArithmeticSource::Register(Register::H))),
            0xA5 => Some(Instruction::AND(ArithmeticSource::Register(Register::L))),
            0xA6 => Some(Instruction::AND(ArithmeticSource::HL)),
            0xA7 => Some(Instruction::AND(ArithmeticSource::Register(Register::A))),
            0xA8 => Some(Instruction::XOR(ArithmeticSource::Register(Register::B))),
            0xA9 => Some(Instruction::XOR(ArithmeticSource::Register(Register::C))),
            0xAA => Some(Instruction::XOR(ArithmeticSource::Register(Register::D))),
            0xAB => Some(Instruction::XOR(ArithmeticSource::Register(Register::E))),
            0xAC => Some(Instruction::XOR(ArithmeticSource::Register(Register::H))),
            0xAD => Some(Instruction::XOR(ArithmeticSource::Register(Register::L))),
            0xAE => Some(Instruction::XOR(ArithmeticSource::HL)),
            0xAF => Some(Instruction::XOR(ArithmeticSource::Register(Register::A))),

            0xB0 => Some(Instruction::OR(ArithmeticSource::Register(Register::B))),
            0xB1 => Some(Instruction::OR(ArithmeticSource::Register(Register::C))),
            0xB2 => Some(Instruction::OR(ArithmeticSource::Register(Register::D))),
            0xB3 => Some(Instruction::OR(ArithmeticSource::Register(Register::E))),
            0xB4 => Some(Instruction::OR(ArithmeticSource::Register(Register::H))),
            0xB5 => Some(Instruction::OR(ArithmeticSource::Register(Register::L))),
            0xB6 => Some(Instruction::OR(ArithmeticSource::HL)),
            0xB7 => Some(Instruction::OR(ArithmeticSource::Register(Register::A))),
            0xB8 => Some(Instruction::CP(ArithmeticSource::Register(Register::B))),
            0xB9 => Some(Instruction::CP(ArithmeticSource::Register(Register::C))),
            0xBA => Some(Instruction::CP(ArithmeticSource::Register(Register::D))),
            0xBB => Some(Instruction::CP(ArithmeticSource::Register(Register::E))),
            0xBC => Some(Instruction::CP(ArithmeticSource::Register(Register::H))),
            0xBD => Some(Instruction::CP(ArithmeticSource::Register(Register::L))),
            0xBE => Some(Instruction::CP(ArithmeticSource::HL)),
            0xBF => Some(Instruction::CP(ArithmeticSource::Register(Register::A))),

            _ => None,
        }
    }

    /// Returns the non-prefix instruction corresponding to the given byte in group 3.
    /// Group 3 consists of the non-prefixed instructions where the higher nibble is C, D, E or F.
    /// See [Interactive CPU Instructions](https://meganesu.github.io/generate-gb-opcodes/) or [CPU opcode reference](https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7)
    /// for details.
    ///
    /// Group 3 consists of control flow and miscellaneous instructions.
    pub(super) fn from_byte_not_prefixed_group_3(byte: u8) -> Option<Instruction> {
        match byte {
            _ => None,
        }
    }
}
