use super::Register;
use crate::cpu::CPU;

/// Represents the possible targets for an inc or dec instruction.
#[derive(Clone, Copy, Debug)]
pub(super) enum IncDecTarget {
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
    pub fn handle_inc_instruction(&mut self, target: IncDecTarget) -> u16 {
        match target {
            IncDecTarget::Register(register) => {
                let new_value = self.inc(register.get_register(&self.registers));
                register.set_register(&mut self.registers, new_value);
                self.increment_cycle_counter(1);
            }
            IncDecTarget::HLRef => {
                let address = self.registers.get_hl();
                let value = self.bus.read_byte(address);
                let new_value = self.inc(value);
                self.bus.write_byte(address, new_value);
                self.increment_cycle_counter(3);
            }
            IncDecTarget::BC => {
                self.registers
                    .set_bc(self.registers.get_bc().wrapping_add(1));
                self.increment_cycle_counter(2);
            }
            IncDecTarget::DE => {
                self.registers
                    .set_de(self.registers.get_de().wrapping_add(1));
                self.increment_cycle_counter(2);
            }
            IncDecTarget::HL => {
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_add(1));
                self.increment_cycle_counter(2);
            }
            IncDecTarget::SP => {
                self.set_sp(self.sp.wrapping_add(1));
                self.increment_cycle_counter(2);
            }
        }
        self.pc.wrapping_add(1)
    }

    /// Adds 1 to a value and sets the corresponding flags in the flags register
    fn inc(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_add(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        // The half carry flag is set if there is an overflow from the lower 4 bits to the fifth bit.
        // This is the case if the addition of the lower 4 bits of the A register and the value is greater
        // than 0xF. That is, if the lower 4 bits of the A register are greater than 0xF.
        self.registers.f.half_carry = (value & 0xF).wrapping_add(1) > 0xF;
        new_value
    }

    /// Handles the dec instruction for the given [IncDecTarget].
    /// The DEC instruction takes 1 cycle if the target is a register, 3 if it is HLRef
    /// and 2 if it is BC, DE, HL or SP.
    pub fn handle_dec_instruction(&mut self, target: IncDecTarget) -> u16 {
        match target {
            IncDecTarget::Register(register) => {
                let new_value = self.dec(register.get_register(&self.registers));
                register.set_register(&mut self.registers, new_value);
                self.increment_cycle_counter(1);
            }
            IncDecTarget::HLRef => {
                let address = self.registers.get_hl();
                let value = self.bus.read_byte(address);
                let new_value = self.dec(value);
                self.bus.write_byte(address, new_value);
                self.increment_cycle_counter(3);
            }
            IncDecTarget::BC => {
                self.registers
                    .set_bc(self.registers.get_bc().wrapping_sub(1));
                self.increment_cycle_counter(2);
            }
            IncDecTarget::DE => {
                self.registers
                    .set_de(self.registers.get_de().wrapping_sub(1));
                self.increment_cycle_counter(2);
            }
            IncDecTarget::HL => {
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_sub(1));
                self.increment_cycle_counter(2);
            }
            IncDecTarget::SP => {
                self.set_sp(self.sp.wrapping_sub(1));
                self.increment_cycle_counter(2);
            }
        }
        self.pc.wrapping_add(1)
    }

    /// Subtracts 1 from a value and sets the corresponding flags in the flags register
    fn dec(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_sub(1);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = true;
        // The half carry flag is set if there is an overflow from the lower 4 bits to the fifth bit.
        // This is the case if the subtraction of 1 from the lower 4 bits of the value is less
        // than 0. That is, if there is a wrap around and the new_value is greater than 0xF.
        self.registers.f.half_carry = (value & 0xF).wrapping_sub(1) > 0xF;
        new_value
    }
}
