use crate::RustBoy;

const ROM_BANK_0_BEGIN: u16 = 0x0000;
const ROM_BANK_0_END: u16 = 0x4000;
const BIOS_BEGIN: u16 = 0x0000;
const BIOS_END: u16 = 0x00FF;
const ROM_BANK_1_BEGIN: u16 = 0x4000;
const ROM_BANK_1_END: u16 = 0x8000;
pub const VRAM_BEGIN: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;

impl RustBoy {
    /// Reads the instruction byte from the memory at the given address. Used separately to check
    /// if the CPU is starting up.
    ///
    /// If the address is 0x0100 and the CPU is starting up, it returns the byte at that address.
    /// Otherwise, it just calls [MemoryBus::read_byte] returns the byte at the given address.
    pub(super) fn read_instruction_byte(&mut self, address: u16) -> u8 {
        if address == 0x0100 && self.starting_up {
            self.starting_up = false;
            self.memory[0x0100]
        } else {
            self.read_byte(address)
        }
    }

    /// Read a byte from the memory at the given address.
    pub(super) fn read_byte(&self, address: u16) -> u8 {
        match address {
            ROM_BANK_0_BEGIN..ROM_BANK_0_END => {
                if self.starting_up {
                    match address {
                        BIOS_BEGIN..BIOS_END => self.bios[address as usize],
                        _ => self.memory[address as usize],
                    }
                } else {
                    self.memory[address as usize]
                }
            }
            ROM_BANK_1_BEGIN..ROM_BANK_1_END => self.memory[address as usize],
            VRAM_BEGIN..VRAM_END => self.gpu.read_vram(address),
            0xFF40 | 0xFF41 | 0xFF42 | 0xFF43 | 0xFF44 | 0xFF45 | 0xFF47 => {
                self.gpu.read_registers(address)
            }
            _ => self.memory[address as usize],
        }
    }

    /// Write a byte to the memory at the given address.
    pub(super) fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            VRAM_BEGIN..VRAM_END => self.gpu.write_vram(address, value),
            0xFF40 | 0xFF41 | 0xFF42 | 0xFF43 | 0xFF44 | 0xFF45 | 0xFF47 => {
                self.gpu.write_registers(address, value)
            }
            _ => {
                self.memory[address as usize] = value;
            }
        }
    }

    /// Reads the word (2 bytes) at the provided address from the memory in little endian order
    /// and returns the result. That is, the least significant byte is read first and then the address
    /// is incremented by 1 and the most significant byte is read.
    pub(super) fn read_word_little_endian(&self, address: u16) -> u16 {
        let low_byte = self.read_byte(address) as u16;
        let high_byte = self.read_byte(address + 1) as u16;
        (high_byte << 8) | low_byte
    }

    /// Reads the next word (2 bytes) from the memory in little endian order and returns the result.
    /// That is, the least significant byte is read first.
    pub(super) fn read_next_word_little_endian(&self, pc: u16) -> u16 {
        self.read_word_little_endian(pc + 1)
    }

    pub(super) fn read_work_big_endian(&self, address: u16) -> u16 {
        let high_byte = self.read_byte(address) as u16;
        let low_byte = self.read_byte(address + 1) as u16;
        (high_byte << 8) | low_byte
    }

    pub(super) fn read_next_work_big_endian(&self, pc: u16) -> u16 {
        self.read_work_big_endian(pc + 1)
    }

    /// Writes data to the memory at the given address.
    pub(super) fn load(&mut self, address: u16, data: &Vec<u8>) {
        for (i, &byte) in data.iter().enumerate() {
            self.write_byte(address + i as u16, byte);
        }
    }

    /// Returns a string representation of the memory bus.
    /// The string is rows of 8 bytes each.
    pub fn memory_to_string(&self) -> String {
        let mut string = String::new();
        string.push_str("MemoryBus: \n");
        for i in 0..self.memory.len() / 8 {
            if i == 4096 {
                string.push_str("End of ROM Bank reached \n");
                break;
            }
            if i % 2 == 0 {
                string.push_str("\n");
            }
            let tmp_string = format!(
                "{:#04X} {:#04X} {:#04X} {:#04X} {:#04X} {:#04X} {:#04X} {:#04X} ",
                self.memory[i],
                self.memory[i + 1],
                self.memory[i + 2],
                self.memory[i + 3],
                self.memory[i + 4],
                self.memory[i + 5],
                self.memory[i + 6],
                self.memory[i + 7]
            );
            string.push_str(&tmp_string);
        }
        string.push('\n');
        string
    }
}
