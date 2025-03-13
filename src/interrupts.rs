const VBLANK_INTERRUPT_LOCATION: u16 = 0x0040;
const LCD_STAT_INTERRUPT_LOCATION: u16 = 0x0048;
const TIMER_INTERRUPT_LOCATION: u16 = 0x0050;
const SERIAL_INTERRUPT_LOCATION: u16 = 0x0058;
const JOYPAD_INTERRUPT_LOCATION: u16 = 0x0050;
const VBLANK_INTERRUPT_BIT: u8 = 0;
const LCD_STAT_INTERRUPT_BIT: u8 = 1;
const TIMER_INTERRUPT_BIT: u8 = 2;
const SERIAL_INTERRUPT_BIT: u8 = 3;
const JOYPAD_INTERRUPT_BIT: u8 = 4;

/// The interrupt enable register (IE 0xFFFF) is a 8-bit register that controls the enabling of interrupts.
/// The individual bits of the register correspond to different interrupts. If these and the IME
/// (Interrupt Master Enable) flag are set, the corresponding interrupt is enabled and can be called
/// if the corresponding bit in the interrupt request register (IF 0xFF0F) is set.
///
/// The bits have the following meaning:
/// - Bit 0: V-Blank interrupt
/// - Bit 1: LCD STAT interrupt
/// - Bit 2: Timer interrupt
/// - Bit 3: Serial interrupt
/// - Bit 4: Joypad interrupt
/// The other bits are unused.
pub struct InterruptEnableRegister {
    pub vblank: bool,
    pub lcd_stat: bool,
    pub timer: bool,
    pub serial: bool,
    pub joypad: bool,
}

/// The interrupt flag register (IF 0xFF0F) is a 8-bit register that contains the interrupt request flags.
/// These only indicate that an interrupt is being requested. It is only actually called if the
/// corresponding bit in the interrupt enable register (IE 0xFFFF) and the IME (Interrupt Master Enable)
/// are set.
///
/// The bits have the following meaning:
/// - Bit 0: V-Blank interrupt
/// - Bit 1: LCD STAT interrupt
/// - Bit 2: Timer interrupt
/// - Bit 3: Serial interrupt
/// - Bit 4: Joypad interrupt
/// The other bits are unused.
pub struct InterruptFlagRegister {
    pub vblank: bool,
    pub lcd_stat: bool,
    pub timer: bool,
    pub serial: bool,
    pub joypad: bool,
}

impl From<InterruptFlagRegister> for u8 {
    fn from(register: InterruptFlagRegister) -> Self {
        let mut value = 0;
        if register.vblank {
            value |= 1 << VBLANK_INTERRUPT_BIT;
        }
        if register.lcd_stat {
            value |= 1 << LCD_STAT_INTERRUPT_BIT;
        }
        if register.timer {
            value |= 1 << TIMER_INTERRUPT_BIT;
        }
        if register.serial {
            value |= 1 << SERIAL_INTERRUPT_BIT;
        }
        if register.joypad {
            value |= 1 << JOYPAD_INTERRUPT_BIT;
        }
        value
    }
}

impl From<u8> for InterruptFlagRegister {
    fn from(value: u8) -> Self {
        InterruptFlagRegister {
            vblank: value & (1 << VBLANK_INTERRUPT_BIT) != 0,
            lcd_stat: value & (1 << LCD_STAT_INTERRUPT_BIT) != 0,
            timer: value & (1 << TIMER_INTERRUPT_BIT) != 0,
            serial: value & (1 << SERIAL_INTERRUPT_BIT) != 0,
            joypad: value & (1 << JOYPAD_INTERRUPT_BIT) != 0,
        }
    }
}

impl InterruptEnableRegister {
    /// Creates a new interrupt enable register with all interrupts disabled.
    pub fn new() -> Self {
        InterruptEnableRegister {
            vblank: false,
            lcd_stat: false,
            timer: false,
            serial: false,
            joypad: false,
        }
    }

    /// Sets the V-Blank interrupt enable flag.
    pub fn set_vblank(&mut self, value: bool) {
        self.vblank = value;
    }

    /// Gets the V-Blank interrupt enable flag.
    pub fn get_vblank(&self) -> bool {
        self.vblank
    }

    /// Sets the LCD STAT interrupt enable flag.
    pub fn set_lcd_stat(&mut self, value: bool) {
        self.lcd_stat = value;
    }

    /// Gets the LCD STAT interrupt enable flag.
    pub fn get_lcd_stat(&self) -> bool {
        self.lcd_stat
    }

    /// Sets the Timer interrupt enable flag.
    pub fn set_timer(&mut self, value: bool) {
        self.timer = value;
    }

    /// Gets the Timer interrupt enable flag.
    pub fn get_timer(&self) -> bool {
        self.timer
    }

    /// Sets the Serial interrupt enable flag.
    pub fn set_serial(&mut self, value: bool) {
        self.serial = value;
    }

    /// Gets the Serial interrupt enable flag.
    pub fn get_serial(&self) -> bool {
        self.serial
    }

    /// Sets the Joypad interrupt enable flag.
    pub fn set_joypad(&mut self, value: bool) {
        self.joypad = value;
    }

    /// Gets the Joypad interrupt enable flag.
    pub fn get_joypad(&self) -> bool {
        self.joypad
    }
}

impl InterruptFlagRegister {
    /// Creates a new interrupt flag register with all interrupts disabled.
    pub fn new() -> Self {
        InterruptFlagRegister {
            vblank: false,
            lcd_stat: false,
            timer: false,
            serial: false,
            joypad: false,
        }
    }

    /// Sets the V-Blank interrupt flag.
    pub fn set_vblank(&mut self, value: bool) {
        self.vblank = value;
    }

    /// Gets the V-Blank interrupt flag.
    pub fn get_vblank(&self) -> bool {
        self.vblank
    }

    /// Sets the LCD STAT interrupt flag.
    pub fn set_lcd_stat(&mut self, value: bool) {
        self.lcd_stat = value;
    }

    /// Gets the LCD STAT interrupt flag.
    pub fn get_lcd_stat(&self) -> bool {
        self.lcd_stat
    }

    /// Sets the Timer interrupt flag.
    pub fn set_timer(&mut self, value: bool) {
        self.timer = value;
    }

    /// Gets the Timer interrupt flag.
    pub fn get_timer(&self) -> bool {
        self.timer
    }

    /// Sets the Serial interrupt flag.
    pub fn set_serial(&mut self, value: bool) {
        self.serial = value;
    }

    /// Gets the Serial interrupt flag.
    pub fn get_serial(&self) -> bool {
        self.serial
    }

    /// Sets the Joypad interrupt flag.
    pub fn set_joypad(&mut self, value: bool) {
        self.joypad = value;
    }

    /// Gets the Joypad interrupt flag.
    pub fn get_joypad(&self) -> bool {
        self.joypad
    }
}
