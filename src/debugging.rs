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

    let log_file_paths = ["logs/doctor.log", "logs/instructions_and_registers.log"];
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
    let data = format!("A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}\n"
                       , rust_boy.registers.a, u8::from(&rust_boy.registers.f), rust_boy.registers.b, rust_boy.registers.c,
                       rust_boy.registers.d, rust_boy.registers.e, rust_boy.registers.h, rust_boy.registers.l, rust_boy.sp, rust_boy.pc,
                       rust_boy.read_byte(rust_boy.pc), rust_boy.read_byte(rust_boy.pc.wrapping_add(1)),
                       rust_boy.read_byte(rust_boy.pc.wrapping_add(2)), rust_boy.read_byte(rust_boy.pc.wrapping_add(3))
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

/// Log the instruction bytes to the log file.
#[cfg(debug_assertions)]
pub fn instruction_log(
    rust_boy: &RustBoy,
    log_file: &str,
    instruction: crate::cpu::instructions::Instruction,
) {
    use std::fs;
    let file_name = format!("logs/{}.log", log_file);
    let data = entire_instruction_to_string(rust_boy, instruction);
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
    let mut res = format!("{:?}", instruction);
    match instruction {
        crate::cpu::instructions::Instruction::LD(load_type) => match load_type {
            crate::cpu::instructions::load::LoadType::Byte(target, source) => {}
            crate::cpu::instructions::load::LoadType::Word(target, source) => {
                match target {
                    _ => {}
                };
                match source {
                    crate::cpu::instructions::load::LoadWordSource::D16 => {
                        let first_immediate_byte = rust_boy.read_byte(rust_boy.pc + 1);
                        let second_immediate_byte = rust_boy.read_byte(rust_boy.pc + 2);
                        res.push_str(&format!(
                            " {:08b} {:08b} ",
                            first_immediate_byte, second_immediate_byte
                        ));
                    }
                    _ => {}
                }
            }
        },
        _ => {}
    }
    res.push_str(" ");
    res
}
