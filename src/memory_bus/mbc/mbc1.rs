/// Struct to represent the MBC1 memory bank controller.
/// This struct handles the memory (ram and rom) mapping for cartridges using MBC1.
///
/// The fields of this struct are:
/// - `rom`: A vector of bytes representing the ROM data.
/// - `ram`: A vector of bytes representing the RAM data.
/// - `ram_enabled`: A boolean indicating whether reading/writing of external RAM is enabled.
/// - `rom_bank_number`: The current ROM bank number. Is a 5-bit register (range $01-$1F) which
/// selects the ROM bank number for the 4000-7FFF region.
/// - `ram_bank_number`: The current RAM bank number. Is a 2-bit register (range $00-$03) which
/// selects the RAM bank (32 KiB ram carts only), or to select the upper 2 bits (4-5) of the ROM
/// bank number (1 MiB ROM or larger carts only).
/// - `mode`: A 1-bit register (range $00-$01) which selects the mode of operation.
pub struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    rom_bank_number: u8,
    ram_bank_number: u8,
    mode: bool,
}

impl MBC1 {
    /// Creates a new MBC1 instance with the given ROM data and RAM size.
    pub(super) fn new(rom_data: Vec<u8>, ram_size: usize) -> Self {
        let ram = vec![0; ram_size];
        MBC1 {
            rom: rom_data,
            ram,
            ram_enabled: false,
            rom_bank_number: 1,
            ram_bank_number: 0,
            mode: false,
        }
    }

    /// Read a byte from the memory controlled by the MBC1.
    ///
    /// Panics if the address is not in the range of 0x0000..=0x7FFF or 0xA000..=0xBFFF.
    pub(super) fn read_byte(&self, address: u16) -> u8 {
        match address {
            // ROM Bank 0
            0x0000..=0x3FFF => self.rom[address as usize],
            // ROM Bank 0x01-0x1F
            0x4000..=0x7FFF => {
                let bank_offset = (self.rom_bank_number as usize) * 0x4000;
                self.rom[bank_offset + (address as usize - 0x4000)]
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let bank_offset = (self.ram_bank_number as usize) * 0x2000;
                    self.ram[bank_offset + (address as usize - 0xA000)]
                } else {
                    0
                }
            }
            _ => panic!("Invalid read address in MBC: {:#X}", address),
        }
    }

    /// Write a byte to the memory controlled by the MBC1.
    ///
    /// Panics if the address is not in the range of 0x000..=0x7FFF or 0xA000..=0xBFFF.
    pub(super) fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // RAM Enable/Disable. Ram is enabled if the value is 0x0A.
            0x0000..=0x1FFF => {
                if self.ram.len() > 0 {
                    // The RAM can only be enabled if the cartridge has RAM.
                    self.ram_enabled = (value & 0x0A) == 0x0A;
                }
            }
            // ROM Bank Number. Only the lower 5 bits are used, and bank_number 0 is considered as
            // bank_number 1.
            0x2000..=0x3FFF => {
                let bank_number = value & 0x1F;
                if bank_number == 0 {
                    self.rom_bank_number = 1;
                } else {
                    self.rom_bank_number = bank_number;
                }
            }
            // RAM Bank Number / Upper Bits of ROM Bank Number. Depending on the mode, this sets the
            // RAM bank number (if mode is 0) or the upper bits of the ROM bank number (if mode is 1).
            0x4000..=0x5FFF => {
                if self.mode {
                    self.ram_bank_number = value & 0x03;
                } else {
                    let upper_bits = value & 0x03;
                    self.rom_bank_number = (self.rom_bank_number & 0x1F) | (upper_bits << 5);
                }
            }
            // Mode Selection
            0x6000..=0x7FFF => {
                self.mode = value == 1;
            }
            // RAM Write
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let bank_offset = (self.ram_bank_number as usize) * 0x2000;
                    self.ram[bank_offset + (address as usize - 0xA000)] = value;
                }
            }
            _ => panic!("Invalid write address in MBC: {:#X}", address),
        }
    }
}
