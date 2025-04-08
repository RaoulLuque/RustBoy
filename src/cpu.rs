//! CPU module
//! This module contains the CPU struct and its methods.
//! The execution of instructions is handled/implemented in the [instructions] module.

pub(crate) mod instructions;
mod memory_bus;
pub mod registers;

use crate::cpu::registers::CPURegisters;
use crate::debugging::{DebugInfo, LOG_FILE_NAME};
#[cfg(debug_assertions)]
use crate::debugging::{doctor_log, instruction_log};
use crate::interrupts::{InterruptEnableRegister, InterruptFlagRegister};
use crate::{MemoryBus, PPU};
use instructions::Instruction;

/// Struct to represent the CPU of the RustBoy.
///
/// The CPU has 8 registers, a program counter (PC), a stack pointer (SP), and a memory bus.
/// For details please refer to [Pan Docs](https://gbdev.io/pandocs/CPU_Registers_and_Flags.html).
/// The CPU also has a cycle counter as well as a cycles_current_instruction field to keep track
/// of the number of cycles executed.
///
/// Additionally, the CPU has an interrupt master enable (IME) flag to control the handling of
/// interrupts, see [Pan Docs](https://gbdev.io/pandocs/Interrupts.html). ime_to_be_set is used
/// to set the IME flag after the current instruction is executed, which is necessary for the
/// correct execution of the EI instruction.
///
/// In addition to the IME flag, the CPU has a halted flag to indicate if the CPU is halted.
/// See [Pan Docs](https://gbdev.io/pandocs/halt.html#halt) for more information on this.
///
/// For implementations of the CPU instructions, please see [instructions].
pub struct CPU {
    /// The 8 registers of the CPU.
    pub registers: CPURegisters,
    /// The program counter (PC) of the CPU.
    pub pc: u16,
    /// The stack pointer (SP) of the CPU.
    pub sp: u16,
    cycle_counter: u64,
    pub(crate) cycles_current_instruction: Option<u8>,
    pub(crate) ime: bool,
    ime_to_be_set: bool,
    halted: bool,
    just_entered_halt: bool,

    // Debugging Flags
    pub(crate) debugging_flags: DebugInfo,
}

impl CPU {
    /// Sets the stack pointer (SP) to the provided value.
    fn set_sp(&mut self, value: u16) {
        self.sp = value;
    }

    /// Increment the cycle counter by the provided value.
    pub fn increment_cycle_counter(&mut self, value: u32) {
        self.cycle_counter += value as u64;
        self.cycles_current_instruction = match self.cycles_current_instruction {
            Some(cycles) => Some(cycles + value as u8),
            None => Some(value as u8),
        };
    }

    /// Reads the next instruction and executes it in the CPU.
    /// Doing so, the program counter (pc) is updated to point to the address of the next instruction.
    ///
    /// Also handles interrupts and the halt mode of the CPU. This method is called in a loop
    /// alternating with [crate::PPU::ppu_step].
    pub fn cpu_step(&mut self, memory_bus: &mut MemoryBus, ppu: &PPU) {
        // Log the current state of the registers if in debug mode. Don't want all this in release
        // builds, which is why we use the cfg conditional compilation feature.
        #[cfg(debug_assertions)]
        if !self.halted {
            // We only log the current state right after an instruction is executed, so we don't
            // have to log the state of the registers if we are in halt mode.
            if self.debugging_flags.doctor {
                doctor_log(self, memory_bus, ppu, "doctor");
            }
            if self.debugging_flags.file_logs {
                doctor_log(self, memory_bus, ppu, LOG_FILE_NAME)
            }
        }

        // This variable tracks if an interrupt was requested, to possibly override the check
        // whether to step out of halt mode.
        let mut interrupt_requested = false;

        // Check if an interrupt needs to be handled. If so, Some(u16) is returned with the
        // interrupt location. If no interrupt is requested, None is returned.
        // If an interrupt is requested, the corresponding bit in the interrupt flag register
        // and the IME (Interrupt Master Enable) flag are set to 0.
        if let Some(interrupt_location) = self.check_if_interrupt_is_requested(memory_bus) {
            // The flag register and IME (Interrupt Master Enable) flag are already set to 0 by
            // the check_if_interrupt_is_requested function, so we don't need to do it again here.

            // Push the current program counter (PC) onto the stack and set the program counter to
            // the interrupt location.
            self.push(memory_bus, self.pc);
            self.pc = interrupt_location;
            self.increment_cycle_counter(5);

            // Set flag that interrupt was requested
            interrupt_requested = true;

            // Log the interrupt if in debug mode
            #[cfg(debug_assertions)]
            if self.debugging_flags.file_logs {
                instruction_log(
                    &self,
                    memory_bus,
                    LOG_FILE_NAME,
                    None,
                    Some(interrupt_location),
                );
            }
        }

        // No interrupt was requested, so we can continue executing instructions.
        // Except if the cpu is halted, in which case need to check if an interrupt is requested
        // and if so, go out of halt mode.

        // We use the following flag to track the halt bug. That is, if the IME flag is set to 0
        // and the CPU just entered halt mode and an interrupt both is requested and enabled, the
        // CPU will go out of halt, but the next instruction will be executed twice instead of once,
        // which we simulate by not setting the new program counter (PC) to the next instruction.
        // See [Pan Docs](https://gbdev.io/pandocs/halt.html#halt-bug)
        let mut halt_bug = false;

        if self.halted {
            // Check if an interrupt is requested. If so, go out of halt mode.
            if InterruptFlagRegister::get_interrupt_flag_register(memory_bus)
                & InterruptEnableRegister::get_interrupt_enable_register(memory_bus)
                != 0
                || interrupt_requested
            {
                // The cpu wakes up from halt mode and the next instruction is executed twice
                // due to the halt bug
                // TODO: Handle edge cases of the halt bug, see https://gbdev.io/pandocs/halt.html#halt-bug
                self.halted = false;
                if self.just_entered_halt {
                    halt_bug = true;
                }
                self.increment_cycle_counter(1);

                // Log the current state of the registers if in debug mode. Don't want all this in release
                // builds, which is why we use the cfg conditional compilation feature.
                #[cfg(debug_assertions)]
                {
                    // We are leaving halt mode, so we log the current state of the registers
                    if self.debugging_flags.doctor {
                        doctor_log(self, memory_bus, ppu, "doctor");
                    }
                    if self.debugging_flags.file_logs {
                        doctor_log(self, memory_bus, ppu, LOG_FILE_NAME)
                    }
                }
            } else {
                // If no interrupt is requested, just increment the cycle counter and return.
                self.increment_cycle_counter(1);

                // We also set the just_entered_halt flag to false, so that we don't trigger the halt
                // bug, because it just triggers if the cpu just entered halt mode.
                self.just_entered_halt = false;

                return;
            }
        }

        let mut instruction_byte = memory_bus.read_instruction_byte(self.pc);

        // Check if the instruction is a CB instruction (prefix)
        let prefixed = instruction_byte == 0xCB;
        if prefixed {
            instruction_byte = memory_bus.read_byte(self.pc.wrapping_add(1));
        }

        let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_byte, prefixed)
        {
            // Log the instruction byte if in debug mode.
            #[cfg(debug_assertions)]
            if self.debugging_flags.file_logs {
                instruction_log(&self, memory_bus, LOG_FILE_NAME, Some(instruction), None);
            }

            self.execute(memory_bus, instruction)
        } else {
            let panic_description = format!(
                "0x{}{:02x}",
                if prefixed { "CB" } else { "" },
                instruction_byte
            );
            panic!("Invalid instruction found for: {}", panic_description);
        };

