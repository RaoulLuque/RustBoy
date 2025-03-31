use super::{
    DOTS_IN_HBLANK_PLUS_TRANSFER, DOTS_IN_VBLANK, GPU, GPU_MODE_WHILE_LCD_TURNED_OFF,
    RenderingInfo, RenderingMode,
};
use crate::cpu::{clear_bit, is_bit_set, set_bit};

use crate::MEMORY_SIZE;
use crate::debugging::DebuggingFlagsWithoutFileHandles;
use crate::gpu::information_for_shader::ChangesToPropagateToShader;
use crate::interrupts::{Interrupt, InterruptFlagRegister};

// Addresses of the GPU registers
const LCDC_REGISTER_ADDRESS: usize = 0xFF40;
const LCD_STATUS_REGISTER_ADDRESS: usize = 0xFF41;
const BG_SCROLL_Y_REGISTER_ADDRESS: usize = 0xFF42;
const BG_SCROLL_X_REGISTER_ADDRESS: usize = 0xFF43;
const SCANLINE_REGISTER_ADDRESS: usize = 0xFF44;
const SCANLINE_COMPARE_REGISTER_ADDRESS: usize = 0xFF45;
const BACKGROUND_PALETTE_REGISTER_ADDRESS: usize = 0xFF47;
const OBJECT_PALETTE_ZERO_REGISTER_ADDRESS: usize = 0xFF48;
const OBJECT_PALETTE_ONE_REGISTER_ADDRESS: usize = 0xFF49;
const WINDOW_Y_POSITION_REGISTER_ADDRESS: usize = 0xFF4A;
const WINDOW_X_POSITION_REGISTER_ADDRESS: usize = 0xFF4B;

// Positions of the bits in the LCD Control register
const LCD_ENABLE_BIT_POSITION: usize = 7;
const WINDOW_TILE_MAP_BIT_POSITION: usize = 6;
const BG_AND_WINDOW_TILE_DATA_BIT_POSITION: usize = 4;
const BG_TILE_MAP_BIT_POSITION: usize = 3;
const OBJ_SIZE_BIT_POSITION: usize = 2;

// Positions of the bits in the LCD Status register
const LYC_LY_COINCIDENCE_FLAG_BIT_POSITION: usize = 2;
const MODE_0_INT_SELECT_BIT_POSITION: usize = 3;
const MODE_1_INT_SELECT_BIT_POSITION: usize = 4;
const MODE_2_INT_SELECT_BIT_POSITION: usize = 5;
const LYC_INT_SELECT_BIT_POSITION: usize = 6;

/// Represents the registers that control the GPU.
///
/// This struct is empty and has no fields. Instead it is just used to group the GPU registers and make
/// the interface nicer. The actual data of the registers is held in the [MemoryBus](crate::MemoryBus).
///
/// TODO: Explain static function setup
///
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
    pub(super) debugging_flags: DebuggingFlagsWithoutFileHandles,
}

/// Represents the LCDC register of the GPU. This struct is empty and has no fields. Instead, it is
/// just used to make the interface nicer and the actual register is held in the [MemoryBus](crate::MemoryBus).
///
/// TODO: Explain static function setup
///
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
pub struct LCDCRegister {}

/// Represents the LCD status register of the GPU. This struct is empty and has no fields. Instead, it is
/// just used to make the interface nicer and the actual register is held in the [MemoryBus](crate::MemoryBus).
///
/// TODO: Explain static function setup
///
/// The LCD status register is used to control the LCD status.
/// It is an 8-bit register with the following bits (see https://gbdev.io/pandocs/STAT.html#ff41--stat-lcd-status)
/// - Bit 0 and 1: PPU Mode (Rendering Mode of GPU)
/// - Bit 2: LYC=LY Coincidence Flag
/// - Bit 3: Mode 0 int select
/// - Bit 4: Mode 1 int select
/// - Bit 5: Mode 2 int select
/// - Bit 6: LYC int select
/// - Bit 7: None (Zero)
pub struct LCDStatusRegister {}

