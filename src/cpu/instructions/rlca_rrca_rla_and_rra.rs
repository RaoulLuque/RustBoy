use crate::RustBoy;

impl RustBoy {
    /// Handles the RLCA instruction. In comparison to the RLC instruction, the RLCA instruction
    /// sets the zero flag to false.
    ///
    /// The RLCA instruction takes 1 cycle.
    pub fn handle_rlca_instruction(&mut self) -> u16 {
        let value = self.registers.a;
        let new_value = self.rlc(value);
        self.registers.a = new_value;
        self.increment_cycle_counter(1);
        self.registers.f.zero = false;
        self.pc.wrapping_add(1)
    }

    /// Handles the RRCA instruction. In comparison to the RRC instruction, the RRCA instruction
    /// sets the zero flag to false.
    ///
    /// The RRCA instruction takes 1 cycle.
    pub fn handle_rrca_instruction(&mut self) -> u16 {
        let value = self.registers.a;
        let new_value = self.rrc(value);
        self.registers.a = new_value;
        self.increment_cycle_counter(1);
        self.registers.f.zero = false;
        self.pc.wrapping_add(1)
    }

    /// Handles the RLA instruction. In comparison to the RL instruction, the RLA instruction
    /// sets the zero flag to false.
    ///
    /// The RLA instruction takes 1 cycle.
    pub fn handle_rla_instruction(&mut self) -> u16 {
        let value = self.registers.a;
        let new_value = self.rl(value);
        self.registers.a = new_value;
        self.increment_cycle_counter(1);
        self.registers.f.zero = false;
        self.pc.wrapping_add(1)
    }

    /// Handles the RRA instruction. In comparison to the RR instruction, the RRA instruction
    /// sets the zero flag to false.
    ///
    /// The RRA instruction takes 1 cycle.
    pub fn handle_rra_instruction(&mut self) -> u16 {
        let value = self.registers.a;
        let new_value = self.rr(value);
        self.registers.a = new_value;
        self.increment_cycle_counter(1);
        self.registers.f.zero = false;
        self.pc.wrapping_add(1)
    }
}
