use super::{
    DOTS_IN_HBLANK_PLUS_TRANSFER, DOTS_IN_VBLANK, GPU, GPU_MODE_WHILE_LCD_TURNED_OFF,
    RenderingInfo, RenderingMode,
};
use crate::cpu::{clear_bit, is_bit_set, set_bit};

use crate::debugging::DebuggingFlagsWithoutFileHandles;
use crate::gpu::information_for_shader::ChangesToPropagateToShader;
use crate::interrupts::InterruptFlagRegister;

const LCD_ENABLE_BIT_POSITION: usize = 7;
const WINDOW_TILE_MAP_BIT_POSITION: usize = 6;
const BG_AND_WINDOW_TILE_DATA_BIT_POSITION: usize = 4;
const BG_TILE_MAP_BIT_POSITION: usize = 3;
const OBJ_SIZE_BIT_POSITION: usize = 2;

const LYC_LY_COINCIDENCE_FLAG_BIT_POSITION: usize = 2;
const MODE_0_INT_SELECT_BIT_POSITION: usize = 3;
const MODE_1_INT_SELECT_BIT_POSITION: usize = 4;
const MODE_2_INT_SELECT_BIT_POSITION: usize = 5;
const LYC_INT_SELECT_BIT_POSITION: usize = 6;

/// Represents the registers that control the GPU.
/// The registers have the following address and function:
/// - 0xFF40: LCDC - LCD Control Register
/// - 0xFF41: STAT - LCD Status Register
/// - 0xFF42: SCY - Background Scroll Y Register
/// - 0xFF43: SCX - Background Scroll X Register
/// - 0xFF44: LY - Current Scanline Register
/// - 0xFF45: LYC - LY Compare Register TODO: Implement
/// - 0xFF47: BGP - Background Palette Data Register
/// - 0xFF48: OBP0 - Object Palette 0 Data Register
/// - 0xFF49: OBP1 - Object Palette 1 Data Register
/// - 0xFF4A: WY - Window Y Position Register
/// - 0xFF4B: WX - Window X Position Register
pub struct GPURegisters {
    pub(super) lcd_control: LCDCRegister,
    pub(super) lcd_status: LCDStatusRegister,
    bg_scroll_y: u8,
    bg_scroll_x: u8,
    current_scanline: u8,
    scanline_compare: u8,
    background_palette: u8,
    object_palette_zero: u8,
    object_palette_one: u8,
    wd_pos_y: u8,
    wd_pos_x: u8,
    pub(super) debugging_flags: DebuggingFlagsWithoutFileHandles,
}

/// Represents the LCDC register of the GPU.
/// The LCDC register is used to control the LCD.
/// It is an 8-bit register with the following bits:
/// - Bit 0: Background on/off (0 = off, 1 = on)
/// - Bit 1: Sprites on/off (0 = off, 1 = on)
/// - Bit 2: Sprite size (0 = 8x8, 1 = 8x16)
/// - Bit 3: Background tilemap (0 = #0 (0x9800), 1 = #1 (0x9C00))
/// - Bit 4: Background and window tile data (0 = #0 (0x8800), 1 = #1 (0x8000))
/// - Bit 5: Window on/off (0 = off, 1 = on) - gets overridden by bit 0 on DMG
/// - Bit 6: Window tilemap (0 = #0 (0x9800), 1 = #1 (0x9C00))
/// - Bit 7: Display off/on (0 = off, 1 = on)
pub struct LCDCRegister {
    register: u8,
}

/// Represents the LCD status register of the GPU.
/// The LCD status register is used to control the LCD status.
/// It is an 8-bit register with the following bits (see https://gbdev.io/pandocs/STAT.html#ff41--stat-lcd-status)
/// - Bit 0 and 1: PPU Mode (Rendering Mode of GPU)
/// - Bit 2: LYC=LY Coincidence Flag
/// - Bit 3: Mode 0 int select
/// - Bit 4: Mode 1 int select
/// - Bit 5: Mode 2 int select
/// - Bit 6: LYC int select
/// - Bit 7: None (Zero)
pub struct LCDStatusRegister {
    register: u8,
}

