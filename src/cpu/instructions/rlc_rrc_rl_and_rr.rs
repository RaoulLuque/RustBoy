use super::SixteenBitInstructionTarget;
use crate::RustBoy;

impl RustBoy {
    /// Handles the RLC instruction for the given [super::SixteenBitInstructionTarget].
    ///
    /// The RLC instruction takes 2 cycles if the target is a register and 4 otherwise.
    pub fn handle_rlc_instruction(&mut self, target: SixteenBitInstructionTarget) -> u16 {
        let value = target.get_value(&self);
        let new_value = self.rlc(value);
        target.set_value(self, new_value);
        match target {
            SixteenBitInstructionTarget::HLRef => self.increment_cycle_counter(4),
            _ => self.increment_cycle_counter(2),
        }
        self.pc.wrapping_add(2)
    }

    fn rlc(&mut self, value: u8) -> u8 {
        let new_value = value.rotate_left(1);
        let carry = value & 0b1000_0000 != 0;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;
        new_value
    }
}