impl GPU {
    pub fn read_registers(
        &self,
        memory: &[u8; MEMORY_SIZE],
        address: u16,
        cycles_current_instruction: u8,
    ) -> u8 {
        match address {
            0xFF40 => GPURegisters::get_lcd_control(memory),
            0xFF41 => GPURegisters::get_lcd_status(memory),
            0xFF42 => GPURegisters::get_bg_scroll_y(memory),
            0xFF43 => GPURegisters::get_bg_scroll_x(memory),
            0xFF44 => self.gpu_registers.get_scanline(
                memory,
                Some(&self.rendering_info),
                Some(GPURegisters::get_gpu_mode(memory)),
                Some(cycles_current_instruction),
                true,
            ),
            0xFF45 => GPURegisters::get_scanline_compare(memory),
            0xFF47 => GPURegisters::get_background_palette(memory),
            0xFF48 => GPURegisters::get_object_palette_zero(memory),
            0xFF49 => GPURegisters::get_object_palette_one(memory),
            0xFF4A => GPURegisters::get_window_y_position(memory),
            0xFF4B => GPURegisters::get_window_x_position(memory),
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
    pub fn write_registers(&mut self, memory: &mut [u8; MEMORY_SIZE], address: u16, value: u8) {
        match address {
            0xFF40 => GPURegisters::set_lcd_control(memory, value, &mut self.memory_changed),
            0xFF41 => GPURegisters::set_lcd_status(memory, value),
            0xFF42 => GPURegisters::set_bg_scroll_y(memory, value, &mut self.memory_changed),
            0xFF43 => GPURegisters::set_bg_scroll_x(memory, value, &mut self.memory_changed),
            // If the rom tries writing to the scanline register, it gets reset to 0
            0xFF44 => GPURegisters::set_scanline(memory, 0),
            0xFF45 => GPURegisters::set_scanline_compare(memory, value),
            0xFF47 => GPURegisters::set_background_palette(memory, value, &mut self.memory_changed),
            0xFF48 => {
                GPURegisters::set_object_palette_zero(memory, value, &mut self.memory_changed)
            }
            0xFF49 => GPURegisters::set_object_palette_one(memory, value, &mut self.memory_changed),
            0xFF4A => GPURegisters::set_window_y_position(memory, value, &mut self.memory_changed),
            0xFF4B => GPURegisters::set_window_x_position(memory, value, &mut self.memory_changed),
            _ => panic!("Writing to invalid GPU register address: {:#04X}", address),
        }
    }
}

impl GPURegisters {
    /// Creates a new instance of the GPURegisters struct with all registers set to their default
    /// startup values.
    pub fn new(debugging_flags: DebuggingFlagsWithoutFileHandles) -> Self {
        Self { debugging_flags }
    }

    /// Set the LCD Control register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_lcd_control(
        memory: &mut [u8; MEMORY_SIZE],
        value: u8,
        memory_changed: &mut ChangesToPropagateToShader,
    ) {
        let old_value = GPURegisters::get_lcd_control(memory);
        memory[LCDC_REGISTER_ADDRESS] = value;
        let distinct_bits = old_value ^ value;
        if is_bit_set(distinct_bits, LCD_ENABLE_BIT_POSITION as u8) {
            if LCDCRegister::get_display_on_flag(memory) {
                log::debug!("LCD is turned on");
            } else {
                log::debug!("LCD is turned off");
            }
        }

        // We need to check if the tile data area or background our window tilemap area changed
        // and set flags accordingly to make sure the GPU/Shader receives these changes in the
        // rendering step
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
    pub fn set_lcd_status(memory: &mut [u8; MEMORY_SIZE], value: u8) {
        memory[LCD_STATUS_REGISTER_ADDRESS] = LCDStatusRegister::with_self_from_u8(memory, value);
        LCDStatusRegister::set_lyc_ly_coincidence_flag(
            memory,
            GPURegisters::get_scanline_internal(memory)
                == GPURegisters::get_scanline_compare(memory),
        );
    }

    /// Set the Background Scroll Y register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_bg_scroll_y(
        memory: &mut [u8; MEMORY_SIZE],
        value: u8,
        memory_changed: &mut ChangesToPropagateToShader,
    ) {
        memory[BG_SCROLL_Y_REGISTER_ADDRESS] = value;
        memory_changed.background_viewport_position_changed = true;
    }

    /// Set the Background Scroll X register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_bg_scroll_x(
        memory: &mut [u8; MEMORY_SIZE],
        value: u8,
        memory_changed: &mut ChangesToPropagateToShader,
    ) {
        memory[BG_SCROLL_X_REGISTER_ADDRESS] = value;
        memory_changed.background_viewport_position_changed = true;
    }

