use super::{
    DOTS_IN_HBLANK_PLUS_TRANSFER, DOTS_IN_VBLANK, PPU, PPU_MODE_WHILE_LCD_TURNED_OFF,
    RenderingInfo, RenderingMode,
};
use crate::cpu::{clear_bit, is_bit_set, set_bit};

use crate::debugging::DebuggingFlagsWithoutFileHandles;
use crate::interrupts::{Interrupt, InterruptFlagRegister};
use crate::ppu::information_for_shader::ChangesToPropagateToShader;
use crate::{MEMORY_SIZE, MemoryBus};

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
pub struct PPURegisters {
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

impl PPU {
    pub fn read_registers(memory_bus: &MemoryBus, address: u16) -> u8 {
        match address {
            0xFF40 => PPURegisters::get_lcd_control(memory_bus),
            0xFF41 => PPURegisters::get_lcd_status(memory_bus),
            0xFF42 => PPURegisters::get_bg_scroll_y(memory_bus),
            0xFF43 => PPURegisters::get_bg_scroll_x(memory_bus),
            0xFF44 => PPURegisters::get_scanline(memory_bus),
            0xFF45 => PPURegisters::get_scanline_compare(memory_bus),
            0xFF47 => PPURegisters::get_background_palette(memory_bus),
            0xFF48 => PPURegisters::get_object_palette_zero(memory_bus),
            0xFF49 => PPURegisters::get_object_palette_one(memory_bus),
            0xFF4A => PPURegisters::get_window_y_position(memory_bus),
            0xFF4B => PPURegisters::get_window_x_position(memory_bus),
            _ => panic!(
                "Reading from invalid PPU register address: {:#04X}",
                address
            ),
        }
    }

    /// Writes to the GPU registers.
    ///
    /// Needs a reference to the interrupt flag register, if a write
    /// to the scanline register is made and LY=LYC is set, in which case a stat interrupt might
    /// be requested.
    pub fn write_registers(memory_bus: &mut MemoryBus, address: u16, value: u8) {
        match address {
            0xFF40 => PPURegisters::set_lcd_control(memory_bus, value),
            0xFF41 => PPURegisters::set_lcd_status(memory_bus, value),
            0xFF42 => PPURegisters::set_bg_scroll_y(memory_bus, value),
            0xFF43 => PPURegisters::set_bg_scroll_x(memory_bus, value),
            // If the rom tries writing to the scanline register, it gets reset to 0
            0xFF44 => PPURegisters::set_scanline(memory_bus, 0),
            0xFF45 => PPURegisters::set_scanline_compare(memory_bus, value),
            0xFF47 => PPURegisters::set_background_palette(memory_bus, value),
            0xFF48 => PPURegisters::set_object_palette_zero(memory_bus, value),
            0xFF49 => PPURegisters::set_object_palette_one(memory_bus, value),
            0xFF4A => PPURegisters::set_window_y_position(memory_bus, value),
            0xFF4B => PPURegisters::set_window_x_position(memory_bus, value),
            _ => panic!("Writing to invalid PPU register address: {:#04X}", address),
        }
    }
}

impl PPURegisters {
    /// Creates a new instance of the GPURegisters struct with all registers set to their default
    /// startup values.
    pub fn new(debugging_flags: DebuggingFlagsWithoutFileHandles) -> Self {
        Self { debugging_flags }
    }

    /// Set the LCD Control register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory_bus changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_lcd_control(memory_bus: &mut MemoryBus, value: u8) {
        let old_value = PPURegisters::get_lcd_control(memory_bus);
        memory_bus.memory[LCDC_REGISTER_ADDRESS] = value;
        let distinct_bits = old_value ^ value;
        if is_bit_set(distinct_bits, LCD_ENABLE_BIT_POSITION as u8) {
            if LCDCRegister::get_display_on_flag(memory_bus) {
                log::debug!("LCD is turned on");
            } else {
                log::debug!("LCD is turned off");
            }
        }

