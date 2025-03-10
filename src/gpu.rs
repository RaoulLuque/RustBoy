mod registers;
pub(crate) mod tile_handling;

use crate::memory_bus::{VRAM_BEGIN, VRAM_END};

use crate::debugging::DebuggingFlags;
use registers::GPURegisters;
use tile_handling::{Tile, TilePixelValue};

const TILEMAP_ONE_START: usize = 0x9800;
const TILEMAP_TWO_START: usize = 0x9C00;
const TILEMAP_SIZE: usize = 1024;

/// Represents the GPU of the Rust Boy.
/// It has a video RAM (VRAM) of 8KB (0x8000 - 0x9FFF) containing the tile set with 384 tiles
/// and two tile maps of 32 * 32 = 1024 bytes each.
///
/// The tile set is a 2D array of 8x8 tile pixel values which redundantly stores the tiles
/// which are already in vram. They are however more accessible than via the vram.
///
/// The tile maps are two 2D arrays of 32x32 tile u8 indices which are used to determine which tile
/// to draw at which position on the screen. They are just stored directly in the vram field.
///
/// Also has a tile_data_changed flag to indicate if the tile data has changed.
pub struct GPU {
    vram: [u8; VRAM_END as usize - VRAM_BEGIN as usize + 1],
    pub tile_set: [Tile; 384],
    rendering_info: RenderingInfo,
    pub gpu_registers: GPURegisters,
    tile_data_changed: bool,
    tile_map_changed: bool,
    background_viewport_changed: bool,

    debugging_flags: DebuggingFlags,
}

/// Struct to collect the information about the current rendering state of the GPU.
pub struct RenderingInfo {
    dots_clock_sum: u32,
    dots_clock: u32,
    dots_for_transfer: u32,
}

/// Represents the possible rendering modes of the GPU.
/// Rendering modes are used to determine what the GPU is currently doing.
/// The GPU can be in one of four rendering modes:
/// - HBlank: The GPU is currently in the horizontal blanking period.
/// - VBlank: The GPU is currently in the vertical blanking period.
/// - OAMSearch: The GPU is currently searching for sprites.
/// - Transfer: The GPU is currently transferring data to the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenderingMode {
    HBlank0,
    VBlank1,
    OAMScan2,
    Transfer3,
}

/// Represents the possible tasks of the GPU.
pub enum RenderTask {
    None,
    Render,
}

impl GPU {
    /// Steps the GPU by the given number of cycles.
    /// Returns a RenderTask indicating what the GPU should do next.
    /// For now, the GPU only renders the entire frame before entering VBlank.
    /// In the future, the GPU should render by lines.
    ///
    /// The GPU steps through four different [RenderingMode]s.
    pub fn gpu_step(&mut self, cycles: u32) -> RenderTask {
        self.rendering_info.dots_clock += cycles;
        self.rendering_info.dots_clock_sum += cycles;
        match self.gpu_registers.lcd_status.gpu_mode {
            RenderingMode::HBlank0 => {
                // TODO: Implement rendering by lines instead of entire frame
                if self.rendering_info.dots_clock >= 456 - self.rendering_info.dots_for_transfer {
                    self.rendering_info.dots_clock -= 456 - self.rendering_info.dots_for_transfer;
                    self.gpu_registers
                        .set_scanline(self.gpu_registers.get_scanline() + 1);
                    // For now: Render the entire frame before entering VBlank
                    if self.gpu_registers.get_scanline() == 144 {
                        self.gpu_registers.lcd_status.gpu_mode = RenderingMode::VBlank1;
                        self.gpu_registers.set_ppu_mode(RenderingMode::VBlank1);
                        return RenderTask::Render;
                    } else {
                        self.gpu_registers.lcd_status.gpu_mode = RenderingMode::OAMScan2;
                        self.gpu_registers.set_ppu_mode(RenderingMode::OAMScan2);
                    }
                }
            }
            RenderingMode::VBlank1 => {
                if self.rendering_info.dots_clock >= 456 {
                    self.rendering_info.dots_clock -= 456;
                    self.gpu_registers
                        .set_scanline(self.gpu_registers.get_scanline() + 1);
                    if self.gpu_registers.get_scanline() == 154 {
                        self.gpu_registers.set_scanline(0);
                        self.gpu_registers.lcd_status.gpu_mode = RenderingMode::OAMScan2;
                        self.gpu_registers.set_ppu_mode(RenderingMode::OAMScan2);
                    }
                }
            }
            RenderingMode::OAMScan2 => {
                if self.rendering_info.dots_clock >= 80 {
                    self.rendering_info.dots_clock -= 80;
                    self.gpu_registers.lcd_status.gpu_mode = RenderingMode::Transfer3;
                    self.gpu_registers.set_ppu_mode(RenderingMode::Transfer3);
                }
            }
            RenderingMode::Transfer3 => {
                // TODO: Implement possible delay in this Mode if background scrolling or sprite fetching happened
                if self.rendering_info.dots_clock >= 172 {
                    self.rendering_info.dots_clock -= 172;
                    self.rendering_info.dots_for_transfer = 172;
                    self.gpu_registers.lcd_status.gpu_mode = RenderingMode::HBlank0;
                    self.gpu_registers.set_ppu_mode(RenderingMode::HBlank0);
                }
            }
        }
        RenderTask::None
    }

