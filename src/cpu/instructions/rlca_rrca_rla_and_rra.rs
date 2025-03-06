use super::SixteenBitInstructionTarget;
use crate::RustBoy;

impl RustBoy {
    /// Handles the RLCA instruction.
    ///
    /// The RLCA instruction takes 1 cycle.
    pub fn handle_rlca_instruction(&mut self) -> u16 {
        let value = self.registers.a;
        let new_value = self.rlc(value);
        self.registers.a = new_value;
        self.increment_cycle_counter(1);
        self.pc.wrapping_add(1)
    }

    /// Handles the RRCA instruction.
    ///
    /// The RRCA instruction takes 1 cycle.
    pub fn handle_rrca_instruction(&mut self) -> u16 {
        let value = self.registers.a;
        let new_value = self.rrc(value);
        self.registers.a = new_value;
        self.increment_cycle_counter(1);
        self.pc.wrapping_add(1)
    }

    /// Handles the RLA instruction.
    ///
    /// The RLA instruction takes 1 cycle.
    pub fn handle_rla_instruction(&mut self) -> u16 {
        let value = self.registers.a;
        let new_value = self.rl(value);
        self.registers.a = new_value;
        self.increment_cycle_counter(1);
        self.pc.wrapping_add(1)
    }

    /// Handles the RRA instruction.
    ///
    /// The RRA instruction takes 1 cycle.
    pub fn handle_rra_instruction(&mut self) -> u16 {
        let value = self.registers.a;
        let new_value = self.rr(value);
        self.registers.a = new_value;
        self.increment_cycle_counter(1);
        self.pc.wrapping_add(1)
    }
}