    /// Set the current scanline register to the provided value.
    ///
    /// Needs a reference to the interrupt flag register, if LY=LYC and a stat interrupt might be
    /// requested.
    pub(super) fn set_scanline(memory: &mut [u8; MEMORY_SIZE], value: u8) {
        memory[SCANLINE_REGISTER_ADDRESS] = value;
        LCDStatusRegister::set_lyc_ly_coincidence_flag(
            memory,
            GPURegisters::get_scanline_internal(memory)
                == GPURegisters::get_scanline_compare(memory),
        );
    }

    /// Set the LY (Scanline) Compare register to the provided value.
    ///
    /// Needs a reference to the interrupt flag register to possibly request a stat interrupt, if
    /// LY=LYC and the LYC int select is set.
    fn set_scanline_compare(memory: &mut [u8; MEMORY_SIZE], value: u8) {
        memory[SCANLINE_COMPARE_REGISTER_ADDRESS] = value;
        LCDStatusRegister::set_lyc_ly_coincidence_flag(
            memory,
            GPURegisters::get_scanline_internal(memory)
                == GPURegisters::get_scanline_compare(memory),
        );
    }

    /// Set the GPU/PPU Mode to the provided value.
    ///
    /// Needs a reference to the interrupt flag register to possibly request a stat interrupt, if
    /// the corresponding mode int select flag is set to the provided mode which is being entered.
    pub(crate) fn set_ppu_mode(memory: &mut [u8; MEMORY_SIZE], mode: RenderingMode) {
        LCDStatusRegister::set_gpu_mode(memory, mode);
        match mode {
            RenderingMode::HBlank0 => {
                if LCDStatusRegister::get_mode_0_int_select(memory) {
                    InterruptFlagRegister::set_flag(memory, Interrupt::LcdStat, true);
                }
            }
            RenderingMode::VBlank1 => {
                if LCDStatusRegister::get_mode_1_int_select(memory) {
                    InterruptFlagRegister::set_flag(memory, Interrupt::LcdStat, true);
                }
            }
            RenderingMode::OAMScan2 => {
                if LCDStatusRegister::get_mode_2_int_select(memory) {
                    InterruptFlagRegister::set_flag(memory, Interrupt::LcdStat, true);
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
        memory: &mut [u8; MEMORY_SIZE],
        value: u8,
        memory_changed: &mut ChangesToPropagateToShader,
    ) {
        if GPURegisters::get_background_palette(memory) != value {
            memory_changed.palette_changed = true;
            memory[BACKGROUND_PALETTE_REGISTER_ADDRESS] = value;
        }
    }

    /// Set the object palette 0 register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_object_palette_zero(
        memory: &mut [u8; MEMORY_SIZE],
        value: u8,
        memory_changed: &mut ChangesToPropagateToShader,
    ) {
        if GPURegisters::get_object_palette_zero(memory) != value {
            memory_changed.palette_changed = true;
            memory[OBJECT_PALETTE_ZERO_REGISTER_ADDRESS] = value;
        }
    }

    /// Set the object palette 1 register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_object_palette_one(
        memory: &mut [u8; MEMORY_SIZE],
        value: u8,
        memory_changed: &mut ChangesToPropagateToShader,
    ) {
        if GPURegisters::get_object_palette_one(memory) != value {
            memory_changed.palette_changed = true;
            memory[OBJECT_PALETTE_ONE_REGISTER_ADDRESS] = value;
        }
    }

    /// Set the window Y position register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_window_y_position(
        memory: &mut [u8; MEMORY_SIZE],
        value: u8,
        memory_changed: &mut ChangesToPropagateToShader,
    ) {
        if GPURegisters::get_window_y_position(memory) != value {
            memory_changed.window_viewport_position_changed = true;
            memory[WINDOW_Y_POSITION_REGISTER_ADDRESS] = value;
        }
    }

