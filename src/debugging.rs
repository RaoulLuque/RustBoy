/// This module contains the debugging functions for the RustBoy emulator. Therefore, it is a bit
/// all over the place.
use wasm_timer::Instant;

use crate::RustBoy;
use crate::cpu::instructions::ArithmeticOrLogicalSource;
use crate::gpu::GPU;
use crate::gpu::registers::{GPURegisters, LCDCRegister};
use crate::gpu::tile_handling::{Tile, TilePixelValue};
use crate::interrupts::{InterruptEnableRegister, InterruptFlagRegister};
use std::fs;
use std::io::Write;

pub const LOG_FILE_NAME: &str = "extensive_logs";

/// Struct to represent the debugging information/flags.
/// The flags are:
/// - `doctor`: If true, the emulator runs in game boy doctor compatible mode.
/// - `file_logs`: If true, the emulator writes logs to a file.
/// - `binjgb_mode`: If true, the emulator runs in binjgb mode, that is, it runs in a mode where
/// the logs of this and the binjgb emulator are compatible.
/// - `timing_mode`: If true, the emulator runs in timing mode, that is, it exits when the serial
/// output is 'P' (capital letter).
/// - `start_time`: The time when the emulator started running. Used in combination with timing mode.
/// - `sb_to_terminal`: If true, the emulator prints the serial output to the terminal.
/// see https://github.com/robert/gameboy-doctor

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

/// Write the gameboy doctor logs to the log file.
/// Don't want all this in release builds, which is why we use the cfg conditional
/// compilation feature.
#[cfg(debug_assertions)]
pub fn doctor_log(rust_boy: &mut RustBoy, log_file: &str) {
    let mut data = format!(
        "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}\n",
        rust_boy.registers.a,
        rust_boy.registers.f.get(),
        rust_boy.registers.b,
        rust_boy.registers.c,
        rust_boy.registers.d,
        rust_boy.registers.e,
        rust_boy.registers.h,
        rust_boy.registers.l,
        rust_boy.sp,
        rust_boy.pc,
        rust_boy.read_byte(rust_boy.pc),
        rust_boy.read_byte(rust_boy.pc.wrapping_add(1)),
        rust_boy.read_byte(rust_boy.pc.wrapping_add(2)),
        rust_boy.read_byte(rust_boy.pc.wrapping_add(3))
    );

    if log_file == LOG_FILE_NAME {
        data.pop();
        data.push_str(&format!(
            " SPMEM:{:02X},{:02X},{:02X},{:02X},CURR:{:02X},{:02X},{:02X},{:02X},{:02X}",
            rust_boy.read_byte(rust_boy.sp.saturating_sub(4)),
            rust_boy.read_byte(rust_boy.sp.saturating_sub(3)),
            rust_boy.read_byte(rust_boy.sp.saturating_sub(2)),
            rust_boy.read_byte(rust_boy.sp.saturating_sub(1)),
            rust_boy.read_byte(rust_boy.sp),
            rust_boy.read_byte(rust_boy.sp.saturating_add(1)),
            rust_boy.read_byte(rust_boy.sp.saturating_add(2)),
            rust_boy.read_byte(rust_boy.sp.saturating_add(3)),
            rust_boy.read_byte(rust_boy.sp.saturating_add(4)),
        ));

        let ppu_mode_as_u8 = GPURegisters::get_gpu_mode(&rust_boy.memory).as_u8();
        let ppu_mode_sign = if GPURegisters::get_lcd_control(&rust_boy.memory) & 0b1000_0000 != 0 {
            "+"
        } else {
            "-"
        };
        data.push_str(&format!(" PPU:{}{}", ppu_mode_sign, ppu_mode_as_u8));

        let stat_register = GPURegisters::get_lcd_status(&rust_boy.memory);
        data.push_str(&format!(" STAT:{:<08b}", stat_register));

        let lyc = GPURegisters::get_scanline_compare(&rust_boy.memory);
        data.push_str(&format!(" LYC:{:<3}", lyc));

        // We use the get scanline internal function to get the current scanline immediately
        // from the memory without any additional sync checks done for syncing GPU and CPU.
        let current_scanline = GPURegisters::get_scanline_internal(&rust_boy.memory);
        data.push_str(&format!(" SCANLINE:{:<3}", current_scanline));

        data.push_str(&format!(" IME:{}", u8::from(rust_boy.ime)));
        data.push_str(&format!(
            " IF:{:02X}",
            InterruptFlagRegister::get_interrupt_flag_register(&rust_boy.memory)
        ));
        data.push_str(&format!(
            " IE:{:02X}",
            InterruptEnableRegister::get_interrupt_enable_register(&rust_boy.memory)
        ));

        let cycles_in_dots: u32 = rust_boy.gpu.rendering_info.dots_clock;
        data.push_str(&format!(" CY_DOTS:{:<3}", cycles_in_dots));

        let total_cycles: u128 = rust_boy.gpu.rendering_info.total_dots;
        data.push_str(&format!(" TOTAL_CY_DOTS:{:<10}\n", total_cycles));
    }
    if log_file == "doctor" {
        rust_boy
            .debugging_flags
            .file_handle_doctor_logs
            .as_ref()
            .expect("Doctor log file handle should be created")
            .write_all(data.as_bytes())
            .expect("Should be able to write data to doctor log file");
    } else {
        rust_boy.debugging_flags.current_number_of_lines_in_log_file += 1;
        if rust_boy.debugging_flags.current_number_of_lines_in_log_file == 2_000_000 {
            rust_boy.debugging_flags.current_number_of_lines_in_log_file = 0;
            rust_boy.debugging_flags.log_file_index += 1;
            setup_debugging_logs_files(&mut rust_boy.debugging_flags);
        }
        rust_boy
            .debugging_flags
            .file_handle_extensive_logs
            .as_ref()
            .expect("Doctor log file handle should be created")
            .write_all(data.as_bytes())
            .expect("Should be able to write data to doctor log file");
    }
}

