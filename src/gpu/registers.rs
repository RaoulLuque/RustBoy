use super::{
    DOTS_IN_HBLANK_PLUS_TRANSFER, DOTS_IN_VBLANK, GPU, GPU_MODE_WHILE_LCD_TURNED_OFF,
    RenderingInfo, RenderingMode,
};

use crate::debugging::DebuggingFlags;
use crate::interrupts::InterruptFlagRegister;

const LCD_ENABLE_BYTE_POSITION: usize = 7;
const WINDOW_TILE_MAP_BYTE_POSITION: usize = 6;
const WINDOW_ENABLE_BYTE_POSITION: usize = 5;
const BG_TILE_DATA_BYTE_POSITION: usize = 4;
const BG_TILE_MAP_BYTE_POSITION: usize = 3;
const OBJ_SIZE_BYTE_POSITION: usize = 2;
const OBJ_ENABLE_BYTE_POSITION: usize = 1;
const BG_ENABLE_BYTE_POSITION: usize = 0;

const LYC_LY_COINCIDENCE_FLAG_BYTE_POSITION: usize = 2;
const MODE_0_INT_SELECT_BYTE_POSITION: usize = 3;
const MODE_1_INT_SELECT_BYTE_POSITION: usize = 4;
const MODE_2_INT_SELECT_BYTE_POSITION: usize = 5;
const LYC_INT_SELECT_BYTE_POSITION: usize = 6;

/// Represents the registers that control the GPU.
/// The registers have the following address and function:
/// - 0xFF40: LCDC - LCD Control Register
/// - 0xFF41: STAT - LCD Status Register
/// - 0xFF42: SCY - Scroll Y Register
/// - 0xFF43: SCX - Scroll X Register
/// - 0xFF44: LY - Current Scanline Register
/// - 0xFF45: LYC - LY Compare Register TODO: Implement
/// - 0xFF47: BGP - Background Palette Data Register TODO: Implement
pub struct GPURegisters {
    pub(super) lcd_control: LCDCRegister,
    pub(super) lcd_status: LCDStatusRegister,
    scroll_y: u8,
    scroll_x: u8,
    current_scanline: u8,
    scanline_compare: u8,
    background_palette: u8,
    pub(super) debugging_flags: DebuggingFlags,
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
/// - Bit 7: Display off/on (0 = off, 1 = on)
pub struct LCDCRegister {
    pub(super) background_on_off: bool,
    pub(super) sprites_on_off: bool,
    pub(super) sprite_size: bool,
    pub(super) background_tile_map: bool,
    pub(super) background_and_window_tile_data: bool,
    pub(super) window_on_off: bool,
    pub(super) window_tile_map: bool,
    pub(super) display_on: bool,
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
    pub(super) gpu_mode: RenderingMode,
    lyc_ly_coincidence_flag: bool,
    mode_0_int_select: bool,
    mode_1_int_select: bool,
    mode_2_int_select: bool,
    lyc_int_select: bool,
}

impl GPU {
    pub fn read_registers(&self, address: u16, cycles_current_instruction: u8) -> u8 {
        match address {
            0xFF40 => self.gpu_registers.get_lcd_control(),
            0xFF41 => self.gpu_registers.get_lcd_status(),
            0xFF42 => self.gpu_registers.get_scroll_y(),
            0xFF43 => self.gpu_registers.get_scroll_x(),
            0xFF44 => self.gpu_registers.get_scanline(
                Some(&self.rendering_info),
                Some(self.gpu_registers.lcd_status.gpu_mode),
                Some(cycles_current_instruction),
                true,
            ),
            0xFF45 => self.gpu_registers.get_scanline_compare(),
            0xFF47 => self.gpu_registers.get_background_palette(),
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
            0xFF40 => self.gpu_registers.set_lcd_control(value),
            0xFF41 => self
                .gpu_registers
                .set_lcd_status(value, interrupt_flag_register),
            0xFF42 => self.gpu_registers.set_scroll_y(value),
            0xFF43 => self.gpu_registers.set_scroll_x(value),
            // If the rom tries writing to the scanline register, it gets reset to 0
            0xFF44 => self.gpu_registers.set_scanline(0, interrupt_flag_register),
            0xFF45 => self
                .gpu_registers
                .set_scanline_compare(value, interrupt_flag_register),
            0xFF47 => self.gpu_registers.set_background_palette(value),
            _ => panic!("Writing to invalid GPU register address: {:#04X}", address),
        }
    }
}

impl GPURegisters {
    /// Creates a new instance of the GPURegisters struct with all registers set to their default
    /// startup values.
    pub fn new(debugging_flags: DebuggingFlags) -> Self {
        Self {
            lcd_control: LCDCRegister {
                background_on_off: false,
                sprites_on_off: false,
                sprite_size: false,
                background_tile_map: false,
                background_and_window_tile_data: false,
                window_on_off: false,
                window_tile_map: false,
                display_on: false,
            },
            lcd_status: LCDStatusRegister {
                gpu_mode: RenderingMode::HBlank0,
                lyc_ly_coincidence_flag: false,
                mode_0_int_select: false,
                mode_1_int_select: false,
                mode_2_int_select: false,
                lyc_int_select: false,
            },
            scroll_x: 0,
            scroll_y: 0,
            current_scanline: 0,
            scanline_compare: 0,
            background_palette: 0,
            debugging_flags,
        }
    }

