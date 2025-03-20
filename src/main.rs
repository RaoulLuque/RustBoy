use clap::Parser;
use rustboy::run;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(name = "Rust Boy")]
#[command(about = "A Game Boy emulator written in Rust", long_about = None)]
struct Args {
    /// If present, runs the emulator in headless mode
    #[arg(long = "HEADLESS", default_value_t = false)]
    headless: bool,

    /// If present, runs in game boy doctor mode
    #[arg(short, long = "DOCTOR", default_value_t = false)]
    game_boy_doctor: bool,

    /// If present, sends logs to extensive_logs.log file
    #[arg(short, long = "LOGS", default_value_t = false)]
    file_logs: bool,

    /// If present, prints serial output to the console
    #[arg(short, long = "SB", default_value_t = false)]
    print_serial_output_to_terminal: bool,

    /// Specify the path of the ROM file to run
    #[arg(short, long = "ROM", value_name = "ROM_PATH")]
    rom_path: String,
}

fn main() {
    let args = Args::parse();

    pollster::block_on(run(
        args.headless,
        args.game_boy_doctor,
        args.file_logs,
        args.print_serial_output_to_terminal,
        &args.rom_path,
    ));
}
