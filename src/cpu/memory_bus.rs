/// Struct to represent the memory bus.
/// It is an array that represents the memory of the RustBoy.
/// 0xFFFF = 65536 is the size of the memory in bytes
pub struct MemoryBus {
    pub memory: [u8; 0xFFFF],
}

impl MemoryBus {
    /// Read a byte from the memory at the given address.
    pub(super) fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    /// Write a byte to the memory at the given address.
    pub(super) fn write_byte(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
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

    /// Writes data to the memory at the given address.
    pub(super) fn load(&mut self, address: u16, data: &Vec<u8>) {
        for (i, &byte) in data.iter().enumerate() {
            self.write_byte(address + i as u16, byte);
        }
    }
}