impl GPU {
    pub fn read_registers(&self, address: u16, cycles_current_instruction: u8) -> u8 {
        match address {
            0xFF40 => self.gpu_registers.get_lcd_control(),
            0xFF41 => self.gpu_registers.get_lcd_status(),
            0xFF42 => self.gpu_registers.get_bg_scroll_y(),
            0xFF43 => self.gpu_registers.get_bg_scroll_x(),
            0xFF44 => self.gpu_registers.get_scanline(
                Some(&self.rendering_info),
                Some(self.gpu_registers.lcd_status.get_gpu_mode()),
                Some(cycles_current_instruction),
                true,
                false,
            ),
            0xFF45 => self.gpu_registers.get_scanline_compare(),
            0xFF47 => self.gpu_registers.get_background_palette(),
            0xFF48 => self.gpu_registers.get_object_palette_zero(),
            0xFF49 => self.gpu_registers.get_object_palette_one(),
            _ => panic!(
                "Reading from invalid GPU register address: {:#04X}",
                address
            ),
        }
    }

    /// Writes to the GPU registers.
    ///
    /// Needs a reference to the interrupt flag register, if a write
    /// to the scanline register is made and LY=LYC is set, in which case a stat interrupt might
    /// be requested.
    pub fn write_registers(
        &mut self,
        address: u16,
        value: u8,
        interrupt_flag_register: &mut InterruptFlagRegister,
    ) {
        match address {
            0xFF40 => self
                .gpu_registers
                .set_lcd_control(value, &mut self.memory_changed),
            0xFF41 => self
                .gpu_registers
                .set_lcd_status(value, interrupt_flag_register),
            0xFF42 => self
                .gpu_registers
                .set_bg_scroll_y(value, &mut self.memory_changed),
            0xFF43 => self
                .gpu_registers
                .set_bg_scroll_x(value, &mut self.memory_changed),
            // If the rom tries writing to the scanline register, it gets reset to 0
            0xFF44 => self.gpu_registers.set_scanline(0, interrupt_flag_register),
            0xFF45 => self
                .gpu_registers
                .set_scanline_compare(value, interrupt_flag_register),
            0xFF47 => self
                .gpu_registers
                .set_background_palette(value, &mut self.memory_changed),
            0xFF48 => self
                .gpu_registers
                .set_object_palette_zero(value, &mut self.memory_changed),
            0xFF49 => self
                .gpu_registers
                .set_object_palette_one(value, &mut self.memory_changed),
            _ => panic!("Writing to invalid GPU register address: {:#04X}", address),
        }
    }
}

impl GPURegisters {
    /// Creates a new instance of the GPURegisters struct with all registers set to their default
    /// startup values.
    pub fn new(debugging_flags: DebuggingFlagsWithoutFileHandles) -> Self {
        Self {
            lcd_control: LCDCRegister { register: 0 },
            lcd_status: LCDStatusRegister { register: 0 },
            bg_scroll_x: 0,
            bg_scroll_y: 0,
            current_scanline: 0,
            scanline_compare: 0,
            background_palette: 0,
            object_palette_zero: 0,
            object_palette_one: 0,
            wd_pos_y: 0,
            wd_pos_x: 0,
            debugging_flags,
        }
    }

    /// Set the LCD Control register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_lcd_control(&mut self, value: u8, memory_changed: &mut ChangesToPropagateToShader) {
        let old_value = self.lcd_control.register;
        self.lcd_control.register = value;
        if self.lcd_control.get_display_on_flag() {
            log::debug!("LCD is turned on");
        } else {
            log::debug!("LCD is turned off");
        }

