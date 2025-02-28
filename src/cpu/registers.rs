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
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: FlagsRegister,
    pub h: u8,
    pub l: u8,
}

impl Registers {
    /// Creates a new instance of the Registers struct with all registers set to 0.
    pub fn default() -> Self {
        Registers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: FlagsRegister {
                zero: false,
                subtract: false,
                half_carry: false,
                carry: false,
            },
            h: 0,
            l: 0,
        }
    }

    /// Returns the value of the AF register pair.
    pub fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | (u16::from(&self.f))
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
        self.b = (value & 0xF00) as u8;
    }

    /// Sets the value of the BC register pair.
    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0xF00) as u8;
    }

    /// Sets the value of the DE register pair.
    pub fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0xF00) as u8;
    }

    /// Sets the value of the HL register pair.
    pub fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0xF00) as u8;
    }
}

/// Struct to represent the flags register.
/// The flags register is a special register that contains 4 flags.
/// Therefore, in comparison to the other registers, it only uses 4 bits of the 8 bits available.
/// The other 4 bits are always zero. The flags are:
/// - zero (Z) - set to true if the result of the last operation was zero
/// - subtract (N) - set to true if the last operation was a subtraction
/// - half carry (H) - set to true if there was a carry from bit 3 to bit 4
/// - carry (C/CY) - set to true if there was a carry from bit 7 (an overflow)
#[derive(Debug)]
pub struct FlagsRegister {
    pub zero: bool,
    pub subtract: bool,
    pub half_carry: bool,
    pub carry: bool,
}

impl std::convert::From<FlagsRegister> for u8 {
    fn from(flags: FlagsRegister) -> Self {
        let mut value = 0;
        if flags.zero {
            value |= 1 << ZERO_FLAG_BYTE_POSITION;
        }
        if flags.subtract {
            value |= 1 << SUBTRACT_FLAG_BYTE_POSITION;
        }
        if flags.half_carry {
            value |= 1 << HALF_CARRY_FLAG_BYTE_POSITION;
        }
        if flags.carry {
            value |= 1 << CARRY_FLAG_BYTE_POSITION;
        }
        value
    }
}

impl std::convert::From<&FlagsRegister> for u8 {
    fn from(flags: &FlagsRegister) -> Self {
        let mut value = 0;
        if flags.zero {
            value |= 1 << ZERO_FLAG_BYTE_POSITION;
        }
        if flags.subtract {
            value |= 1 << SUBTRACT_FLAG_BYTE_POSITION;
        }
        if flags.half_carry {
            value |= 1 << HALF_CARRY_FLAG_BYTE_POSITION;
        }
        if flags.carry {
            value |= 1 << CARRY_FLAG_BYTE_POSITION;
        }
        value
    }
}

impl std::convert::From<FlagsRegister> for u16 {
    fn from(flags: FlagsRegister) -> Self {
        u8::from(flags) as u16
    }
}

impl std::convert::From<&FlagsRegister> for u16 {
    fn from(flags: &FlagsRegister) -> Self {
        u8::from(flags) as u16
    }
}

impl std::convert::From<u8> for FlagsRegister {
    fn from(value: u8) -> Self {
        FlagsRegister {
            zero: value & (1 << ZERO_FLAG_BYTE_POSITION) != 0,
            subtract: value & (1 << SUBTRACT_FLAG_BYTE_POSITION) != 0,
            half_carry: value & (1 << HALF_CARRY_FLAG_BYTE_POSITION) != 0,
            carry: value & (1 << CARRY_FLAG_BYTE_POSITION) != 0,
        }
    }
}

impl std::convert::From<u16> for FlagsRegister {
    fn from(value: u16) -> Self {
        FlagsRegister::from(value as u8)
    }
}
