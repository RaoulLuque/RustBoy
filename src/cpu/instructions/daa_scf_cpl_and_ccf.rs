use crate::CPU;

impl CPU {
    /// Handles the DAA instruction.
    ///
    /// The DAA instruction takes 1 cycle.
    pub fn handle_daa_instruction(&mut self) -> u16 {
        self.increment_cycle_counter(1);
        self.registers.a = self.daa();
        self.pc.wrapping_add(1)
    }

    /// Decimal Adjust for Addition
    fn daa(&mut self) -> u8 {
        let mut a = self.registers.a;
        if self.registers.f.get_subtract_flag() {
            if self.registers.f.get_half_carry_flag() {
                a = a.wrapping_sub(0x06);
            }
            if self.registers.f.get_carry_flag() {
                a = a.wrapping_sub(0x60);
            }
        } else {
            let mut adjustment = 0;
            if self.registers.f.get_half_carry_flag() || (self.registers.a & 0x0F) > 0x09 {
                adjustment += 0x06;
            }
            if self.registers.f.get_carry_flag() || self.registers.a > 0x99 {
                adjustment += 0x60;
                self.registers.f.set_carry_flag(true);
            }
            a = a.wrapping_add(adjustment);
        }
        self.registers.f.set_zero_flag(a == 0);
        self.registers.f.set_half_carry_flag(false);
        a
    }

    /// Handles the SCF instruction.
    ///
    /// The SCF instruction takes 1 cycle.
    pub fn handle_scf_instruction(&mut self) -> u16 {
        self.increment_cycle_counter(1);
        self.scf();
        self.pc.wrapping_add(1)
    }

    /// Set Carry Flag. Also clears the half carry and subtract flags.
    fn scf(&mut self) {
        self.registers.f.set_carry_flag(true);
        self.registers.f.set_subtract_flag(false);
        self.registers.f.set_half_carry_flag(false);
    }

    /// Handles the CPL instruction.
    ///
    /// The CPL instruction takes 1 cycle.
    pub fn handle_cpl_instruction(&mut self) -> u16 {
        self.increment_cycle_counter(1);
        self.cpl();
        self.pc.wrapping_add(1)
    }

    /// Complement A. Sets the subtract flag and the half carry flag.
    fn cpl(&mut self) {
        self.registers.a = !self.registers.a;
        self.registers.f.set_subtract_flag(true);
        self.registers.f.set_half_carry_flag(true);
    }

    /// Handles the CCF instruction.
    ///
    /// The CCF instruction takes 1 cycle.
    pub fn handle_ccf_instruction(&mut self) -> u16 {
        self.increment_cycle_counter(1);
        self.ccf();
        self.pc.wrapping_add(1)
    }

    /// Complement Carry Flag. Sets the subtract flag and clears the half carry flag.
    fn ccf(&mut self) {
        self.registers
            .f
            .set_carry_flag(!self.registers.f.get_carry_flag());
        self.registers.f.set_subtract_flag(false);
        self.registers.f.set_half_carry_flag(false);
    }
}
