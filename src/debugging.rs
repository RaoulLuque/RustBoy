//! This module contains the debugging functions for the RustBoy emulator.
//! It provides utilities for logging, debugging, and inspecting the state of the emulator.
//! The functions and structs in this module are primarily used during development and testing.
use wasm_timer::Instant;

use crate::interrupts::{InterruptEnableRegister, InterruptFlagRegister};
use crate::ppu::registers::{LCDCRegister, PPURegisters};
use crate::ppu::tile_handling::{Tile, TilePixelValue};
use crate::{CPU, MemoryBus, PPU};
use std::fs;
use std::io::Write;

pub const LOG_FILE_NAME: &str = "extensive_logs";

/// Struct to represent the debugging information/flags.
/// This struct contains various flags and handles used for debugging the emulator.
///
/// Fields:
/// - `file_handle_doctor_logs`: Optional file handle for writing doctor logs.
/// - `file_handle_extensive_logs`: Optional file handle for writing extensive logs.
/// - `log_file_index`: Index of the current log file.
/// - `current_number_of_lines_in_log_file`: Number of lines written to the current log file.
/// - `doctor`: Flag indicating if the emulator runs in Game Boy Doctor compatible mode.
/// - `file_logs`: Flag indicating if logs should be written to a file.
/// - `binjgb_mode`: Flag indicating if the emulator runs in binjgb mode.
/// - `timing_mode`: Flag indicating if the emulator runs in timing mode.
/// - `start_time`: Optional start time of the emulator, used in timing mode.
/// - `sb_to_terminal`: Flag indicating if serial output should be printed to the terminal.
#[derive(Debug)]
pub struct DebugInfo {
    pub file_handle_doctor_logs: Option<std::fs::File>,
    pub file_handle_extensive_logs: Option<std::fs::File>,
    pub log_file_index: u8,
    pub current_number_of_lines_in_log_file: u32,
    pub doctor: bool,
    pub file_logs: bool,
    pub binjgb_mode: bool,
    pub timing_mode: bool,
    pub start_time: Option<Instant>,
    pub sb_to_terminal: bool,
}

/// Struct to represent the debugging information/flags. This struct is similar to [DebugInfo],
/// but does not contain handles to the log files, which makes it easier to pass around.
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub struct DebuggingFlagsWithoutFileHandles {
    pub doctor: bool,
    pub file_logs: bool,
    pub binjgb_mode: bool,
    pub timing_mode: bool,
    pub start_time: Option<Instant>,
    pub sb_to_terminal: bool,
}

impl DebuggingFlagsWithoutFileHandles {
    pub fn from_debugging_flags(debugging_flags: &DebugInfo) -> Self {
        Self {
            doctor: debugging_flags.doctor,
            file_logs: debugging_flags.file_logs,
            binjgb_mode: debugging_flags.binjgb_mode,
            timing_mode: debugging_flags.timing_mode,
            start_time: debugging_flags.start_time,
            sb_to_terminal: debugging_flags.sb_to_terminal,
        }
    }
}

/// Sets up the debugging log files for the emulator.
/// This function creates the necessary log directory and initializes file handles
/// for doctor logs and extensive logs based on the current log file index.
#[cfg(debug_assertions)]
pub fn setup_debugging_logs_files(debugging_flags: &mut DebugInfo) {
    let log_file_index = debugging_flags.log_file_index;

    // Create the log directory if it doesn't exist
    fs::create_dir_all("logs").unwrap();

    let log_file_paths = [
        format!("logs/doctor_{log_file_index}.log"),
        format!("logs/{LOG_FILE_NAME}_{log_file_index}.log"),
    ];
    for path in log_file_paths {
        if path == format!("logs/doctor_{log_file_index}.log") {
            debugging_flags.file_handle_doctor_logs = Some(
                fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(&path)
                    .expect(&format!("{} File should be openable", &path)),
            );
        } else {
            debugging_flags.file_handle_extensive_logs = Some(
                fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(&path)
                    .expect(&format!("{} File should be openable", &path)),
            );
        }
    }
}

