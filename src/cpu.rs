mod instructions;
mod registers;

use instructions::Instruction;
use registers::Registers;

/// Struct to represent the CPU.
/// The CPU has 8 registers, a program counter (PC), a stack pointer (SP), and a memory bus.
/// For details please refer to [Pan Docs](https://gbdev.io/pandocs/CPU_Registers_and_Flags.html).
///
/// For implementations of the CPU instructions please see [instructions].
pub struct CPU {
    registers: Registers,
    pc: u16,
    sp: u16,
    bus: MemoryBus,
}

impl CPU {
    /// Reads the next instruction and executes it in the CPU.
    /// Doing so, the program counter (pc) is updated to point to the address of the next instruction.
    fn step(&mut self) {
        let mut instruction_byte = self.bus.read_byte(self.pc);

        // Check if the instruction is a CB instruction (prefix)
        let prefixed = instruction_byte == 0xCB;
        if prefixed {
            instruction_byte = self.bus.read_byte(self.pc + 1);
        }

        let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_byte, prefixed)
        {
            self.execute(instruction)
        } else {
            let panic_description = format!(
                "0x{}{:x}",
                if prefixed { "CB" } else { "" },
                instruction_byte
            );
            panic!("Invalid instruction found for: {}", panic_description);
        };

        self.pc = next_pc;
    }
}

/// Struct to represent the memory bus.
/// It is an array that represents the memory of the RustBoy.
/// 0xFFFF = 65536 is the size of the memory in bytes
struct MemoryBus {
    memory: [u8; 0xFFFF],
}

impl MemoryBus {
    /// Read a byte from the memory at the given address.
    fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    /// Write a byte to the memory at the given address.
    fn write_byte(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }
}
