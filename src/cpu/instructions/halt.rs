use crate::RustBoy;

impl RustBoy {
    /// Handles the halt instruction.
    ///
    /// Takes 1 cycle to execute.
    pub fn handle_halt_instruction(&mut self) -> u16 {
        self.increment_cycle_counter(1);
        self.halt();
        self.pc.wrapping_add(1)
    }

    /// Sets the CPU to halt mode. In this mode, the CPU will not execute any instructions until an
    /// interrupt is requested.
    fn halt(&mut self) {
        self.halted = true;
    }
}
