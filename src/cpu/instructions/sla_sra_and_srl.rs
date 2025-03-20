use super::SixteenBitInstructionTarget;
use crate::RustBoy;

impl RustBoy {
    /// Handles the SLA instruction for the given [super::SixteenBitInstructionTarget].
    ///
    /// The SLA instruction takes 2 cycles if the target is a register and 4 otherwise.
    pub fn handle_sla_instruction(&mut self, target: SixteenBitInstructionTarget) -> u16 {
        match target {
            SixteenBitInstructionTarget::HLRef => self.increment_cycle_counter(4),
            _ => self.increment_cycle_counter(2),
        }
        let value = target.get_value(&self);
        let new_value = self.sla(value);
        target.set_value(self, new_value);
        self.pc.wrapping_add(2)
    }

    /// Shifts the given value left and sets the carry flag if the shift wraps around.
    /// Also sets the zero flag if the result is zero.
    fn sla(&mut self, value: u8) -> u8 {
        let new_value = value << 1;
        let carry = value & 0b1000_0000 != 0;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;
        new_value
    }

    /// Handles the SRA instruction for the given [super::SixteenBitInstructionTarget].
    ///
    /// The SRA instruction takes 2 cycles if the target is a register and 4 otherwise.
    pub fn handle_sra_instruction(&mut self, target: SixteenBitInstructionTarget) -> u16 {
        match target {
            SixteenBitInstructionTarget::HLRef => self.increment_cycle_counter(4),
            _ => self.increment_cycle_counter(2),
        }
        let value = target.get_value(&self);
        let new_value = self.sra(value);
        target.set_value(self, new_value);
        self.pc.wrapping_add(2)
    }

    /// Shifts the given value right and sets the carry flag if the shift wraps around.
    /// Also sets the zero flag if the result is zero.
    fn sra(&mut self, value: u8) -> u8 {
        let new_value = (value as i8) >> 1;
        let carry = value & 0b0000_0001 != 0;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;
        new_value as u8
    }

    /// Handles the SRL instruction for the given [super::SixteenBitInstructionTarget].
    ///
    /// The SRL instruction takes 2 cycles if the target is a register and 4 otherwise.
    pub fn handle_srl_instruction(&mut self, target: SixteenBitInstructionTarget) -> u16 {
        match target {
            SixteenBitInstructionTarget::HLRef => self.increment_cycle_counter(4),
            _ => self.increment_cycle_counter(2),
        }
        let value = target.get_value(&self);
        let new_value = self.srl(value);
        target.set_value(self, new_value);
        self.pc.wrapping_add(2)
    }

    /// Shifts the given value right and sets the carry flag if the shift wraps around.
    /// Also sets the zero flag if the result is zero.
    fn srl(&mut self, value: u8) -> u8 {
        let new_value = value >> 1;
        let carry = value & 0b0000_0001 != 0;
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;
        new_value
    }
}