/// Helper function to log debugging information. Calls [doctor_log] for [LOG_FILE_NAME] and a provided log file name
#[cfg(debug_assertions)]
pub fn doctor_log_helper(
    cpu: &mut CPU,
    memory_bus: &MemoryBus,
    ppu: &PPU,
    log_file: &str,
    doctor_flag: bool,
    file_logs_flag: bool,
) {
    if doctor_flag {
        doctor_log(cpu, memory_bus, ppu, log_file);
    }
    if file_logs_flag {
        doctor_log(cpu, memory_bus, ppu, LOG_FILE_NAME)
    }
}

/// Logs the state of the emulator to a log file.
/// This function writes detailed debugging information about the CPU, memory, and PPU state
/// to the specified log file. It is only included in debug builds.
#[cfg(debug_assertions)]
pub fn doctor_log(cpu: &mut CPU, memory_bus: &MemoryBus, ppu: &PPU, log_file: &str) {
    let mut data = format!(
        "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}\n",
        cpu.registers.a,
        cpu.registers.f.get(),
        cpu.registers.b,
        cpu.registers.c,
        cpu.registers.d,
        cpu.registers.e,
        cpu.registers.h,
        cpu.registers.l,
        cpu.sp,
        cpu.pc,
        memory_bus.read_byte(cpu.pc),
        memory_bus.read_byte(cpu.pc.wrapping_add(1)),
        memory_bus.read_byte(cpu.pc.wrapping_add(2)),
        memory_bus.read_byte(cpu.pc.wrapping_add(3))
    );

    if log_file == LOG_FILE_NAME {
        data.pop();
        data.push_str(&format!(
            " SPMEM:{:02X},{:02X},{:02X},{:02X},CURR:{:02X},{:02X},{:02X},{:02X},{:02X}",
            memory_bus.read_byte(cpu.sp.saturating_sub(4)),
            memory_bus.read_byte(cpu.sp.saturating_sub(3)),
            memory_bus.read_byte(cpu.sp.saturating_sub(2)),
            memory_bus.read_byte(cpu.sp.saturating_sub(1)),
            memory_bus.read_byte(cpu.sp),
            memory_bus.read_byte(cpu.sp.saturating_add(1)),
            memory_bus.read_byte(cpu.sp.saturating_add(2)),
            memory_bus.read_byte(cpu.sp.saturating_add(3)),
            memory_bus.read_byte(cpu.sp.saturating_add(4)),
        ));

        let stat_register = PPURegisters::get_lcd_status(memory_bus);
        data.push_str(&format!(" STAT:{:<08b}", stat_register));

        let lyc = PPURegisters::get_scanline_compare(memory_bus);
        data.push_str(&format!(" LYC:{:<3}", lyc));

        let ppu_mode_as_u8 = PPURegisters::get_ppu_mode(memory_bus).as_u8();
        let ppu_mode_sign = if PPURegisters::get_lcd_control(memory_bus) & 0b1000_0000 != 0 {
            "+"
        } else {
            "-"
        };
        data.push_str(&format!(" PPU:{}{}", ppu_mode_sign, ppu_mode_as_u8));

        // We use the get scanline internal function to get the current scanline immediately
        // from the memory without any additional sync checks done for syncing GPU and CPU.
        let current_scanline = PPURegisters::get_scanline_internal(memory_bus);
        data.push_str(&format!(" SCANLINE:{:<3}", current_scanline));

        data.push_str(&format!(" IME:{}", u8::from(cpu.ime)));
        data.push_str(&format!(
            " IF:{:02X}",
            InterruptFlagRegister::get_interrupt_flag_register(memory_bus)
        ));
        data.push_str(&format!(
            " IE:{:02X}",
            InterruptEnableRegister::get_interrupt_enable_register(memory_bus)
        ));

        let cycles_in_dots: u32 = ppu.rendering_info.dots_clock;
        data.push_str(&format!(" CY_DOTS:{:<3}", cycles_in_dots));

        let total_cycles: u128 = ppu.rendering_info.total_dots;
        data.push_str(&format!(" TOTAL_CY_DOTS:{:<10}\n", total_cycles));
    }
    if log_file == "doctor" {
        cpu.debugging_flags
            .file_handle_doctor_logs
            .as_ref()
            .expect("Doctor log file handle should be created")
            .write_all(data.as_bytes())
            .expect("Should be able to write data to doctor log file");
    } else {
        cpu.debugging_flags.current_number_of_lines_in_log_file += 1;
        if cpu.debugging_flags.current_number_of_lines_in_log_file == 2_000_000 {
            cpu.debugging_flags.current_number_of_lines_in_log_file = 0;
            cpu.debugging_flags.log_file_index += 1;
            setup_debugging_logs_files(&mut cpu.debugging_flags);
        }
        cpu.debugging_flags
            .file_handle_extensive_logs
            .as_ref()
            .expect("Doctor log file handle should be created")
            .write_all(data.as_bytes())
            .expect("Should be able to write data to doctor log file");
    }
}