/// Log the instruction bytes to the log file.
#[cfg(debug_assertions)]
pub fn instruction_log(
    rust_boy: &RustBoy,
    log_file: &str,
    instruction: Option<crate::cpu::instructions::Instruction>,
    interrupt_location: Option<u16>,
) {
    let data = if let Some(instruction) = instruction {
        format!(
            "{:<50}",
            entire_instruction_to_string(rust_boy, instruction)
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
        rust_boy
            .debugging_flags
            .file_handle_doctor_logs
            .as_ref()
            .expect("Doctor log file handle should be created")
            .write_all(data.as_bytes())
            .expect("Should be able to write data to doctor log file");
    } else {
        rust_boy
            .debugging_flags
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
    rust_boy: &RustBoy,
    instruction: crate::cpu::instructions::Instruction,
) -> String {
    use crate::cpu::instructions::Instruction;
    use crate::cpu::instructions::add_and_adc::AddWordSource;
    use crate::cpu::instructions::load::{LoadType, LoadWordSource};
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
                push_next_immediate_byte_as_hex_to_string(rust_boy, &mut res)
            }
            _ => {}
        },
        Instruction::ADDWord(_, source) => match source {
            AddWordSource::E8 => {
                push_next_immediate_byte_as_hex_to_string(rust_boy, &mut res);
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
                        push_next_two_immediate_bytes_to_string(rust_boy, &mut res);
                    }
                    _ => {}
                }
            }
        },
        Instruction::LDH(crate::cpu::instructions::ldh::LDHType::LDH(target, source)) => {
            match (target, source) {
                (
                    crate::cpu::instructions::ldh::LDHSourceOrTarget::A,
                    crate::cpu::instructions::ldh::LDHSourceOrTarget::A8Ref,
                ) => push_next_immediate_byte_as_hex_to_string(rust_boy, &mut res),
                _ => {}
            }
        }
        Instruction::JR(_) => {
            push_next_immediate_byte_as_signed_integer_to_string(rust_boy, &mut res);
        }
        Instruction::JP(_) => {
            push_next_two_immediate_bytes_as_hex_big_endian_to_string(rust_boy, &mut res);
        }
        Instruction::CALL(_) => {
            push_next_four_immediate_bytes_as_hex_to_string(rust_boy, &mut res);
        }
        _ => {}
    }
    res.push_str(" ");
    res
}

#[cfg(debug_assertions)]
fn push_next_immediate_byte_as_hex_to_string(rust_boy: &RustBoy, string: &mut String) {
    let first_immediate_byte = rust_boy.read_byte(rust_boy.pc + 1);
    string.push_str(&format!(" {:02X} ", first_immediate_byte,));
}

