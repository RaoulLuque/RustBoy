pub(crate) mod object_handling;
mod registers;
pub(crate) mod tile_handling;

use crate::memory_bus::{VRAM_BEGIN, VRAM_END};

use crate::debugging::{DebugInfo, DebuggingFlagsWithoutFileHandles};
use crate::interrupts::InterruptFlagRegister;
use object_handling::Object;
use registers::GPURegisters;
use tile_handling::Tile;

const TILE_DATA_BLOCK_0_START: usize = 0x8000;
const TILE_DATA_BLOCK_1_START: usize = 0x8800;
const TILE_DATA_BLOCK_2_START: usize = 0x9000;
const TILE_DATA_BLOCK_SIZE: usize = 2048;
const TILEMAP_ZERO_START: usize = 0x9800;
const TILEMAP_ONE_START: usize = 0x9C00;
const TILEMAP_SIZE: usize = 1024;

/// The number of dots (GPU cycles) in the Transfer Mode.
const DOTS_IN_TRANSFER: u32 = 172;
/// The number of dots (GPU cycles) in the HBlank plus in the Transfer Mode.
pub(crate) const DOTS_IN_HBLANK_PLUS_TRANSFER: u32 = 376;
/// The number of dots (GPU cycles) in the OAM Scan Mode.
const DOTS_IN_OAM_SCAN: u32 = 80;
/// The number of dots (GPU cycles) in the VBlank Mode.
pub(crate) const DOTS_IN_VBLANK: u32 = 4560;

/// The GPU mode the GPU is in when the LCD is turned off.
pub(crate) const GPU_MODE_WHILE_LCD_TURNED_OFF: RenderingMode = RenderingMode::HBlank0;

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
    pub(crate) rendering_info: RenderingInfo,
    pub gpu_registers: GPURegisters,
    background_viewport_changed: bool,

    pub(crate) oam: [Object; 40],

    pub memory_changed: ChangesToPropagateToShader,

    debugging_flags: DebuggingFlagsWithoutFileHandles,
}

/// Struct to collect the information about the current rendering state of the GPU.
pub struct RenderingInfo {
    pub(crate) dots_clock: u32,
    pub(crate) total_dots: u128,
    dots_for_transfer: u32,
    lcd_was_turned_off: bool,
    first_scanline_after_lcd_was_turned_on: bool,
}

/// Represents the possible rendering modes of the GPU.
/// Rendering modes are used to determine what the GPU is currently doing.
/// The GPU can be in one of four rendering modes:
/// - HBlank: The GPU is currently in the horizontal blanking period.
/// - VBlank: The GPU is currently in the vertical blanking period.
/// - OAMSearch: The GPU is currently searching for sprites.
/// - Transfer: The GPU is currently transferring data to the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderingMode {
    HBlank0,
    VBlank1,
    OAMScan2,
    Transfer3,
}

/// Represents the possible tasks of the GPU.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderTask {
    None,
    WriteLineToBuffer(u8),
    RenderFrame,
}

/// Struct to keep track of changes/writes to tile data, tile map, viewport position, and OAM.
///
/// We reset this struct after rendering each scanline. Therefore, it tracks the resources that
/// changed since the last scanline which the render step can use to only (re)send the data that
/// actually changed to the Shader/GPU.
pub struct ChangesToPropagateToShader {
    pub(crate) tile_data_flag_changed: bool,
    pub(crate) tile_data_block_0_1_changed: bool,
    pub(crate) tile_data_block_2_1_changed: bool,
    pub(crate) background_tile_map_flag_changed: bool,
    pub(crate) window_tile_map_flag_changed: bool,
    pub(crate) tile_map_0_changed: bool,
    pub(crate) tile_map_1_changed: bool,
    pub(crate) background_viewport_position_changed: bool,
}

impl ChangesToPropagateToShader {
    /// Returns a new instance of MemoryChanged with only false values
    pub(crate) fn new() -> Self {
        Self {
            tile_data_flag_changed: false,
            tile_data_block_0_1_changed: false,
            tile_data_block_2_1_changed: false,
            background_tile_map_flag_changed: false,
            window_tile_map_flag_changed: false,
            tile_map_0_changed: false,
            tile_map_1_changed: false,
            background_viewport_position_changed: false,
        }
    }
}