    /// Set the window X position register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_window_x_position(
        memory: &mut [u8; MEMORY_SIZE],
        value: u8,
        memory_changed: &mut ChangesToPropagateToShader,
    ) {
        if GPURegisters::get_window_x_position(memory) != value {
            memory_changed.window_viewport_position_changed = true;
            memory[WINDOW_X_POSITION_REGISTER_ADDRESS] = value;
        }
    }

    /// Get the LCD Control register.
    pub fn get_lcd_control(memory: &[u8; MEMORY_SIZE]) -> u8 {
        memory[LCDC_REGISTER_ADDRESS]
    }

    /// Get the LCD Status register.
    ///
    /// If the LCD is turned off, we return VBlank mode (0b01) as the current mode (lower two
    /// bits of the LCD status register), because the CPU might read this register before the
    /// GPU has a chance to update it.
    pub fn get_lcd_status(memory: &[u8; MEMORY_SIZE]) -> u8 {
        let before_lcd_enable = memory[LCD_STATUS_REGISTER_ADDRESS];
        if LCDCRegister::get_display_on_flag(memory) {
            // If the LCD is turned off, we return VBlank mode (0b01) as the current mode (lower two
            // bits of the LCD status register)
            before_lcd_enable & (0b1111_1100 | GPU_MODE_WHILE_LCD_TURNED_OFF.as_u8())
        } else {
            before_lcd_enable
        }
    }

    /// Get the Background Scroll Y register.
    pub fn get_bg_scroll_y(memory: &[u8; MEMORY_SIZE]) -> u8 {
        memory[BG_SCROLL_Y_REGISTER_ADDRESS]
    }

    /// Get the Background Scroll X register.
    pub fn get_bg_scroll_x(memory: &[u8; MEMORY_SIZE]) -> u8 {
        memory[BG_SCROLL_X_REGISTER_ADDRESS]
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
    /// TODO: Change calling from parameters to an ENUM
    pub fn get_scanline(
        &self,
        memory: &[u8; MEMORY_SIZE],
        rendering_info: Option<&RenderingInfo>,
        current_rendering_mode: Option<RenderingMode>,
        cycles_current_instruction: Option<u8>,
        calling_from_memory_bus: bool,
    ) -> u8 {
        if self.debugging_flags.doctor {
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
                                    return GPURegisters::get_scanline_internal(memory) + 1;
                                }
                            } else if current_rendering_mode == RenderingMode::VBlank1 {
                                if rendering_info.dots_clock + cycles_current_instruction as u32 * 4
                                    >= DOTS_IN_VBLANK / 10
                                {
                                    return GPURegisters::get_scanline_internal(memory) + 1;
                                }
                            }
                        }
                    }
                }
            }
            GPURegisters::get_scanline_internal(memory)
        }
    }

    pub(crate) fn get_scanline_internal(memory: &[u8; MEMORY_SIZE]) -> u8 {
        memory[SCANLINE_REGISTER_ADDRESS]
    }

    /// Get the LY (Scanline) Compare register.
    pub fn get_scanline_compare(memory: &[u8; MEMORY_SIZE]) -> u8 {
        memory[SCANLINE_COMPARE_REGISTER_ADDRESS]
    }

    /// Get the background palette register.
    pub fn get_background_palette(memory: &[u8; MEMORY_SIZE]) -> u8 {
        memory[BACKGROUND_PALETTE_REGISTER_ADDRESS]
    }

    /// Get the object palette 0 register.
    pub fn get_object_palette_zero(memory: &[u8; MEMORY_SIZE]) -> u8 {
        memory[OBJECT_PALETTE_ZERO_REGISTER_ADDRESS]
    }

    /// Get the object palette 1 register.
    pub fn get_object_palette_one(memory: &[u8; MEMORY_SIZE]) -> u8 {
        memory[OBJECT_PALETTE_ONE_REGISTER_ADDRESS]
    }

    /// Get the window Y position register.
    pub fn get_window_y_position(memory: &[u8; MEMORY_SIZE]) -> u8 {
        memory[WINDOW_Y_POSITION_REGISTER_ADDRESS]
    }

    /// Get the window X position register.
    pub fn get_window_x_position(memory: &[u8; MEMORY_SIZE]) -> u8 {
        memory[WINDOW_X_POSITION_REGISTER_ADDRESS]
    }

    /// Get the GPU Mode
    pub fn get_gpu_mode(memory: &[u8; MEMORY_SIZE]) -> RenderingMode {
        LCDStatusRegister::get_gpu_mode(memory)
    }
}

impl LCDCRegister {
    /// Returns the state of the sprite size flag.
    pub fn get_sprite_size_flag(memory: &[u8; MEMORY_SIZE]) -> bool {
        is_bit_set(memory[LCDC_REGISTER_ADDRESS], OBJ_SIZE_BIT_POSITION as u8)
    }

    /// Returns the state of the background tilemap flag.
    pub fn get_background_tile_map_flag(memory: &[u8; MEMORY_SIZE]) -> bool {
        is_bit_set(
            memory[LCDC_REGISTER_ADDRESS],
            BG_TILE_MAP_BIT_POSITION as u8,
        )
    }

