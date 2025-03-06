use super::load::{LoadByteSource, LoadByteTarget, LoadType, LoadWordSource, LoadWordTarget};
use super::{
    IncDecTarget, Instruction, InstructionCondition, JumpType, LDHType, PopTarget, PushSource,
    Register, SixteenBitInstructionTarget,
};
use crate::cpu::instructions::add_and_adc::{AddWordSource, AddWordTarget};
use crate::cpu::instructions::ldh::LDHSourceOrTarget;
use crate::cpu::instructions::ArithmeticOrLogicalSource;

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
            0x03 => Some(Instruction::INC(IncDecTarget::BC)),
            0x04 => Some(Instruction::INC(IncDecTarget::Register(Register::B))),
            0x05 => Some(Instruction::DEC(IncDecTarget::Register(Register::B))),
            0x06 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::B),
                LoadByteSource::D8,
            ))),
            // TODO: Add missing instructions
            0x08 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::A16Ref,
                LoadWordSource::SP,
            ))),
            0x09 => Some(Instruction::ADDWord(AddWordTarget::HL, AddWordSource::BC)),
            0x0A => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::BCRef,
            ))),
            0x0B => Some(Instruction::DEC(IncDecTarget::BC)),
            0x0C => Some(Instruction::INC(IncDecTarget::Register(Register::C))),
            0x0D => Some(Instruction::DEC(IncDecTarget::Register(Register::C))),
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
            0x13 => Some(Instruction::INC(IncDecTarget::DE)),
            0x14 => Some(Instruction::INC(IncDecTarget::Register(Register::D))),
            0x15 => Some(Instruction::DEC(IncDecTarget::Register(Register::D))),
            0x16 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::D),
                LoadByteSource::D8,
            ))),
            // TODO: Add missing instructions
            0x18 => Some(Instruction::JR(InstructionCondition::Always)),
            0x19 => Some(Instruction::ADDWord(AddWordTarget::HL, AddWordSource::DE)),
            0x1A => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::DERef,
            ))),
            0x1B => Some(Instruction::DEC(IncDecTarget::DE)),
            0x1C => Some(Instruction::INC(IncDecTarget::Register(Register::E))),
            0x1D => Some(Instruction::DEC(IncDecTarget::Register(Register::E))),
            0x1E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::E),
                LoadByteSource::D8,
            ))),
            // TODO: Add missing instructions
            0x20 => Some(Instruction::JR(InstructionCondition::NotZero)),
            0x21 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::HL,
                LoadWordSource::D16,
            ))),
            0x22 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLRefIncrement,
                LoadByteSource::REGISTER(Register::A),
            ))),
            0x23 => Some(Instruction::INC(IncDecTarget::HL)),
            0x24 => Some(Instruction::INC(IncDecTarget::Register(Register::H))),
            0x25 => Some(Instruction::DEC(IncDecTarget::Register(Register::H))),
            0x26 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::H),
                LoadByteSource::D8,
            ))),
            0x27 => Some(Instruction::DAA),
            0x28 => Some(Instruction::JR(InstructionCondition::Zero)),
            0x29 => Some(Instruction::ADDWord(AddWordTarget::HL, AddWordSource::HL)),
            0x2A => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::HLRefIncrement,
            ))),
            0x2B => Some(Instruction::DEC(IncDecTarget::HL)),
            0x2C => Some(Instruction::INC(IncDecTarget::Register(Register::L))),
            0x2D => Some(Instruction::DEC(IncDecTarget::Register(Register::L))),
            0x2E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::L),
                LoadByteSource::D8,
            ))),
            0x2F => Some(Instruction::CPL),
            0x30 => Some(Instruction::JR(InstructionCondition::NotCarry)),
            0x31 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::SP,
                LoadWordSource::D16,
            ))),
            0x32 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLRefDecrement,
                LoadByteSource::REGISTER(Register::A),
            ))),
            0x33 => Some(Instruction::INC(IncDecTarget::SP)),
            0x34 => Some(Instruction::INC(IncDecTarget::HLRef)),
            0x35 => Some(Instruction::DEC(IncDecTarget::HLRef)),
            0x36 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLRef,
                LoadByteSource::D8,
            ))),
            0x37 => Some(Instruction::SCF),
            0x38 => Some(Instruction::JR(InstructionCondition::Carry)),
            0x39 => Some(Instruction::ADDWord(AddWordTarget::HL, AddWordSource::SP)),
            0x3A => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::HLRefDecrement,
            ))),
            0x3B => Some(Instruction::DEC(IncDecTarget::SP)),
            0x3C => Some(Instruction::INC(IncDecTarget::Register(Register::A))),
            0x3D => Some(Instruction::DEC(IncDecTarget::Register(Register::A))),
            0x3E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::D8,
            ))),
            0x3F => Some(Instruction::CCF),
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
            0x80 => Some(Instruction::ADDByte(ArithmeticOrLogicalSource::Register(
                Register::B,
            ))),
            0x81 => Some(Instruction::ADDByte(ArithmeticOrLogicalSource::Register(
                Register::C,
            ))),
            0x82 => Some(Instruction::ADDByte(ArithmeticOrLogicalSource::Register(
                Register::D,
            ))),
            0x83 => Some(Instruction::ADDByte(ArithmeticOrLogicalSource::Register(
                Register::E,
            ))),
            0x84 => Some(Instruction::ADDByte(ArithmeticOrLogicalSource::Register(
                Register::H,
            ))),
            0x85 => Some(Instruction::ADDByte(ArithmeticOrLogicalSource::Register(
                Register::L,
            ))),
            0x86 => Some(Instruction::ADDByte(ArithmeticOrLogicalSource::HLRef)),
            0x87 => Some(Instruction::ADDByte(ArithmeticOrLogicalSource::Register(
                Register::A,
            ))),
            0x88 => Some(Instruction::ADC(ArithmeticOrLogicalSource::Register(
                Register::B,
            ))),
            0x89 => Some(Instruction::ADC(ArithmeticOrLogicalSource::Register(
                Register::C,
            ))),
            0x8A => Some(Instruction::ADC(ArithmeticOrLogicalSource::Register(
                Register::D,
            ))),
            0x8B => Some(Instruction::ADC(ArithmeticOrLogicalSource::Register(
                Register::E,
            ))),
            0x8C => Some(Instruction::ADC(ArithmeticOrLogicalSource::Register(
                Register::H,
            ))),
            0x8D => Some(Instruction::ADC(ArithmeticOrLogicalSource::Register(
                Register::L,
            ))),
            0x8E => Some(Instruction::ADC(ArithmeticOrLogicalSource::HLRef)),
            0x8F => Some(Instruction::ADC(ArithmeticOrLogicalSource::Register(
                Register::A,
            ))),

            0x90 => Some(Instruction::SUB(ArithmeticOrLogicalSource::Register(
                Register::B,
            ))),
            0x91 => Some(Instruction::SUB(ArithmeticOrLogicalSource::Register(
                Register::C,
            ))),
            0x92 => Some(Instruction::SUB(ArithmeticOrLogicalSource::Register(
                Register::D,
            ))),
            0x93 => Some(Instruction::SUB(ArithmeticOrLogicalSource::Register(
                Register::E,
            ))),
            0x94 => Some(Instruction::SUB(ArithmeticOrLogicalSource::Register(
                Register::H,
            ))),
            0x95 => Some(Instruction::SUB(ArithmeticOrLogicalSource::Register(
                Register::L,
            ))),
            0x96 => Some(Instruction::SUB(ArithmeticOrLogicalSource::HLRef)),
            0x97 => Some(Instruction::SUB(ArithmeticOrLogicalSource::Register(
                Register::A,
            ))),
            0x98 => Some(Instruction::SBC(ArithmeticOrLogicalSource::Register(
                Register::B,
            ))),
            0x99 => Some(Instruction::SBC(ArithmeticOrLogicalSource::Register(
                Register::C,
            ))),
            0x9A => Some(Instruction::SBC(ArithmeticOrLogicalSource::Register(
                Register::D,
            ))),
            0x9B => Some(Instruction::SBC(ArithmeticOrLogicalSource::Register(
                Register::E,
            ))),
            0x9C => Some(Instruction::SBC(ArithmeticOrLogicalSource::Register(
                Register::H,
            ))),
            0x9D => Some(Instruction::SBC(ArithmeticOrLogicalSource::Register(
                Register::L,
            ))),
            0x9E => Some(Instruction::SBC(ArithmeticOrLogicalSource::HLRef)),
            0x9F => Some(Instruction::SBC(ArithmeticOrLogicalSource::Register(
                Register::A,
            ))),

            0xA0 => Some(Instruction::AND(ArithmeticOrLogicalSource::Register(
                Register::B,
            ))),
            0xA1 => Some(Instruction::AND(ArithmeticOrLogicalSource::Register(
                Register::C,
            ))),
            0xA2 => Some(Instruction::AND(ArithmeticOrLogicalSource::Register(
                Register::D,
            ))),
            0xA3 => Some(Instruction::AND(ArithmeticOrLogicalSource::Register(
                Register::E,
            ))),
            0xA4 => Some(Instruction::AND(ArithmeticOrLogicalSource::Register(
                Register::H,
            ))),
            0xA5 => Some(Instruction::AND(ArithmeticOrLogicalSource::Register(
                Register::L,
            ))),
            0xA6 => Some(Instruction::AND(ArithmeticOrLogicalSource::HLRef)),
            0xA7 => Some(Instruction::AND(ArithmeticOrLogicalSource::Register(
                Register::A,
            ))),
            0xA8 => Some(Instruction::XOR(ArithmeticOrLogicalSource::Register(
                Register::B,
            ))),
            0xA9 => Some(Instruction::XOR(ArithmeticOrLogicalSource::Register(
                Register::C,
            ))),
            0xAA => Some(Instruction::XOR(ArithmeticOrLogicalSource::Register(
                Register::D,
            ))),
            0xAB => Some(Instruction::XOR(ArithmeticOrLogicalSource::Register(
                Register::E,
            ))),
            0xAC => Some(Instruction::XOR(ArithmeticOrLogicalSource::Register(
                Register::H,
            ))),
            0xAD => Some(Instruction::XOR(ArithmeticOrLogicalSource::Register(
                Register::L,
            ))),
            0xAE => Some(Instruction::XOR(ArithmeticOrLogicalSource::HLRef)),
            0xAF => Some(Instruction::XOR(ArithmeticOrLogicalSource::Register(
                Register::A,
            ))),

            0xB0 => Some(Instruction::OR(ArithmeticOrLogicalSource::Register(
                Register::B,
            ))),
            0xB1 => Some(Instruction::OR(ArithmeticOrLogicalSource::Register(
                Register::C,
            ))),
            0xB2 => Some(Instruction::OR(ArithmeticOrLogicalSource::Register(
                Register::D,
            ))),
            0xB3 => Some(Instruction::OR(ArithmeticOrLogicalSource::Register(
                Register::E,
            ))),
            0xB4 => Some(Instruction::OR(ArithmeticOrLogicalSource::Register(
                Register::H,
            ))),
            0xB5 => Some(Instruction::OR(ArithmeticOrLogicalSource::Register(
                Register::L,
            ))),
            0xB6 => Some(Instruction::OR(ArithmeticOrLogicalSource::HLRef)),
            0xB7 => Some(Instruction::OR(ArithmeticOrLogicalSource::Register(
                Register::A,
            ))),
            0xB8 => Some(Instruction::CP(ArithmeticOrLogicalSource::Register(
                Register::B,
            ))),
            0xB9 => Some(Instruction::CP(ArithmeticOrLogicalSource::Register(
                Register::C,
            ))),
            0xBA => Some(Instruction::CP(ArithmeticOrLogicalSource::Register(
                Register::D,
            ))),
            0xBB => Some(Instruction::CP(ArithmeticOrLogicalSource::Register(
                Register::E,
            ))),
            0xBC => Some(Instruction::CP(ArithmeticOrLogicalSource::Register(
                Register::H,
            ))),
            0xBD => Some(Instruction::CP(ArithmeticOrLogicalSource::Register(
                Register::L,
            ))),
            0xBE => Some(Instruction::CP(ArithmeticOrLogicalSource::HLRef)),
            0xBF => Some(Instruction::CP(ArithmeticOrLogicalSource::Register(
                Register::A,
            ))),

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
            0xC0 => Some(Instruction::RET(InstructionCondition::NotZero)),
            0xC1 => Some(Instruction::POP(PopTarget::BC)),
            0xC2 => Some(Instruction::JP(JumpType::JumpToImmediateOperand(
                InstructionCondition::NotZero,
            ))),
            0xC3 => Some(Instruction::JP(JumpType::JumpToImmediateOperand(
                InstructionCondition::Always,
            ))),
            0xC4 => Some(Instruction::CALL(InstructionCondition::NotZero)),
            0xC5 => Some(Instruction::PUSH(PushSource::BC)),
            0xC6 => Some(Instruction::ADDByte(ArithmeticOrLogicalSource::D8)),
            0xC7 => Some(Instruction::RST(0x00)),
            0xC8 => Some(Instruction::RET(InstructionCondition::Zero)),
            0xC9 => Some(Instruction::RET(InstructionCondition::Always)),
            0xCA => Some(Instruction::JP(JumpType::JumpToImmediateOperand(
                InstructionCondition::Zero,
            ))),
            0xCC => Some(Instruction::CALL(InstructionCondition::Zero)),
            0xCD => Some(Instruction::CALL(InstructionCondition::Always)),
            0xCE => Some(Instruction::ADC(ArithmeticOrLogicalSource::D8)),
            0xCF => Some(Instruction::RST(0x08)),

            0xD0 => Some(Instruction::RET(InstructionCondition::NotCarry)),
            0xD1 => Some(Instruction::POP(PopTarget::DE)),
            0xD2 => Some(Instruction::JP(JumpType::JumpToImmediateOperand(
                InstructionCondition::NotCarry,
            ))),
            0xD4 => Some(Instruction::CALL(InstructionCondition::NotCarry)),
            0xD5 => Some(Instruction::PUSH(PushSource::DE)),
            0xD6 => Some(Instruction::SUB(ArithmeticOrLogicalSource::D8)),
            0xD7 => Some(Instruction::RST(0x10)),
            0xD8 => Some(Instruction::RET(InstructionCondition::Carry)),
            0xD9 => Some(Instruction::RETI),
            0xDA => Some(Instruction::JP(JumpType::JumpToImmediateOperand(
                InstructionCondition::Carry,
            ))),
            0xDC => Some(Instruction::CALL(InstructionCondition::Carry)),
            0xDE => Some(Instruction::SBC(ArithmeticOrLogicalSource::D8)),
            0xDF => Some(Instruction::RST(0x18)),

            0xE0 => Some(Instruction::LDH(LDHType::LDH(
                LDHSourceOrTarget::A8Ref,
                LDHSourceOrTarget::A,
            ))),
            0xE1 => Some(Instruction::POP(PopTarget::HL)),
            0xE2 => Some(Instruction::LDH(LDHType::LDH(
                LDHSourceOrTarget::CRef,
                LDHSourceOrTarget::A,
            ))),
            0xE5 => Some(Instruction::PUSH(PushSource::HL)),
            0xE6 => Some(Instruction::AND(ArithmeticOrLogicalSource::D8)),
            0xE7 => Some(Instruction::RST(0x20)),
            0xE8 => Some(Instruction::ADDWord(AddWordTarget::SP, AddWordSource::E8)),
            0xE9 => Some(Instruction::JP(JumpType::JumpToHL)),
            0xEA => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A16Ref,
                LoadByteSource::REGISTER(Register::A),
            ))),
            0xEE => Some(Instruction::XOR(ArithmeticOrLogicalSource::D8)),
            0xEF => Some(Instruction::RST(0x28)),

            0xF0 => Some(Instruction::LDH(LDHType::LDH(
                LDHSourceOrTarget::A,
                LDHSourceOrTarget::A8Ref,
            ))),
            0xF1 => Some(Instruction::POP(PopTarget::AF)),
            0xF2 => Some(Instruction::LDH(LDHType::LDH(
                LDHSourceOrTarget::A,
                LDHSourceOrTarget::CRef,
            ))),
            0xF3 => Some(Instruction::DI),
            0xF5 => Some(Instruction::PUSH(PushSource::AF)),
            0xF6 => Some(Instruction::OR(ArithmeticOrLogicalSource::D8)),
            0xF7 => Some(Instruction::RST(0x30)),
            0xF8 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::HL,
                LoadWordSource::SPPlusE8,
            ))),
            0xF9 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::SP,
                LoadWordSource::HL,
            ))),
            0xFA => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::REGISTER(Register::A),
                LoadByteSource::A16Ref,
            ))),
            0xFB => Some(Instruction::EI),
            0xFE => Some(Instruction::CP(ArithmeticOrLogicalSource::D8)),
            0xFF => Some(Instruction::RST(0x38)),
            _ => None,
        }
    }

    /// Returns the prefix instruction corresponding to the given byte in group 0.
    /// Group 0 consists of the prefixed instructions where the higher nibble is 0, 1, 2 or 3.
    /// See [Interactive CPU Instructions](https://meganesu.github.io/generate-gb-opcodes/)
    /// or [CPU opcode reference](https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7) for details.
    ///
    /// Group 0 consists of the instructions RLC, RRC, RL, RR, SLA, SRA, SWAP and SRL.
    pub(super) fn from_byte_prefixed_group_0(byte: u8) -> Option<Instruction> {
        let instruction = match byte {
            0x00 => Instruction::RLC(SixteenBitInstructionTarget::B),
            0x01 => Instruction::RLC(SixteenBitInstructionTarget::C),
            0x02 => Instruction::RLC(SixteenBitInstructionTarget::D),
            0x03 => Instruction::RLC(SixteenBitInstructionTarget::E),
            0x04 => Instruction::RLC(SixteenBitInstructionTarget::H),
            0x05 => Instruction::RLC(SixteenBitInstructionTarget::L),
            0x06 => Instruction::RLC(SixteenBitInstructionTarget::HLRef),
            0x07 => Instruction::RLC(SixteenBitInstructionTarget::A),
            0x08 => Instruction::RRC(SixteenBitInstructionTarget::B),
            0x09 => Instruction::RRC(SixteenBitInstructionTarget::C),
            0x0A => Instruction::RRC(SixteenBitInstructionTarget::D),
            0x0B => Instruction::RRC(SixteenBitInstructionTarget::E),
            0x0C => Instruction::RRC(SixteenBitInstructionTarget::H),
            0x0D => Instruction::RRC(SixteenBitInstructionTarget::L),
            0x0E => Instruction::RRC(SixteenBitInstructionTarget::HLRef),
            0x0F => Instruction::RRC(SixteenBitInstructionTarget::A),

            0x10 => Instruction::RL(SixteenBitInstructionTarget::B),
            0x11 => Instruction::RL(SixteenBitInstructionTarget::C),
            0x12 => Instruction::RL(SixteenBitInstructionTarget::D),
            0x13 => Instruction::RL(SixteenBitInstructionTarget::E),
            0x14 => Instruction::RL(SixteenBitInstructionTarget::H),
            0x15 => Instruction::RL(SixteenBitInstructionTarget::L),
            0x16 => Instruction::RL(SixteenBitInstructionTarget::HLRef),
            0x17 => Instruction::RL(SixteenBitInstructionTarget::A),
            0x18 => Instruction::RR(SixteenBitInstructionTarget::B),
            0x19 => Instruction::RR(SixteenBitInstructionTarget::C),
            0x1A => Instruction::RR(SixteenBitInstructionTarget::D),
            0x1B => Instruction::RR(SixteenBitInstructionTarget::E),
            0x1C => Instruction::RR(SixteenBitInstructionTarget::H),
            0x1D => Instruction::RR(SixteenBitInstructionTarget::L),
            0x1E => Instruction::RR(SixteenBitInstructionTarget::HLRef),
            0x1F => Instruction::RR(SixteenBitInstructionTarget::A),

            0x20 => Instruction::SLA(SixteenBitInstructionTarget::B),
            0x21 => Instruction::SLA(SixteenBitInstructionTarget::C),
            0x22 => Instruction::SLA(SixteenBitInstructionTarget::D),
            0x23 => Instruction::SLA(SixteenBitInstructionTarget::E),
            0x24 => Instruction::SLA(SixteenBitInstructionTarget::H),
            0x25 => Instruction::SLA(SixteenBitInstructionTarget::L),
            0x26 => Instruction::SLA(SixteenBitInstructionTarget::HLRef),
            0x27 => Instruction::SLA(SixteenBitInstructionTarget::A),
            0x28 => Instruction::SRA(SixteenBitInstructionTarget::B),
            0x29 => Instruction::SRA(SixteenBitInstructionTarget::C),
            0x2A => Instruction::SRA(SixteenBitInstructionTarget::D),
            0x2B => Instruction::SRA(SixteenBitInstructionTarget::E),
            0x2C => Instruction::SRA(SixteenBitInstructionTarget::H),
            0x2D => Instruction::SRA(SixteenBitInstructionTarget::L),
            0x2E => Instruction::SRA(SixteenBitInstructionTarget::HLRef),
            0x2F => Instruction::SRA(SixteenBitInstructionTarget::A),

            0x30 => Instruction::SWAP(SixteenBitInstructionTarget::B),
            0x31 => Instruction::SWAP(SixteenBitInstructionTarget::C),
            0x32 => Instruction::SWAP(SixteenBitInstructionTarget::D),
            0x33 => Instruction::SWAP(SixteenBitInstructionTarget::E),
            0x34 => Instruction::SWAP(SixteenBitInstructionTarget::H),
            0x35 => Instruction::SWAP(SixteenBitInstructionTarget::L),
            0x36 => Instruction::SWAP(SixteenBitInstructionTarget::HLRef),
            0x37 => Instruction::SWAP(SixteenBitInstructionTarget::A),
            // TODO: Add more instructions
            _ => return None,
        };
        Some(instruction)
    }

    /// Returns the prefix instruction corresponding to the given byte in group 1.
    /// Group 1 consists of the prefixed instructions where the higher nibble is 4, 5, 6 or 7.
    /// See [Interactive CPU Instructions](https://meganesu.github.io/generate-gb-opcodes/)
    /// or [CPU opcode reference](https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7) for details.
    ///
    /// Group 1 consists only of the BIT instruction.
    pub(super) fn from_byte_prefixed_group_1(byte: u8) -> Option<Instruction> {
        let instruction = match byte {
            // TODO: Add more instructions
            _ => return None,
        };
        Some(instruction)
    }

    /// Returns the prefix instruction corresponding to the given byte in group 2.
    /// Group 2 consists of the prefixed instructions where the higher nibble is 8, 9, A or B.
    /// See [Interactive CPU Instructions](https://meganesu.github.io/generate-gb-opcodes/)
    /// or [CPU opcode reference](https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7) for details.
    ///
    /// Group 2 consists only of the RES instruction.
    pub(super) fn from_byte_prefixed_group_2(byte: u8) -> Option<Instruction> {
        let instruction = match byte {
            // TODO: Add more instructions
            _ => return None,
        };
        Some(instruction)
    }

    /// Returns the prefix instruction corresponding to the given byte in group 3.
    /// Group 3 consists of the prefixed instructions where the higher nibble is C, D, E or F.
    /// See [Interactive CPU Instructions](https://meganesu.github.io/generate-gb-opcodes/)
    /// or [CPU opcode reference](https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7) for details.
    ///
    /// Group 3 consists only of the SET instruction.
    pub(super) fn from_byte_prefixed_group_3(byte: u8) -> Option<Instruction> {
        let instruction = match byte {
            // TODO: Add more instructions
            _ => return None,
        };
        Some(instruction)
    }
}
