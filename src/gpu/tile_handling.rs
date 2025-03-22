use super::{GPU, TILEMAP_ONE_START, TILEMAP_SIZE, TILEMAP_TWO_START};
use crate::memory_bus::VRAM_BEGIN;

/// Represents the possible values of a tile pixel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TilePixelValue {
    Zero,
    One,
    Two,
    Three,
}

impl TilePixelValue {
    /// Converts the two bits of a tile pixel to a TilePixelValue.
    pub(crate) fn from_bits(lower_bit: u8, upper_bit: u8) -> TilePixelValue {
        match (lower_bit != 0, upper_bit != 0) {
            (true, true) => TilePixelValue::Three,
            (false, true) => TilePixelValue::Two,
            (true, false) => TilePixelValue::One,
            (false, false) => TilePixelValue::Zero,
        }
    }

    /// Converts the TilePixelValue to an RGBA color.
    fn to_rgba(&self) -> [u8; 4] {
        match self {
            TilePixelValue::Zero => [255, 255, 255, 255], // White
            TilePixelValue::One => [192, 192, 192, 255],  // Light Gray
            TilePixelValue::Two => [96, 96, 96, 255],     // Dark Gray
            TilePixelValue::Three => [0, 0, 0, 255],      // Black
        }
    }
}

impl GPU {
    /// Handles a change in the tile data. The change is simply applied to the tile set.
    pub(crate) fn handle_tile_data_change(&mut self, address: u16) {
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

    /// Returns the current tile set for the background and window. Switches the addressing mode
    /// automatically according to LCDC bit 6 (window_tile_map).
    pub fn get_window_tile_map(&self) -> &[u8; 1024] {
        if self.gpu_registers.lcd_control.window_tile_map {
            self.vram[TILEMAP_ONE_START - VRAM_BEGIN as usize
                ..TILEMAP_ONE_START + TILEMAP_SIZE - VRAM_BEGIN as usize]
                .try_into()
                .expect("Slice should be of correct length, work with me here compiler")
        } else {
            self.vram[TILEMAP_TWO_START - VRAM_BEGIN as usize
                ..TILEMAP_TWO_START + TILEMAP_SIZE - VRAM_BEGIN as usize]
                .try_into()
                .expect("Slice should be of correct length, work with me here compiler")
        }
    }

    /// Returns the current tile set for the background and window. Switches the addressing mode
    /// automatically according to LCDC bit 4 (background_and_window_tile_data).
    pub fn get_background_and_window_tile_data(&self) -> [Tile; 256] {
        if self
            .gpu_registers
            .lcd_control
            .background_and_window_tile_data
        {
            self.get_background_and_window_tile_data_block_0_and_1()
        } else {
            self.get_background_and_window_tile_data_block_2_and_1()
        }
    }

    /// Returns the current tile set for the objects. That is, the tile set in
    /// Block 0 (0x8000 - 0x87FF) and Block 1 (0x8800 - 0x8FFF).
    pub fn get_object_tile_data(&self) -> [Tile; 256] {
        self.get_background_and_window_tile_data_block_0_and_1()
    }

    /// Returns the tile data in Block 0 (0x8000 - 0x87FF) and Block 1 (0x8800 - 0x8FFF).
    pub fn get_background_and_window_tile_data_block_0_and_1(&self) -> [Tile; 256] {
        self.tile_set[0..256]
            .try_into()
            .expect("Slice should be of correct length, work with me here compiler")
    }

    /// Returns the tile data in Block 2 (0x9000 - 0x97FF) and Block 1 (0x8800 - 0x8FFF).
    pub fn get_background_and_window_tile_data_block_2_and_1(&self) -> [Tile; 256] {
        [&self.tile_set[256..384], &self.tile_set[128..256]]
            .concat()
            .try_into()
            .expect("Slice should be of correct length, work with me here compiler")
    }

    /// Returns the current tile map for the background. Switches the addressing mode
    /// automatically according to LCDC bit 3 (background_tile_map).
    pub fn get_background_tile_map(&self) -> &[u8; 1024] {
        if !self.gpu_registers.lcd_control.background_tile_map {
            self.get_background_tile_map_one()
        } else {
            self.get_background_tile_map_two()
        }
    }

    /// Returns the first tile map (0x9800 - 0x9BFF).
    pub fn get_background_tile_map_one(&self) -> &[u8; 1024] {
        self.vram[TILEMAP_ONE_START - VRAM_BEGIN as usize
            ..TILEMAP_ONE_START + TILEMAP_SIZE - VRAM_BEGIN as usize]
            .try_into()
            .expect("Slice should be of correct length, work with me here compiler")
    }

    /// Returns the second tile map (0x9C00 - 0x9FFF).
    pub fn get_background_tile_map_two(&self) -> &[u8; 1024] {
        self.vram[TILEMAP_TWO_START - VRAM_BEGIN as usize
            ..TILEMAP_TWO_START + TILEMAP_SIZE - VRAM_BEGIN as usize]
            .try_into()
            .expect("Slice should be of correct length, work with me here compiler")
    }
}

/// Represents a tile in the tile set. Is a 2D array of 8x8 tile pixel values.
pub type Tile = [[TilePixelValue; 8]; 8];

pub fn empty_tile() -> Tile {
    [[TilePixelValue::Zero; 8]; 8]
}

/// Converts a tile array to a RGBA array.
/// The tile array is a 2D array of 8x8 tile pixel values.
/// The RGBA array is a 1D array of 4 bytes per pixel.
/// The RGBA array is 8 times the width and height of the tile array.
/// The RGBA array is in the format [R, G, B, A, R, G, B, A, ...].
/// The RGBA array is in row major order.
///
/// TODO: Optimize this because it seems to take a long time. Possibly move this to the GPU
/// to handle.
pub fn tile_array_to_rgba_array(tiles: &[Tile; 256]) -> [u8; 65536] {
    let mut rgba_array = [0u8; 65536];
    // Loop over the 16 rows of tiles
    // for tile_row_index in 0..16 {
    //     // Loop over the 8 rows of pixels per tile
    //     for in_tile_row_index in 0..8 {
    //         // Loop over the 16 columns of tiles
    //         for tile_column_index in 0..16 {
    //             // Loop over the 8 columns of pixels per tile
    //             for in_tile_column_index in 0..8 {
    //                 let tile_index = tile_row_index * 16 + tile_column_index;
    //                 let tile = tiles[tile_index];
    //                 let pixel_value = tile[in_tile_row_index][in_tile_column_index];
    //                 let rgba = pixel_value.to_rgba();
    //                 let index_in_rgba_array = (tile_row_index * 8 + in_tile_row_index) * 16 * 8 * 4
    //                     + (tile_column_index * 8 + in_tile_column_index) * 4;
    //                 rgba_array[index_in_rgba_array..index_in_rgba_array + 4].copy_from_slice(&rgba);
    //             }
    //         }
    //     }
    // }

    // Loop over the 256 tiles
    for tile_index in 0..256 {
        // Loop over the 8 rows of pixels per tile
        for in_tile_row_index in 0..8 {
            // Copy over the entire row of pixels as one chunk to the final rgba array
            let rgba_row = tile_pixel_value_row_to_rgba(tiles[tile_index][in_tile_row_index]);
            let index_in_rgba_array = ((tile_index / 16) * 16 * 8 * 4 * 8)
                + (in_tile_row_index * 16 * 8 * 4)
                + ((tile_index % 16) * 8 * 4);
            rgba_array[index_in_rgba_array..index_in_rgba_array + 32].copy_from_slice(&rgba_row);
        }
    }
    rgba_array
}

fn tile_pixel_value_row_to_rgba(row: [TilePixelValue; 8]) -> [u8; 32] {
    let mut rgba_row = [0u8; 32];
    for (i, pixel_value) in row.iter().enumerate() {
        let rgba = pixel_value.to_rgba();
        rgba_row[i * 4..i * 4 + 4].copy_from_slice(&rgba);
    }
    rgba_row
}

#[allow(dead_code)]
pub fn tile_to_string(tile: &Tile) -> String {
    let mut string = String::new();
    for row in tile {
        for pixel in row {
            string.push_str(&convert_pixel_to_string(pixel));
            string.push_str(" ");
        }
        string.push('\n');
    }
    string
}

pub fn tile_data_to_string(tile_data: &[Tile; 256]) -> String {
    let mut res_string = String::new();
    for tile_row in 0..16 {
        for in_tile_row in 0..8 {
            for tile_column in 0..16 {
                for in_tile_column in 0..8 {
                    if in_tile_row == 0 && tile_column == 0 && in_tile_column == 0 {
                        let tile_index_for_printing: usize = tile_row * 16 + tile_column;
                        for i in 0..16 {
                            res_string.push_str(&format!(
                                "{:<17}",
                                tile_n_string(tile_index_for_printing + i),
                            ));
                        }
                        res_string.push_str("\n");
                    }
                    let tile_index = tile_row * 16 + tile_column;
                    let tile: Tile = tile_data[tile_index];
                    let pixel_value = tile[in_tile_row][in_tile_column];
                    res_string.push_str(&convert_pixel_to_string(&pixel_value));
                    res_string.push_str(" ");
                }
                res_string.push_str(" ");
            }
            res_string.push('\n');
        }
        res_string.push('\n');
    }
    res_string
}

fn tile_n_string(tile_index: usize) -> String {
    format!("Tile {}:", tile_index)
}

pub fn convert_pixel_to_string(pixel: &TilePixelValue) -> String {
    match pixel {
        TilePixelValue::Zero => "▫".to_string(),
        TilePixelValue::One => "▪".to_string(),
        TilePixelValue::Two => "□".to_string(),
        TilePixelValue::Three => "■".to_string(),
    }
}

pub fn tile_map_to_string(tile_map: &[u8; 1024]) -> String {
    let mut string = String::new();
    for row in 0..32 {
        for column in 0..32 {
            let tile_index = (row * 32 + column) as usize;
            let tile_value = tile_map[tile_index];
            string.push_str(&format!("{} ", tile_value));
        }
        string.push('\n');
    }
    string
}