        if memory_bus.dma_happened && !self.debugging_flags.binjgb_mode {
            self.increment_cycle_counter(160);
            memory_bus.dma_happened = false;
        }

        if !halt_bug {
            self.pc = next_pc;
        }
    }

    pub fn new_before_boot_rom(debugging_flags: DebugInfo) -> Self {
        CPU {
            registers: CPURegisters::new_zero(),
            pc: 0x0000,
            sp: 0xFFFE,
            cycle_counter: 0,
            cycles_current_instruction: None,
            ime: false,
            ime_to_be_set: false,
            halted: false,
            just_entered_halt: false,
            debugging_flags,
        }
    }

    /// Initializes the hardware registers to their default values after the boot rom ran.
    /// See [Pan Docs](https://gbdev.io/pandocs/Power_Up_Sequence.html#obp)
    pub(crate) fn initialize_hardware_registers(memory_bus: &mut MemoryBus) {
        memory_bus.write_byte(0xFF00, 0xCF);
        memory_bus.write_byte(0xFF01, 0x00);
        memory_bus.write_byte(0xFF02, 0x7E);
        memory_bus.write_byte(0xFF04, 0xAB);
        memory_bus.write_byte(0xFF05, 0x00);
        memory_bus.write_byte(0xFF06, 0x00);
        memory_bus.write_byte(0xFF07, 0xF8);
        memory_bus.write_byte(0xFF0F, 0xE1);
        memory_bus.write_byte(0xFF10, 0x80);
        memory_bus.write_byte(0xFF11, 0xBF);
        memory_bus.write_byte(0xFF12, 0xF3);
        memory_bus.write_byte(0xFF13, 0xFF);
        memory_bus.write_byte(0xFF14, 0xBF);
        memory_bus.write_byte(0xFF16, 0x3F);
        memory_bus.write_byte(0xFF17, 0x00);
        memory_bus.write_byte(0xFF19, 0xBF);
        memory_bus.write_byte(0xFF1A, 0x7F);
        memory_bus.write_byte(0xFF1B, 0xFF);
        memory_bus.write_byte(0xFF1C, 0x9F);
        memory_bus.write_byte(0xFF1D, 0xFF);
        memory_bus.write_byte(0xFF1E, 0xBF);
        memory_bus.write_byte(0xFF20, 0xFF);
        memory_bus.write_byte(0xFF21, 0x00);
        memory_bus.write_byte(0xFF22, 0x00);
        memory_bus.write_byte(0xFF23, 0xBF);
        memory_bus.write_byte(0xFF24, 0x77);
        memory_bus.write_byte(0xFF25, 0xF3);
        memory_bus.write_byte(0xFF26, 0xF1);
        memory_bus.write_byte(0xFF40, 0x91);
        memory_bus.write_byte(0xFF41, 0x85);
        memory_bus.write_byte(0xFF42, 0x00);
        memory_bus.write_byte(0xFF43, 0x00);
        memory_bus.write_byte(0xFF44, 0x00);
        memory_bus.write_byte(0xFF45, 0x00);
        memory_bus.write_byte(0xFF46, 0xFF);
        memory_bus.write_byte(0xFF47, 0xFC);
        memory_bus.write_byte(0xFF4A, 0x00);
        memory_bus.write_byte(0xFF4B, 0x00);
        memory_bus.write_byte(0xFFFF, 0x00);
    }
}

/// Checks if the bit at the given position is set in the given value.
pub fn is_bit_set(value: u8, bit_position: u8) -> bool {
    (value & (1 << bit_position)) != 0
}

/// Sets the bit at the given position in the given value.
pub fn set_bit(value: u8, bit_position: u8) -> u8 {
    value | (1 << bit_position)
}

/// Clears the bit at the given position in the given value.
pub fn clear_bit(value: u8, bit_position: u8) -> u8 {
    value & !(1 << bit_position)
}