impl GPU {
    /// Steps the GPU by the given number of dots.
    /// Returns a RenderTask indicating what the GPU should do next.
    /// For now, the GPU only renders the entire frame before entering VBlank.
    /// In the future, the GPU should render by lines.
    ///
    /// The GPU steps through four different [RenderingMode]s. When VBlank is entered, or rather,
    /// when HBlank is exited, the flag for a VBlank interrupt is set.
    pub fn gpu_step(
        &mut self,
        interrupt_flags: &mut InterruptFlagRegister,
        dots: u32,
    ) -> RenderTask {
        // Always increment total dots (for debugging purposes)
        self.rendering_info.total_dots += dots as u128;

        if self.gpu_registers.lcd_control.get_display_on_flag() == false {
            if self.rendering_info.lcd_was_turned_off == false {
                // If the LCD is not enabled, there is no rendering task and we can reset the GPU
                // to its initial state. We only do this once when the LCD is turned off.
                self.rendering_info.dots_clock = 0;
                self.rendering_info.dots_for_transfer = 0;
                self.gpu_registers
                    .set_ppu_mode(GPU_MODE_WHILE_LCD_TURNED_OFF, interrupt_flags);
                self.gpu_registers.set_scanline(0, interrupt_flags);
                self.rendering_info.lcd_was_turned_off = true;
            }
            RenderTask::None
        } else {
            if self.rendering_info.lcd_was_turned_off {
                // If the LCD was turned off, the GPU starts in HBlank mode and after this goes
                // RenderingMode::Transfer3, which happens only after the lcd was turned on for the
                // first "HBlank cycle", see:
                // https://www.reddit.com/r/EmuDev/comments/1cykjdr/gameboy_ppu_timing_question/
                // To make sure this irregularity is handled, we set the first_scanline_after_lcd_was_turned_on
                // flag.
                // TODO: Possibly handle that first frame after turning on the LCD is not actually
                // sent to the screen, but rather just a blank screen.
                self.gpu_registers
                    .set_ppu_mode(RenderingMode::HBlank0, interrupt_flags);
                self.rendering_info.first_scanline_after_lcd_was_turned_on = true;
                self.rendering_info.lcd_was_turned_off = false;
            }
            self.rendering_info.dots_clock += dots;
            match self.gpu_registers.get_gpu_mode() {
                RenderingMode::HBlank0 => {
                    if self.rendering_info.first_scanline_after_lcd_was_turned_on {
                        // If the LCD was turned off, it immediately enters HBlank mode which only
                        // lasts [DOTS_IN_OAM_SCAN] dots and then enters Transfer mode.
                        if self.rendering_info.dots_clock >= DOTS_IN_OAM_SCAN {
                            self.rendering_info.dots_clock -= DOTS_IN_OAM_SCAN;
                            self.gpu_registers
                                .set_ppu_mode(RenderingMode::Transfer3, interrupt_flags);
                            // We can now set the first_scanline_after_lcd_was_turned_on flag to
                            // false, since after this we are in Transfer mode and then regular
                            // HBlank mode, so the GPU can return to normal operation.
                            self.rendering_info.first_scanline_after_lcd_was_turned_on = false;
                        }
                    } else {
                        if self.rendering_info.dots_clock
                            >= DOTS_IN_HBLANK_PLUS_TRANSFER - self.rendering_info.dots_for_transfer
                        {
                            self.rendering_info.dots_clock -= DOTS_IN_HBLANK_PLUS_TRANSFER
                                - self.rendering_info.dots_for_transfer;
                            self.gpu_registers.set_scanline(
                                self.gpu_registers
                                    .get_scanline(None, None, None, false, true)
                                    + 1,
                                interrupt_flags,
                            );
                            if self
                                .gpu_registers
                                .get_scanline(None, None, None, false, true)
                                == 144
                            {
                                // We are entering VBlank, so we need to set the VBlank flag
                                // and set the GPU mode to VBlank. Also, we send a render frame request to
                                // the GPU, which renders the framebuffer to the screen.
                                self.gpu_registers
                                    .set_ppu_mode(RenderingMode::VBlank1, interrupt_flags);
                                interrupt_flags.vblank = true;
                                return RenderTask::RenderFrame;
                            } else {
                                // We are still in HBlank, so we need to set the GPU mode to OAMScan2.
                                // Also we send a request to the GPU to write the current line to the
                                // framebuffer
                                self.gpu_registers
                                    .set_ppu_mode(RenderingMode::OAMScan2, interrupt_flags);
                                return RenderTask::WriteLineToBuffer(
                                    self.gpu_registers
                                        .get_scanline(None, None, None, false, true)
                                        - 1,
                                );
                            }
                        }
                    }
                }
                RenderingMode::VBlank1 => {
                    if self.rendering_info.dots_clock >= DOTS_IN_VBLANK / 10 {
                        self.rendering_info.dots_clock -= DOTS_IN_VBLANK / 10;
                        self.gpu_registers.set_scanline(
                            self.gpu_registers
                                .get_scanline(None, None, None, false, true)
                                + 1,
                            interrupt_flags,
                        );
                        if self
                            .gpu_registers
                            .get_scanline(None, None, None, false, true)
                            == 154
                        {
                            self.gpu_registers.set_scanline(0, interrupt_flags);
                            self.gpu_registers
                                .set_ppu_mode(RenderingMode::OAMScan2, interrupt_flags);
                        }
                    }
                }
                RenderingMode::OAMScan2 => {
                    if self.rendering_info.dots_clock >= DOTS_IN_OAM_SCAN {
                        self.rendering_info.dots_clock -= DOTS_IN_OAM_SCAN;
                        self.gpu_registers
                            .set_ppu_mode(RenderingMode::Transfer3, interrupt_flags);
                    }
                }
                RenderingMode::Transfer3 => {
                    // TODO: Implement possible delay in this Mode if background scrolling or sprite fetching happened
                    if self.rendering_info.dots_clock >= DOTS_IN_TRANSFER {
                        self.rendering_info.dots_clock -= DOTS_IN_TRANSFER;
                        self.rendering_info.dots_for_transfer = DOTS_IN_TRANSFER;
                        self.gpu_registers
                            .set_ppu_mode(RenderingMode::HBlank0, interrupt_flags);
                    }
                }
            }
            RenderTask::None
        }
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
    ///
    /// Also sets flags in self.memory_changed, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn write_vram(&mut self, address: u16, value: u8) {
        let normalized_address = address - VRAM_BEGIN;
        self.vram[normalized_address as usize] = value;

        // If our index is greater than or equal to 0x1800, we are not writing to the tile set storage
        // so we can simply return
        if normalized_address >= 0x1800 {
            if address < 0x9C00 {
                // We are writing to tile map 0. Therefore, we set the changed flag to make sure
                // the GPU receives the new tile map later in rendering.
                self.memory_changed.tile_map_0_changed = true;
            } else {
                // We are writing to tile map 1. Therefore, we set the changed flag to make sure
                // the GPU receives the new tile map later in rendering.
                self.memory_changed.tile_map_1_changed = true;
            }
            return;
        } else {
            self.handle_tile_data_change(normalized_address);
        }
    }

