use super::{BitTarget, SixteenBitInstructionTarget};
use crate::{CPU, MemoryBus};

/// Represents the possible types of RES or SET instructions
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResAndSetInstructionType {
    Type(SixteenBitInstructionTarget, BitTarget),
}

impl CPU {
    /// Handles the res instruction for the given [ResAndSetInstructionType].
    ///
    /// The RES instruction takes 2 cycles if the target is a register and 4 if it is the memory
    /// where HL points to.
    pub fn handle_res_instruction(
        &mut self,
        memory_bus: &mut MemoryBus,
        res_instruction_type: ResAndSetInstructionType,
    ) -> u16 {
        match res_instruction_type {
            ResAndSetInstructionType::Type(target, bit_to_reset) => {
                match target {
                    SixteenBitInstructionTarget::HLRef => {
                        self.increment_cycle_counter(4);
                    }
                    _ => {
                        self.increment_cycle_counter(2);
                    }
                }
                let value = target.get_value(memory_bus, &self);
                let new_value = self.res(value, bit_to_reset);
                target.set_value(memory_bus, self, new_value);
                self.pc.wrapping_add(2)
            }
        }
    }

    /// Sets the given bit in the given value to 0. No flags are set or reset.
    fn res(&self, value: u8, bit_to_reset: BitTarget) -> u8 {
        // bit_to_check converts the enum to a u8. This is done by just assigning the different
        // enum variants their indexes in the declaration starting at 0
        let mask = 1 << (bit_to_reset as u8);
        let new_value = value & !mask;
        new_value
    }

    /// Handles the set instruction for the given [SetInstructionType].
    ///
    /// The SET instruction takes 2 cycles if the source is a register and 3 otherwise.
    pub fn handle_set_instruction(
        &mut self,
        memory_bus: &mut MemoryBus,
        set_instruction_type: ResAndSetInstructionType,
    ) -> u16 {
        match set_instruction_type {
            ResAndSetInstructionType::Type(target, bit_to_set) => {
                match target {
                    SixteenBitInstructionTarget::HLRef => {
                        self.increment_cycle_counter(4);
                    }
                    _ => {
                        self.increment_cycle_counter(2);
                    }
                }
                let value = target.get_value(memory_bus, &self);
                let new_value = self.set(value, bit_to_set);
                target.set_value(memory_bus, self, new_value);
                self.pc.wrapping_add(2)
            }
        }
    }

    /// Sets the given bit in the given value to 1. No flags are set or reset.
    fn set(&self, value: u8, bit_to_reset: BitTarget) -> u8 {
        // bit_to_check converts the enum to a u8. This is done by just assigning the different
        // enum variants their indexes in the declaration starting at 0
        let mask = 1 << (bit_to_reset as u8);
        let new_value = value | mask;
        new_value
    }
}