        // We need to check if the tile data area or background our window tilemap area changed
        // and set flags accordingly to make sure the GPU/Shader receives these changes in the
        // rendering step
        let distinct_bits = old_value ^ value;
        if is_bit_set(distinct_bits, WINDOW_TILE_MAP_BIT_POSITION as u8) {
            memory_changed.window_tile_map_flag_changed = true;
        }
        if is_bit_set(distinct_bits, BG_AND_WINDOW_TILE_DATA_BIT_POSITION as u8) {
            memory_changed.tile_data_flag_changed = true;
        }
        if is_bit_set(distinct_bits, BG_TILE_MAP_BIT_POSITION as u8) {
            memory_changed.background_tile_map_flag_changed = true;
        }
    }

    /// Set the LCD Status register to the provided value.
    ///
    /// Needs a reference to the interrupt flag register, if the LYC=LY Coincidence Flag is set,
    /// in which case a stat interrupt might be requested.
    pub fn set_lcd_status(
        &mut self,
        value: u8,
        interrupt_flag_register: &mut InterruptFlagRegister,
    ) {
        self.lcd_status = self.lcd_status.with_self_from_u8(&self, value);
        self.set_lyc_ly_coincidence_flag(
            self.current_scanline == self.scanline_compare,
            interrupt_flag_register,
        );
    }

    /// Set the Background Scroll Y register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_bg_scroll_y(&mut self, value: u8, memory_changed: &mut ChangesToPropagateToShader) {
        self.bg_scroll_y = value;
        memory_changed.background_viewport_position_changed = true;
    }

    /// Set the Background Scroll X register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_bg_scroll_x(&mut self, value: u8, memory_changed: &mut ChangesToPropagateToShader) {
        self.bg_scroll_x = value;
        memory_changed.background_viewport_position_changed = true;
    }

    /// Set the current scanline register to the provided value.
    ///
    /// Needs a reference to the interrupt flag register, if LY=LYC and a stat interrupt might be
    /// requested.
    pub(super) fn set_scanline(
        &mut self,
        value: u8,
        interrupt_flag_register: &mut InterruptFlagRegister,
    ) {
        self.current_scanline = value;
        self.set_lyc_ly_coincidence_flag(
            self.current_scanline == self.scanline_compare,
            interrupt_flag_register,
        );
    }

    /// Set the LY (Scanline) Compare register to the provided value.
    ///
    /// Needs a reference to the interrupt flag register to possibly request a stat interrupt, if
    /// LY=LYC and the LYC int select is set.
    fn set_scanline_compare(
        &mut self,
        value: u8,
        interrupt_flag_register: &mut InterruptFlagRegister,
    ) {
        self.scanline_compare = value;
        self.set_lyc_ly_coincidence_flag(
            self.current_scanline == self.scanline_compare,
            interrupt_flag_register,
        );
    }

    /// Set the LYC=LY Coincidence Flag to the provided value.
    ///
    /// Needs a reference to the interrupt flag register to possibly request a stat interrupt, if
    /// LY=LYC and the LYC int select is set.
    fn set_lyc_ly_coincidence_flag(
        &mut self,
        value: bool,
        interrupt_flag_register: &mut InterruptFlagRegister,
    ) {
        self.lcd_status.set_lyc_ly_coincidence_flag(value);
        if value {
            if self.lcd_status.get_lyc_int_select() {
                interrupt_flag_register.lcd_stat = true;
            }
        }
    }

    /// Set the GPU/PPU Mode to the provided value.
    ///
    /// Needs a reference to the interrupt flag register to possibly request a stat interrupt, if
    /// the corresponding mode int select flag is set to the provided mode which is being entered.
    pub(crate) fn set_ppu_mode(
        &mut self,
        mode: RenderingMode,
        interrupt_flag_register: &mut InterruptFlagRegister,
    ) {
        self.lcd_status.set_gpu_mode(mode);
        match mode {
            RenderingMode::HBlank0 => {
                if self.lcd_status.get_mode_0_int_select() {
                    interrupt_flag_register.lcd_stat = true;
                }
            }
            RenderingMode::VBlank1 => {
                if self.lcd_status.get_mode_1_int_select() {
                    interrupt_flag_register.lcd_stat = true;
                }
            }
            RenderingMode::OAMScan2 => {
                if self.lcd_status.get_mode_2_int_select() {
                    interrupt_flag_register.lcd_stat = true;
                }
            }
            RenderingMode::Transfer3 => {}
        }
    }

    /// Set the background palette register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_background_palette(
        &mut self,
        value: u8,
        memory_changed: &mut ChangesToPropagateToShader,
    ) {
        if self.background_palette != value {
            memory_changed.palette_changed = true;
        }
        self.background_palette = value;
    }

    /// Set the object palette 0 register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_object_palette_zero(
        &mut self,
        value: u8,
        memory_changed: &mut ChangesToPropagateToShader,
    ) {
        if self.object_palette_zero != value {
            memory_changed.palette_changed = true;
        }
        self.object_palette_zero = value;
    }

    /// Set the object palette 1 register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_object_palette_one(
        &mut self,
        value: u8,
        memory_changed: &mut ChangesToPropagateToShader,
    ) {
        if self.object_palette_one != value {
            memory_changed.palette_changed = true;
        }
        self.object_palette_one = value;
    }

    /// Set the window Y position register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_window_y_position(
        &mut self,
        value: u8,
        memory_changed: &mut ChangesToPropagateToShader,
    ) {
        self.wd_pos_y = value;
        memory_changed.window_viewport_position_changed = true;
    }

    /// Set the window X position register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_window_x_position(
        &mut self,
        value: u8,
        memory_changed: &mut ChangesToPropagateToShader,
    ) {
        self.wd_pos_x = value;
        memory_changed.window_viewport_position_changed = true;
    }

    /// Get the LCD Control register.
    pub fn get_lcd_control(&self) -> u8 {
        self.lcd_control.register
    }

    /// Get the LCD Status register.
    ///
    /// If the LCD is turned off, we return VBlank mode (0b01) as the current mode (lower two
    /// bits of the LCD status register), because the CPU might read this register before the
    /// GPU has a chance to update it.
    pub fn get_lcd_status(&self) -> u8 {
        let before_lcd_enable = self.lcd_status.register;
        if self.lcd_control.get_display_on_flag() {
            // If the LCD is turned off, we return VBlank mode (0b01) as the current mode (lower two
            // bits of the LCD status register)
            before_lcd_enable & GPU_MODE_WHILE_LCD_TURNED_OFF.as_u8()
        } else {
            before_lcd_enable
        }
    }

    /// Get the Background Scroll Y register.
    pub fn get_bg_scroll_y(&self) -> u8 {
        self.bg_scroll_y
    }

    /// Get the Background Scroll X register.
    pub fn get_bg_scroll_x(&self) -> u8 {
        self.bg_scroll_x
    }

    /// Get the current scanline register.
    ///
    /// This function has rendering info, the current rendering mode and the cycles of the current
    /// instruction as optional parameters. These are used to correctly determine the current scanline
    /// based on a quirk if called from the memory bus (that is, called by the CPU). If the GPU is
    /// in HBlank mode and about to increment the scanline, we want to return this already incremented
    /// scanline instead of the current one since in the real Game Boy they (GPU and CPU) would run in
    /// parallel.
    ///
    /// TODO: Update docstring
    pub fn get_scanline(
        &self,
        rendering_info: Option<&RenderingInfo>,
        current_rendering_mode: Option<RenderingMode>,
        cycles_current_instruction: Option<u8>,
        calling_from_memory_bus: bool,
        calling_from_gpu: bool,
    ) -> u8 {
        if self.debugging_flags.doctor && !calling_from_gpu {
            // Game Boy Doctor specifies that reading from the LY register (scanline) should always
            // return 0x90.
            0x90
        } else {
            // If we are calling from the memory bus, we are calling from the CPU, so we need to
            // work around a quirk with our implementation. The GPU and CPU run in parallel
            // in the real hardware, and in sequence (CPU step then GPU step) in our implementation.
            // Therefore, it can happen that right at the moment that CPU wants to read the current scanline (LY),
            // the GPU would actually increment the scanline. We account for this by checking if
            // the GPU would increment the scanline in the next step, and if so, we return the current
            // scanline + 1.
            if calling_from_memory_bus {
                if let Some(rendering_info) = rendering_info {
                    if let Some(current_rendering_mode) = current_rendering_mode {
                        if let Some(cycles_current_instruction) = cycles_current_instruction {
                            if current_rendering_mode == RenderingMode::HBlank0 {
                                if rendering_info.dots_clock + cycles_current_instruction as u32 * 4
                                    >= (DOTS_IN_HBLANK_PLUS_TRANSFER
                                        - rendering_info.dots_for_transfer)
                                {
                                    return self.current_scanline + 1;
                                }
                            } else if current_rendering_mode == RenderingMode::VBlank1 {
                                if rendering_info.dots_clock + cycles_current_instruction as u32 * 4
                                    >= DOTS_IN_VBLANK / 10
                                {
                                    return self.current_scanline + 1;
                                }
                            }
                        }
                    }
                }
            }
            self.current_scanline
        }
    }

    /// Get the LY (Scanline) Compare register.
    pub fn get_scanline_compare(&self) -> u8 {
        self.scanline_compare
    }

    /// Get the background palette register.
    pub fn get_background_palette(&self) -> u8 {
        self.background_palette
    }

    /// Get the object palette 0 register.
    pub fn get_object_palette_zero(&self) -> u8 {
        self.object_palette_zero
    }

    /// Get the object palette 1 register.
    pub fn get_object_palette_one(&self) -> u8 {
        self.object_palette_one
    }

    /// Get the window Y position register.
    pub fn get_window_y_position(&self) -> u8 {
        self.wd_pos_y
    }

    /// Get the window X position register.
    pub fn get_window_x_position(&self) -> u8 {
        self.wd_pos_x
    }

    /// Get the GPU Mode
    pub fn get_gpu_mode(&self) -> RenderingMode {
        self.lcd_status.get_gpu_mode()
    }

    /// Get the state of the sprite/object size flag (sprite size)
    pub fn get_sprite_size_flag(&self) -> bool {
        self.lcd_control.get_sprite_size_flag()
    }
}

