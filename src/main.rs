use clap::Parser;
use rustboy::run;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(name = "Rust Boy")]
#[command(about = "A Game Boy emulator written in Rust", long_about = None)]
struct Args {
    /// If present, runs in game boy doctor mode
    #[arg(short, long, default_value_t = false)]
    doctor: bool,
}

fn main() {
    let args = Args::parse();

    pollster::block_on(run(args.doctor));
}
