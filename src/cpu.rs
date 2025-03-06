//! CPU module
//! This module contains the CPU struct and its methods.
//! The execution of instructions is handled/implemented in the [instructions] module.

pub(crate) mod instructions;
mod memory_bus;
pub mod registers;

use super::GPU;
#[cfg(debug_assertions)]
use crate::debugging::{doctor_log, instruction_log};
use crate::RustBoy;
use instructions::Instruction;
use registers::CPURegisters;

impl RustBoy {
    /// Loads a program into the memory bus at address 0x0000.
    pub fn load_program(&mut self, program_directory: &str) {
        let program = std::fs::read(program_directory)
            .expect(&format!("Should be able to read file {program_directory}"));
        self.load(0x0000, &program);
    }

    /// Runs the CPU.
    pub fn run(&mut self) {
        loop {
            self.cpu_step();
        }
    }

    /// Sets the stackpointer (SP) to the provided value.
    fn set_sp(&mut self, value: u16) {
        self.sp = value;
    }

    /// Increment the cycle counter by the provided value.
    fn increment_cycle_counter(&mut self, value: u32) {
        self.cycle_counter += value as u64;
    }

    /// Reads the next instruction and executes it in the CPU.
    /// Doing so, the program counter (pc) is updated to point to the address of the next instruction.
    pub fn cpu_step(&mut self) {
        // Log the current state of the registers if in debug mode. Don't want all this in release
        // builds, which is why we use the cfg conditional compilation feature.
        #[cfg(debug_assertions)]
        if self.debugging_flags.doctor {
            doctor_log(&self, "doctor");
            doctor_log(&self, "doctors_augmented")
        }

        let mut instruction_byte = self.read_instruction_byte(self.pc);

        // Check if the instruction is a CB instruction (prefix)
        let prefixed = instruction_byte == 0xCB;
        if prefixed {
            instruction_byte = self.read_byte(self.pc.wrapping_add(1));
        }

        let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_byte, prefixed)
        {
            // Log the instruction byte if in debug mode.
            #[cfg(debug_assertions)]
            if self.debugging_flags.doctor {
                instruction_log(&self, "doctors_augmented", instruction);
            }

            log::trace!("Executing instruction: {:?} ", instruction);
            self.execute(instruction)
        } else {
            let panic_description = format!(
                "0x{}{:02x}",
                if prefixed { "CB" } else { "" },
                instruction_byte
            );
            panic!("Invalid instruction found for: {}", panic_description);
        };

        self.pc = next_pc;
    }

    /// Initializes the hardware registers to their default values.
    /// See [Pan Docs](https://gbdev.io/pandocs/Power_Up_Sequence.html#obp)
    pub(crate) fn initialize_hardware_registers(&mut self) {
        self.write_byte(0xFF00, 0xCF);
        self.write_byte(0xFF01, 0x00);
        self.write_byte(0xFF02, 0x7E);
        self.write_byte(0xFF04, 0xAB);
        self.write_byte(0xFF05, 0x00);
        self.write_byte(0xFF06, 0x00);
        self.write_byte(0xFF07, 0xF8);
        self.write_byte(0xFF0F, 0xE1);
        self.write_byte(0xFF10, 0x80);
        self.write_byte(0xFF11, 0xBF);
        self.write_byte(0xFF12, 0xF3);
        self.write_byte(0xFF13, 0xFF);
        self.write_byte(0xFF14, 0xBF);
        self.write_byte(0xFF16, 0x3F);
        self.write_byte(0xFF17, 0x00);
        self.write_byte(0xFF19, 0xBF);
        self.write_byte(0xFF1A, 0x7F);
        self.write_byte(0xFF1B, 0xFF);
        self.write_byte(0xFF1C, 0x9F);
        self.write_byte(0xFF1D, 0xFF);
        self.write_byte(0xFF1E, 0xBF);
        self.write_byte(0xFF20, 0xFF);
        self.write_byte(0xFF21, 0x00);
        self.write_byte(0xFF22, 0x00);
        self.write_byte(0xFF23, 0xBF);
        self.write_byte(0xFF24, 0x77);
        self.write_byte(0xFF25, 0xF3);
        self.write_byte(0xFF26, 0xF1);
        self.write_byte(0xFF40, 0x91);
        self.write_byte(0xFF41, 0x85);
        self.write_byte(0xFF42, 0x00);
        self.write_byte(0xFF43, 0x00);
        self.write_byte(0xFF44, 0x00);
        self.write_byte(0xFF45, 0x00);
        self.write_byte(0xFF46, 0xFF);
        self.write_byte(0xFF47, 0xFC);
        self.write_byte(0xFF4A, 0x00);
        self.write_byte(0xFF4B, 0x00);
        self.write_byte(0xFFFF, 0x00);
    }
}