impl LCDCRegister {
    /// Returns the state of the sprite size flag.
    pub fn get_sprite_size_flag(&self) -> bool {
        is_bit_set(self.register, OBJ_SIZE_BIT_POSITION as u8)
    }

    /// Returns the state of the background tilemap flag.
    pub fn get_background_tile_map_flag(&self) -> bool {
        is_bit_set(self.register, BG_TILE_MAP_BIT_POSITION as u8)
    }

    /// Returns the state of the background and window tile data flag.
    pub fn get_background_and_window_tile_data_flag(&self) -> bool {
        is_bit_set(self.register, BG_AND_WINDOW_TILE_DATA_BIT_POSITION as u8)
    }

    /// Returns the state of the window tilemap flag.
    pub fn get_window_tile_map_flag(&self) -> bool {
        is_bit_set(self.register, WINDOW_TILE_MAP_BIT_POSITION as u8)
    }

    /// Returns the state of the lcd/display enable flag.
    pub fn get_display_on_flag(&self) -> bool {
        is_bit_set(self.register, LCD_ENABLE_BIT_POSITION as u8)
    }
}

impl LCDStatusRegister {
    /// Returns the GPU mode as a [super::RenderingMode] enum.
    fn get_gpu_mode(&self) -> RenderingMode {
        RenderingMode::from_u8(self.register & 0b0000_0011)
    }

