use crate::cpu::registers::CPURegisters;
use crate::{CPU, MemoryBus};

/// Represents the possible sources for the values of a push instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PushSource {
    BC,
    DE,
    HL,
    AF,
}

/// Represents the possible targets of a pop instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopTarget {
    BC,
    DE,
    HL,
    AF,
}

impl PushSource {
    /// Returns the value of the register pair corresponding to the enum variant.
    fn get_register_pair(&self, registers: &CPURegisters) -> u16 {
        match &self {
            PushSource::BC => registers.get_bc(),
            PushSource::DE => registers.get_de(),
            PushSource::HL => registers.get_hl(),
            PushSource::AF => registers.get_af(),
        }
    }
}

impl PopTarget {
    /// Sets the value of the register pair corresponding to the enum variant.
    fn set_register_pair(&self, registers: &mut CPURegisters, value: u16) {
        match &self {
            PopTarget::BC => registers.set_bc(value),
            PopTarget::DE => registers.set_de(value),
            PopTarget::HL => registers.set_hl(value),
            PopTarget::AF => registers.set_af(value),
        }
    }
}

impl CPU {
    /// Handles the push instruction for the given [PushSource].
    ///
    /// The PUSH instruction takes 4 cycles.
    pub fn handle_push_instruction(
        &mut self,
        memory_bus: &mut MemoryBus,
        register_pair_to_push: PushSource,
    ) -> u16 {
        self.increment_cycle_counter(4);
        let value_to_push = register_pair_to_push.get_register_pair(&self.registers);

        self.push(memory_bus, value_to_push);
        self.pc.wrapping_add(1)
    }

    /// Pushes the given value onto the stack decreasing the stack pointer by 2 (increasing the
    /// size of the stack). The value is stored in little endian format, so the least significant byte is
    /// stored first, that is, on top of the stack.
    pub fn push(&mut self, memory_bus: &mut MemoryBus, value_to_push: u16) {
        self.sp = self.sp.wrapping_sub(1);
        memory_bus.write_byte(self.sp, ((value_to_push & 0xFF00) >> 8) as u8);

        self.sp = self.sp.wrapping_sub(1);
        memory_bus.write_byte(self.sp, value_to_push as u8);
    }

    /// Handles the pop instruction for the given [PopTarget].
    ///
    /// The POP instruction takes 3 cycles.
    pub fn handle_pop_instruction(
        &mut self,
        memory_bus: &MemoryBus,
        register_pair_to_pop_to: PopTarget,
    ) -> u16 {
        self.increment_cycle_counter(3);
        let pop_result = self.pop(memory_bus);
        register_pair_to_pop_to.set_register_pair(&mut self.registers, pop_result);
        self.pc.wrapping_add(1)
    }

    /// Pops a value from the stack increasing the stack pointer by 2 (decreasing the size of the
    /// stack). The value is stored in little endian format, so the least significant byte is read first,
    /// that is, it is at the top of the stack.
    pub fn pop(&mut self, memory_bus: &MemoryBus) -> u16 {
        let lower_byte = memory_bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        let upper_byte = memory_bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        (upper_byte << 8) | lower_byte
    }
}