    /// Reads a byte from the VRAM at the given address.
    /// Since the address is the absolute address in the grand scheme of the total Rust Boy's
    /// memory, we have to convert it to the relative address in terms of the VRAM. That is the
    /// absolute address 0x8000 would be the relative address 0x0000.
    pub fn read_vram(&self, address: u16) -> u8 {
        self.vram[(address - VRAM_BEGIN) as usize]
    }

    /// Writes a byte to the VRAM at the given address.
    /// Since the address is the absolute address in the grand scheme of the total Rust Boy's
    /// memory, we have to convert it to the relative address in terms of the VRAM. That is the
    /// absolute address 0x8000 would be the relative address 0x0000.
    pub fn write_vram(&mut self, address: u16, value: u8) {
        let address = address - VRAM_BEGIN;
        self.vram[address as usize] = value;

        // If our index is greater than or equal to 0x1800, we are not writing to the tile set storage
        // so we can simply return
        if address >= 0x1800 {
            // TODO Set tile_map_changed flag only if the currently used tile map has actually changed and not the other one
            self.tile_map_changed = true;
            return;
        } else {
            self.handle_tile_data_change(address);
        }
    }

    /// Handles a change in the tile data. The change is simply applied to the tile set.
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

    /// Returns a new GPU with empty tile set and empty VRAM.
    pub fn new_empty(debugging_flags: DebuggingFlags) -> Self {
        Self {
            vram: [0; VRAM_END as usize - VRAM_BEGIN as usize + 1],
            tile_set: [tile_handling::empty_tile(); 384],
            rendering_info: RenderingInfo {
                dots_clock_sum: 0,
                dots_clock: 0,
                dots_for_transfer: 0,
            },
            gpu_registers: GPURegisters::new(debugging_flags),
            tile_data_changed: true,
            tile_map_changed: true,
            background_viewport_changed: true,

            debugging_flags,
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
            self.tile_set[0..256]
                .try_into()
                .expect("Slice should be of correct length, work with me here compiler")
        } else {
            [&self.tile_set[256..384], &self.tile_set[0..128]]
                .concat()
                .try_into()
                .expect("Slice should be of correct length, work with me here compiler")
        }
    }

    /// Returns the current tile map for the background. Switches the addressing mode
    /// automatically according to LCDC bit 3 (background_tile_map).
    pub fn get_background_tile_map(&self) -> &[u8; 1024] {
        if self.gpu_registers.lcd_control.background_tile_map {
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
}
