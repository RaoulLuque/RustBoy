use super::SixteenBitInstructionTarget;
use crate::RustBoy;

impl RustBoy {
    /// Handles the SWAP instruction for the given [super::SixteenBitInstructionTarget].
    ///
    /// The SWAP instruction takes 2 cycles if the target is a register and 4 otherwise.
    pub fn handle_swap_instruction(&mut self, target: SixteenBitInstructionTarget) -> u16 {
        match target {
            SixteenBitInstructionTarget::HLRef => self.increment_cycle_counter(4),
            _ => self.increment_cycle_counter(2),
        }
        let value = target.get_value(&self);
        let new_value = self.swap(value);
        target.set_value(self, new_value);
        self.pc.wrapping_add(2)
    }

    /// Swaps the upper and lower nibble of the given value and sets the zero flag if the result is
    /// zero.
    fn swap(&mut self, value: u8) -> u8 {
        let new_value = (value << 4) | (value >> 4);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
        new_value
    }
}
