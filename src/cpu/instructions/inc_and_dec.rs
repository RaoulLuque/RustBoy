use super::Register;
use crate::{CPU, MemoryBus};

/// Represents the possible targets for an inc or dec instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IncDecTarget {
    Register(Register),
    HLRef,
    BC,
    DE,
    HL,
    SP,
}

impl CPU {
    /// Handles the inc instruction for the given [IncDecTarget].
    ///
    /// The INC instruction takes 1 cycle if the target is a register, 3 if it is HLRef
    /// and 2 if it is BC, DE, HL or SP.
    pub fn handle_inc_instruction(
        &mut self,
        memory_bus: &mut MemoryBus,
        target: IncDecTarget,
    ) -> u16 {
        match target {
            IncDecTarget::Register(register) => {
                self.increment_cycle_counter(1);
                let new_value = self.inc(register.get_register(&self.registers));
                register.set_register(&mut self.registers, new_value);
            }
            IncDecTarget::HLRef => {
                self.increment_cycle_counter(3);
                let address = self.registers.get_hl();
                let value = memory_bus.read_byte(address);
                let new_value = self.inc(value);
                memory_bus.write_byte(address, new_value);
            }
            IncDecTarget::BC => {
                self.increment_cycle_counter(2);
                self.registers
                    .set_bc(self.registers.get_bc().wrapping_add(1));
            }
            IncDecTarget::DE => {
                self.increment_cycle_counter(2);
                self.registers
                    .set_de(self.registers.get_de().wrapping_add(1));
            }
            IncDecTarget::HL => {
                self.increment_cycle_counter(2);
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_add(1));
            }
            IncDecTarget::SP => {
                self.increment_cycle_counter(2);
                self.set_sp(self.sp.wrapping_add(1));
            }
        }
        self.pc.wrapping_add(1)
    }

    /// Adds 1 to a value and sets the corresponding flags in the flags register
    fn inc(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_add(1);
        self.registers.f.set_zero_flag(new_value == 0);
        self.registers.f.set_subtract_flag(false);
        // The half carry flag is set if there is an overflow from the lower 4 bits to the fifth bit.
        // This is the case if the addition of the lower 4 bits of the A register and the value is greater
        // than 0xF. That is, if the lower 4 bits of the A register are greater than 0xF.
        self.registers
            .f
            .set_half_carry_flag((value & 0xF).wrapping_add(1) > 0xF);
        new_value
    }

    /// Handles the dec instruction for the given [IncDecTarget].
    /// The DEC instruction takes 1 cycle if the target is a register, 3 if it is HLRef
    /// and 2 if it is BC, DE, HL or SP.
    pub fn handle_dec_instruction(
        &mut self,
        memory_bus: &mut MemoryBus,
        target: IncDecTarget,
    ) -> u16 {
        match target {
            IncDecTarget::Register(register) => {
                self.increment_cycle_counter(1);
                let new_value = self.dec(register.get_register(&self.registers));
                register.set_register(&mut self.registers, new_value);
            }
            IncDecTarget::HLRef => {
                self.increment_cycle_counter(3);
                let address = self.registers.get_hl();
                let value = memory_bus.read_byte(address);
                let new_value = self.dec(value);
                memory_bus.write_byte(address, new_value);
            }
            IncDecTarget::BC => {
                self.increment_cycle_counter(2);
                self.registers
                    .set_bc(self.registers.get_bc().wrapping_sub(1));
            }
            IncDecTarget::DE => {
                self.increment_cycle_counter(2);
                self.registers
                    .set_de(self.registers.get_de().wrapping_sub(1));
            }
            IncDecTarget::HL => {
                self.increment_cycle_counter(2);
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_sub(1));
            }
            IncDecTarget::SP => {
                self.increment_cycle_counter(2);
                self.set_sp(self.sp.wrapping_sub(1));
            }
        }
        self.pc.wrapping_add(1)
    }

    /// Subtracts 1 from a value and sets the corresponding flags in the flags register
    fn dec(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_sub(1);
        self.registers.f.set_zero_flag(new_value == 0);
        self.registers.f.set_subtract_flag(true);
        // The half carry flag is set if there is an overflow from the lower 4 bits to the fifth bit.
        // This is the case if the subtraction of 1 from the lower 4 bits of the value is less
        // than 0. That is, if there is a wrap around and the new_value is greater than 0xF.
        self.registers
            .f
            .set_half_carry_flag((value & 0xF).wrapping_sub(1) > 0xF);
        new_value
    }
}
