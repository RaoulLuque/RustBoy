//! CPU module
//! This module contains the CPU struct and its methods.
//! The execution of instructions is handled/implemented in the [instructions] module.

mod instructions;
mod memory_bus;
mod registers;

use instructions::Instruction;
use log::trace;
use memory_bus::MemoryBus;
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
    cycle_counter: u32,
    pub bus: MemoryBus,
}

impl CPU {
    /// Creates a new instance of the CPU struct.
    /// The registers are set to 0, the program counter (PC) is set to 0x0000,
    /// the stack pointer (SP) is set to 0xFFFE, and the cycle counter is set to 0.
    /// The memory bus is also initialized.
    pub fn new() -> CPU {
        CPU {
            registers: Registers::default(),
            pc: 0x0000,
            sp: 0xFFFE,
            cycle_counter: 0,
            bus: MemoryBus {
                memory: [0; 0xFFFF],
                bios: [0; 0x0100],
                starting_up: true,
            },
        }
    }

    /// Loads a program into the memory bus at address 0x0000.
    pub fn load_program(&mut self, program_directory: &str) {
        let program = std::fs::read(program_directory)
            .expect(&format!("Should be able to read file {program_directory}"));
        self.bus.load(0x0000, &program);
    }

    /// Runs the CPU.
    pub fn run(&mut self) {
        loop {
            self.step();
        }
    }

    /// Sets the stackpointer (SP) to the provided value.
    fn set_sp(&mut self, value: u16) {
        self.sp = value;
    }

    /// Increment the cycle counter by the provided value.
    fn increment_cycle_counter(&mut self, value: u32) {
        self.cycle_counter += value;
    }

    /// Reads the next instruction and executes it in the CPU.
    /// Doing so, the program counter (pc) is updated to point to the address of the next instruction.
    fn step(&mut self) {
        let mut instruction_byte = self.bus.read_instruction_byte(self.pc);

        // Check if the instruction is a CB instruction (prefix)
        let prefixed = instruction_byte == 0xCB;
        if prefixed {
            instruction_byte = self.bus.read_byte(self.pc.wrapping_add(1));
        }

        let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_byte, prefixed)
        {
            println!("Executing instruction: {:?} ", instruction);
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
