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

    /// Rotates the given value left and sets the flags in the flags register if the rotation wraps
    /// around.
    fn rlc(&mut self, value: u8) -> u8 {
        let new_value = value.rotate_left(1);
        let carry = value & 0b1000_0000 != 0;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;
        new_value
    }

    /// Handles the RRC instruction for the given [super::SixteenBitInstructionTarget].
    ///
    /// The RRC instruction takes 2 cycles if the target is a register and 4 otherwise.
    pub fn handle_rrc_instruction(&mut self, target: SixteenBitInstructionTarget) -> u16 {
        let value = target.get_value(&self);
        let new_value = self.rrc(value);
        target.set_value(self, new_value);
        match target {
            SixteenBitInstructionTarget::HLRef => self.increment_cycle_counter(4),
            _ => self.increment_cycle_counter(2),
        }
        self.pc.wrapping_add(2)
    }

    /// Rotates the given value right and sets the flags in the flags register if the rotation wraps
    /// around.
    fn rrc(&mut self, value: u8) -> u8 {
        let new_value = value.rotate_right(1);
        let carry = value & 0b0000_0001 != 0;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;
        new_value
    }

    /// Handles the RL instruction for the given [super::SixteenBitInstructionTarget].
    ///
    /// The RL instruction takes 2 cycles if the target is a register and 4 otherwise.
    pub fn handle_rl_instruction(&mut self, target: SixteenBitInstructionTarget) -> u16 {
        let value = target.get_value(&self);
        let new_value = self.rl(value);
        target.set_value(self, new_value);
        match target {
            SixteenBitInstructionTarget::HLRef => self.increment_cycle_counter(4),
            _ => self.increment_cycle_counter(2),
        }
        self.pc.wrapping_add(2)
    }

    /// Rotates the given value left through the carry flag
    fn rl(&mut self, value: u8) -> u8 {
        let carry = self.registers.f.carry;
        let new_value = value << 1 | (carry as u8);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = value & 0b1000_0000 != 0;
        new_value
    }
}
