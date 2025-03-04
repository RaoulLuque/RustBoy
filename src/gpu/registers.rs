use crate::cpu::registers::FlagsRegister;

const LCD_ENABLE_BYTE_POSITION: usize = 7;
const WINDOW_TILE_MAP_BYTE_POSITION: usize = 6;
const WINDOW_ENABLE_BYTE_POSITION: usize = 5;
const BG_TILE_DATA_BYTE_POSITION: usize = 4;
const BG_TILE_MAP_BYTE_POSITION: usize = 3;
const OBJ_SIZE_BYTE_POSITION: usize = 2;
const OBJ_ENABLE_BYTE_POSITION: usize = 1;
const BG_ENABLE_BYTE_POSITION: usize = 0;

/// Represents the registers that control the GPU.
/// The registers have the following address and function:
/// - 0xFF40: LCDC - LCD Control Register
/// - 0xFF41: STAT - LCD Status Register
/// - 0xFF42: SCY - Scroll Y Register
/// - 0xFF43: SCX - Scroll X Register
/// - 0xFF44: LY - Current Scanline Register
/// - 0xFF45: LYC - LY Compare Register TODO: Implement
///
pub struct GPURegisters {
    lcd_control: LCDCRegister,
    lcd_status: LCDStatusRegister,
    scroll_y: u8,
    scroll_x: u8,
    current_scanline: u8,
    ly_compare: u8,
    background_palette: u8,
}

/// Represents the LCDC register of the GPU.
/// The LCDC register is used to control the LCD.
/// It is an 8-bit register with the following bits:
/// - Bit 0: Background on/off (0 = off, 1 = on)
/// - Bit 1: Sprites on/off (0 = off, 1 = on)
/// - Bit 2: Sprite size (0 = 8x8, 1 = 8x16)
/// - Bit 3: Background tile map (0 = #0 (0x9800), 1 = #1 (0x9C00))
/// - Bit 4: Background and window tile data (0 = #0 (0x8800), 1 = #1 (0x8000))
/// - Bit 5: Window on/off (0 = off, 1 = on) - gets overridden by bit 0 on DMG
/// - Bit 6: Window tile map (0 = #0 (0x9800), 1 = #1 (0x9C00))
/// - Bit 7: Display on/off (0 = off, 1 = on)
pub struct LCDCRegister {
    background_on_off: bool,
    sprites_on_off: bool,
    sprite_size: bool,
    background_tile_map: bool,
    background_and_window_tile_data: bool,
    window_on_off: bool,
    window_tile_map: bool,
    display_on_off: bool,
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
    ppu_mode: PPUMode,
    lyc_ly_coincidence_flag: bool,
    mode_0_int_select: bool,
    mode_1_int_select: bool,
    mode_2_int_select: bool,
    lyc_int_select: bool,
}

enum PPUMode {
    RenderingMode,
    Disabled,
}

impl GPURegisters {
    pub fn new_zero() -> Self {
        Self {
            lcd_control: LCDCRegister {
                background_on_off: false,
                sprites_on_off: false,
                sprite_size: false,
                background_tile_map: false,
                background_and_window_tile_data: false,
                window_on_off: false,
                window_tile_map: false,
                display_on_off: false,
            },
            lcd_status: LCDStatusRegister {
                ppu_mode: PPUMode::RenderingMode,
                lyc_ly_coincidence_flag: false,
                mode_0_int_select: false,
                mode_1_int_select: false,
                mode_2_int_select: false,
                lyc_int_select: false,
            },
            scroll_x: 0,
            scroll_y: 0,
            current_scanline: 0,
            ly_compare: 0,
            background_palette: 0,
        }
    }
}

impl From<LCDCRegister> for u8 {
    fn from(register: LCDCRegister) -> Self {
        let mut value = 0;
        if register.display_on_off {
            value |= 1 << LCD_ENABLE_BYTE_POSITION;
        }
        if register.window_tile_map {
            value |= 1 << WINDOW_TILE_MAP_BYTE_POSITION;
        }
        if register.window_on_off {
            value |= 1 << WINDOW_ENABLE_BYTE_POSITION;
        }
        if register.background_and_window_tile_data {
            value |= 1 << BG_TILE_DATA_BYTE_POSITION;
        }
        if register.background_tile_map {
            value |= 1 << BG_TILE_MAP_BYTE_POSITION;
        }
        if register.sprite_size {
            value |= 1 << OBJ_SIZE_BYTE_POSITION;
        }
        if register.sprites_on_off {
            value |= 1 << OBJ_ENABLE_BYTE_POSITION;
        }
        if register.background_on_off {
            value |= 1 << BG_ENABLE_BYTE_POSITION;
        }
        value
    }
}

impl From<u8> for LCDCRegister {
    fn from(value: u8) -> Self {
        LCDCRegister {
            display_on_off: value & (1 << LCD_ENABLE_BYTE_POSITION) != 0,
            window_tile_map: value & (1 << WINDOW_TILE_MAP_BYTE_POSITION) != 0,
            window_on_off: value & (1 << WINDOW_ENABLE_BYTE_POSITION) != 0,
            background_and_window_tile_data: value & (1 << BG_TILE_DATA_BYTE_POSITION) != 0,
            background_tile_map: value & (1 << BG_TILE_MAP_BYTE_POSITION) != 0,
            sprite_size: value & (1 << OBJ_SIZE_BYTE_POSITION) != 0,
            sprites_on_off: value & (1 << OBJ_ENABLE_BYTE_POSITION) != 0,
            background_on_off: value & (1 << BG_ENABLE_BYTE_POSITION) != 0,
        }
    }
}
