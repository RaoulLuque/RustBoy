/// This module contains the debugging functions for the RustBoy emulator. Therefore, it is a bit
/// all over the place.
use wasm_timer::Instant;

use crate::RustBoy;
use crate::cpu::instructions::ArithmeticOrLogicalSource;
use std::fs;

pub const LOG_FILE_NAME: &str = "extensive_logs";

/// Struct to represent the debugging flags.
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

#[derive(Copy, Clone, Debug)]
pub struct DebuggingFlags {
    pub doctor: bool,
    pub file_logs: bool,
    pub binjgb_mode: bool,
    pub timing_mode: bool,
    pub start_time: Option<Instant>,
    pub sb_to_terminal: bool,
}

#[cfg(debug_assertions)]
pub fn setup_debugging_logs_files(_: DebuggingFlags, rom_path: &str) {
    // Create the log directory if it doesn't exist
    fs::create_dir_all("logs").unwrap();

    let log_file_paths = ["logs/doctor.log", &format!("logs/{LOG_FILE_NAME}.log")];
    for path in log_file_paths {
        fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .unwrap();
    }

    let mut data = rom_path.to_string();
    data.push_str("\n");

    let file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&format!("logs/{LOG_FILE_NAME}.log"));
    if let Ok(mut file) = file {
        use std::io::Write;
        file.write_all(data.as_bytes())
            .expect("Unable to write data");
    } else {
        panic!("Unable to open file: {:?}", file);
    }
}

/// Write the gameboy doctor logs to the log file.
/// Don't want all this in release builds, which is why we use the cfg conditional
/// compilation feature.
#[cfg(debug_assertions)]
pub fn doctor_log(rust_boy: &RustBoy, log_file: &str) {
    use std::fs;
    let file_name = format!("logs/{}.log", log_file);
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

        let ppu_mode_as_u8 = rust_boy.gpu.gpu_registers.get_gpu_mode().as_u8();
        let ppu_mode_sign = if rust_boy.gpu.gpu_registers.get_lcd_control() & 0b1000_0000 != 0 {
            "+"
        } else {
            "-"
        };
        data.push_str(&format!(" PPU:{}{}", ppu_mode_sign, ppu_mode_as_u8));

        let current_scanline = rust_boy
            .gpu
            .gpu_registers
            .get_scanline(None, None, None, false);
        data.push_str(&format!(" SCANLINE:{:<3}", current_scanline));

        data.push_str(&format!(" IME:{}", u8::from(rust_boy.ime)));
        data.push_str(&format!(
            " IF:{:02X}",
            u8::from(&rust_boy.interrupt_flag_register)
        ));
        data.push_str(&format!(
            " IE:{:02X}",
            u8::from(&rust_boy.interrupt_enable_register)
        ));

        let cycles_in_dots: u32 = rust_boy.gpu.rendering_info.dots_clock;
        data.push_str(&format!(" CY_DOTS:{:<3}", cycles_in_dots));

        let total_cycles: u128 = rust_boy.gpu.rendering_info.total_dots;
        data.push_str(&format!(" TOTAL_CY_DOTS:{:<10}\n", total_cycles));
    }
    let file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(file_name);
    if let Ok(mut file) = file {
        use std::io::Write;
        file.write_all(data.as_bytes())
            .expect("Unable to write data");
    } else {
        panic!("Unable to open file: {:?}", file);
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
    use std::fs;
    let file_name = format!("logs/{}.log", log_file);
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

    let file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(file_name);
    if let Ok(mut file) = file {
        use std::io::Write;
        file.write_all(data.as_bytes())
            .expect("Unable to write data");
    } else {
        panic!("Unable to open file: {:?}", file);
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
