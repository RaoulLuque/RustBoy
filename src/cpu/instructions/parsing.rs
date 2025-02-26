use super::load::{LoadByteSource, LoadByteTarget, LoadType};
use super::{Instruction, Register};

impl Instruction {
    /// Returns the non-prefix instruction corresponding to the given byte in group 0.
    /// Group 0 consists of the non-prefixed instructions where the higher nibble is 0, 1, 2 or 3.
    /// See [Interactive CPU Instructions](https://meganesu.github.io/generate-gb-opcodes/)
    /// for details.
    pub(super) fn from_byte_not_prefixed_group_0(byte: u8) -> Option<Instruction> {
        match byte {
            _ => None,
        }
    }

    /// Returns the non-prefix instruction corresponding to the given byte in group 1.
    /// Group 1 consists of the non-prefixed instructions where the higher nibble is 4, 5, 6 or 7.
    /// See [Interactive CPU Instructions](https://meganesu.github.io/generate-gb-opcodes/)
    /// for details.
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
            _ => None,
        }
    }

    /// Returns the non-prefix instruction corresponding to the given byte in group 2.
    /// Group 2 consists of the non-prefixed instructions where the higher nibble is 8, 9, A or B.
    /// See [Interactive CPU Instructions](https://meganesu.github.io/generate-gb-opcodes/)
    /// for details.
    pub(super) fn from_byte_not_prefixed_group_2(byte: u8) -> Option<Instruction> {
        match byte {
            0x80 => Some(Instruction::ADD(Register::B)),
            0x81 => Some(Instruction::ADD(Register::C)),
            0x82 => Some(Instruction::ADD(Register::D)),
            0x83 => Some(Instruction::ADD(Register::E)),
            0x84 => Some(Instruction::ADD(Register::H)),
            0x85 => Some(Instruction::ADD(Register::L)),
            0x87 => Some(Instruction::ADD(Register::A)),
            _ => None,
        }
    }

    /// Returns the non-prefix instruction corresponding to the given byte in group 3.
    /// Group 3 consists of the non-prefixed instructions where the higher nibble is C, D, E or F.
    /// See [Interactive CPU Instructions](https://meganesu.github.io/generate-gb-opcodes/)
    /// for details.
    pub(super) fn from_byte_not_prefixed_group_3(byte: u8) -> Option<Instruction> {
        match byte {
            _ => None,
        }
    }
}
