use super::{clear_bit, is_bit_set, set_bit};

const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

/// Struct to represent the registers of the CPU.
/// The CPU has 8 registers, each 8 bits (1 byte) wide.
/// The registers purposes are:
/// - a: accumulator
/// - f: [FlagsRegister]
/// - else: general purpose
///
/// The registers can either be accessed in pairs or individually.
///
/// For details please refer to [Pan Docs](https://gbdev.io/pandocs/CPU_Registers_and_Flags.html).
#[derive(Debug)]
pub struct CPURegisters {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: FlagsRegister,
    pub h: u8,
    pub l: u8,
}

impl CPURegisters {
    /// Creates a new instance of the Registers struct with all registers set to 0. This
    /// is the state of the registers before the boot rom has been executed.
    pub fn new_zero() -> Self {
        CPURegisters {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: FlagsRegister { register: 0 },
            h: 0,
            l: 0,
        }
    }

    /// Creates a new instance of the Registers struct with the registers set to their values
    /// after the boot rom has been executed.
    pub fn new_after_boot() -> Self {
        CPURegisters {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            f: FlagsRegister { register: 0xB0 },
            h: 0x01,
            l: 0x4D,
        }
    }

    /// Returns the value of the AF register pair.
    pub fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f.register as u16)
    }

    /// Returns the value of the BC register pair.
    pub fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    /// Returns the value of the DE register pair.
    pub fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    /// Returns the value of the HL register pair.
    pub fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    /// Sets the value of the AF register pair.
    pub fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xFF00) >> 8) as u8;
        self.f = FlagsRegister {
            register: (value & 0x00FF) as u8,
        };
    }

    /// Sets the value of the BC register pair.
    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0x00FF) as u8;
    }

    /// Sets the value of the DE register pair.
    pub fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0x00FF) as u8;
    }

    /// Sets the value of the HL register pair.
    pub fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0x00FF) as u8;
    }
}

/// Struct to represent the flags register.
/// The flags register is a special register that contains 4 flags.
/// Therefore, in comparison to the other registers, it only uses 4 bits of the 8 bits available.
/// The other 4 bits are always zero. The flags are:
/// - Bit 7: zero (Z) - set to true if the result of the last operation was zero
/// - Bit 6: subtract (N) - set to true if the last operation was a subtraction
/// - Bit 5: half carry (H) - set to true if there was a carry from bit 3 to bit 4
/// - Bit 4: carry (C/CY) - set to true if there was a carry from bit 7 (an overflow)
#[derive(Debug)]
pub struct FlagsRegister {
    register: u8,
}

impl FlagsRegister {
    pub fn get(&self) -> u8 {
        self.register & 0xF0
    }

    pub fn get_zero_flag(&self) -> bool {
        is_bit_set(self.register, ZERO_FLAG_BYTE_POSITION)
    }

    pub fn get_subtract_flag(&self) -> bool {
        is_bit_set(self.register, SUBTRACT_FLAG_BYTE_POSITION)
    }

    pub fn get_half_carry_flag(&self) -> bool {
        is_bit_set(self.register, HALF_CARRY_FLAG_BYTE_POSITION)
    }

    pub fn get_carry_flag(&self) -> bool {
        is_bit_set(self.register, CARRY_FLAG_BYTE_POSITION)
    }

    pub fn set_zero_flag(&mut self, value: bool) {
        if value {
            self.register = set_bit(self.register, ZERO_FLAG_BYTE_POSITION);
        } else {
            self.register = clear_bit(self.register, ZERO_FLAG_BYTE_POSITION);
        }
    }

    pub fn set_subtract_flag(&mut self, value: bool) {
        if value {
            self.register = set_bit(self.register, SUBTRACT_FLAG_BYTE_POSITION);
        } else {
            self.register = clear_bit(self.register, SUBTRACT_FLAG_BYTE_POSITION);
        }
    }

    pub fn set_half_carry_flag(&mut self, value: bool) {
        if value {
            self.register = set_bit(self.register, HALF_CARRY_FLAG_BYTE_POSITION);
        } else {
            self.register = clear_bit(self.register, HALF_CARRY_FLAG_BYTE_POSITION);
        }
    }

    pub fn set_carry_flag(&mut self, value: bool) {
        if value {
            self.register = set_bit(self.register, CARRY_FLAG_BYTE_POSITION);
        } else {
            self.register = clear_bit(self.register, CARRY_FLAG_BYTE_POSITION);
        }
    }
}
