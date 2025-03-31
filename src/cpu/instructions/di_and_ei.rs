use crate::CPU;

impl CPU {
    /// Handles the di instruction
    ///
    /// Disables interrupts. Takes 1 cycle.
    pub fn handle_di_instruction(&mut self) -> u16 {
        self.increment_cycle_counter(1);
        self.di();
        self.pc.wrapping_add(1)
    }

    /// Sets the IME (Interrupt Master Enable) flag to false, which disables interrupts.
    fn di(&mut self) {
        self.ime = false;
    }

    /// Handles the ei instruction
    ///
    /// Enables interrupts. Takes 1 cycle.
    pub fn handle_ei_instruction(&mut self) -> u16 {
        self.increment_cycle_counter(1);
        self.ei();
        self.pc.wrapping_add(1)
    }

    /// Sets the ime_to_be_set flag to true, which will enable interrupts on the next instruction,
    /// that is, will set the IME (Interrupt Master Enable) flag to true.
    fn ei(&mut self) {
        self.ime_to_be_set = true;
    }
}
