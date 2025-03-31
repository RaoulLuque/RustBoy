use crate::{CPU, MemoryBus};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LDHSourceOrTarget {
    A,
    CRef,
    A8Ref,
}

/// Represents the LDH instruction.
/// The enum has only one variant, which is a tuple of two [LDHSourceOrTarget]s. The first
/// element is the target and the second element is the source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LDHType {
    LDH(LDHSourceOrTarget, LDHSourceOrTarget),
}

impl CPU {
    /// Handles the LDH instruction.
    pub fn handle_ldh_instruction(
        &mut self,
        memory_bus: &mut MemoryBus,
        source_or_target: LDHType,
    ) -> u16 {
        match source_or_target {
            LDHType::LDH(target, source) => match (target, source) {
                (LDHSourceOrTarget::CRef, LDHSourceOrTarget::A) => {
                    self.increment_cycle_counter(2);
                    memory_bus.write_byte(0xFF00 + self.registers.c as u16, self.registers.a);
                    self.pc.wrapping_add(1)
                }
                (LDHSourceOrTarget::A, LDHSourceOrTarget::CRef) => {
                    self.increment_cycle_counter(2);
                    self.registers.a = memory_bus.read_byte(0xFF00 + self.registers.c as u16);
                    self.pc.wrapping_add(1)
                }
                (LDHSourceOrTarget::A, LDHSourceOrTarget::A8Ref) => {
                    self.increment_cycle_counter(3);
                    let address = memory_bus.read_byte(self.pc.wrapping_add(1)) as u16;
                    self.registers.a = memory_bus.read_byte(0xFF00 + address);
                    self.pc.wrapping_add(2)
                }
                (LDHSourceOrTarget::A8Ref, LDHSourceOrTarget::A) => {
                    self.increment_cycle_counter(3);
                    let address = memory_bus.read_byte(self.pc.wrapping_add(1)) as u16;
                    memory_bus.write_byte(0xFF00 + address, self.registers.a);
                    self.pc.wrapping_add(2)
                }
                _ => panic!("Invalid LDH instruction"),
            },
        }
    }
}
