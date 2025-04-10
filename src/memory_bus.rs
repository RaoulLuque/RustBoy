//! This module contains the [MemoryBus] struct, which handles all reads and writes to the main
//! memory of the RustBoy.
//!
//! The main functionality is provided by [MemoryBus::read_byte] and [MemoryBus::write_byte],
//! which handle the reading and writing of bytes to the memory.

use crate::debugging::{DebugInfo, DebuggingFlagsWithoutFileHandles};
use crate::input::{ButtonState, Joypad};
use crate::interrupts::{InterruptEnableRegister, InterruptFlagRegister};
use crate::ppu::information_for_shader::ChangesToPropagateToShader;
use crate::ppu::tile_handling::{Tile, empty_tile};
use crate::{MEMORY_SIZE, PPU};

const ROM_BANK_0_BEGIN: u16 = 0x0000;
const ROM_BANK_0_END: u16 = 0x4000;
const BIOS_BEGIN: u16 = 0x0000;
const BIOS_END: u16 = 0x00FF;
const ROM_BANK_1_BEGIN: u16 = 0x4000;
const ROM_BANK_1_END: u16 = 0x8000;
pub const VRAM_BEGIN: u16 = 0x8000;
pub const VRAM_END: u16 = 0xA000;
pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFEA0;
const UNUSABLE_RAM_BEGIN: u16 = 0xFEA0;
const UNUSABLE_RAM_END: u16 = 0xFF00;
pub(crate) const JOYPAD_REGISTER: u16 = 0xFF00;
pub(crate) const INTERRUPT_FLAG_REGISTER: u16 = 0xFF0F;
pub(crate) const INTERRUPT_ENABLE_REGISTER: u16 = 0xFFFF;

/// Struct to represent the memory bus of the RustBoy.
///
/// - `memory`: An array representing the main memory of the RustBoy, with a size of [MEMORY_SIZE] bytes.
/// - `bios`: An array representing the BIOS of the RustBoy, used during startup instead of the
///     first 0x0100 bytes of memory.
/// - `being_initialized`: A flag indicating if the memory bus is being initialized.
///     During this phase, writes to certain registers (e.g., DMA Register) should not cause side effects.
/// - `starting_up`: A flag indicating if the RustBoy is in the startup phase, where the BIOS is
///     used instead of the ROM.
/// - `debugging_flags_without_file_handles`: Flags used for debugging purposes.
/// - `memory_changed`: Tracks changes to memory that need to be propagated to the shader for rendering.
/// - `tile_set`: An array of tiles representing the graphics data of the RustBoy.
///
/// For details on memory mapping and behavior, refer to [Pan Docs - Memory Map](https://gbdev.io/pandocs/Memory_Map.html)
/// and [Pan Docs - Hardware Registers](https://gbdev.io/pandocs/Hardware_Reg_List.html).
pub struct MemoryBus {
    /// An array representing the main memory of the RustBoy, with a size of [MEMORY_SIZE] bytes.
    pub memory: [u8; MEMORY_SIZE],
    bios: [u8; 0x0100],
    pub(crate) being_initialized: bool,
    pub(crate) starting_up: bool,

    pub(crate) debugging_flags_without_file_handles: DebuggingFlagsWithoutFileHandles,

    pub(crate) memory_changed: ChangesToPropagateToShader,

    // The following should be tried to get rid of
    pub(crate) tile_set: [Tile; 384],

    pub(crate) dma_happened: bool,

    pub(crate) action_button_state: ButtonState,
    pub(crate) direction_button_state: ButtonState,
}

impl MemoryBus {
    /// Loads a program into the memory bus at address 0x0000.
    pub fn load_program(&mut self, rom_data: &[u8]) {
        self.load(0x0000, &rom_data);
    }

