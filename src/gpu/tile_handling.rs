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
/// TODO: Write tests or smth for this, this is a bit tricky
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