/// Log the instruction as a pretty string to the provided log file.
#[cfg(debug_assertions)]
pub fn instruction_log(
    cpu: &CPU,
    memory_bus: &MemoryBus,
    log_file: &str,
    instruction: Option<crate::cpu::instructions::Instruction>,
    interrupt_location: Option<u16>,
) {
    let data = if let Some(instruction) = instruction {
        format!(
            "{:<50}",
            entire_instruction_to_string(cpu, memory_bus, instruction)
        )
    } else if let Some(interrupt_location) = interrupt_location {
        format!(
            "{:<50}",
            format!(
                "Interrupt: {}",
                push_match_interrupt_location_to_interrupt_name(interrupt_location)
                    .expect("Should be valid interrupt that is being called")
            )
        )
    } else {
        format!("{:<50}", "No instruction")
    };

    if log_file == "doctor" {
        cpu.debugging_flags
            .file_handle_doctor_logs
            .as_ref()
            .expect("Doctor log file handle should be created")
            .write_all(data.as_bytes())
            .expect("Should be able to write data to doctor log file");
    } else {
        cpu.debugging_flags
            .file_handle_extensive_logs
            .as_ref()
            .expect("Doctor log file handle should be created")
            .write_all(data.as_bytes())
            .expect("Should be able to write data to doctor log file");
    }
}

/// Match the instruction to the length of the instruction to copy its entire bytes
#[cfg(debug_assertions)]
pub fn entire_instruction_to_string(
    cpu: &CPU,
    memory_bus: &MemoryBus,
    instruction: crate::cpu::instructions::Instruction,
) -> String {
    use crate::cpu::instructions::add_and_adc::AddWordSource;
    use crate::cpu::instructions::load::{LoadType, LoadWordSource};
    use crate::cpu::instructions::*;
    let mut res = format!("{:?}", instruction);
    match instruction {
        Instruction::ADDByte(source)
        | Instruction::ADC(source)
        | Instruction::SUB(source)
        | Instruction::SBC(source)
        | Instruction::AND(source)
        | Instruction::OR(source)
        | Instruction::XOR(source)
        | Instruction::CP(source) => match source {
            ArithmeticOrLogicalSource::D8 => {
                push_next_immediate_byte_as_hex_to_string(cpu, memory_bus, &mut res)
            }
            _ => {}
        },
        Instruction::ADDWord(_, source) => match source {
            AddWordSource::E8 => {
                push_next_immediate_byte_as_hex_to_string(cpu, memory_bus, &mut res);
            }
            _ => {}
        },
        Instruction::LD(load_type) => match load_type {
            LoadType::Byte(_, _) => {}
            LoadType::Word(target, source) => {
                match target {
                    _ => {}
                };
                match source {
                    LoadWordSource::D16 => {
                        push_next_two_immediate_bytes_to_string(cpu, memory_bus, &mut res);
                    }
                    _ => {}
                }
            }
        },
        Instruction::LDH(ldh::LDHType::LDH(target, source)) => match (target, source) {
            (ldh::LDHSourceOrTarget::A, ldh::LDHSourceOrTarget::A8Ref) => {
                push_next_immediate_byte_as_hex_to_string(cpu, memory_bus, &mut res)
            }
            _ => {}
        },
        Instruction::JR(_) => {
            push_next_immediate_byte_as_signed_integer_to_string(cpu, memory_bus, &mut res);
        }
        Instruction::JP(_) => {
            push_next_two_immediate_bytes_as_hex_big_endian_to_string(cpu, memory_bus, &mut res);
        }
        Instruction::CALL(_) => {
            push_next_four_immediate_bytes_as_hex_to_string(cpu, memory_bus, &mut res);
        }
        _ => {}
    }
    res.push_str(" ");
    res
}