    /// Set the LCD Control register to the provided value.
    pub fn set_lcd_control(&mut self, value: u8) {
        self.lcd_control = LCDCRegister::from(value);
        if self.lcd_control.display_on {
            log::debug!("LCD is turned on");
        } else {
            log::debug!("LCD is turned off");
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

    /// Set the scroll y register to the provided value.
    pub fn set_scroll_y(&mut self, value: u8) {
        self.scroll_y = value;
    }

    /// Set the scroll x register to the provided value.
    pub fn set_scroll_x(&mut self, value: u8) {
        self.scroll_x = value;
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
        self.lcd_status.lyc_ly_coincidence_flag = value;
        if value {
            if self.lcd_status.lyc_int_select {
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
        self.lcd_status.gpu_mode = mode;
        match mode {
            RenderingMode::HBlank0 => {
                if self.lcd_status.mode_0_int_select {
                    interrupt_flag_register.lcd_stat = true;
                }
            }
            RenderingMode::VBlank1 => {
                if self.lcd_status.mode_1_int_select {
                    interrupt_flag_register.lcd_stat = true;
                }
            }
            RenderingMode::OAMScan2 => {
                if self.lcd_status.mode_2_int_select {
                    interrupt_flag_register.lcd_stat = true;
                }
            }
            RenderingMode::Transfer3 => {}
        }
    }

    /// Set the background palette register to the provided value.
    pub fn set_background_palette(&mut self, value: u8) {
        self.background_palette = value;
    }

    /// Get the LCD Control register.
    pub fn get_lcd_control(&self) -> u8 {
        u8::from(&self.lcd_control)
    }

    /// Get the LCD Status register.
    ///
    /// If the LCD is turned off, we return VBlank mode (0b01) as the current mode (lower two
    /// bits of the LCD status register), because the CPU might read this register before the
    /// GPU has a chance to update it.
    pub fn get_lcd_status(&self) -> u8 {
        let before_lcd_enable = u8::from(&self.lcd_status);
        if !self.lcd_control.display_on {
            // If the LCD is turned off, we return VBlank mode (0b01) as the current mode (lower two
            // bits of the LCD status register)
            before_lcd_enable & GPU_MODE_WHILE_LCD_TURNED_OFF.as_u8()
        } else {
            before_lcd_enable
        }
    }

    /// Get the scroll y register.
    pub fn get_scroll_y(&self) -> u8 {
        self.scroll_y
    }

    /// Get the scroll x register.
    pub fn get_scroll_x(&self) -> u8 {
        self.scroll_x
    }

    /// Get the current scanline register.
    ///
    /// This function has rendering info, the current rendering mode and the cycles of the current
    /// instruction as optional parameters. These are used to correctly determine the current scanline
    /// based on a quirk if called from the memory bus (that is, called by the CPU). If the GPU is
    /// in HBlank mode and about to increment the scanline, we want to return this already incremented
    /// scanline instead of the current one since in the real Game Boy they (GPU and CPU) would run in
    /// parallel.
    pub fn get_scanline(
        &self,
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

    /// Get the GPU Mode
    pub fn get_gpu_mode(&self) -> RenderingMode {
        self.lcd_status.gpu_mode
    }

    /// Get the state of the obj size flag (sprite size)
    pub fn get_obj_size(&self) -> bool {
        self.lcd_control.sprite_size
    }
}

impl From<LCDCRegister> for u8 {
    fn from(register: LCDCRegister) -> Self {
        let mut value = 0;
        if register.display_on {
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

impl From<&LCDCRegister> for u8 {
    fn from(register: &LCDCRegister) -> Self {
        let mut value = 0;
        if register.display_on {
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
            display_on: value & (1 << LCD_ENABLE_BYTE_POSITION) != 0,
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

impl From<LCDStatusRegister> for u8 {
    fn from(register: LCDStatusRegister) -> Self {
        let mut value = 0;
        match register.gpu_mode {
            rendering_mode => match rendering_mode {
                RenderingMode::HBlank0 => value |= 0b00,
                RenderingMode::VBlank1 => value |= 0b01,
                RenderingMode::OAMScan2 => value |= 0b10,
                RenderingMode::Transfer3 => value |= 0b11,
            },
        }
        if register.lyc_ly_coincidence_flag {
            value |= 1 << LYC_LY_COINCIDENCE_FLAG_BYTE_POSITION;
        }
        if register.mode_0_int_select {
            value |= 1 << MODE_0_INT_SELECT_BYTE_POSITION;
        }
        if register.mode_1_int_select {
            value |= 1 << MODE_1_INT_SELECT_BYTE_POSITION;
        }
        if register.mode_2_int_select {
            value |= 1 << MODE_2_INT_SELECT_BYTE_POSITION;
        }
        if register.lyc_int_select {
            value |= 1 << LYC_INT_SELECT_BYTE_POSITION;
        }

        value
    }
}

impl From<&LCDStatusRegister> for u8 {
    fn from(register: &LCDStatusRegister) -> Self {
        let mut value = 0;
        match &register.gpu_mode {
            rendering_mode => match rendering_mode {
                RenderingMode::HBlank0 => value |= 0b00,
                RenderingMode::VBlank1 => value |= 0b01,
                RenderingMode::OAMScan2 => value |= 0b10,
                RenderingMode::Transfer3 => value |= 0b11,
            },
        }
        if register.lyc_ly_coincidence_flag {
            value |= 1 << LYC_LY_COINCIDENCE_FLAG_BYTE_POSITION;
        }
        if register.mode_0_int_select {
            value |= 1 << MODE_0_INT_SELECT_BYTE_POSITION;
        }
        if register.mode_1_int_select {
            value |= 1 << MODE_1_INT_SELECT_BYTE_POSITION;
        }
        if register.mode_2_int_select {
            value |= 1 << MODE_2_INT_SELECT_BYTE_POSITION;
        }
        if register.lyc_int_select {
            value |= 1 << LYC_INT_SELECT_BYTE_POSITION;
        }

        value
    }
}

impl LCDStatusRegister {
    /// Returns a new instance of the LCDStatusRegister struct with the fields set according to
    /// the provided value except for PPU Mode and LYC=LY Coincidence Flag.
    /// Needs a reference to the GPURegisters to get the value of the LYC=LY Coincidence Flag.
    fn with_self_from_u8(&self, gpu_registers: &GPURegisters, value: u8) -> Self {
        LCDStatusRegister {
            gpu_mode: self.gpu_mode,
            lyc_ly_coincidence_flag: gpu_registers.scanline_compare
                == gpu_registers.current_scanline,
            mode_0_int_select: value & (1 << MODE_0_INT_SELECT_BYTE_POSITION) != 0,
            mode_1_int_select: value & (1 << MODE_1_INT_SELECT_BYTE_POSITION) != 0,
            mode_2_int_select: value & (1 << MODE_2_INT_SELECT_BYTE_POSITION) != 0,
            lyc_int_select: value & (1 << LYC_INT_SELECT_BYTE_POSITION) != 0,
        }
    }
}
