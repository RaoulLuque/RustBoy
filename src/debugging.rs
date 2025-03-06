use crate::RustBoy;

/// Struct to represent the debugging flags.
/// The flags are:
/// - 'doctor': If true, the emulator runs in game boy doctor compatible mode,
/// see https://github.com/robert/gameboy-doctor

#[derive(Copy, Clone, Debug)]
pub struct DebuggingFlags {
    pub doctor: bool,
}

#[cfg(debug_assertions)]
pub fn setup_debugging_logs_files(debugging_flags: DebuggingFlags) {
    // Create the log directory if it doesn't exist
    std::fs::create_dir_all("logs").unwrap();

    let log_file_paths = ["logs/doctor.log", "logs/doctors_augmented.log"];
    for path in log_file_paths {
        std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .unwrap();
    }
}

/// Write the gameboy doctor logs to the log file.
/// Don't want all this in release builds, which is why we use the cfg conditional
/// compilation feature.
#[cfg(debug_assertions)]
pub fn doctor_log(rust_boy: &RustBoy, log_file: &str) {
    use std::fs;
    let file_name = format!("logs/{}.log", log_file);
    let mut data = format!("A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}\n"
                       , rust_boy.registers.a, u8::from(&rust_boy.registers.f), rust_boy.registers.b, rust_boy.registers.c,
                       rust_boy.registers.d, rust_boy.registers.e, rust_boy.registers.h, rust_boy.registers.l, rust_boy.sp, rust_boy.pc,
                       rust_boy.read_byte(rust_boy.pc), rust_boy.read_byte(rust_boy.pc.wrapping_add(1)),
                       rust_boy.read_byte(rust_boy.pc.wrapping_add(2)), rust_boy.read_byte(rust_boy.pc.wrapping_add(3))
    );

    if log_file == "doctors_augmented" {
        data.pop();
        data.push_str(&format!(
            " SPMEM:{:02X},{:02X},{:02X},{:02X},CURR:{:02X},{:02X},{:02X},{:02X},{:02X}\n",
            rust_boy.read_byte(rust_boy.sp.saturating_sub(4)),
            rust_boy.read_byte(rust_boy.sp.saturating_sub(3)),
            rust_boy.read_byte(rust_boy.sp.saturating_sub(2)),
            rust_boy.read_byte(rust_boy.sp.saturating_sub(1)),
            rust_boy.read_byte(rust_boy.sp),
            rust_boy.read_byte(rust_boy.sp.saturating_add(1)),
            rust_boy.read_byte(rust_boy.sp.saturating_add(2)),
            rust_boy.read_byte(rust_boy.sp.saturating_add(3)),
            rust_boy.read_byte(rust_boy.sp.saturating_add(4)),
        ))
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
    instruction: crate::cpu::instructions::Instruction,
) {
    use std::fs;
    let file_name = format!("logs/{}.log", log_file);
    let data = format!(
        "{:<40}",
        entire_instruction_to_string(rust_boy, instruction)
    );
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
    use crate::cpu::instructions::load::{LoadType, LoadWordSource, LoadWordTarget};
    use crate::cpu::instructions::Instruction;
    let mut res = format!("{:?}", instruction);
    match instruction {
        Instruction::LD(load_type) => match load_type {
            LoadType::Byte(target, source) => {}
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
        Instruction::CALL(_) => {
            push_next_four_immediate_bytes_as_hex_to_string(rust_boy, &mut res);
        }
        _ => {}
    }
    res.push_str(" ");
    res
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