        // We need to check if the tile data area or background our window tilemap area changed
        // and set flags accordingly to make sure the GPU/Shader receives these changes in the
        // rendering step
        if is_bit_set(distinct_bits, WINDOW_TILE_MAP_BIT_POSITION as u8) {
            memory_bus.memory_changed.window_tile_map_flag_changed = true;
        }
        if is_bit_set(distinct_bits, BG_AND_WINDOW_TILE_DATA_BIT_POSITION as u8) {
            memory_bus.memory_changed.tile_data_flag_changed = true;
        }
        if is_bit_set(distinct_bits, BG_TILE_MAP_BIT_POSITION as u8) {
            memory_bus.memory_changed.background_tile_map_flag_changed = true;
        }
    }

    /// Set the LCD Status register to the provided value.
    ///
    /// Needs a reference to the interrupt flag register, if the LYC=LY Coincidence Flag is set,
    /// in which case a stat interrupt might be requested.
    pub fn set_lcd_status(memory_bus: &mut MemoryBus, value: u8) {
        memory_bus.memory[LCD_STATUS_REGISTER_ADDRESS] =
            LCDStatusRegister::with_self_from_u8(memory_bus, value);
        LCDStatusRegister::set_lyc_ly_coincidence_flag(
            memory_bus,
            PPURegisters::get_scanline_internal(memory_bus)
                == PPURegisters::get_scanline_compare(memory_bus),
        );
    }

