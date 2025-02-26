use crate::cpu::instructions::Register;
use crate::cpu::CPU;

/// Represents the possible targets for a byte load instruction.
#[derive(Clone, Copy, Debug)]
pub enum LoadByteTarget {
    REGISTER(Register),
    HLRef,
    HLRefIncrement,
    HLRefDecrement,
    BCRef,
    DERef,
}

/// Represents the possible sources for a byte load instruction.
#[derive(Clone, Copy, Debug)]
pub enum LoadByteSource {
    REGISTER(Register),
    D8,
    HLRef,
    HLRefIncrement,
    HLRefDecrement,
    BCRef,
    DERef,
}

/// Represents the possible targets for a word load instruction.
#[derive(Clone, Copy, Debug)]
pub enum LoadWordTarget {
    BC,
    DE,
    HL,
    SP,
    A16Ref,
}

/// Represents the possible sources for a word load instruction.
#[derive(Clone, Copy, Debug)]
pub enum LoadWordSource {
    D16,
    SP,
}

/// Represents the possible types of load instructions.
#[derive(Clone, Copy, Debug)]
pub(super) enum LoadType {
    Byte(LoadByteTarget, LoadByteSource),
    Word(LoadWordTarget, LoadWordSource),
}

impl CPU {
    /// Handles the load instruction for the given [LoadType].
    pub fn handle_load_instruction(&mut self, type_of_load: LoadType) -> u16 {
        match type_of_load {
            LoadType::Byte(target, source) => {
                let value = self.get_value_from_load_byte_source(source);
                match target {
                    LoadByteTarget::REGISTER(register) => {
                        register.set_register(&mut self.registers, value);
                    }
                    LoadByteTarget::HLRef => {
                        self.bus.write_byte(self.registers.get_hl(), value);
                    }
                    LoadByteTarget::HLRefIncrement => {
                        self.bus.write_byte(self.registers.get_hl(), value);
                        self.registers
                            .set_hl(self.registers.get_hl().wrapping_add(1));
                    }
                    LoadByteTarget::HLRefDecrement => {
                        self.bus.write_byte(self.registers.get_hl(), value);
                        self.registers
                            .set_hl(self.registers.get_hl().wrapping_sub(1));
                    }
                    LoadByteTarget::BCRef => {
                        self.bus.write_byte(self.registers.get_bc(), value);
                    }
                    LoadByteTarget::DERef => {
                        self.bus.write_byte(self.registers.get_de(), value);
                    }
                }
                match source {
                    LoadByteSource::D8 => self.pc.wrapping_add(2),
                    _ => self.pc.wrapping_add(1),
                }
            }
            LoadType::Word(target, source) => {
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
                        let address_to_store_to = self.bus.read_next_word_little_endian(self.pc);
                        self.bus.write_byte(address_to_store_to, value as u8);
                        self.bus
                            .write_byte(address_to_store_to.wrapping_add(1), (value >> 8) as u8);
                    }
                }
                self.pc.wrapping_add(3)
            }
        }
    }

    fn get_value_from_load_byte_source(&mut self, source: LoadByteSource) -> u8 {
        match source {
            LoadByteSource::REGISTER(register) => register.get_register(&self.registers),
            LoadByteSource::D8 => self.bus.read_byte(self.pc + 1),
            LoadByteSource::HLRef => self.bus.read_byte(self.registers.get_hl()),
            LoadByteSource::HLRefIncrement => {
                let hl = self.registers.get_hl();
                let value = self.bus.read_byte(hl);
                self.registers.set_hl(hl.wrapping_add(1));
                value
            }
            LoadByteSource::HLRefDecrement => {
                let hl = self.registers.get_hl();
                let value = self.bus.read_byte(hl);
                self.registers.set_hl(hl.wrapping_sub(1));
                value
            }
            LoadByteSource::BCRef => self.bus.read_byte(self.registers.get_bc()),
            LoadByteSource::DERef => self.bus.read_byte(self.registers.get_de()),
        }
    }

    fn get_value_from_load_word_source(&self, source: LoadWordSource) -> u16 {
        match source {
            LoadWordSource::D16 => self.bus.read_next_word_little_endian(self.pc + 1),
            LoadWordSource::SP => self.sp,
        }
    }
}
