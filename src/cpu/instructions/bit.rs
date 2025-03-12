use super::{BitTarget, SixteenBitInstructionTarget};
use crate::RustBoy;

/// Represents the possible types of bit instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitInstructionType {
    Bit(SixteenBitInstructionTarget, BitTarget),
}

impl RustBoy {
    /// Handles the bit instruction for the given [BitInstructionType].
    ///
    /// The BIT instruction takes 2 cycles if the target is a register and 3 if it is the memory
    /// where HL points to.
    pub fn handle_bit_instruction(&mut self, bit_instruction_type: BitInstructionType) -> u16 {
        match bit_instruction_type {
            BitInstructionType::Bit(target, bit_to_check) => {
                let value = target.get_value(&self);
                self.bit(value, bit_to_check);
                match target {
                    SixteenBitInstructionTarget::HLRef => {
                        self.increment_cycle_counter(3);
                    }
                    _ => {
                        self.increment_cycle_counter(2);
                    }
                }
                self.pc.wrapping_add(2)
            }
        }
    }

    /// Sets the zero flag if the given bit is not set in the given value. The subtract and half
    /// carry flags are set to false and true respectively.
    fn bit(&mut self, value: u8, bit_to_check: BitTarget) {
        // bit_to_check converts the enum to a u8. This is done by just assigning the different
        // enum variants their indexes in the declaration starting at 0
        let mask = 1 << (bit_to_check as u8);
        self.registers.f.zero = (value & mask) == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = true;
    }
}
