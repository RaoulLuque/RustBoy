pub(crate) mod information_for_shader;
pub(crate) mod object_handling;
pub mod registers;
pub(crate) mod tile_handling;

use crate::cpu::is_bit_set;
use crate::debugging::{DebugInfo, DebuggingFlagsWithoutFileHandles};
use crate::gpu::registers::LCDCRegister;
use crate::interrupts::InterruptFlagRegister;
use crate::{MEMORY_SIZE, RustBoy};
use information_for_shader::{BuffersForRendering, ChangesToPropagateToShader};
use registers::GPURegisters;

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
/// and two tilemaps of 32 * 32 = 1024 bytes each.
///
/// The tile set is a 2D array of 8x8 tile pixel values which redundantly stores the tiles
/// which are already in vram. They are however more accessible than via the vram.
///
/// The tilemaps are two 2D arrays of 32x32 tile u8 indices which are used to determine which tile
/// to draw at which position on the screen. They are just stored directly in the vram field.
///
/// Also has a tile_data_changed flag to indicate if the tile data has changed.
pub struct GPU {
    pub(crate) rendering_info: RenderingInfo,
    pub(crate) buffers_for_rendering: BuffersForRendering,
    pub gpu_registers: GPURegisters,

    pub memory_changed: ChangesToPropagateToShader,
}