    /// Set the Background Scroll Y register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory_bus changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_bg_scroll_y(memory_bus: &mut MemoryBus, value: u8) {
        memory_bus.memory[BG_SCROLL_Y_REGISTER_ADDRESS] = value;
        memory_bus
            .memory_changed
            .background_viewport_position_changed = true;
    }

    /// Set the Background Scroll X register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory_bus changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_bg_scroll_x(memory_bus: &mut MemoryBus, value: u8) {
        memory_bus.memory[BG_SCROLL_X_REGISTER_ADDRESS] = value;
        memory_bus
            .memory_changed
            .background_viewport_position_changed = true;
    }

    /// Set the current scanline register to the provided value.
    ///
    /// Needs a reference to the interrupt flag register, if LY=LYC and a stat interrupt might be
    /// requested.
    pub(super) fn set_scanline(memory_bus: &mut MemoryBus, value: u8) {
        memory_bus.memory[SCANLINE_REGISTER_ADDRESS] = value;
        LCDStatusRegister::set_lyc_ly_coincidence_flag(
            memory_bus,
            PPURegisters::get_scanline_internal(memory_bus)
                == PPURegisters::get_scanline_compare(memory_bus),
        );
    }

    /// Set the LY (Scanline) Compare register to the provided value.
    ///
    /// Needs a reference to the interrupt flag register to possibly request a stat interrupt, if
    /// LY=LYC and the LYC int select is set.
    fn set_scanline_compare(memory_bus: &mut MemoryBus, value: u8) {
        memory_bus.memory[SCANLINE_COMPARE_REGISTER_ADDRESS] = value;
        LCDStatusRegister::set_lyc_ly_coincidence_flag(
            memory_bus,
            PPURegisters::get_scanline_internal(memory_bus)
                == PPURegisters::get_scanline_compare(memory_bus),
        );
    }

    /// Set the GPU/PPU Mode to the provided value.
    ///
    /// Needs a reference to the interrupt flag register to possibly request a stat interrupt, if
    /// the corresponding mode int select flag is set to the provided mode which is being entered.
    pub(crate) fn set_ppu_mode(memory_bus: &mut MemoryBus, mode: RenderingMode) {
        LCDStatusRegister::set_ppu_mode(memory_bus, mode);
        match mode {
            RenderingMode::HBlank0 => {
                if LCDStatusRegister::get_mode_0_int_select(memory_bus) {
                    InterruptFlagRegister::set_flag(memory_bus, Interrupt::LcdStat, true);
                }
            }
            RenderingMode::VBlank1 => {
                if LCDStatusRegister::get_mode_1_int_select(memory_bus) {
                    InterruptFlagRegister::set_flag(memory_bus, Interrupt::LcdStat, true);
                }
            }
            RenderingMode::OAMScan2 => {
                if LCDStatusRegister::get_mode_2_int_select(memory_bus) {
                    InterruptFlagRegister::set_flag(memory_bus, Interrupt::LcdStat, true);
                }
            }
            RenderingMode::Transfer3 => {}
        }
    }

    /// Set the background palette register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory_bus changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_background_palette(memory_bus: &mut MemoryBus, value: u8) {
        if PPURegisters::get_background_palette(memory_bus) != value {
            memory_bus.memory_changed.palette_changed = true;
            memory_bus.memory[BACKGROUND_PALETTE_REGISTER_ADDRESS] = value;
        }
    }

    /// Set the object palette 0 register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory_bus changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_object_palette_zero(memory_bus: &mut MemoryBus, value: u8) {
        if PPURegisters::get_object_palette_zero(memory_bus) != value {
            memory_bus.memory_changed.palette_changed = true;
            memory_bus.memory[OBJECT_PALETTE_ZERO_REGISTER_ADDRESS] = value;
        }
    }

    /// Set the object palette 1 register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory_bus changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_object_palette_one(memory_bus: &mut MemoryBus, value: u8) {
        if PPURegisters::get_object_palette_one(memory_bus) != value {
            memory_bus.memory_changed.palette_changed = true;
            memory_bus.memory[OBJECT_PALETTE_ONE_REGISTER_ADDRESS] = value;
        }
    }

    /// Set the window Y position register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory_bus changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_window_y_position(memory_bus: &mut MemoryBus, value: u8) {
        if PPURegisters::get_window_y_position(memory_bus) != value {
            memory_bus.memory_changed.window_viewport_position_changed = true;
            memory_bus.memory[WINDOW_Y_POSITION_REGISTER_ADDRESS] = value;
        }
    }

    /// Set the window X position register to the provided value.
    ///
    /// Also sets flags in the provided [super::ChangesToPropagateToShader] struct, to keep track of which parts
    /// of the GPU memory_bus changed for the next scanline/frame rendering to propagate these changes
    /// to the shader.
    pub fn set_window_x_position(memory_bus: &mut MemoryBus, value: u8) {
        if PPURegisters::get_window_x_position(memory_bus) != value {
            memory_bus.memory_changed.window_viewport_position_changed = true;
            memory_bus.memory[WINDOW_X_POSITION_REGISTER_ADDRESS] = value;
        }
    }

    /// Get the LCD Control register.
    pub fn get_lcd_control(memory_bus: &MemoryBus) -> u8 {
        memory_bus.memory[LCDC_REGISTER_ADDRESS]
    }

    /// Get the LCD Status register.
    ///
    /// If the LCD is turned off, we return VBlank mode (0b01) as the current mode (lower two
    /// bits of the LCD status register), because the CPU might read this register before the
    /// GPU has a chance to update it.
    pub fn get_lcd_status(memory_bus: &MemoryBus) -> u8 {
        let before_lcd_enable = memory_bus.memory[LCD_STATUS_REGISTER_ADDRESS];
        if LCDCRegister::get_display_on_flag(memory_bus) {
            // If the LCD is turned off, we return VBlank mode (0b01) as the current mode (lower two
            // bits of the LCD status register)
            before_lcd_enable & (0b1111_1100 | PPU_MODE_WHILE_LCD_TURNED_OFF.as_u8())
        } else {
            before_lcd_enable
        }
    }

    /// Get the Background Scroll Y register.
    pub fn get_bg_scroll_y(memory_bus: &MemoryBus) -> u8 {
        memory_bus.memory[BG_SCROLL_Y_REGISTER_ADDRESS]
    }

    /// Get the Background Scroll X register.
    pub fn get_bg_scroll_x(memory_bus: &MemoryBus) -> u8 {
        memory_bus.memory[BG_SCROLL_X_REGISTER_ADDRESS]
    }

    /// Get the current scanline register.
    ///
    /// This function has rendering info, the current rendering mode and the cycles of the current
    /// instruction as optional parameters. These are used to correctly determine the current scanline
    /// based on a quirk if called from the memory_bus bus (that is, called by the CPU). If the GPU is
    /// in HBlank mode and about to increment the scanline, we want to return this already incremented
    /// scanline instead of the current one since in the real Game Boy they (GPU and CPU) would run in
    /// parallel.
    ///
    /// TODO: Update docstring
    pub fn get_scanline(memory_bus: &MemoryBus) -> u8 {
        if memory_bus.debugging_flags_without_file_handles.doctor {
            // Game Boy Doctor specifies that reading from the LY register (scanline) should always
            // return 0x90.
            0x90
        } else {
            PPURegisters::get_scanline_internal(memory_bus)
        }
    }

    pub(crate) fn get_scanline_internal(memory_bus: &MemoryBus) -> u8 {
        memory_bus.memory[SCANLINE_REGISTER_ADDRESS]
    }

    /// Get the LY (Scanline) Compare register.
    pub fn get_scanline_compare(memory_bus: &MemoryBus) -> u8 {
        memory_bus.memory[SCANLINE_COMPARE_REGISTER_ADDRESS]
    }

    /// Get the background palette register.
    pub fn get_background_palette(memory_bus: &MemoryBus) -> u8 {
        memory_bus.memory[BACKGROUND_PALETTE_REGISTER_ADDRESS]
    }

    /// Get the object palette 0 register.
    pub fn get_object_palette_zero(memory_bus: &MemoryBus) -> u8 {
        memory_bus.memory[OBJECT_PALETTE_ZERO_REGISTER_ADDRESS]
    }

    /// Get the object palette 1 register.
    pub fn get_object_palette_one(memory_bus: &MemoryBus) -> u8 {
        memory_bus.memory[OBJECT_PALETTE_ONE_REGISTER_ADDRESS]
    }

    /// Get the window Y position register.
    pub fn get_window_y_position(memory_bus: &MemoryBus) -> u8 {
        memory_bus.memory[WINDOW_Y_POSITION_REGISTER_ADDRESS]
    }

    /// Get the window X position register.
    pub fn get_window_x_position(memory_bus: &MemoryBus) -> u8 {
        memory_bus.memory[WINDOW_X_POSITION_REGISTER_ADDRESS]
    }

    /// Get the GPU Mode
    pub fn get_ppu_mode(memory_bus: &MemoryBus) -> RenderingMode {
        LCDStatusRegister::get_ppu_mode(memory_bus)
    }
}

