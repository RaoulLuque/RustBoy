//! CPU module
//! This module contains the CPU struct and its methods.
//! The execution of instructions is handled/implemented in the [instructions] module.

pub(crate) mod instructions;
mod memory_bus;
pub mod registers;

use crate::RustBoy;
use crate::debugging::LOG_FILE_NAME;
#[cfg(debug_assertions)]
use crate::debugging::{doctor_log, instruction_log};
use crate::memory_bus::{OAM_END, OAM_START};
use instructions::Instruction;

impl RustBoy {
    /// Loads a program into the memory bus at address 0x0000.
    pub fn load_program(&mut self, program_directory: &str) {
        let program = std::fs::read(program_directory)
            .expect(&format!("Should be able to read file {program_directory}"));
        self.load(0x0000, &program);
    }

    /// Sets the stackpointer (SP) to the provided value.
    fn set_sp(&mut self, value: u16) {
        self.sp = value;
    }

    /// Increment the cycle counter by the provided value.
    pub fn increment_cycle_counter(&mut self, value: u32) {
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
        }
        #[cfg(debug_assertions)]
        if self.debugging_flags.file_logs {
            doctor_log(&self, LOG_FILE_NAME)
        }

        // Check if an interrupt needs to be handled. If so, Some(u16) is returned with the
        // interrupt location. If no interrupt is requested, None is returned.
        // If an interrupt is requested, the corresponding bit in the interrupt flag register
        // and the IME (Interrupt Master Enable) flag are set to 0.
        if let Some(interrupt_location) = self.check_if_interrupt_is_requested() {
            // Log the interrupt location
            log::trace!("Interrupt requested at: 0x{:04X}", interrupt_location);

            // The flag register and IME (Interrupt Master Enable) flag are already set to 0 by
            // the check_if_interrupt_is_requested function, so we don't need to do it again here.

            // Push the current program counter (PC) onto the stack and set the program counter to
            // the interrupt location.
            self.push(self.pc);
            self.pc = interrupt_location;
            self.increment_cycle_counter(5);
        } else {
            // No interrupt was requested, so we can continue executing instructions.
            // Except if the cpu is halted, in which case need to check if an interrupt is requested
            // and if so, go out of halt mode.

            // We use the following flag to track the halt bug. That is, if the IME flag is set to 0
            // and the CPU is in halt mode and an interrupt is requested, the CPU will go out of halt,
            // but the next instruction will be executed twice instead of once, which we simulate
            // by not setting the new program counter (PC) to the next instruction.
            // See [Pan Docs](https://gbdev.io/pandocs/halt.html#halt-bug)
            let mut halt_bug = false;

            if self.halted {
                // Check if an interrupt is requested. If so, go out of halt mode.
                if u8::from(&self.interrupt_flag_register)
                    & u8::from(&self.interrupt_enable_register)
                    != 0
                {
                    // The cpu wakes up from halt mode and the next instruction is executed twice
                    // due to the halt bug
                    // TODO: Handle edge cases of the halt bug, see https://gbdev.io/pandocs/halt.html#halt-bug
                    self.halted = false;
                    halt_bug = true;
                    self.increment_cycle_counter(1);
                } else {
                    // If no interrupt is requested, just increment the cycle counter and return.
                    self.increment_cycle_counter(1);
                    return;
                }
            }

            let mut instruction_byte = self.read_instruction_byte(self.pc);

            // Check if the instruction is a CB instruction (prefix)
            let prefixed = instruction_byte == 0xCB;
            if prefixed {
                instruction_byte = self.read_byte(self.pc.wrapping_add(1));
            }

            let next_pc =
                if let Some(instruction) = Instruction::from_byte(instruction_byte, prefixed) {
                    // Log the instruction byte if in debug mode.
                    #[cfg(debug_assertions)]
                    if self.debugging_flags.file_logs {
                        instruction_log(&self, LOG_FILE_NAME, instruction);
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

            if !halt_bug {
                self.pc = next_pc;
            }
        }
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

    /// The DMA transfer is started by writing to the DMA register at 0xFF46. The value written
    /// is the starting address of the transfer divided by 0x100 (= 256). The transfer takes 160
    /// cycles.
    ///
    /// TODO: Possibly split the copy instruction into 40 individual writes each taking 4 cycles
    /// to simulate the transfer speed of the DMG.
    pub(crate) fn handle_dma(&mut self, address: u8) {
        let address = (address as u16) << 8;
        for i in 0..(OAM_END - OAM_START) + 1 {
            let value = self.read_byte(address + i);
            self.write_byte(OAM_START + i, value);
        }

        self.increment_cycle_counter(160);
    }
}