#[cfg(debug_assertions)]
fn push_next_immediate_byte_as_hex_to_string(
    cpu: &CPU,
    memory_bus: &MemoryBus,
    string: &mut String,
) {
    let first_immediate_byte = memory_bus.read_byte(cpu.pc + 1);
    string.push_str(&format!(" {:02X} ", first_immediate_byte,));
}

#[cfg(debug_assertions)]
fn push_next_immediate_byte_as_signed_integer_to_string(
    cpu: &CPU,
    memory_bus: &MemoryBus,
    string: &mut String,
) {
    let first_immediate_byte = memory_bus.read_byte(cpu.pc + 1) as i8;
    string.push_str(&format!(" {} ", first_immediate_byte,));
}

#[cfg(debug_assertions)]
fn push_next_two_immediate_bytes_to_string(cpu: &CPU, memory_bus: &MemoryBus, string: &mut String) {
    let first_immediate_byte = memory_bus.read_byte(cpu.pc + 1);
    let second_immediate_byte = memory_bus.read_byte(cpu.pc + 2);
    string.push_str(&format!(
        " {:08b} {:08b} ",
        first_immediate_byte, second_immediate_byte
    ));
}

#[cfg(debug_assertions)]
fn push_next_two_immediate_bytes_as_hex_big_endian_to_string(
    cpu: &CPU,
    memory_bus: &MemoryBus,
    string: &mut String,
) {
    let first_immediate_byte = memory_bus.read_byte(cpu.pc + 1);
    let second_immediate_byte = memory_bus.read_byte(cpu.pc + 2);
    string.push_str(&format!(
        " {:02X} {:02X} ",
        second_immediate_byte, first_immediate_byte
    ));
}

#[cfg(debug_assertions)]
fn push_next_four_immediate_bytes_as_hex_to_string(
    cpu: &CPU,
    memory_bus: &MemoryBus,
    string: &mut String,
) {
    let first_immediate_byte = memory_bus.read_byte(cpu.pc + 1);
    let second_immediate_byte = memory_bus.read_byte(cpu.pc + 2);
    let third_immediate_byte = memory_bus.read_byte(cpu.pc + 3);
    let fourth_immediate_byte = memory_bus.read_byte(cpu.pc + 4);
    string.push_str(&format!(
        " {:02X} {:02X} {:02X} {:02X} ",
        first_immediate_byte, second_immediate_byte, third_immediate_byte, fourth_immediate_byte
    ));
}

#[cfg(debug_assertions)]
fn push_match_interrupt_location_to_interrupt_name(interrupt_location: u16) -> Option<String> {
    match interrupt_location {
        0x0040 => Some("VBLANK".to_string()),
        0x0048 => Some("LCD STAT".to_string()),
        0x0050 => Some("TIMER".to_string()),
        0x0058 => Some("SERIAL".to_string()),
        0x0060 => Some("JOYPAD".to_string()),
        _ => None,
    }
}

impl PPU {
    /// Returns the current tile set for the background and window. Switches the addressing mode
    /// automatically according to LCDC bit 4 (background_and_window_tile_data) as tile structs.
    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    pub fn get_background_and_window_tile_data_debug(&self, memory_bus: &MemoryBus) -> [Tile; 256] {
        if LCDCRegister::get_background_and_window_tile_data_flag(memory_bus) {
            self.get_background_and_window_tile_data_block_0_and_1_debug(memory_bus)
        } else {
            self.get_background_and_window_tile_data_block_2_and_1_debug(memory_bus)
        }
    }

