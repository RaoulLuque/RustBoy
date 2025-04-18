mod mbc1;

pub(super) enum MBCType {
    MBC1,
}

pub(super) enum MBC {
    MBC1(mbc1::MBC1),
}

impl MBC {
    pub fn new(mbc_type: MBCType, rom_data: Vec<u8>, ram_size: usize) -> Self {
        match mbc_type {
            MBCType::MBC1 => MBC::MBC1(mbc1::MBC1::new(rom_data, ram_size)),
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match self {
            MBC::MBC1(mbc) => mbc.read_byte(address),
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match self {
            MBC::MBC1(mbc) => mbc.write_byte(address, value),
        }
    }
}