impl LCDCRegister {
    /// Returns the state of the sprite size flag.
    pub fn get_sprite_size_flag(memory_bus: &MemoryBus) -> bool {
        is_bit_set(
            memory_bus.memory[LCDC_REGISTER_ADDRESS],
            OBJ_SIZE_BIT_POSITION as u8,
        )
    }

    /// Returns the state of the background tilemap flag.
    pub fn get_background_tile_map_flag(memory_bus: &MemoryBus) -> bool {
        is_bit_set(
            memory_bus.memory[LCDC_REGISTER_ADDRESS],
            BG_TILE_MAP_BIT_POSITION as u8,
        )
    }

    /// Returns the state of the background and window tile data flag.
    pub fn get_background_and_window_tile_data_flag(memory_bus: &MemoryBus) -> bool {
        is_bit_set(
            memory_bus.memory[LCDC_REGISTER_ADDRESS],
            BG_AND_WINDOW_TILE_DATA_BIT_POSITION as u8,
        )
    }

    /// Returns the state of the window tilemap flag.
    pub fn get_window_tile_map_flag(memory_bus: &MemoryBus) -> bool {
        is_bit_set(
            memory_bus.memory[LCDC_REGISTER_ADDRESS],
            WINDOW_TILE_MAP_BIT_POSITION as u8,
        )
    }

