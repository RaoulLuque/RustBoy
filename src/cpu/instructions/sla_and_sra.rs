use super::SixteenBitInstructionTarget;
use crate::RustBoy;

impl RustBoy {
    /// Handles the SLA instruction for the given [super::SixteenBitInstructionTarget].
    ///
    /// The SLA instruction takes 2 cycles if the target is a register and 4 otherwise.
    pub fn handle_sla_instruction(&mut self, target: SixteenBitInstructionTarget) -> u16 {
        let value = target.get_value(&self);
        let new_value = self.sla(value);
        target.set_value(self, new_value);
        match target {
            SixteenBitInstructionTarget::HLRef => self.increment_cycle_counter(4),
            _ => self.increment_cycle_counter(2),
        }
        self.pc.wrapping_add(2)
    }

    /// Shifts the given value left and sets the carry flag if the shift wraps around.
    fn sla(&mut self, value: u8) -> u8 {
        let new_value = value << 1;
        let carry = value & 0b1000_0000 != 0;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;
        new_value
    }
}