/// Struct to collect the information about the current rendering state of the GPU.
///
/// TODO: Add more detailed docstring
/// - `window_internal_line_counter`: Determines how many lines have been rendered where the window
/// was part of the line. Its value is incremented after transfer mode (3). That is, before it,
/// it indicates the next line that will be used from the window tilemap and after transfer mode (3)
/// it indicates both how many lines have been rendered already and what the next line used from
/// the window tilemap will be.
/// - `wy_condition_was_met_this_frame`: Indicates if the window y position (wy) was equal to the current
/// scanline at some point already throughout this frame.
/// - `window_is_rendered_this_scanline`: Indicates after exiting transfer mode (3), if the window is rendered
/// on the current scanline. Before exiting transfer mode, it indicates the state for the last scanline
pub struct RenderingInfo {
    // GPU rendering info
    pub(crate) dots_clock: u32,
    pub(crate) total_dots: u128,
    dots_for_transfer: u32,
    lcd_was_turned_off: bool,
    first_scanline_after_lcd_was_turned_on: bool,
    // Window rendering info
    window_internal_line_counter: u8,
    wy_condition_was_met_this_frame: bool,
    window_is_rendered_this_scanline: bool,
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
    /// Steps the GPU by the given number of dots.
    /// Returns a RenderTask indicating what the GPU should do next.
    /// For now, the GPU only renders the entire frame before entering VBlank.
    /// In the future, the GPU should render by lines.
    ///
    /// The GPU steps through four different [RenderingMode]s. When VBlank is entered, or rather,
    /// when HBlank is exited, the flag for a VBlank interrupt is set.
    pub fn gpu_step(
        &mut self,
        memory: &mut [u8; MEMORY_SIZE],
        interrupt_flags: &mut InterruptFlagRegister,
        dots: u32,
    ) -> RenderTask {
        // Always increment total dots (for debugging purposes)
        self.rendering_info.total_dots += dots as u128;

        if LCDCRegister::get_display_on_flag(memory) == false {
            if self.rendering_info.lcd_was_turned_off == false {
                // If the LCD is not enabled, there is no rendering task and we can reset the GPU
                // to its initial state. We only do this once when the LCD is turned off.
                self.rendering_info.dots_clock = 0;
                self.rendering_info.dots_for_transfer = 0;
                GPURegisters::set_ppu_mode(memory, GPU_MODE_WHILE_LCD_TURNED_OFF, interrupt_flags);
                GPURegisters::set_scanline(memory, 0, interrupt_flags);
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
                GPURegisters::set_ppu_mode(memory, RenderingMode::HBlank0, interrupt_flags);
                self.rendering_info.first_scanline_after_lcd_was_turned_on = true;
                self.rendering_info.lcd_was_turned_off = false;
            }
            self.rendering_info.dots_clock += dots;
            match GPURegisters::get_gpu_mode(memory) {
                RenderingMode::HBlank0 => {
                    if self.rendering_info.first_scanline_after_lcd_was_turned_on {
                        // If the LCD was turned off, it immediately enters HBlank mode which only
                        // lasts [DOTS_IN_OAM_SCAN] dots and then enters Transfer mode.
                        if self.rendering_info.dots_clock >= DOTS_IN_OAM_SCAN {
                            self.rendering_info.dots_clock -= DOTS_IN_OAM_SCAN;
                            GPURegisters::set_ppu_mode(
                                memory,
                                RenderingMode::Transfer3,
                                interrupt_flags,
                            );
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
                            GPURegisters::set_scanline(
                                memory,
                                GPURegisters::get_scanline_internal(memory) + 1,
                                interrupt_flags,
                            );
                            if GPURegisters::get_scanline_internal(memory) == 144 {
                                // We are entering VBlank, so we need to set the VBlank flag
                                // and set the GPU mode to VBlank. Also, we send a render frame request to
                                // the GPU, which renders the framebuffer to the screen.
                                GPURegisters::set_ppu_mode(
                                    memory,
                                    RenderingMode::VBlank1,
                                    interrupt_flags,
                                );
                                interrupt_flags.vblank = true;
                                return RenderTask::RenderFrame;
                            } else {
                                // We are still in HBlank, so we need to set the GPU mode to OAMScan2.
                                // Also we send a request to the GPU to write the current line to the
                                // framebuffer
                                // We need to return current scanline - 1, since we are already in the next
                                // scanline.
                                GPURegisters::set_ppu_mode(
                                    memory,
                                    RenderingMode::OAMScan2,
                                    interrupt_flags,
                                );
                                let next_scanline = GPURegisters::get_scanline_internal(memory);

                                // Since we are now entering OAMScan2, we want to check whether
                                // the WY condition is met
                                self.rendering_info.check_wy_condition(
                                    next_scanline,
                                    GPURegisters::get_window_y_position(memory),
                                );

                                return RenderTask::WriteLineToBuffer(next_scanline - 1);
                            }
                        }
                    }
                }
                RenderingMode::VBlank1 => {
                    if self.rendering_info.dots_clock >= DOTS_IN_VBLANK / 10 {
                        self.rendering_info.dots_clock -= DOTS_IN_VBLANK / 10;
                        GPURegisters::set_scanline(
                            memory,
                            GPURegisters::get_scanline_internal(memory) + 1,
                            interrupt_flags,
                        );
                        if GPURegisters::get_scanline_internal(memory) == 154 {
                            // On exiting VBlank, we update (reset) the window internal line counter
                            self.rendering_info.update_window_internal_line_counter(
                                memory,
                                // The current scanline is guaranteed to be 154 by the if condition
                                154,
                            );
                            // We also need to reset the wy_condition_was_triggered_this_frame and
                            // window_is_rendered_this_scanline flags for the next frame
                            self.rendering_info.wy_condition_was_met_this_frame = false;
                            self.rendering_info.window_is_rendered_this_scanline = false;

                            GPURegisters::set_scanline(memory, 0, interrupt_flags);

                            // Since we are now entering OAMScan2, we want to check whether
                            // the WY condition is met
                            self.rendering_info
                                .check_wy_condition(0, GPURegisters::get_window_y_position(memory));

                            GPURegisters::set_ppu_mode(
                                memory,
                                RenderingMode::OAMScan2,
                                interrupt_flags,
                            );
                        }
                    }
                }
                RenderingMode::OAMScan2 => {
                    if self.rendering_info.dots_clock >= DOTS_IN_OAM_SCAN {
                        self.rendering_info.dots_clock -= DOTS_IN_OAM_SCAN;
                        self.fetch_objects_in_scanline_to_rendering_buffer(
                            memory,
                            GPURegisters::get_scanline_internal(memory),
                        );

                        GPURegisters::set_ppu_mode(
                            memory,
                            RenderingMode::Transfer3,
                            interrupt_flags,
                        );
                    }
                }
                RenderingMode::Transfer3 => {
                    // TODO: Implement possible delay in this Mode if background scrolling or sprite fetching happened
                    if self.rendering_info.dots_clock >= DOTS_IN_TRANSFER {
                        self.rendering_info.dots_clock -= DOTS_IN_TRANSFER;
                        self.rendering_info.dots_for_transfer = DOTS_IN_TRANSFER;
                        let current_scanline = GPURegisters::get_scanline_internal(memory);
                        // On exiting Transfer mode, before buffering the information for
                        // the next scanline, we update the window internal line counter
                        self.rendering_info
                            .update_window_internal_line_counter(memory, current_scanline);
                        self.fetch_rendering_information_to_rendering_buffer(
                            memory,
                            current_scanline,
                        );

                        GPURegisters::set_ppu_mode(memory, RenderingMode::HBlank0, interrupt_flags);
                    }
                }
            }
            RenderTask::None
        }
    }