    /// Reads the instruction byte from the memory at the given address. Used separately to check
    /// if the CPU is starting up.
    ///
    /// If the address is 0x0100 and the CPU is starting up, it returns the byte at that address.
    /// Otherwise, it just calls [MemoryBus::read_byte] returns the byte at the given address.
    pub(super) fn read_instruction_byte(&mut self, address: u16) -> u8 {
        if address == 0x0100 && self.starting_up {
            self.starting_up = false;
            self.memory[0x0100]
        } else {
            self.read_byte(address)
        }
    }

    /// Read a byte from memory at the given address.
    pub(super) fn read_byte(&self, address: u16) -> u8 {
        match address {
            ROM_BANK_0_BEGIN..ROM_BANK_0_END => {
                if self.starting_up {
                    match address {
                        BIOS_BEGIN..BIOS_END => self.bios[address as usize],
                        _ => self.memory[address as usize],
                    }
                } else {
                    self.memory[address as usize]
                }
            }
            ROM_BANK_1_BEGIN..ROM_BANK_1_END => self.memory[address as usize],

            VRAM_BEGIN..VRAM_END => self.memory[address as usize],
            OAM_START..OAM_END => self.memory[address as usize],
            UNUSABLE_RAM_BEGIN..UNUSABLE_RAM_END => {
                // When trying to read from unusable RAM, we return 0xFF
                0xFF
            }

            // Joypad register
            JOYPAD_REGISTER => Joypad::get_joypad_register(&self),

            // GPU registers
            0xFF40 | 0xFF41 | 0xFF42 | 0xFF43 | 0xFF44 | 0xFF45 | 0xFF47 | 0xFF48 | 0xFF49
            | 0xFF4A | 0xFF4B => PPU::read_registers(&self, address),

            // Interrupt flag register
            0xFF0F => InterruptFlagRegister::get_interrupt_flag_register(&self),

            // Interrupt enable register
            0xFFFF => InterruptEnableRegister::get_interrupt_enable_register(&self),

            _ => self.memory[address as usize],
        }
    }

    /// Write a byte to memory at the given address.
    pub(super) fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // TODO: Add Memory bank controller
            ROM_BANK_0_BEGIN..ROM_BANK_0_END => {
                // When trying to write to ROM, we just do nothing (for now)
            }
            ROM_BANK_1_BEGIN..ROM_BANK_1_END => {
                // When trying to write to ROM, we just do nothing (for now)
            }

            VRAM_BEGIN..VRAM_END => PPU::write_vram(self, address, value),
            OAM_START..OAM_END => self.memory[address as usize] = value,
            UNUSABLE_RAM_BEGIN..UNUSABLE_RAM_END => {
                // When trying to write to unusable RAM, we just do nothing
            }

            // Joypad register
            0xFF00 => Joypad::write_joypad_register(self, value),

            // GPU registers
            0xFF40 | 0xFF41 | 0xFF42 | 0xFF43 | 0xFF44 | 0xFF45 | 0xFF47 | 0xFF48 | 0xFF49
            | 0xFF4A | 0xFF4B => {
                PPU::write_registers(self, address, value);
            }

            // DMA transfer register
            0xFF46 => {
                // If the RustBoy and Memory is being initialized by the BIOS, we do not want to
                // trigger a DMA transfer
                if !self.being_initialized {
                    // The value written to the DMA register is the starting address of the transfer
                    // divided by 0x100 (= 256). The transfer takes 160 cycles.
                    self.handle_dma(value);
                }
            }

            // Serial transfer register
            0xFF01 => {
                if self.debugging_flags_without_file_handles.timing_mode {
                    if value as char == 'P' {
                        println!(
                            "Run took: {} seconds",
                            self.debugging_flags_without_file_handles
                                .start_time
                                .expect("Start time should be set")
                                .elapsed()
                                .as_micros() as f32
                                / 1_000_000f32
                        );
                    }
                }
                if self.debugging_flags_without_file_handles.sb_to_terminal {
                    println!("Write to SB: {}", value as char);
                }
                self.memory[address as usize] = value;
            }

