use crate::cpu::CPU;

#[derive(Clone, Copy, Debug)]
pub enum LDHSourceOrTarget {
    A,
    CRef,
    A8Ref,
}

#[derive(Clone, Copy, Debug)]
pub enum LDHType {
    LDH(LDHSourceOrTarget, LDHSourceOrTarget),
}

impl CPU {
    pub fn handle_ldh_instruction(&mut self, source_or_target: LDHType) -> u16 {
        match source_or_target {
            LDHType::LDH(source, target) => match (source, target) {
                (LDHSourceOrTarget::CRef, LDHSourceOrTarget::A) => {
                    self.bus
                        .write_byte(0xFF00 + self.registers.c as u16, self.registers.a);
                    self.increment_cycle_counter(2);
                    self.pc.wrapping_add(1)
                }
                (LDHSourceOrTarget::A, LDHSourceOrTarget::CRef) => {
                    self.registers.a = self.bus.read_byte(0xFF00 + self.registers.c as u16);
                    self.increment_cycle_counter(2);
                    self.pc.wrapping_add(1)
                }
                (LDHSourceOrTarget::A, LDHSourceOrTarget::A8Ref) => {
                    let address = self.bus.read_byte(self.pc.wrapping_add(1)) as u16;
                    self.registers.a = self.bus.read_byte(0xFF00 + address);
                    self.increment_cycle_counter(3);
                    self.pc.wrapping_add(2)
                }
                (LDHSourceOrTarget::A8Ref, LDHSourceOrTarget::A) => {
                    let address = self.bus.read_byte(self.pc.wrapping_add(1)) as u16;
                    self.bus.write_byte(0xFF00 + address, self.registers.a);
                    self.increment_cycle_counter(3);
                    self.pc.wrapping_add(2)
                }
                _ => panic!("Invalid LDH instruction"),
            },
        }
    }
}
