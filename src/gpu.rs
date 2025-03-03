use crate::memory_bus::{VRAM_BEGIN, VRAM_END};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TilePixelValue {
    Zero,
    One,
    Two,
    Three,
}

type Tile = [[TilePixelValue; 8]; 8];
fn empty_tile() -> Tile {
    [[TilePixelValue::Zero; 8]; 8]
}

pub struct GPU {
    v_ram: [u8; VRAM_END as usize - VRAM_BEGIN as usize + 1],
    tile_set: [Tile; 384],
}

impl GPU {
    pub fn read_vram(&self, address: u16) -> u8 {
        self.v_ram[address as usize]
    }

    pub fn write_vram(&mut self, address: u16, value: u8) {
        self.v_ram[address as usize] = value;
    }

    pub fn new_empty() -> Self {
        Self {
            v_ram: [0; VRAM_END as usize - VRAM_BEGIN as usize + 1],
            tile_set: [empty_tile(); 384],
        }
    }
}