            // Divider register
            0xFF04 => {
                // When a write happens to the divider register, it just resets to 0
                self.memory[address as usize] = 0;
            }

            // Interrupt flag register
            INTERRUPT_FLAG_REGISTER => {
                InterruptFlagRegister::set_interrupt_flag_register(self, value);
            }

            // Interrupt enable register
            INTERRUPT_ENABLE_REGISTER => {
                InterruptEnableRegister::set_interrupt_enable_register(self, value);
            }

            _ => {
                self.memory[address as usize] = value;
            }
        }
    }

    /// Reads the word (2 bytes) at the provided address from the memory in little endian order
    /// and returns the result. That is, the least significant byte is read first and then the address
    /// is incremented by 1 and the most significant byte is read.
    pub(super) fn read_word_little_endian(&self, address: u16) -> u16 {
        let low_byte = self.read_byte(address) as u16;
        let high_byte = self.read_byte(address + 1) as u16;
        (high_byte << 8) | low_byte
    }

    /// Reads the next word (2 bytes) from the memory in little endian order and returns the result.
    /// That is, the least significant byte is read first.
    pub(super) fn read_next_word_little_endian(&self, pc: u16) -> u16 {
        self.read_word_little_endian(pc + 1)
    }

    /// Writes data immediately to the memory at the given address.
    pub(super) fn load(&mut self, address: u16, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            self.memory[address as usize + i] = byte;
        }
    }

    /// The DMA transfer is started by writing to the DMA register at 0xFF46. The value written
    /// is the starting address of the transfer divided by 0x100 (= 256). The transfer takes 160
    /// cycles.
    ///
    /// TODO: Possibly split the dma into 40 individual writes each taking 4 cycles
    /// to simulate the transfer speed of the DMG.
    pub(crate) fn handle_dma(&mut self, address: u8) {
        if !self.debugging_flags_without_file_handles.binjgb_mode {
            // In the binjgb emulator, the DMA transfer does not seem to increment the cycle counter
            self.dma_happened = true;
        }
        let address = (address as u16) << 8;
        for i in 0..(OAM_END - OAM_START) + 1 {
            let value = self.read_byte(address + i);
            self.write_byte(OAM_START + i, value);
        }
    }

    /// Creates a new instance of the [MemoryBus] struct with the given [DebugInfo]. The memory,
    /// including the bios' memory, is set to 0 and the starting_up and being_initialized
    /// flags are set to true.
    pub fn new_before_boot(debug_info: &DebugInfo) -> Self {
        MemoryBus {
            memory: [0; 65536],
            bios: [0; 0x0100],
            starting_up: true,
            being_initialized: true,

            debugging_flags_without_file_handles:
                DebuggingFlagsWithoutFileHandles::from_debugging_flags(debug_info),

            memory_changed: ChangesToPropagateToShader::new_true(),

            tile_set: [empty_tile(); 384],

            dma_happened: false,

            action_button_state: ButtonState::new_nothing_pressed(),
            direction_button_state: ButtonState::new_nothing_pressed(),
        }
    }

    /// Returns a string representation of the memory bus.
    /// The string is rows of 8 bytes each.
    pub fn memory_to_string(&self) -> String {
        let mut string = String::new();
        string.push_str("MemoryBus: \n");
        for i in 0..self.memory.len() / 8 {
            if i == 4096 {
                string.push_str("End of ROM Bank reached \n");
                break;
            }
            if i % 2 == 0 {
                string.push_str("\n");
            }
            let tmp_string = format!(
                "{:#04X} {:#04X} {:#04X} {:#04X} {:#04X} {:#04X} {:#04X} {:#04X} ",
                self.memory[i],
                self.memory[i + 1],
                self.memory[i + 2],
                self.memory[i + 3],
                self.memory[i + 4],
                self.memory[i + 5],
                self.memory[i + 6],
                self.memory[i + 7]
            );
            string.push_str(&tmp_string);
        }
        string.push('\n');
        string
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
