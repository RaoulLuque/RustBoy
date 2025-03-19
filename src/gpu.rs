mod object_handling;
mod registers;
pub(crate) mod tile_handling;

use crate::memory_bus::{VRAM_BEGIN, VRAM_END};

use crate::debugging::DebuggingFlags;
use crate::interrupts::InterruptFlagRegister;
use object_handling::Object;
use registers::GPURegisters;
use tile_handling::Tile;

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
    pub(crate) rendering_info: RenderingInfo,
    pub gpu_registers: GPURegisters,
    tile_data_changed: bool,
    tile_map_changed: bool,
    background_viewport_changed: bool,

    pub(crate) oam: [Object; 40],

    debugging_flags: DebuggingFlags,
}

/// Struct to collect the information about the current rendering state of the GPU.
pub struct RenderingInfo {
    pub(crate) dots_clock: u32,
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

impl GPU {
    /// Steps the GPU by the given number of cycles.
    /// Returns a RenderTask indicating what the GPU should do next.
    /// For now, the GPU only renders the entire frame before entering VBlank.
    /// In the future, the GPU should render by lines.
    ///
    /// The GPU steps through four different [RenderingMode]s. When VBlank is entered, or rather,
    /// when HBlank is exited, the flag for a VBlank interrupt is set.
    pub fn gpu_step(
        &mut self,
        interrupt_flags: &mut InterruptFlagRegister,
        cycles: u32,
    ) -> RenderTask {
        if self.gpu_registers.lcd_control.display_on_off == false {
            // If the LCD is not enabled, there is no rendering task and we can reset the GPU
            self.rendering_info.dots_clock = 0;
            self.rendering_info.dots_for_transfer = 0;
            self.gpu_registers
                .set_ppu_mode(RenderingMode::VBlank1, interrupt_flags);
            self.gpu_registers.set_scanline(0, interrupt_flags);
            RenderTask::None
        } else {
            self.rendering_info.dots_clock += cycles;
            match self.gpu_registers.get_gpu_mode() {
                RenderingMode::HBlank0 => {
                    if self.rendering_info.dots_clock >= 456 - self.rendering_info.dots_for_transfer
                    {
                        self.rendering_info.dots_clock -=
                            456 - self.rendering_info.dots_for_transfer;
                        self.gpu_registers
                            .set_scanline(self.gpu_registers.get_scanline() + 1, interrupt_flags);
                        if self.gpu_registers.get_scanline() == 144 {
                            // We are entering VBlank, so we need to set the VBlank flag
                            // and set the GPU mode to VBlank. Also, we send a render frame request to
                            // the GPU, which renders the framebuffer to the screen.
                            self.gpu_registers.lcd_status.gpu_mode = RenderingMode::VBlank1;
                            self.gpu_registers
                                .set_ppu_mode(RenderingMode::VBlank1, interrupt_flags);
                            interrupt_flags.vblank = true;
                            return RenderTask::RenderFrame;
                        } else {
                            // We are still in HBlank, so we need to set the GPU mode to OAMScan2.
                            // Also we send a request to the GPU to write the current line to the
                            // framebuffer
                            self.gpu_registers.lcd_status.gpu_mode = RenderingMode::OAMScan2;
                            self.gpu_registers
                                .set_ppu_mode(RenderingMode::OAMScan2, interrupt_flags);
                            return RenderTask::WriteLineToBuffer(
                                self.gpu_registers.get_scanline() - 1,
                            );
                        }
                    }
                }
                RenderingMode::VBlank1 => {
                    if self.rendering_info.dots_clock >= 456 {
                        self.rendering_info.dots_clock -= 456;
                        self.gpu_registers
                            .set_scanline(self.gpu_registers.get_scanline() + 1, interrupt_flags);
                        if self.gpu_registers.get_scanline() == 154 {
                            self.gpu_registers.set_scanline(0, interrupt_flags);
                            self.gpu_registers.lcd_status.gpu_mode = RenderingMode::OAMScan2;
                            self.gpu_registers
                                .set_ppu_mode(RenderingMode::OAMScan2, interrupt_flags);
                        }
                    }
                }
                RenderingMode::OAMScan2 => {
                    if self.rendering_info.dots_clock >= 80 {
                        self.rendering_info.dots_clock -= 80;
                        self.gpu_registers.lcd_status.gpu_mode = RenderingMode::Transfer3;
                        self.gpu_registers
                            .set_ppu_mode(RenderingMode::Transfer3, interrupt_flags);
                    }
                }
                RenderingMode::Transfer3 => {
                    // TODO: Implement possible delay in this Mode if background scrolling or sprite fetching happened
                    if self.rendering_info.dots_clock >= 172 {
                        self.rendering_info.dots_clock -= 172;
                        self.rendering_info.dots_for_transfer = 172;
                        self.gpu_registers.lcd_status.gpu_mode = RenderingMode::HBlank0;
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

    /// Reads from the OAM (Object Attribute Memory) at the given address. Valid addresses are 0xFE00 - 0xFE9F.
    pub fn read_oam(&self, address: u16) -> u8 {
        self.handle_oam_read(address)
    }

    /// Writes to the OAM (Object Attribute Memory) at the given address. Valid addresses are 0xFE00 - 0xFE9F.
    pub fn write_oam(&mut self, address: u16, value: u8) {
        self.handle_oam_write(address, value)
    }

    /// Returns a new GPU with empty tile set and empty VRAM.
    pub fn new_empty(debugging_flags: DebuggingFlags) -> Self {
        Self {
            vram: [0; VRAM_END as usize - VRAM_BEGIN as usize + 1],
            tile_set: [tile_handling::empty_tile(); 384],
            rendering_info: RenderingInfo {
                dots_clock: 0,
                dots_for_transfer: 0,
            },
            gpu_registers: GPURegisters::new(debugging_flags),
            tile_data_changed: true,
            tile_map_changed: true,
            background_viewport_changed: true,
            oam: [Object::default(); 40],

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
}
