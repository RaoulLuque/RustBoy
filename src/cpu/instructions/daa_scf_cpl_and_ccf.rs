use crate::RustBoy;

impl RustBoy {
    /// Handles the DAA instruction.
    ///
    /// The DAA instruction takes 1 cycle.
    pub fn handle_daa_instruction(&mut self) -> u16 {
        self.registers.a = self.daa();
        self.increment_cycle_counter(1);
        self.pc.wrapping_add(1)
    }

    /// Decimal Adjust for Addition
    fn daa(&mut self) -> u8 {
        let mut a = self.registers.a;
        if self.registers.f.subtract {
            if self.registers.f.half_carry {
                a = a.wrapping_sub(0x06);
            }
            if self.registers.f.carry {
                a = a.wrapping_sub(0x60);
            }
        } else {
            let mut adjustment = 0;
            if self.registers.f.half_carry || (self.registers.a & 0x0F) > 0x09 {
                adjustment += 0x06;
            }
            if self.registers.f.carry || self.registers.a > 0x99 {
                adjustment += 0x60;
                self.registers.f.carry = true;
            }
            a = a.wrapping_add(adjustment);
        }
        self.registers.f.zero = a == 0;
        self.registers.f.half_carry = false;
        a
    }

    /// Handles the SCF instruction.
    ///
    /// The SCF instruction takes 1 cycle.
    pub fn handle_scf_instruction(&mut self) -> u16 {
        self.scf();
        self.increment_cycle_counter(1);
        self.pc.wrapping_add(1)
    }

    /// Set Carry Flag. Also clears the half carry and subtract flags.
    fn scf(&mut self) {
        self.registers.f.carry = true;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
    }

    /// Handles the CPL instruction.
    ///
    /// The CPL instruction takes 1 cycle.
    pub fn handle_cpl_instruction(&mut self) -> u16 {
        self.cpl();
        self.increment_cycle_counter(1);
        self.pc.wrapping_add(1)
    }

    /// Complement A. Sets the subtract flag and the half carry flag.
    fn cpl(&mut self) {
        self.registers.a = !self.registers.a;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = true;
    }

    /// Handles the CCF instruction.
    ///
    /// The CCF instruction takes 1 cycle.
    pub fn handle_ccf_instruction(&mut self) -> u16 {
        self.ccf();
        self.increment_cycle_counter(1);
        self.pc.wrapping_add(1)
    }

    /// Complement Carry Flag. Sets the subtract flag and clears the half carry flag.
    fn ccf(&mut self) {
        self.registers.f.carry = !self.registers.f.carry;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
    }
}
