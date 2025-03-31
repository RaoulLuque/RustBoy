use crate::cpu::{clear_bit, is_bit_set, set_bit};
use crate::memory_bus::{INTERRUPT_ENABLE_REGISTER, INTERRUPT_FLAG_REGISTER};
use crate::{MEMORY_SIZE, RustBoy};

const VBLANK_INTERRUPT_LOCATION: u16 = 0x0040;
const LCD_STAT_INTERRUPT_LOCATION: u16 = 0x0048;
const TIMER_INTERRUPT_LOCATION: u16 = 0x0050;
const SERIAL_INTERRUPT_LOCATION: u16 = 0x0058;
const JOYPAD_INTERRUPT_LOCATION: u16 = 0x0060;
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
/// This struct is empty and has no fields. Instead it is just used offer some static methods to
/// simplify access to this register. The actual register data are all held in the [MemoryBus](crate::MemoryBus).
///
/// The bits have the following meaning:
/// - Bit 0: V-Blank interrupt
/// - Bit 1: LCD STAT interrupt
/// - Bit 2: Timer interrupt
/// - Bit 3: Serial interrupt
/// - Bit 4: Joypad interrupt
/// The other bits are unused.
pub struct InterruptEnableRegister {}

/// The interrupt flag register (IF 0xFF0F) is a 8-bit register that contains the interrupt request flags.
/// These only indicate that an interrupt is being requested. It is only actually called if the
/// corresponding bit in the interrupt enable register (IE 0xFFFF) and the IME (Interrupt Master Enable)
/// are set.
///
/// This struct is empty and has no fields. Instead it is just used offer some static methods to
/// simplify access to this register. The actual register data are all held in the [MemoryBus](crate::MemoryBus).
///
/// The bits have the following meaning:
/// - Bit 0: V-Blank interrupt
/// - Bit 1: LCD STAT interrupt
/// - Bit 2: Timer interrupt
/// - Bit 3: Serial interrupt
/// - Bit 4: Joypad interrupt
/// The other bits are unused.
pub struct InterruptFlagRegister {}

#[derive(Debug, Clone, Copy)]
pub enum Interrupt {
    VBlank,
    LcdStat,
    Timer,
    Serial,
    Joypad,
}

impl RustBoy {
    /// Handles interrupts by checking all possible interrupts according to the
    /// [InterruptEnableRegister] and the [InterruptFlagRegister] and requesting if it should be.
    /// Returns true if an interrupt was requested.
    ///
    /// If an interrupt was requested, [request_interrupt] is called with the corresponding
    /// interrupt location. In that case, this function counts as an executed instruction on the
    /// CPU and the cpu step should be called again.
    pub fn check_if_interrupt_is_requested(&mut self) -> Option<u16> {
        if self.ime {
            // VBLANK
            if self.check_if_specific_interrupt_is_requested_and_handle(Interrupt::VBlank) {
                return Some(VBLANK_INTERRUPT_LOCATION);
            }

            // LCD STAT
            if self.check_if_specific_interrupt_is_requested_and_handle(Interrupt::LcdStat) {
                return Some(LCD_STAT_INTERRUPT_LOCATION);
            }

            // TIMER
            if self.check_if_specific_interrupt_is_requested_and_handle(Interrupt::Timer) {
                return Some(TIMER_INTERRUPT_LOCATION);
            }

            // SERIAL
            if self.check_if_specific_interrupt_is_requested_and_handle(Interrupt::Serial) {
                return Some(SERIAL_INTERRUPT_LOCATION);
            }

            // JOYPAD
            if self.check_if_specific_interrupt_is_requested_and_handle(Interrupt::Joypad) {
                return Some(JOYPAD_INTERRUPT_LOCATION);
            }
        }
        None
    }

    fn check_if_specific_interrupt_is_requested_and_handle(
        &mut self,
        interrupt: Interrupt,
    ) -> bool {
        if InterruptEnableRegister::get_flag(&self.memory, interrupt) {
            if InterruptFlagRegister::get_flag(&self.memory, interrupt) {
                // Clear the interrupt flags
                InterruptFlagRegister::set_flag(&mut self.memory, interrupt, false);
                self.ime = false;

                return true;
            }
        }
        false
    }
}

impl InterruptEnableRegister {
    /// Returns the value of the interrupt enable register (IE 0xFFFF).
    pub fn get_interrupt_enable_register(memory: &[u8; MEMORY_SIZE]) -> u8 {
        memory[INTERRUPT_ENABLE_REGISTER as usize]
    }

