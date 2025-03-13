use crate::RustBoy;

impl RustBoy {
    /// Handles the halt instruction
    ///
    /// TODO: Handle [halt bug](https://gbdev.io/pandocs/halt.html#halt-bug)
    pub fn handle_halt_instruction(&mut self) -> u16 {
        self.halt();
        self.increment_cycle_counter(1);
        self.pc.wrapping_add(1)
    }

    fn halt(&mut self) {
        self.halted = true;
    }
}
