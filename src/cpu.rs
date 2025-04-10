//! This module contains the CPU struct and its methods.
//! The execution of instructions is handled/implemented in the [instructions] (sub-)module.

pub(crate) mod instructions;
pub mod registers;

use crate::cpu::registers::CPURegisters;
use crate::debugging::{DebugInfo, LOG_FILE_NAME};
#[cfg(debug_assertions)]
use crate::debugging::{doctor_log_helper, instruction_log};
use crate::interrupts::{InterruptEnableRegister, InterruptFlagRegister};
use crate::{MemoryBus, PPU};
use instructions::Instruction;

/// Struct to represent the CPU of the RustBoy.
///
/// - `registers`: The 8 general-purpose registers of the CPU, including the accumulator and flag register.
///     For details, refer to [Pan Docs - CPU Registers and Flags](https://gbdev.io/pandocs/CPU_Registers_and_Flags.html).
/// - `pc`: The program counter, which points to the address of the next instruction to be executed.
/// - `sp`: The stack pointer, which points at the top of the stack. Note that the stack grows downwards.
/// - `cycle_counter`: A counter to track the total number of cycles executed by the CPU.
/// - `cycles_current_instruction`: Tracks the number of cycles taken by the current instruction being executed.
/// - `ime`: The interrupt master enable (IME) flag, which controls whether interrupts are enabled or disabled.
///     See [Pan Docs - Interrupts](https://gbdev.io/pandocs/Interrupts.html) for more details.
/// - `ime_to_be_set`: A flag used to set the IME flag after the current instruction is executed,
///     necessary for the correct execution of the EI instruction.
/// - `halted`: Indicates whether the CPU is in a halted state. See [Pan Docs - Halt](https://gbdev.io/pandocs/halt.html#halt).
/// - `just_entered_halt`: A flag to track if the CPU has just entered the halt state, used to handle the halt bug.
///     See [Pan Docs - Halt Bug](https://gbdev.io/pandocs/halt.html#halt-bug) for more details.
/// - `debugging_flags`: Flags used for debugging purposes, such as logging the state of the CPU.
///
/// For implementations of the CPU instructions, please see [instructions].
pub struct CPU {
    /// The 8 general-purpose registers of the CPU, including the accumulator and flag register.
    /// For details, refer to [Pan Docs - CPU Registers and Flags](https://gbdev.io/pandocs/CPU_Registers_and_Flags.html).
    pub registers: CPURegisters,
    /// The program counter, which points to the address of the next instruction to be executed.
    pub pc: u16,
    /// The stack pointer, which points at the top of the stack. Note that the stack grows downwards.
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
    /// Sets the stack pointer (sp) to the provided value.
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
    ///
    /// Needs access to the memory bus to read the instruction byte, execute it and possibly change
    /// memory during execution of the instruction.
    pub fn cpu_step(&mut self, memory_bus: &mut MemoryBus, ppu: &PPU) {
        // Log the current state of the registers if in debug mode.
        #[cfg(debug_assertions)]
        if !self.halted {
            // We only log the current state right after an instruction is executed, so we don't
            // have to log the state of the registers if we are in halt mode.
            doctor_log_helper(
                self,
                memory_bus,
                ppu,
                "doctor",
                self.debugging_flags.doctor,
                self.debugging_flags.file_logs,
            );
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

        // We use the following flag to track the halt bug. That is, if the IME flag is set to 0
        // and the CPU just entered halt mode and an interrupt both is requested and enabled, the
        // CPU will go out of halt, but the next instruction will be executed twice instead of once,
        // which we simulate by not setting the new program counter (PC) to the next instruction.
        // See [Pan Docs - Halt Bug](https://gbdev.io/pandocs/halt.html#halt-bug)
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

                // Log the current state of the registers if in debug mode.
                #[cfg(debug_assertions)]
                doctor_log_helper(
                    self,
                    memory_bus,
                    ppu,
                    "doctor",
                    self.debugging_flags.doctor,
                    self.debugging_flags.file_logs,
                );
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

    /// Creates a new CPU instance with all registers and flags set to 0 and/or false. The debugging
    /// flags are set to the provided value.
    ///
    /// This is used to initialize the CPU to a state it is in before the boot ROM in executed.
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
    /// See [Pan Docs - Power up Sequence](https://gbdev.io/pandocs/Power_Up_Sequence.html#obp)
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