    /// Sets the value of the interrupt enable register (IE 0xFFFF).
    pub fn set_interrupt_enable_register(memory: &mut [u8; MEMORY_SIZE], value: u8) {
        memory[INTERRUPT_ENABLE_REGISTER as usize] = value;
    }

    /// Returns the value of the provided [Interrupt] from the interrupt enable register.
    pub fn get_flag(memory: &[u8; MEMORY_SIZE], interrupt: Interrupt) -> bool {
        let interrupt_enable = InterruptEnableRegister::get_interrupt_enable_register(memory);
        interrupt.is_set(interrupt_enable)
    }

    /// Sets the value of the provided [Interrupt] in the interrupt enable register to the provided
    /// value.
    pub fn set_flag(memory: &mut [u8; MEMORY_SIZE], interrupt: Interrupt, value: bool) {
        let mut interrupt_enable = InterruptEnableRegister::get_interrupt_enable_register(memory);
        interrupt_enable = if value {
            interrupt.set(interrupt_enable)
        } else {
            interrupt.clear(interrupt_enable)
        };
        InterruptEnableRegister::set_interrupt_enable_register(memory, interrupt_enable);
    }
}

impl InterruptFlagRegister {
    /// Returns the value of the interrupt flag register (IF 0xFF0F).
    pub fn get_interrupt_flag_register(memory: &[u8; MEMORY_SIZE]) -> u8 {
        memory[INTERRUPT_FLAG_REGISTER as usize]
    }

    /// Sets the value of the interrupt flag register (IF 0xFF0F).
    pub fn set_interrupt_flag_register(memory: &mut [u8; MEMORY_SIZE], value: u8) {
        memory[INTERRUPT_FLAG_REGISTER as usize] = value;
    }

    /// Returns the value of the provided [Interrupt] from the interrupt flag register.
    pub fn get_flag(memory: &[u8; MEMORY_SIZE], interrupt: Interrupt) -> bool {
        let interrupt_enable = InterruptFlagRegister::get_interrupt_flag_register(memory);
        interrupt.is_set(interrupt_enable)
    }

    /// Sets the value of the provided [Interrupt] in the interrupt flag register to the provided
    /// value.
    pub fn set_flag(memory: &mut [u8; MEMORY_SIZE], interrupt: Interrupt, value: bool) {
        let mut interrupt_enable = InterruptFlagRegister::get_interrupt_flag_register(memory);
        interrupt_enable = if value {
            interrupt.set(interrupt_enable)
        } else {
            interrupt.clear(interrupt_enable)
        };
        InterruptFlagRegister::set_interrupt_flag_register(memory, interrupt_enable);
    }
}

impl Interrupt {
    fn is_set(&self, value: u8) -> bool {
        use Interrupt::*;
        match self {
            VBlank => is_bit_set(value, VBLANK_INTERRUPT_BIT),
            LcdStat => is_bit_set(value, LCD_STAT_INTERRUPT_BIT),
            Timer => is_bit_set(value, TIMER_INTERRUPT_BIT),
            Serial => is_bit_set(value, SERIAL_INTERRUPT_BIT),
            Joypad => is_bit_set(value, JOYPAD_INTERRUPT_BIT),
        }
    }

    fn set(&self, value: u8) -> u8 {
        use Interrupt::*;
        match self {
            VBlank => set_bit(value, VBLANK_INTERRUPT_BIT),
            LcdStat => set_bit(value, LCD_STAT_INTERRUPT_BIT),
            Timer => set_bit(value, TIMER_INTERRUPT_BIT),
            Serial => set_bit(value, SERIAL_INTERRUPT_BIT),
            Joypad => set_bit(value, JOYPAD_INTERRUPT_BIT),
        }
    }

    fn clear(&self, value: u8) -> u8 {
        use Interrupt::*;
        match self {
            VBlank => clear_bit(value, VBLANK_INTERRUPT_BIT),
            LcdStat => clear_bit(value, LCD_STAT_INTERRUPT_BIT),
            Timer => clear_bit(value, TIMER_INTERRUPT_BIT),
            Serial => clear_bit(value, SERIAL_INTERRUPT_BIT),
            Joypad => clear_bit(value, JOYPAD_INTERRUPT_BIT),
        }
    }
}