    /// Returns the state of the mode 0 interrupt select flag.
    fn get_mode_0_int_select(&self) -> bool {
        is_bit_set(self.register, MODE_0_INT_SELECT_BIT_POSITION as u8)
    }

    /// Returns the state of the mode 1 interrupt select flag.
    fn get_mode_1_int_select(&self) -> bool {
        is_bit_set(self.register, MODE_1_INT_SELECT_BIT_POSITION as u8)
    }

    /// Returns the state of the mode 2 interrupt select flag.
    fn get_mode_2_int_select(&self) -> bool {
        is_bit_set(self.register, MODE_2_INT_SELECT_BIT_POSITION as u8)
    }

    /// Returns the state of the LYC interrupt select flag.
    fn get_lyc_int_select(&self) -> bool {
        is_bit_set(self.register, LYC_INT_SELECT_BIT_POSITION as u8)
    }

    /// Sets the gpu/ppu mode to the provided value.
    fn set_gpu_mode(&mut self, mode: RenderingMode) {
        self.register = (self.register & 0b1111_1100) | mode.as_u8();
    }

    /// Sets the LYC = LY Coincidence Flag to the provided value.
    fn set_lyc_ly_coincidence_flag(&mut self, value: bool) {
        self.register = if value {
            set_bit(self.register, LYC_LY_COINCIDENCE_FLAG_BIT_POSITION as u8)
        } else {
            clear_bit(self.register, LYC_LY_COINCIDENCE_FLAG_BIT_POSITION as u8)
        };
    }

    /// Returns a new instance of the LCDStatusRegister struct with the fields set according to
    /// the provided value except for PPU Mode and LYC=LY Coincidence Flag. So only the bits
    /// 3 to 6 are set according to the provided value.
    /// Needs a reference to the GPURegisters to get the value of the LYC=LY Coincidence Flag.
    fn with_self_from_u8(&self, gpu_registers: &GPURegisters, value: u8) -> Self {
        let mut register = value & 0b0111_1000;
        if gpu_registers.scanline_compare == gpu_registers.current_scanline {
            register |= 1 << LYC_LY_COINCIDENCE_FLAG_BIT_POSITION;
        }
        register |= self.register & 0b11;
        LCDStatusRegister { register }
    }
}
