use crate::cpu::instructions::Register;
use crate::RustBoy;

/// Represents the possible targets for a byte load instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadByteTarget {
    REGISTER(Register),
    HLRef,
    HLRefIncrement,
    HLRefDecrement,
    BCRef,
    DERef,
    A16Ref,
}

/// Represents the possible sources for a byte load instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadByteSource {
    REGISTER(Register),
    D8,
    HLRef,
    HLRefIncrement,
    HLRefDecrement,
    BCRef,
    DERef,
    A16Ref,
}

/// Represents the possible targets for a word load instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadWordTarget {
    BC,
    DE,
    HL,
    SP,
    A16Ref,
}

/// Represents the possible sources for a word load instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadWordSource {
    D16,
    SP,
    SPPlusE8,
    HL,
}

/// Represents the possible types of load instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadType {
    Byte(LoadByteTarget, LoadByteSource),
    Word(LoadWordTarget, LoadWordSource),
}

impl RustBoy {
    /// Handles the load instruction for the given [LoadType].
    ///
    /// The LD instruction takes 1 cycle if the source and targets are registers.
    /// It takes an additional cycle if the source if a reference or immediate operand like
    /// HLRef, HLRefIncrement, HLRefDecrement or D8.
    pub fn handle_load_instruction(&mut self, type_of_load: LoadType) -> u16 {
        // The LD instruction takes 1 cycle by default.
        self.increment_cycle_counter(1);
        match type_of_load {
            LoadType::Byte(target, source) => {
                let value = self.get_value_from_load_byte_source(source);
                // Increment the cycle counter if the source is a reference or immediate operand.
                match target {
                    LoadByteTarget::REGISTER(register) => {
                        register.set_register(&mut self.registers, value);
                    }
                    LoadByteTarget::HLRef => {
                        self.increment_cycle_counter(1);
                        self.write_byte(self.registers.get_hl(), value);
                    }
                    LoadByteTarget::HLRefIncrement => {
                        self.increment_cycle_counter(1);
                        self.write_byte(self.registers.get_hl(), value);
                        self.registers
                            .set_hl(self.registers.get_hl().wrapping_add(1));
                    }
                    LoadByteTarget::HLRefDecrement => {
                        self.increment_cycle_counter(1);
                        self.write_byte(self.registers.get_hl(), value);
                        self.registers
                            .set_hl(self.registers.get_hl().wrapping_sub(1));
                    }
                    LoadByteTarget::BCRef => {
                        self.increment_cycle_counter(1);
                        self.write_byte(self.registers.get_bc(), value);
                    }
                    LoadByteTarget::DERef => {
                        self.increment_cycle_counter(1);
                        self.write_byte(self.registers.get_de(), value);
                    }
                    LoadByteTarget::A16Ref => {
                        self.increment_cycle_counter(3);
                        let address = self.read_next_word_little_endian(self.pc);
                        self.write_byte(address, value);
                    }
                }
                match (source, target) {
                    (LoadByteSource::D8, _) => self.pc.wrapping_add(2),
                    (LoadByteSource::A16Ref, _) => self.pc.wrapping_add(3),
                    (_, LoadByteTarget::A16Ref) => self.pc.wrapping_add(3),
                    _ => self.pc.wrapping_add(1),
                }
            }
            LoadType::Word(target, source) => {
                // All word loads take 3 cycles except for the A16Ref target which takes 5 and
                // LD SP, HL which takes 2.
                // So add 2 cycles here if not LD SP, HL and then add 2 more if the target is A16Ref.
                match (target, source) {
                    (LoadWordTarget::SP, LoadWordSource::HL) => self.increment_cycle_counter(1),
                    _ => self.increment_cycle_counter(2),
                };
                let value = self.get_value_from_load_word_source(source);
                match target {
                    LoadWordTarget::BC => {
                        self.registers.set_bc(value);
                    }
                    LoadWordTarget::DE => {
                        self.registers.set_de(value);
                    }
                    LoadWordTarget::HL => {
                        self.registers.set_hl(value);
                    }
                    LoadWordTarget::SP => {
                        self.set_sp(value);
                    }
                    LoadWordTarget::A16Ref => {
                        self.increment_cycle_counter(2);
                        let address_to_store_to = self.read_next_word_little_endian(self.pc);
                        self.write_byte(address_to_store_to, value as u8);
                        self.write_byte(address_to_store_to.wrapping_add(1), (value >> 8) as u8);
                    }
                }
                self.pc.wrapping_add(3)
            }
        }
    }

    /// Returns the value from the given [LoadByteSource].
    ///
    /// The function also increments the cycle counter if the source is an immediate operand or
    /// a reference.
    fn get_value_from_load_byte_source(&mut self, source: LoadByteSource) -> u8 {
        match source {
            LoadByteSource::REGISTER(register) => register.get_register(&self.registers),
            LoadByteSource::D8 => {
                self.increment_cycle_counter(1);
                self.read_byte(self.pc.wrapping_add(1))
            }
            LoadByteSource::HLRef => {
                self.increment_cycle_counter(1);
                self.read_byte(self.registers.get_hl())
            }
            LoadByteSource::HLRefIncrement => {
                self.increment_cycle_counter(1);
                let hl = self.registers.get_hl();
                let value = self.read_byte(hl);
                self.registers.set_hl(hl.wrapping_add(1));
                value
            }
            LoadByteSource::HLRefDecrement => {
                self.increment_cycle_counter(1);
                let hl = self.registers.get_hl();
                let value = self.read_byte(hl);
                self.registers.set_hl(hl.wrapping_sub(1));
                value
            }
            LoadByteSource::BCRef => {
                self.increment_cycle_counter(1);
                self.read_byte(self.registers.get_bc())
            }
            LoadByteSource::DERef => {
                self.increment_cycle_counter(1);
                self.read_byte(self.registers.get_de())
            }
            LoadByteSource::A16Ref => {
                self.increment_cycle_counter(3);
                let address = self.read_next_word_little_endian(self.pc);
                self.read_byte(address)
            }
        }
    }

    /// Returns the value from the given [LoadWordSource].
    fn get_value_from_load_word_source(&self, source: LoadWordSource) -> u16 {
        match source {
            LoadWordSource::D16 => self.read_next_word_little_endian(self.pc),
            LoadWordSource::SP => self.sp,
            LoadWordSource::HL => self.registers.get_hl(),
            LoadWordSource::SPPlusE8 => {
                let value = (self.read_byte(self.pc.wrapping_add(1)) as i8) as i16;
                (self.sp).wrapping_add_signed(value)
            }
        }
    }
}