    /// Returns the current tile set for the objects. That is, the tile set in
    /// Block 0 (0x8000 - 0x87FF) and Block 1 (0x8800 - 0x8FFF).
    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    pub fn get_object_tile_data_debug(&self, memory_bus: &MemoryBus) -> [Tile; 256] {
        self.get_background_and_window_tile_data_block_0_and_1_debug(memory_bus)
    }

    /// Returns the tile data in Block 0 (0x8000 - 0x87FF) and Block 1 (0x8800 - 0x8FFF).
    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    pub fn get_background_and_window_tile_data_block_0_and_1_debug(
        &self,
        memory_bus: &MemoryBus,
    ) -> [Tile; 256] {
        memory_bus.tile_set[0..256].try_into().expect(
            "Slice should be of correct length, work with me here compiler:\
                0 ... 256 = 256 (Tiles)",
        )
    }

    /// Returns the tile data in Block 2 (0x9000 - 0x97FF) and Block 1 (0x8800 - 0x8FFF).
    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    pub fn get_background_and_window_tile_data_block_2_and_1_debug(
        &self,
        memory_bus: &MemoryBus,
    ) -> [Tile; 256] {
        [
            &memory_bus.tile_set[256..384],
            &memory_bus.tile_set[128..256],
        ]
        .concat()
        .try_into()
        .expect(
            "Slice should be of correct length, work with me here compiler:\
                256 ... 384 + 128 ... 256 = 128 + 128 = 256 (Tiles)",
        )
    }
}

/// Converts a tile to a string representation.
#[allow(dead_code)]
pub fn tile_to_string(tile: &Tile) -> String {
    let mut string = String::new();
    for row in tile {
        for pixel in row {
            string.push_str(&convert_pixel_to_string(pixel));
            string.push_str(" ");
        }
        string.push('\n');
    }
    string
}

/// Converts an array of tiles to a string representation.
#[allow(dead_code)]
pub fn tile_data_to_string(tile_data: &[Tile; 256]) -> String {
    let mut res_string = String::new();
    for tile_row in 0..16 {
        for in_tile_row in 0..8 {
            for tile_column in 0..16 {
                for in_tile_column in 0..8 {
                    if in_tile_row == 0 && tile_column == 0 && in_tile_column == 0 {
                        let tile_index_for_printing: usize = tile_row * 16 + tile_column;
                        for i in 0..16 {
                            res_string.push_str(&format!(
                                "{:<17}",
                                tile_n_string(tile_index_for_printing + i),
                            ));
                        }
                        res_string.push_str("\n");
                    }
                    let tile_index = tile_row * 16 + tile_column;
                    let tile: Tile = tile_data[tile_index];
                    let pixel_value = tile[in_tile_row][in_tile_column];
                    res_string.push_str(&convert_pixel_to_string(&pixel_value));
                    res_string.push_str(" ");
                }
                res_string.push_str(" ");
            }
            res_string.push('\n');
        }
        res_string.push('\n');
    }
    res_string
}

#[allow(dead_code)]
fn tile_n_string(tile_index: usize) -> String {
    format!("Tile {}:", tile_index)
}

/// Converts a pixel with a value/color to a string representation.
#[allow(dead_code)]
pub fn convert_pixel_to_string(pixel: &TilePixelValue) -> String {
    match pixel {
        TilePixelValue::Zero => "▫".to_string(),
        TilePixelValue::One => "▪".to_string(),
        TilePixelValue::Two => "□".to_string(),
        TilePixelValue::Three => "■".to_string(),
    }
}

/// Converts a tile map (array of u8) to a string representation.
#[allow(dead_code)]
pub fn tile_map_to_string(tile_map: &[u8; 1024]) -> String {
    let mut string = String::new();
    for row in 0..32 {
        for column in 0..32 {
            let tile_index = (row * 32 + column) as usize;
            let tile_value = tile_map[tile_index];
            string.push_str(&format!("{} ", tile_value));
        }
        string.push('\n');
    }
    string
}
