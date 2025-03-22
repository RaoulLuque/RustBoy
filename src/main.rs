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

    /// If present, runs in Game Boy doctor mode
    #[arg(short, long = "DOCTOR", default_value_t = false)]
    game_boy_doctor: bool,

    /// If present, sends logs to extensive_logs.log file
    #[arg(short, long = "LOGS", default_value_t = false)]
    file_logs: bool,

    /// If present, runs in binjgb mode, which allows for easier debugging with the binjgb emulator
    #[arg(short, long = "BINJGB", default_value_t = false)]
    binjgb_mode: bool,

    /// If present, exits when 'P' (capital letter) is written to the serial output. Used to measure
    /// the execution time of test ROMs.
    #[arg(short, long = "TIMING", default_value_t = false)]
    timing_mode: bool,

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
        args.binjgb_mode,
        args.timing_mode,
        args.print_serial_output_to_terminal,
        &args.rom_path,
    ));
}
