use crate::RustBoy;
use crate::cpu::instructions::ArithmeticOrLogicalSource;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddWordTarget {
    HL,
    SP,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddWordSource {
    BC,
    DE,
    HL,
    SP,
    E8,
}

impl RustBoy {
    /// Handles the add instruction for the given [Register] if it adds bytes. For all of these
    /// instructions it also holds that these add to the A register.
    ///
    /// The ADD instruction takes 1 cycle if the source is a register and 2 otherwise.
    pub fn handle_add_byte_instruction(&mut self, source: ArithmeticOrLogicalSource) -> u16 {
        let new_pc = source.increment_pc_and_cycle(self);
        let value = source.get_value(&self.registers, &self, self.pc);
        let new_value = self.add(value, false);
        self.registers.a = new_value;
        new_pc
    }

    /// Adds a value to the A register and sets the corresponding flags in the flags register
    /// [super::registers::FlagsRegister].
    fn add(&mut self, value: u8, carry_flag: bool) -> u8 {
        let new_value = self
            .registers
            .a
            .wrapping_add(value)
            .wrapping_add(carry_flag as u8);
        self.registers.f.set_zero_flag(new_value == 0);
        self.registers.f.set_subtract_flag(false);
        // The carry flag is set if there is an overflow from the 8th bit to the "9"th bit.
        // This is the case if the sum of the A register and the value are greater than 0xFF = 0b 1111 1111 (binary).
        self.registers
            .f
            .set_carry_flag(self.registers.a as u16 + value as u16 + carry_flag as u16 > 0xFF);
        // The half carry flag is set if there is an overflow from the lower 4 bits to the fifth bit.
        // This is the case if the sum of the lower 4 bits of the A register and the value are greater
        // than 0xF = 0b 0000 1111 (binary).
        self.registers.f.set_half_carry_flag(
            ((self.registers.a & 0xF) + (value & 0xF) + carry_flag as u8) > 0xF,
        );
        new_value
    }

    /// Adds a source_value to a target_value and sets the corresponding flags in the
    /// [super::registers::FlagsRegister].
    pub(crate) fn add_not_to_a(&mut self, target_value: u8, source_value: u8) -> u8 {
        let new_value = target_value.wrapping_add(source_value);
        self.registers.f.set_zero_flag(new_value == 0);
        self.registers.f.set_subtract_flag(false);
        // The carry flag is set if there is an overflow from the 8th bit to the "9"th bit.
        // This is the case if the sum of the A register and the value are greater than 0xFF = 0b 1111 1111 (binary).
        self.registers
            .f
            .set_carry_flag(target_value as u16 + source_value as u16 > 0xFF);
        // The half carry flag is set if there is an overflow from the lower 4 bits to the fifth bit.
        // This is the case if the sum of the lower 4 bits of the A register and the value are greater
        // than 0xF = 0b 0000 1111 (binary).
        self.registers
            .f
            .set_half_carry_flag(((target_value & 0x0F) + (source_value & 0x0F)) > 0xF);
        new_value
    }

    /// Handles the add instruction for the given [Register] if words (2 bytes) are added.
    /// In particular, these instructions do not add to the A register.
    ///
    /// These Instructions take 2 cycles if the target is HL and 4 otherwise.
    pub fn handle_add_word_instruction(
        &mut self,
        type_of_word_add: (AddWordTarget, AddWordSource),
    ) -> u16 {
        let (target, source) = type_of_word_add;
        match target {
            AddWordTarget::HL => {
                self.increment_cycle_counter(2);
            }
            AddWordTarget::SP => {
                self.increment_cycle_counter(4);
            }
        }
        let value = match source {
            AddWordSource::BC => Some(self.registers.get_bc()),
            AddWordSource::DE => Some(self.registers.get_de()),
            AddWordSource::HL => Some(self.registers.get_hl()),
            AddWordSource::SP => Some(self.sp),
            AddWordSource::E8 => None,
        };
        match target {
            AddWordTarget::HL => {
                let value = value.expect(
                    "Should be a valid add instruction and therefore value should be present",
                );
                let new_value = self.add_word(self.registers.get_hl(), value, false);
                self.registers.set_hl(new_value);
                self.pc.wrapping_add(1)
            }
            AddWordTarget::SP => {
                let value = (self.read_byte(self.pc.wrapping_add(1)) as i8) as i16;
                let value_u8 = self.read_byte(self.pc.wrapping_add(1));
                let new_sp = self.sp.wrapping_add_signed(value);
                // Set flags by calling add Instruction, discarding result and overwriting zero flag
                self.add_not_to_a(self.sp as u8, value_u8);
                self.registers.f.set_zero_flag(false);

                self.sp = new_sp;
                self.pc.wrapping_add(2)
            }
        }
    }

    /// Adds a 16bit value to the target and sets the corresponding flags in the flags register
    /// [super::registers::FlagsRegister].
    ///
    /// The zero flag is reset if the target is the stack pointer. Otherwise, it is not changed.
    fn add_word(&mut self, target: u16, value: u16, sp_is_target: bool) -> u16 {
        let new_value = target.wrapping_add(value);
        self.registers.f.set_subtract_flag(false);
        // The carry flag is set if there is an overflow from the 15th bit to the "16"th bit.
        // This is the case if the sum of the A register and the value are greater than 0xFFFF = 0b 1111 1111 1111 1111 (binary).
        self.registers
            .f
            .set_carry_flag(target as u32 + value as u32 > 0xFFFF);
        // The half carry flag is set if there is an overflow from the lower 12 bits to the thirteenth bit.
        // This is the case if the sum of the lower 12 bits of the target and the value are greater
        // than 0x0FFF = 0b 0000 1111 1111 1111 (binary).
        self.registers
            .f
            .set_half_carry_flag(((target & 0x0FFF) + (value & 0x0FFF)) > 0x0FFF);
        if sp_is_target {
            self.registers.f.set_zero_flag(false);
        }
        new_value
    }

    /// Handles the adc instruction for the given [Register].
    /// Does the same as [handle_add_instruction] but adds the carry flag to the value.
    ///
    /// The ADC instruction takes 1 cycle if the source is a register and 2 otherwise.
    pub fn handle_adc_instruction(&mut self, source: ArithmeticOrLogicalSource) -> u16 {
        let new_pc = source.increment_pc_and_cycle(self);
        let value = source.get_value(&self.registers, &self, self.pc);
        let new_value = self.add(value, self.registers.f.get_carry_flag());
        self.registers.a = new_value;
        new_pc
    }
}
