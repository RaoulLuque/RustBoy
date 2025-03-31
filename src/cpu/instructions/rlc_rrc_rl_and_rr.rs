use super::SixteenBitInstructionTarget;
use crate::{CPU, MemoryBus};

impl CPU {
    /// Handles the RLC instruction for the given [super::SixteenBitInstructionTarget].
    ///
    /// The RLC instruction takes 2 cycles if the target is a register and 4 otherwise.
    pub fn handle_rlc_instruction(
        &mut self,
        memory_bus: &mut MemoryBus,
        target: SixteenBitInstructionTarget,
    ) -> u16 {
        match target {
            SixteenBitInstructionTarget::HLRef => self.increment_cycle_counter(4),
            _ => self.increment_cycle_counter(2),
        }
        let value = target.get_value(memory_bus, &self);
        let new_value = self.rlc(value);
        target.set_value(memory_bus, self, new_value);
        self.pc.wrapping_add(2)
    }

    /// Rotates the given value left and sets the flags in the flags register if the rotation wraps
    /// around. Also sets the zero flag if the result is zero.
    pub(crate) fn rlc(&mut self, value: u8) -> u8 {
        let new_value = value.rotate_left(1);
        let carry = value & 0b1000_0000 != 0;
        self.registers.f.set_zero_flag(new_value == 0);
        self.registers.f.set_subtract_flag(false);
        self.registers.f.set_half_carry_flag(false);
        self.registers.f.set_carry_flag(carry);
        new_value
    }

    /// Handles the RRC instruction for the given [super::SixteenBitInstructionTarget].
    ///
    /// The RRC instruction takes 2 cycles if the target is a register and 4 otherwise.
    pub fn handle_rrc_instruction(
        &mut self,
        memory_bus: &mut MemoryBus,
        target: SixteenBitInstructionTarget,
    ) -> u16 {
        match target {
            SixteenBitInstructionTarget::HLRef => self.increment_cycle_counter(4),
            _ => self.increment_cycle_counter(2),
        }
        let value = target.get_value(memory_bus, &self);
        let new_value = self.rrc(value);
        target.set_value(memory_bus, self, new_value);
        self.pc.wrapping_add(2)
    }

    /// Rotates the given value right and sets the flags in the flags register if the rotation wraps
    /// around. Also sets the zero flag if the result is zero.
    pub(crate) fn rrc(&mut self, value: u8) -> u8 {
        let new_value = value.rotate_right(1);
        let carry = value & 0b0000_0001 != 0;
        self.registers.f.set_zero_flag(new_value == 0);
        self.registers.f.set_subtract_flag(false);
        self.registers.f.set_half_carry_flag(false);
        self.registers.f.set_carry_flag(carry);
        new_value
    }

    /// Handles the RL instruction for the given [super::SixteenBitInstructionTarget].
    ///
    /// The RL instruction takes 2 cycles if the target is a register and 4 otherwise.
    pub fn handle_rl_instruction(
        &mut self,
        memory_bus: &mut MemoryBus,
        target: SixteenBitInstructionTarget,
    ) -> u16 {
        match target {
            SixteenBitInstructionTarget::HLRef => self.increment_cycle_counter(4),
            _ => self.increment_cycle_counter(2),
        }
        let value = target.get_value(memory_bus, &self);
        let new_value = self.rl(value);
        target.set_value(memory_bus, self, new_value);
        self.pc.wrapping_add(2)
    }

    /// Rotates the given value left through the carry flag. Sets the zero flag if the result is zero.
    pub(crate) fn rl(&mut self, value: u8) -> u8 {
        let carry = self.registers.f.get_carry_flag();
        let new_value = value << 1 | (carry as u8);
        self.registers.f.set_zero_flag(new_value == 0);
        self.registers.f.set_subtract_flag(false);
        self.registers.f.set_half_carry_flag(false);
        self.registers.f.set_carry_flag(value & 0b1000_0000 != 0);
        new_value
    }

    /// Handles the RR instruction for the given [super::SixteenBitInstructionTarget].
    ///
    /// The RR instruction takes 2 cycles if the target is a register and 4 otherwise.
    pub fn handle_rr_instruction(
        &mut self,
        memory_bus: &mut MemoryBus,
        target: SixteenBitInstructionTarget,
    ) -> u16 {
        match target {
            SixteenBitInstructionTarget::HLRef => self.increment_cycle_counter(4),
            _ => self.increment_cycle_counter(2),
        }
        let value = target.get_value(memory_bus, &self);
        let new_value = self.rr(value);
        target.set_value(memory_bus, self, new_value);
        self.pc.wrapping_add(2)
    }

    /// Rotates the given value right through the carry flag. Sets the zero flag if the result is zero.
    pub(crate) fn rr(&mut self, value: u8) -> u8 {
        let carry = self.registers.f.get_carry_flag();
        let new_value = (value >> 1) | ((carry as u8) << 7);
        self.registers.f.set_zero_flag(new_value == 0);
        self.registers.f.set_subtract_flag(false);
        self.registers.f.set_half_carry_flag(false);
        self.registers.f.set_carry_flag(value & 0b0000_0001 != 0);
        new_value
    }
}