#[cfg(debug_assertions)]
fn push_next_immediate_byte_as_signed_integer_to_string(rust_boy: &RustBoy, string: &mut String) {
    let first_immediate_byte = rust_boy.read_byte(rust_boy.pc + 1) as i8;
    string.push_str(&format!(" {} ", first_immediate_byte,));
}

#[cfg(debug_assertions)]
fn push_next_two_immediate_bytes_to_string(rust_boy: &RustBoy, string: &mut String) {
    let first_immediate_byte = rust_boy.read_byte(rust_boy.pc + 1);
    let second_immediate_byte = rust_boy.read_byte(rust_boy.pc + 2);
    string.push_str(&format!(
        " {:08b} {:08b} ",
        first_immediate_byte, second_immediate_byte
    ));
}

#[cfg(debug_assertions)]
fn push_next_two_immediate_bytes_as_hex_big_endian_to_string(
    rust_boy: &RustBoy,
    string: &mut String,
) {
    let first_immediate_byte = rust_boy.read_byte(rust_boy.pc + 1);
    let second_immediate_byte = rust_boy.read_byte(rust_boy.pc + 2);
    string.push_str(&format!(
        " {:02X} {:02X} ",
        second_immediate_byte, first_immediate_byte
    ));
}

#[cfg(debug_assertions)]
fn push_next_four_immediate_bytes_as_hex_to_string(rust_boy: &RustBoy, string: &mut String) {
    let first_immediate_byte = rust_boy.read_byte(rust_boy.pc + 1);
    let second_immediate_byte = rust_boy.read_byte(rust_boy.pc + 2);
    let third_immediate_byte = rust_boy.read_byte(rust_boy.pc + 3);
    let fourth_immediate_byte = rust_boy.read_byte(rust_boy.pc + 4);
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

impl GPU {
    /// Returns the current tile set for the background and window. Switches the addressing mode
    /// automatically according to LCDC bit 4 (background_and_window_tile_data) as tile structs.
    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    pub fn get_background_and_window_tile_data_debug(&self, rust_boy: &RustBoy) -> [Tile; 256] {
        if LCDCRegister::get_background_and_window_tile_data_flag(&rust_boy.memory) {
            self.get_background_and_window_tile_data_block_0_and_1_debug(rust_boy)
        } else {
            self.get_background_and_window_tile_data_block_2_and_1_debug(rust_boy)
        }
    }

    /// Returns the current tile set for the objects. That is, the tile set in
    /// Block 0 (0x8000 - 0x87FF) and Block 1 (0x8800 - 0x8FFF).
    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    pub fn get_object_tile_data_debug(&self, rust_boy: &RustBoy) -> [Tile; 256] {
        self.get_background_and_window_tile_data_block_0_and_1_debug(rust_boy)
    }

    /// Returns the tile data in Block 0 (0x8000 - 0x87FF) and Block 1 (0x8800 - 0x8FFF).
    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    pub fn get_background_and_window_tile_data_block_0_and_1_debug(
        &self,
        rust_boy: &RustBoy,
    ) -> [Tile; 256] {
        rust_boy.tile_set[0..256].try_into().expect(
            "Slice should be of correct length, work with me here compiler:\
                0 ... 256 = 256 (Tiles)",
        )
    }

    /// Returns the tile data in Block 2 (0x9000 - 0x97FF) and Block 1 (0x8800 - 0x8FFF).
    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    pub fn get_background_and_window_tile_data_block_2_and_1_debug(
        &self,
        rust_boy: &RustBoy,
    ) -> [Tile; 256] {
        [&rust_boy.tile_set[256..384], &rust_boy.tile_set[128..256]]
            .concat()
            .try_into()
            .expect(
                "Slice should be of correct length, work with me here compiler:\
                256 ... 384 + 128 ... 256 = 128 + 128 = 256 (Tiles)",
            )
    }
}

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

#[allow(dead_code)]
pub fn convert_pixel_to_string(pixel: &TilePixelValue) -> String {
    match pixel {
        TilePixelValue::Zero => "▫".to_string(),
        TilePixelValue::One => "▪".to_string(),
        TilePixelValue::Two => "□".to_string(),
        TilePixelValue::Three => "■".to_string(),
    }
}

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
