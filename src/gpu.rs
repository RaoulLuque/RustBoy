use crate::memory_bus::{VRAM_BEGIN, VRAM_END};

/// Represents the possible values of a tile pixel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TilePixelValue {
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

    fn to_rgba(&self) -> [u8; 4] {
        match self {
            TilePixelValue::Zero => [255, 255, 255, 255], // White
            TilePixelValue::One => [192, 192, 192, 255],  // Light Gray
            TilePixelValue::Two => [96, 96, 96, 255],     // Dark Gray
            TilePixelValue::Three => [0, 0, 0, 255],      // Black
        }
    }
}

/// Represents a tile in the tile set. Is a 2D array of 8x8 tile pixel values.
pub type Tile = [[TilePixelValue; 8]; 8];
fn empty_tile() -> Tile {
    [[TilePixelValue::Zero; 8]; 8]
}

/// Represents the GPU of the Rust Boy.
/// It has a video RAM (VRAM) of 8KB (0x8000 - 0x9FFF) and a tile set of 384 tiles.
/// The tile set is a 2D array of 8x8 tile pixel values which redundantly stores the tiles
/// which are already in vram. They are however more accessible than via the vram.
///
/// Also has a tile_data_changed flag to indicate if the tile data has changed.
pub struct GPU {
    vram: [u8; VRAM_END as usize - VRAM_BEGIN as usize + 1],
    pub tile_set: [Tile; 384],
    tile_data_changed: bool,
    tile_map: [[u8; 32]; 32],
    tile_map_changed: bool,
}

impl GPU {
    /// Reads a byte from the VRAM at the given address.
    /// The address is not the actual absolute address in the grand scheme of the total Rust Boy's
    /// memory but instead the address in the VRAM. That is the absolute address 0x8000 would be
    /// 0x0000 in this case.
    pub fn read_vram(&self, address: u16) -> u8 {
        self.vram[address as usize]
    }

    /// Writes a byte to the VRAM at the given address.
    /// The address is not the actual absolute address in the grand scheme of the total Rust Boy's
    /// memory but instead the address in the VRAM. That is the absolute address 0x8000 would be
    /// 0x0000 in this case.
    pub fn write_vram(&mut self, address: u16, value: u8) {
        self.vram[address as usize] = value;

        // If our index is greater than 0x1800 we are not writing to the tile set storage
        // so we can simply return
        if address > 0x1800 {
            return;
        } else {
            self.handle_tile_data_change(address);
        }
    }

    fn handle_tile_data_change(&mut self, address: u16) {
        // Tiles rows are encoded in two bytes with the first byte always
        // on an even address. Bitwise ANDing the address with 0xffe
        // gives us the address of the first byte.
        // For example: `12 & 0xFFFE == 12` and `13 & 0xFFFE == 12`
        let normalized_address = address & 0xFFFE;

        // First we need to get the two bytes that encode the tile row.
        let byte1 = self.vram[normalized_address as usize];
        let byte2 = self.vram[normalized_address as usize + 1];

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
            vram: [0; VRAM_END as usize - VRAM_BEGIN as usize + 1],
            tile_set: [empty_tile(); 384],
            tile_data_changed: false,
            tile_map: [[0; 32]; 32],
            tile_map_changed: false,
        }
    }

    /// Returns true if the tile data has changed since the last time it was checked.
    pub fn tile_data_changed(&mut self) -> bool {
        let res = self.tile_data_changed;
        self.tile_data_changed = false;
        res
    }

    /// Returns true if the tile map has changed since the last time it was checked.
    pub fn tile_map_changed(&mut self) -> bool {
        let res = self.tile_map_changed;
        self.tile_map_changed = false;
        res
    }
}

/// Converts a tile array to a RGBA array.
/// The tile array is a 2D array of 8x8 tile pixel values.
/// The RGBA array is a 1D array of 4 bytes per pixel.
/// The RGBA array is 8 times the width and height of the tile array.
/// The RGBA array is in the format [R, G, B, A, R, G, B, A, ...].
/// The RGBA array is in row major order.
///
/// The generics `I` and `R` are the number of tiles and the number of bytes in the RGBA array,
/// respectively. Therefore, `R` is `I * 8 * 8 * 4`.
pub fn tile_array_to_rgba_array<const I: usize, const R: usize>(tiles: &[Tile; I]) -> [u8; R] {
    let mut rgba_array = [0u8; R];
    for tile_index in 0..tiles.len() {
        for row_in_tile in 0..tiles[tile_index].len() {
            for column_in_tile in 0..tiles[tile_index][row_in_tile].len() {
                let rgba = tiles[tile_index][row_in_tile][column_in_tile].to_rgba();
                let index_in_rgba_array =
                    tile_index * 8 * 8 * 4 + row_in_tile * 8 * 4 + column_in_tile * 4;
                rgba_array[index_in_rgba_array..index_in_rgba_array + 4].copy_from_slice(&rgba);
            }
        }
    }
    rgba_array
}
