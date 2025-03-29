use super::GPU;
use crate::MEMORY_SIZE;
use crate::cpu::is_bit_set;
use crate::frontend::shader::{
    BgAndWdViewportPosition, Palettes, RenderingLinePositionAndObjectSize,
};

/// Struct to keep track of the resources that are fetched during transfer (and OAMScan) mode which are then
/// sent to the shader.
pub struct BuffersForRendering {
    // Transfer mode buffers:
    pub(crate) background_tile_map: [u8; 1024],
    pub(crate) window_tile_map: [u8; 1024],
    pub(crate) bg_and_wd_tile_data: [u8; 4096],
    pub(crate) bg_and_wd_viewport_position: BgAndWdViewportPosition,
    pub(crate) palettes: Palettes,
    pub(crate) rendering_line_lcd_control_and_window_internal_line_info:
        RenderingLinePositionAndObjectSize,
    pub(crate) object_tile_data: [u8; 4096],
    // OAMScan mode buffer:
    pub(crate) objects_in_scanline_buffer: [[u32; 4]; 10],
}

impl BuffersForRendering {
    /// Returns a new BuffersForRendering with empty buffers.
    pub fn new_empty() -> Self {
        Self {
            background_tile_map: [0; 1024],
            window_tile_map: [0; 1024],
            bg_and_wd_tile_data: [0; 4096],
            bg_and_wd_viewport_position: BgAndWdViewportPosition { pos: [0; 4] },
            palettes: Palettes { values: [0; 4] },
            rendering_line_lcd_control_and_window_internal_line_info:
                RenderingLinePositionAndObjectSize { pos: [0; 4] },
            object_tile_data: [0; 4096],
            objects_in_scanline_buffer: [[0; 4]; 10],
        }
    }
}

impl GPU {
    /// Fetches the tile data, tilemap, viewport position, palettes and other data needed for the
    /// next scanline to be rendered using the scanline shader. This data is buffered because the original
    /// RustBoy fetches it in mode 3 (Transfer) and we only actually render it in mode 0 (HBlank).
    /// So, to avoid reading already changed data for rendering, we buffer the "old state".
    pub(super) fn fetch_rendering_information_to_rendering_buffer(
        &mut self,
        memory: &[u8; MEMORY_SIZE],
        current_scanline: u8,
    ) {
        self.buffers_for_rendering.background_tile_map = self.get_background_tile_map(memory);

        self.buffers_for_rendering.window_tile_map = self.get_window_tile_map(memory);

        self.buffers_for_rendering.bg_and_wd_tile_data =
            self.get_background_and_window_tile_data(memory);

        self.buffers_for_rendering.bg_and_wd_viewport_position = BgAndWdViewportPosition {
            pos: [
                self.gpu_registers.get_bg_scroll_x() as u32,
                self.gpu_registers.get_bg_scroll_y() as u32,
                self.gpu_registers.get_window_x_position() as u32,
                self.gpu_registers.get_window_y_position() as u32,
            ],
        };

        self.buffers_for_rendering.palettes = Palettes {
            values: [
                self.gpu_registers.get_background_palette() as u32,
                self.gpu_registers.get_object_palette_zero() as u32,
                self.gpu_registers.get_object_palette_one() as u32,
                0,
            ],
        };

        self.buffers_for_rendering.object_tile_data = self.get_object_tile_data(memory);

        self.buffers_for_rendering
            .rendering_line_lcd_control_and_window_internal_line_info =
            RenderingLinePositionAndObjectSize {
                pos: [
                    current_scanline as u32,
                    self.gpu_registers.get_lcd_control() as u32,
                    // We pass the info necessary for the window internal line counter
                    self.rendering_info.window_is_rendered_this_scanline as u32,
                    // By the documentation of the [window_internal_line_counter](super::RenderingInfo)
                    // field, its value is equal to the window line to be rendered plus 1, if
                    // the window is rendered this scanline.
                    if self.rendering_info.window_is_rendered_this_scanline {
                        self.rendering_info.window_internal_line_counter - 1
                    } else {
                        self.rendering_info.window_internal_line_counter
                    } as u32,
                ],
            };
        // DEBUG
        log::trace!(
            "Window rendered this scanline: {}, Current LCD control: {:<8b}, Current Scanline: {:<3}, Window position: {:<3}/{:<3}",
            self.rendering_info.window_is_rendered_this_scanline as u32,
            self.gpu_registers.get_lcd_control(),
            current_scanline,
            self.gpu_registers.get_window_x_position(),
            self.gpu_registers.get_window_y_position()
        );
    }

    /// Fetches the list of objects for the current scanline. This is needed for the
    /// next scanline to be rendered using the scanline shader. This is buffered because the original
    /// RustBoy fetches it in mode 2 (OAMScan) and we only actually render it in mode 0 (HBlank).
    /// So, to avoid reading already changed data for rendering, we buffer the "old state".
    pub(super) fn fetch_objects_in_scanline_to_rendering_buffer(&mut self, current_scanline: u8) {
        self.buffers_for_rendering.objects_in_scanline_buffer =
            self.get_objects_for_current_scanline(current_scanline);
    }
}

/// Struct to keep track of changes/writes to tile data, tilemap, viewport position, and OAM.
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
    pub(crate) window_viewport_position_changed: bool,
    pub(crate) palette_changed: bool,
}

impl ChangesToPropagateToShader {
    /// Returns a new instance of MemoryChanged with only false values
    pub(crate) fn new_false() -> Self {
        Self {
            tile_data_flag_changed: false,
            tile_data_block_0_1_changed: false,
            tile_data_block_2_1_changed: false,
            background_tile_map_flag_changed: false,
            window_tile_map_flag_changed: false,
            tile_map_0_changed: false,
            tile_map_1_changed: false,
            background_viewport_position_changed: false,
            window_viewport_position_changed: false,
            palette_changed: false,
        }
    }

    pub(crate) fn new_true() -> Self {
        Self {
            tile_data_flag_changed: true,
            tile_data_block_0_1_changed: true,
            tile_data_block_2_1_changed: true,
            background_tile_map_flag_changed: true,
            window_tile_map_flag_changed: true,
            tile_map_0_changed: true,
            tile_map_1_changed: true,
            background_viewport_position_changed: true,
            window_viewport_position_changed: true,
            palette_changed: true,
        }
    }
}