    /// Returns the state of the lcd/display enable flag.
    pub fn get_display_on_flag(memory_bus: &MemoryBus) -> bool {
        is_bit_set(
            memory_bus.memory[LCDC_REGISTER_ADDRESS],
            LCD_ENABLE_BIT_POSITION as u8,
        )
    }
}

impl LCDStatusRegister {
    /// Returns the GPU mode as a [super::RenderingMode] enum.
    fn get_ppu_mode(memory_bus: &MemoryBus) -> RenderingMode {
        RenderingMode::from_u8(memory_bus.memory[LCD_STATUS_REGISTER_ADDRESS] & 0b0000_0011)
    }

    /// Returns the state of the mode 0 interrupt select flag.
    fn get_mode_0_int_select(memory_bus: &MemoryBus) -> bool {
        is_bit_set(
            memory_bus.memory[LCD_STATUS_REGISTER_ADDRESS],
            MODE_0_INT_SELECT_BIT_POSITION as u8,
        )
    }

    /// Returns the state of the mode 1 interrupt select flag.
    fn get_mode_1_int_select(memory_bus: &MemoryBus) -> bool {
        is_bit_set(
            memory_bus.memory[LCD_STATUS_REGISTER_ADDRESS],
            MODE_1_INT_SELECT_BIT_POSITION as u8,
        )
    }

    /// Returns the state of the mode 2 interrupt select flag.
    fn get_mode_2_int_select(memory_bus: &MemoryBus) -> bool {
        is_bit_set(
            memory_bus.memory[LCD_STATUS_REGISTER_ADDRESS],
            MODE_2_INT_SELECT_BIT_POSITION as u8,
        )
    }

    /// Returns the state of the LYC interrupt select flag.
    fn get_lyc_int_select(memory_bus: &MemoryBus) -> bool {
        is_bit_set(
            memory_bus.memory[LCD_STATUS_REGISTER_ADDRESS],
            LYC_INT_SELECT_BIT_POSITION as u8,
        )
    }

    /// Sets the gpu/ppu mode to the provided value.
    fn set_ppu_mode(memory_bus: &mut MemoryBus, mode: RenderingMode) {
        memory_bus.memory[LCD_STATUS_REGISTER_ADDRESS] =
            (memory_bus.memory[LCD_STATUS_REGISTER_ADDRESS] & 0b1111_1100) | mode.as_u8();
    }

    /// Sets the LYC = LY Coincidence Flag to the provided value.
    fn set_lyc_ly_coincidence_flag(memory_bus: &mut MemoryBus, value: bool) {
        memory_bus.memory[LCD_STATUS_REGISTER_ADDRESS] = if value {
            set_bit(
                memory_bus.memory[LCD_STATUS_REGISTER_ADDRESS],
                LYC_LY_COINCIDENCE_FLAG_BIT_POSITION as u8,
            )
        } else {
            clear_bit(
                memory_bus.memory[LCD_STATUS_REGISTER_ADDRESS],
                LYC_LY_COINCIDENCE_FLAG_BIT_POSITION as u8,
            )
        };
        if value {
            if LCDStatusRegister::get_lyc_int_select(memory_bus) {
                InterruptFlagRegister::set_flag(memory_bus, Interrupt::LcdStat, true);
            }
        }
    }

    /// Returns a new u8 containing the new LCDStatusRegister value with the fields set according to
    /// the provided value except for PPU Mode and LYC=LY Coincidence Flag. So only the bits
    /// 3 to 6 are set according to the provided value.
    fn with_self_from_u8(memory_bus: &MemoryBus, value: u8) -> u8 {
        let mut register = value & 0b0111_1000;
        if PPURegisters::get_scanline_compare(memory_bus)
            == PPURegisters::get_scanline_internal(memory_bus)
        {
            register |= 1 << LYC_LY_COINCIDENCE_FLAG_BIT_POSITION;
        }
        register |= memory_bus.memory[LCD_STATUS_REGISTER_ADDRESS] & 0b11;
        register
    }
}
