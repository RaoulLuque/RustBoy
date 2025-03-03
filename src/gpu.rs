use crate::memory_bus::{VRAM_BEGIN, VRAM_END};

/// Represents the possible values of a tile pixel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TilePixelValue {
    Zero,
    One,
    Two,
    Three,
}

impl TilePixelValue {
    fn from_bits(lower_bit: u8, upper_bit: u8) -> TilePixelValue {
        match (lower_bit != 0, upper_bit != 0) {
            (true, true) => TilePixelValue::Three,
            (false, true) => TilePixelValue::Two,
            (true, false) => TilePixelValue::One,
            (false, false) => TilePixelValue::Zero,
        }
    }
}

/// Represents a tile in the tile set. Is a 2D array of 8x8 tile pixel values.
type Tile = [[TilePixelValue; 8]; 8];
fn empty_tile() -> Tile {
    [[TilePixelValue::Zero; 8]; 8]
}

/// Represents the GPU of the Rust Boy.
/// It has a video RAM (VRAM) of 8KB (0x8000 - 0x9FFF) and a tile set of 384 tiles.
/// The tile set is a 2D array of 8x8 tile pixel values which redundantly stores the tiles
/// which are already in v_ram. They are however more accessible than via the v_ram.
///
/// Also has a tile_data_changed flag to indicate if the tile data has changed.
pub struct GPU {
    v_ram: [u8; VRAM_END as usize - VRAM_BEGIN as usize + 1],
    tile_set: [Tile; 384],
    tile_data_changed: bool,
}

impl GPU {
    /// Reads a byte from the VRAM at the given address.
    /// The address is not the actual absolute address in the grand scheme of the total Rust Boy's
    /// memory but instead the address in the VRAM. That is the absolute address 0x8000 would be
    /// 0x0000 in this case.
    pub fn read_vram(&self, address: u16) -> u8 {
        self.v_ram[address as usize]
    }

    /// Writes a byte to the VRAM at the given address.
    /// The address is not the actual absolute address in the grand scheme of the total Rust Boy's
    /// memory but instead the address in the VRAM. That is the absolute address 0x8000 would be
    /// 0x0000 in this case.
    pub fn write_vram(&mut self, address: u16, value: u8) {
        self.v_ram[address as usize] = value;

        // If our index is greater than 0x1800 we are not writing to the tile set storage
        // so we can simply return
        if address > 0x1800 {
            return;
        }

        // Tiles rows are encoded in two bytes with the first byte always
        // on an even address. Bitwise ANDing the address with 0xffe
        // gives us the address of the first byte.
        // For example: `12 & 0xFFFE == 12` and `13 & 0xFFFE == 12`
        let normalized_address = address & 0xFFFE;

        // First we need to get the two bytes that encode the tile row.
        let byte1 = self.v_ram[normalized_address as usize];
        let byte2 = self.v_ram[normalized_address as usize + 1];

        // Then we need to get the tile index from the address.
        let tile_index = (normalized_address / 16) as usize;
        // Address % 16 gives us the row index in the tile. However, two consecutive bytes encode
        // a row so we need to divide by 2.
        let row_index = ((address % 16) / 2) as usize;

        // Next, we override the tile row with the new values.
        for pixel_index in 0..8 {
            // To determine a pixel's value we must first find the corresponding bit that encodes
            // that pixels value:
            // values:  1111_1111
            // indexes: 0123 4567
            //
            // Now the bit that corresponds to the nth pixel is the bit in the nth
            // position *from the left*.
            //
            // To find the first pixel (a.k.a pixel 0) we find the left most bit (a.k.a bit 7). For
            // the second pixel (a.k.a pixel 1) we first the second most left bit (a.k.a bit 6) and
            // so on. To do that, we create a mask with a 1 in the nth position and a 0 in every
            // other position.
            //
            // Bitwise ANDing this mask with our bytes will leave that particular bit with its
            // original value and every other bit with a 0.
            let mask = 1 << (7 - pixel_index);
            let lower_bit = byte1 & mask;
            let upper_bit = byte2 & mask;

            // We can now convert the two bits to the corresponding TilePixelValue.
            let value = TilePixelValue::from_bits(lower_bit, upper_bit);

            self.tile_set[tile_index][row_index][pixel_index] = value;
        }
        self.tile_data_changed = true;
    }

    pub fn new_empty() -> Self {
        Self {
            v_ram: [0; VRAM_END as usize - VRAM_BEGIN as usize + 1],
            tile_set: [empty_tile(); 384],
            tile_data_changed: false,
        }
    }
}
