use super::{
    GPU, TILE_DATA_BLOCK_0_START, TILE_DATA_BLOCK_1_START, TILE_DATA_BLOCK_2_START,
    TILE_DATA_BLOCK_SIZE, TILEMAP_ONE_START, TILEMAP_SIZE, TILEMAP_ZERO_START,
};
use crate::gpu::registers::LCDCRegister;
use crate::memory_bus::VRAM_BEGIN;
use crate::{MEMORY_SIZE, RustBoy};

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
}

impl GPU {
    /// Handles a change in the tile data. The change is simply applied to the tile set.
    ///
    /// Also sets flags in self.memory_changed, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    /// TODO: Make this a non static method and pass in memory bus?
    pub(crate) fn handle_tile_data_change(rust_boy: &mut RustBoy, address: u16) {
        // Tiles rows are encoded in two bytes with the first byte always
        // on an even address. Bitwise ANDing the address with 0xffe
        // gives us the address of the first byte.
        // For example: `12 & 0xFFFE == 12` and `13 & 0xFFFE == 12`
        let address_first_byte = address & 0xFFFE;

        // First we need to get the two bytes that encode the tile row.
        let byte1 = rust_boy.memory[address_first_byte as usize];
        let byte2 = rust_boy.memory[address_first_byte as usize + 1];

        // Then we need to get the tile index from the address.
        let normalized_address = address - VRAM_BEGIN;
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

            rust_boy.tile_set[tile_index][row_index][pixel_index] = value;
        }
        // We set the memory changed flags to make sure the GPU receives the new tilemap later
        // in rendering.
        if address < 0x8800 {
            // The address lies in block 0
            rust_boy.gpu.memory_changed.tile_data_block_0_1_changed = true;
        } else if address < 0x9000 {
            // The address lies in block 1
            rust_boy.gpu.memory_changed.tile_data_block_0_1_changed = true;
            rust_boy.gpu.memory_changed.tile_data_block_2_1_changed = true;
        } else {
            // The address lies only in block 2
            rust_boy.gpu.memory_changed.tile_data_block_2_1_changed = true;
        }
    }

    /// Returns true if the tile data currently used for the background and window has changed since
    /// the last time it was checked (usually the last scanline).
    ///
    /// Also sets flags in self.memory_changed, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn current_bg_and_wd_tile_data_changed(&mut self, memory: &[u8; MEMORY_SIZE]) -> bool {
        if LCDCRegister::get_background_and_window_tile_data_flag(memory) {
            self.memory_changed.tile_data_block_0_1_changed
        } else {
            self.memory_changed.tile_data_block_2_1_changed
        }
    }

    /// Returns true if the tilemap currently used for the background has changed since the last
    /// time it was checked (usually the last scanline).
    ///
    /// Also sets flags in self.memory_changed, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn current_background_tile_map_changed(&mut self, memory: &[u8; MEMORY_SIZE]) -> bool {
        if LCDCRegister::get_background_tile_map_flag(memory) {
            self.memory_changed.tile_map_1_changed
        } else {
            self.memory_changed.tile_map_0_changed
        }
    }

    /// Returns true if the tilemap currently used for the window has changed since the last
    /// time it was checked (usually the last scanline).
    ///
    /// Also sets flags in self.memory_changed, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn current_window_tile_map_changed(&mut self, memory: &[u8; MEMORY_SIZE]) -> bool {
        if LCDCRegister::get_window_tile_map_flag(memory) {
            self.memory_changed.tile_map_1_changed
        } else {
            self.memory_changed.tile_map_0_changed
        }
    }

    /// Returns the current tile set for the background and window. Switches the addressing mode
    /// automatically, according to LCDC bit 6 (window_tile_map).
    pub fn get_window_tile_map(&self, memory: &[u8; MEMORY_SIZE]) -> [u8; 1024] {
        if !LCDCRegister::get_window_tile_map_flag(memory) {
            memory[TILEMAP_ZERO_START..TILEMAP_ZERO_START + TILEMAP_SIZE]
                .try_into()
                .expect(
                    "Slice should be of correct length, work with me here compiler:\
                0x9800 ... (0x9800 + 1024) = 1024 (bytes)",
                )
        } else {
            memory[TILEMAP_ONE_START..TILEMAP_ONE_START + TILEMAP_SIZE]
                .try_into()
                .expect(
                    "Slice should be of correct length, work with me here compiler:\
                0x9C00 ... (0x9C00 + 1024) = 1024 (bytes)",
                )
        }
    }

    /// Returns the current tile set for the background and window. Switches the addressing mode
    /// automatically according to LCDC bit 4 (background_and_window_tile_data).
    pub fn get_background_and_window_tile_data(&self, memory: &[u8; MEMORY_SIZE]) -> [u8; 4096] {
        if LCDCRegister::get_background_and_window_tile_data_flag(memory) {
            self.get_background_and_window_tile_data_block_0_and_1(memory)
        } else {
            self.get_background_and_window_tile_data_block_2_and_1(memory)
        }
    }

    /// Returns the current tile set for the objects. That is, the tile set in
    /// Block 0 (0x8000 - 0x87FF) and Block 1 (0x8800 - 0x8FFF).
    pub fn get_object_tile_data(&self, memory: &[u8; MEMORY_SIZE]) -> [u8; 4096] {
        self.get_background_and_window_tile_data_block_0_and_1(memory)
    }

    /// Returns the tile data in Block 0 (0x8000 - 0x87FF) and Block 1 (0x8800 - 0x8FFF).
    pub fn get_background_and_window_tile_data_block_0_and_1(
        &self,
        memory: &[u8; MEMORY_SIZE],
    ) -> [u8; 4096] {
        memory[TILE_DATA_BLOCK_0_START..TILE_DATA_BLOCK_1_START + TILE_DATA_BLOCK_SIZE]
            .try_into()
            .expect(
                "Slice should be of correct length, work with me here compiler:\
                0x8000 ... (0x8800 + 2048) = 4096 (bytes)",
            )
    }

    /// Returns the tile data in Block 2 (0x9000 - 0x97FF) and Block 1 (0x8800 - 0x8FFF).
    pub fn get_background_and_window_tile_data_block_2_and_1(
        &self,
        memory: &[u8; MEMORY_SIZE],
    ) -> [u8; 4096] {
        [
            &memory[TILE_DATA_BLOCK_2_START..TILE_DATA_BLOCK_2_START + TILE_DATA_BLOCK_SIZE],
            &memory[TILE_DATA_BLOCK_1_START..TILE_DATA_BLOCK_1_START + TILE_DATA_BLOCK_SIZE],
        ]
        .concat()
        .try_into()
        .expect(
            "Slice should be of correct length, work with me here compiler:\
                0x9000 ... (0x9000 + 2048) + 0x8800 ... (0x8800 + 2048) = 4096 (bytes)",
        )
    }

    /// Returns the current tilemap for the background. Switches the addressing mode
    /// automatically according to LCDC bit 3 (background_tile_map).
    pub fn get_background_tile_map(&self, memory: &[u8; MEMORY_SIZE]) -> [u8; 1024] {
        if !LCDCRegister::get_background_tile_map_flag(memory) {
            self.get_background_tile_map_zero(memory)
        } else {
            self.get_background_tile_map_one(memory)
        }
    }

    /// Returns the zeroth tilemap (0x9800 - 0x9BFF).
    pub fn get_background_tile_map_zero(&self, memory: &[u8; MEMORY_SIZE]) -> [u8; 1024] {
        memory[TILEMAP_ZERO_START..TILEMAP_ZERO_START + TILEMAP_SIZE]
            .try_into()
            .expect(
                "Slice should be of correct length, work with me here compiler:\
                0x9800 ... (0x9800 + 1024) = 1024 (bytes)",
            )
    }

    /// Returns the first tilemap (0x9C00 - 0x9FFF).
    pub fn get_background_tile_map_one(&self, memory: &[u8; MEMORY_SIZE]) -> [u8; 1024] {
        memory[TILEMAP_ONE_START..TILEMAP_ONE_START + TILEMAP_SIZE]
            .try_into()
            .expect(
                "Slice should be of correct length, work with me here compiler:\
                0x9C00 ... (0x9C00 + 1024) = 1024 (bytes)",
            )
    }
}

/// Represents a tile in the tile set. Is a 2D array of 8x8 tile pixel values.
pub type Tile = [[TilePixelValue; 8]; 8];

pub fn empty_tile() -> Tile {
    [[TilePixelValue::Zero; 8]; 8]
}