    /// Returns the state of the background and window tile data flag.
    pub fn get_background_and_window_tile_data_flag(memory: &[u8; MEMORY_SIZE]) -> bool {
        is_bit_set(
            memory[LCDC_REGISTER_ADDRESS],
            BG_AND_WINDOW_TILE_DATA_BIT_POSITION as u8,
        )
    }

    /// Returns the state of the window tilemap flag.
    pub fn get_window_tile_map_flag(memory: &[u8; MEMORY_SIZE]) -> bool {
        is_bit_set(
            memory[LCDC_REGISTER_ADDRESS],
            WINDOW_TILE_MAP_BIT_POSITION as u8,
        )
    }

    /// Returns the state of the lcd/display enable flag.
    pub fn get_display_on_flag(memory: &[u8; MEMORY_SIZE]) -> bool {
        is_bit_set(memory[LCDC_REGISTER_ADDRESS], LCD_ENABLE_BIT_POSITION as u8)
    }
}

impl LCDStatusRegister {
    /// Returns the GPU mode as a [super::RenderingMode] enum.
    fn get_gpu_mode(memory: &[u8; MEMORY_SIZE]) -> RenderingMode {
        RenderingMode::from_u8(memory[LCD_STATUS_REGISTER_ADDRESS] & 0b0000_0011)
    }

    /// Returns the state of the mode 0 interrupt select flag.
    fn get_mode_0_int_select(memory: &[u8; MEMORY_SIZE]) -> bool {
        is_bit_set(
            memory[LCD_STATUS_REGISTER_ADDRESS],
            MODE_0_INT_SELECT_BIT_POSITION as u8,
        )
    }

    /// Returns the state of the mode 1 interrupt select flag.
    fn get_mode_1_int_select(memory: &[u8; MEMORY_SIZE]) -> bool {
        is_bit_set(
            memory[LCD_STATUS_REGISTER_ADDRESS],
            MODE_1_INT_SELECT_BIT_POSITION as u8,
        )
    }

    /// Returns the state of the mode 2 interrupt select flag.
    fn get_mode_2_int_select(memory: &[u8; MEMORY_SIZE]) -> bool {
        is_bit_set(
            memory[LCD_STATUS_REGISTER_ADDRESS],
            MODE_2_INT_SELECT_BIT_POSITION as u8,
        )
    }

    /// Returns the state of the LYC interrupt select flag.
    fn get_lyc_int_select(memory: &[u8; MEMORY_SIZE]) -> bool {
        is_bit_set(
            memory[LCD_STATUS_REGISTER_ADDRESS],
            LYC_INT_SELECT_BIT_POSITION as u8,
        )
    }

    /// Sets the gpu/ppu mode to the provided value.
    fn set_gpu_mode(memory: &mut [u8; MEMORY_SIZE], mode: RenderingMode) {
        memory[LCD_STATUS_REGISTER_ADDRESS] =
            (memory[LCD_STATUS_REGISTER_ADDRESS] & 0b1111_1100) | mode.as_u8();
    }

    /// Sets the LYC = LY Coincidence Flag to the provided value.
    fn set_lyc_ly_coincidence_flag(memory: &mut [u8; MEMORY_SIZE], value: bool) {
        memory[LCD_STATUS_REGISTER_ADDRESS] = if value {
            set_bit(
                memory[LCD_STATUS_REGISTER_ADDRESS],
                LYC_LY_COINCIDENCE_FLAG_BIT_POSITION as u8,
            )
        } else {
            clear_bit(
                memory[LCD_STATUS_REGISTER_ADDRESS],
                LYC_LY_COINCIDENCE_FLAG_BIT_POSITION as u8,
            )
        };
        if value {
            if LCDStatusRegister::get_lyc_int_select(memory) {
                InterruptFlagRegister::set_flag(memory, Interrupt::LcdStat, true);
            }
        }
    }

    /// Returns a new u8 containing the new LCDStatusRegister value with the fields set according to
    /// the provided value except for PPU Mode and LYC=LY Coincidence Flag. So only the bits
    /// 3 to 6 are set according to the provided value.
    fn with_self_from_u8(memory: &[u8; MEMORY_SIZE], value: u8) -> u8 {
        let mut register = value & 0b0111_1000;
        if GPURegisters::get_scanline_compare(memory) == GPURegisters::get_scanline_internal(memory)
        {
            register |= 1 << LYC_LY_COINCIDENCE_FLAG_BIT_POSITION;
        }
        register |= memory[LCD_STATUS_REGISTER_ADDRESS] & 0b11;
        register
    }
}