    /// Reads from the OAM (Object Attribute Memory) at the given address. Valid addresses are 0xFE00 - 0xFE9F.
    pub fn read_oam(&self, address: u16) -> u8 {
        self.handle_oam_read(address)
    }

    /// Writes to the OAM (Object Attribute Memory) at the given address. Valid addresses are 0xFE00 - 0xFE9F.
    pub fn write_oam(&mut self, address: u16, value: u8) {
        self.handle_oam_write(address, value)
    }

    /// Returns a new GPU with empty tile set and empty VRAM.
    ///
    /// The lcd_was_turned_off flag is set to
    /// true, so the GPU starts off in HBlank mode instead of OAMScan, which is the supposed
    /// behavior after the LCD was turned on (for the first time or after being turned off).
    pub fn new_empty(debugging_flags: &DebugInfo) -> Self {
        let debugging_flags =
            DebuggingFlagsWithoutFileHandles::from_debugging_flags(debugging_flags);
        Self {
            vram: [0; VRAM_END as usize - VRAM_BEGIN as usize + 1],
            tile_set: [tile_handling::empty_tile(); 384],
            rendering_info: RenderingInfo {
                dots_clock: 0,
                total_dots: 0,
                dots_for_transfer: 0,
                lcd_was_turned_off: true,
                first_scanline_after_lcd_was_turned_on: false,
            },
            gpu_registers: GPURegisters::new(debugging_flags),
            background_viewport_changed: true,
            oam: [Object::default(); 40],

            memory_changed: ChangesToPropagateToShader {
                tile_data_flag_changed: true,
                tile_data_block_0_1_changed: true,
                tile_data_block_2_1_changed: true,
                background_tile_map_flag_changed: true,
                window_tile_map_flag_changed: true,
                tile_map_0_changed: true,
                tile_map_1_changed: true,
                background_viewport_position_changed: true,
            },

            debugging_flags,
        }
    }
}

impl RenderingMode {
    /// Returns the current rendering mode of the GPU as an u8. The conversions are as follows
    /// - HBlank: 0
    /// - VBlank: 1
    /// - OAMScan: 2
    /// - Transfer: 3
    pub fn as_u8(&self) -> u8 {
        match self {
            RenderingMode::HBlank0 => 0,
            RenderingMode::VBlank1 => 1,
            RenderingMode::OAMScan2 => 2,
            RenderingMode::Transfer3 => 3,
        }
    }

    /// Converts a u8 to a [RenderingMode]. The conversions are as follows
    /// - 0: HBlank
    /// - 1: VBlank
    /// - 2: OAMScan
    /// - 3: Transfer
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => RenderingMode::HBlank0,
            1 => RenderingMode::VBlank1,
            2 => RenderingMode::OAMScan2,
            3 => RenderingMode::Transfer3,
            _ => panic!("Invalid GPU mode: {}", value),
        }
    }
}
