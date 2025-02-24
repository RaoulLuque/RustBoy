mod registers;

/// Struct to represent the CPU.
/// The CPU has 8 registers, a program counter (PC), a stack pointer (SP), and a memory bus.
/// For details please refer to [Pan Docs](https://gbdev.io/pandocs/CPU_Registers_and_Flags.html).
struct CPU {
    registers: Registers,
    pc: u16,
    sp: u16,
    bus: MemoryBus,
}

/// Struct to represent the registers of the CPU.
/// The CPU has 8 registers, each 8 bits (1 byte) wide.
/// The registers purposes are:
/// - a: accumulator
/// - f: flags register
/// - else: general purpose
///
/// The registers can either be accessed in pairs or individually.
///
/// For details please refer to [Pan Docs](https://gbdev.io/pandocs/CPU_Registers_and_Flags.html).
struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    // f is also referred to as the flags register
    f: u8,
    h: u8,
    l: u8,
}

struct MemoryBus {}
