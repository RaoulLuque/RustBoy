use clap::Parser;
use rustboy::run;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(name = "Rust Boy")]
#[command(about = "A Game Boy emulator written in Rust", long_about = None)]
struct Args {
    /// If present, runs in game boy doctor mode
    #[arg(short, long = "DOCTOR", default_value_t = false)]
    game_boy_doctor: bool,

    /// Specify the path of the ROM file to run
    #[arg(short, long = "ROM", value_name = "ROM_PATH")]
    rom_path: String,
}

fn main() {
    let args = Args::parse();

    pollster::block_on(run(args.game_boy_doctor, &args.rom_path));
}