    /// Writes a byte to the VRAM at the given address.
    ///
    /// Also sets flags in self.memory_changed, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    ///
    /// TODO: Make this a non static method and pass in memory bus?
    pub fn write_vram(rust_boy: &mut RustBoy, address: u16, value: u8) {
        rust_boy.memory[address as usize] = value;

        // If our index is greater than or equal to 0x1800, we are not writing to the tile set storage
        // so we can simply return
        if address >= 0x9800 {
            if address < 0x9C00 {
                // We are writing to tilemap 0. Therefore, we set the changed flag to make sure
                // the GPU receives the new tilemap later in rendering.
                rust_boy.gpu.memory_changed.tile_map_0_changed = true;
            } else {
                // We are writing to tilemap 1. Therefore, we set the changed flag to make sure
                // the GPU receives the new tilemap later in rendering.
                rust_boy.gpu.memory_changed.tile_map_1_changed = true;
            }
            return;
        } else {
            GPU::handle_tile_data_change(rust_boy, address);
        }
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
            rendering_info: RenderingInfo::new_initial_state(),
            buffers_for_rendering: BuffersForRendering::new_empty(),
            gpu_registers: GPURegisters::new(debugging_flags),

            memory_changed: ChangesToPropagateToShader::new_true(),
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

impl RenderingInfo {
    /// Returns a new RenderingInfo instance with the initial state of the rendering information.
    /// That is, the dots clocks are set to 0, and the lcd was turned off flag is set to true.
    fn new_initial_state() -> Self {
        RenderingInfo {
            dots_clock: 0,
            total_dots: 0,
            dots_for_transfer: 0,
            lcd_was_turned_off: true,
            first_scanline_after_lcd_was_turned_on: false,
            window_internal_line_counter: 0,
            wy_condition_was_met_this_frame: false,
            window_is_rendered_this_scanline: false,
        }
    }

    /// Updates the window internal line counter.
    /// This is used to determine how many lines have been rendered where the window was part of the
    /// line.
    fn update_window_internal_line_counter(
        &mut self,
        memory: &[u8; MEMORY_SIZE],
        current_scanline: u8,
    ) {
        if current_scanline > 143 {
            // If the current scanline is greater than 143, we are in VBlank mode and the window
            // internal line counter is reset to 0.
            self.window_internal_line_counter = 0;
        } else {
            // We are about to exit Transfer mode and we need to check, if the window will be
            // rendered on the current scanline.
            if self.wy_condition_was_met_this_frame
                && GPURegisters::get_window_x_position(memory) < 167
                && is_bit_set(GPURegisters::get_lcd_control(memory), 5)
            {
                // The window will be rendered, if the wy condition was met this frame, the x position
                // of the window is not out of bounds, and the window flag in the lcd control register
                // is set
                self.window_is_rendered_this_scanline = true;
                self.window_internal_line_counter += 1;
            } else {
                // The window will not be rendered, which we need to keep track of to pass to the
                // shader
                self.window_is_rendered_this_scanline = false;
            }
        }
    }

    /// Checks if the window y position (wy) is equal to the current scanline.
    /// If so, we set the wy_condition_was_triggered_this_frame flag to true. Otherwise, we don't
    /// do anything.
    /// This is always checked when entering OAMScan (mode 2), see [Pan Docs](https://gbdev.io/pandocs/Scrolling.html#window)
    fn check_wy_condition(&mut self, current_scanline: u8, wy: u8) {
        // Check if the current scanline is equal to the y position of the window (wy)
        if current_scanline == wy {
            self.wy_condition_was_met_this_frame = true;
        }
    }
}
