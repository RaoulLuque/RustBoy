use crate::cpu::instructions::Register;
use crate::cpu::CPU;

/// Represents the possible targets for a byte load instruction.
#[derive(Clone, Copy, Debug)]
pub enum LoadByteTarget {
    REGISTER(Register),
    HLRef,
    HLRefIncrement,
    HLRefDecrement,
}

/// Represents the possible sources for a byte load instruction.
#[derive(Clone, Copy, Debug)]
pub enum LoadByteSource {
    REGISTER(Register),
    D8,
    HL,
    HLIncrement,
    HLDecrement,
}

/// Represents the possible types of load instructions.
#[derive(Clone, Copy, Debug)]
pub(super) enum LoadType {
    Byte(LoadByteTarget, LoadByteSource),
}

impl CPU {
    /// Handles the load instruction for the given [LoadType].
    pub fn handle_load_instruction(&mut self, type_of_load: LoadType) -> u16 {
        match type_of_load {
            LoadType::Byte(target, source) => {
                let value = match source {
                    LoadByteSource::REGISTER(register) => register.get_register(&self.registers),
                    LoadByteSource::D8 => self.bus.read_byte(self.pc + 1),
                    LoadByteSource::HL => self.bus.read_byte(self.registers.get_hl()),
                    _ => todo!("Not implemented"),
                };
                match target {
                    LoadByteTarget::REGISTER(register) => {
                        register.set_register(&mut self.registers, value);
                    }
                    LoadByteTarget::HLRef => {
                        self.bus.write_byte(self.registers.get_hl(), value);
                    }
                    _ => todo!("Not Implemented"),
                }
                match source {
                    LoadByteSource::D8 => self.pc.wrapping_add(2),
                    _ => self.pc.wrapping_add(1),
                }
            }
        }
    }
}
