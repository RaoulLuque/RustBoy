use super::SixteenBitInstructionTarget;
use crate::{CPU, MemoryBus};

impl CPU {
    /// Handles the SWAP instruction for the given [super::SixteenBitInstructionTarget].
    ///
    /// The SWAP instruction takes 2 cycles if the target is a register and 4 otherwise.
    pub fn handle_swap_instruction(
        &mut self,
        memory_bus: &mut MemoryBus,
        target: SixteenBitInstructionTarget,
    ) -> u16 {
        match target {
            SixteenBitInstructionTarget::HLRef => self.increment_cycle_counter(4),
            _ => self.increment_cycle_counter(2),
        }
        let value = target.get_value(memory_bus, &self);
        let new_value = self.swap(value);
        target.set_value(memory_bus, self, new_value);
        self.pc.wrapping_add(2)
    }

    /// Swaps the upper and lower nibble of the given value and sets the zero flag if the result is
    /// zero.
    fn swap(&mut self, value: u8) -> u8 {
        let new_value = (value << 4) | (value >> 4);
        self.registers.f.set_zero_flag(new_value == 0);
        self.registers.f.set_subtract_flag(false);
        self.registers.f.set_half_carry_flag(false);
        self.registers.f.set_carry_flag(false);
        new_value
    }
}
