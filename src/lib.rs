#![warn(missing_docs)] // Warn if public items lack documentation
#![warn(unused_imports)] // Warn about unused imports
#![warn(unused_variables)] // Warn about unused variables
#![warn(unreachable_code)] // Warn if there is unreachable code
#![warn(dead_code)] // Warn about unused functions, structs, etc.
#![warn(non_snake_case)] // Warn about non-standard naming conventions
#![warn(rust_2021_compatibility)] // Warn about issues with Rust 2021 edition
#![warn(clippy::all)] // Enable all Clippy lints
//! This crate provides the methods used to run a Rust Boy emulator written in Rust, so it can be run both natively and on the web using WebAssembly.

mod cpu;
mod debugging;
mod frontend;
mod gpu;
mod interrupts;
mod memory_bus;
mod timer;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wasm_timer::Instant;

#[cfg(debug_assertions)]
use crate::debugging::setup_debugging_logs_files;
use crate::gpu::RenderTask;
use cpu::registers::CPURegisters;
use debugging::DebuggingFlags;
use frontend::State;
use gpu::GPU;
use interrupts::{InterruptEnableRegister, InterruptFlagRegister};
use timer::TimerInfo;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

const TARGET_FPS: u32 = 60;
const TARGET_FRAME_DURATION: f64 = 1.0 / TARGET_FPS as f64;
const SCREEN_WIDTH: u32 = 160;
const SCREEN_HEIGHT: u32 = 144;
const M_CYCLES_PER_SECOND: u32 = 1_048_576;

/// Struct to represent the Rust Boy.
/// It is split into 3 main parts: The CPU, the memory bus, and the GPU.
///
/// The CPU has 8 registers, a program counter (PC), a stack pointer (SP), and a memory bus.
/// For details please refer to [Pan Docs](https://gbdev.io/pandocs/CPU_Registers_and_Flags.html).
/// The CPU also has a cycle counter to keep track of the number of cycles executed.
///
/// Additionally, the CPU has an interrupt master enable (IME) flag to control the handling of
/// interrupts, see [Pan Docs](https://gbdev.io/pandocs/Interrupts.html). ime_to_be_set is used
/// to set the IME flag after the current instruction is executed which is necessary for the
/// correct execution of the EI instruction.
///
/// In addition to the IME flag, the CPU has a halted flag to indicate if the CPU is halted.
/// See [Pan Docs](https://gbdev.io/pandocs/halt.html#halt) for more information on this.
///
/// At last the CPU has a starting_up flag to indicate if the CPU/System is starting up.
///
/// For implementations of the CPU instructions please see [instructions].
///
/// The memory bus is a struct that represents the memory of the Rust Boy.
/// It is an array that represents the memory of the RustBoy.
/// 65536 is the size of the memory in bytes
/// The memory bus also has a bios array that represents the BIOS of the RustBoy which is used
/// during startup instead of the first 0x0100 bytes of the memory.
///
/// The GPU is a struct that represents the graphics processing unit of the RustBoy. It contains
/// the registers, VRAM and other graphics related data.
///
///
///
/// The debugging flags are used to control flags used in debugging the RustBoy.
pub struct RustBoy {
    // CPU
    registers: CPURegisters,
    pc: u16,
    sp: u16,
    cycle_counter: u64,
    ime: bool,
    ime_to_be_set: bool,
    halted: bool,
    starting_up: bool,

    // Memory
    memory: [u8; 65536],
    bios: [u8; 0x0100],

    // GPU
    gpu: GPU,

    // Timers
    timer_info: TimerInfo,

    // Interrupts
    interrupt_enable_register: InterruptEnableRegister,
    interrupt_flag_register: InterruptFlagRegister,

    // Debugging Flags
    debugging_flags: DebuggingFlags,
}

impl RustBoy {
    /// Creates a new instance of the RustBoy struct.
    /// The registers and pointers are all set to their
    /// defaults, as they are before the boot rom has been executed. More specifically,
    /// The registers are set to 0, the program counter (PC) is set to 0x0000,
    /// the stack pointer (SP) is set to 0xFFFE, and the cycle counter is set to 0.
    /// The memory bus is also initialized.
    /// The GPU is initialized to an empty state.
    pub fn new_before_boot(debugging_flags: DebuggingFlags) -> RustBoy {
        RustBoy {
            registers: CPURegisters::new_zero(),
            pc: 0x0000,
            sp: 0xFFFE,
            cycle_counter: 0,
            memory: [0; 65536],
            bios: [0; 0x0100],
            starting_up: true,
            ime: false,
            ime_to_be_set: false,
            halted: false,
            gpu: GPU::new_empty(debugging_flags),
            timer_info: TimerInfo::new(),
            interrupt_enable_register: InterruptEnableRegister::new(),
            interrupt_flag_register: InterruptFlagRegister::new(),

            debugging_flags,
        }
    }

    /// Creates a new instance of the RustBoy struct.
    /// The registers and pointers are all set to their values which they would have after the
    /// boot rom has been executed. For reference, see in the
    /// [Pan Docs](https://gbdev.io/pandocs/Power_Up_Sequence.html#obp)
    pub fn new_after_boot(debugging_flags: DebuggingFlags) -> RustBoy {
        let mut cpu = RustBoy {
            registers: CPURegisters::new_after_boot(),
            pc: 0x0100,
            sp: 0xFFFE,
            cycle_counter: 0,
            memory: [0; 65536],
            bios: [0; 0x0100],
            starting_up: false,
            ime: false,
            ime_to_be_set: false,
            halted: false,
            gpu: GPU::new_empty(debugging_flags),
            timer_info: TimerInfo::new(),
            interrupt_enable_register: InterruptEnableRegister::new(),
            interrupt_flag_register: InterruptFlagRegister::new(),

            debugging_flags,
        };

        cpu.initialize_hardware_registers();
        cpu
    }
}

/// Run the emulator.
/// This function is the entry point for the emulator. The parameters are as follows:
/// * `headless`: If true, the emulator runs in headless mode. That is, without opening a window
/// and therefore not showing the graphics
/// * `game_boy_doctor_mode`: If true, the emulator runs in a mode which is compatible with
/// debugging using [gameboy doctor](https://github.com/robert/gameboy-doctor).
/// * `print_serial_output_to_terminal`: If true, the emulator prints the serial output to the
/// terminal.
/// * `rom_path`: The path to the ROM file to be loaded.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn run(
    headless: bool,
    game_boy_doctor_mode: bool,
    print_serial_output_to_terminal: bool,
    rom_path: &str,
) {
    // Initialize logger according to the target architecture
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Logger should be able to initialize");
        } else {
            env_logger::init();
        }
    }
    log::info!("Logger initialized");

    let debugging_flags = DebuggingFlags {
        doctor: game_boy_doctor_mode,
        sb_to_terminal: print_serial_output_to_terminal,
    };

    let mut rust_boy = setup_rust_boy(debugging_flags, rom_path);

    #[cfg(debug_assertions)]
    if headless {
        log::info!("Running in headless mode");
        run_headless(&mut rust_boy);
    }

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("RustBoy");

    // Add a canvas to the HTML document
    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::platform::web::WindowExtWebSys;
        let canvas = window.canvas().expect("Canvas not found");
        canvas.style().set_css_text("width: 160px; height: 144px;"); // Enforce size
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("emulator-body")?;
                dst.append_child(&web_sys::Element::from(canvas)).ok()
            })
            .expect("Failed to append canvas");
    }
    // Force a resize event to trigger initial configuration
    let _ = window.request_inner_size(PhysicalSize::new(crate::SCREEN_WIDTH, crate::SCREEN_HEIGHT));

    let mut state = State::new(&window).await;
    let mut surface_configured = false;

    // Track the cpu cycles
    let mut total_num_cpu_cycles = 0;

    // Variable to keep track of the current [gpu::RenderTask] to be executed
    let mut current_rendering_task: RenderTask = RenderTask::None;

    let mut last_frame_time = Instant::now();
    log::info!("Starting event loop");

    event_loop
        .run(move |event, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        } => control_flow.exit(),
                        WindowEvent::Resized(physical_size) => {
                            log::info!("physical_size: {physical_size:?}");
                            surface_configured = true;
                            state.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            // This tells winit that we want another frame after this one
                            state.window().request_redraw();

                            if !surface_configured {
                                log::warn!("Surface not configured");
                                return;
                            }

                            // Make multiple steps per redraw request until something has to be rendered
                            while current_rendering_task != RenderTask::RenderFrame {
                                (total_num_cpu_cycles, current_rendering_task) =
                                    handle_no_rendering_task(&mut rust_boy, total_num_cpu_cycles);

                                // We draw a new line to the framebuffer whenever the gpu requests a new line or when it requests a
                                // new frame, since in the latter case, the last line is still missing
                                if current_rendering_task != RenderTask::None {
                                    state.update();

                                    if let RenderTask::WriteLineToBuffer(current_scanline) =
                                        current_rendering_task
                                    {
                                        // If the current rendering task was to render a line, we need to reset it to none,
                                        // since we have just written a line to the framebuffer. If it was to render a frame,
                                        // it has to stay as is, since we still need to render the frame
                                        current_rendering_task = RenderTask::None;
                                        state.render_compute(&mut rust_boy.gpu, current_scanline);
                                    } else {
                                        // Otherwise, the current rendering task was to render a frame, and we still need to
                                        // write the last line to the framebuffer
                                        state.render_compute(&mut rust_boy.gpu, 143);
                                    }
                                }
                            }

                            if current_rendering_task == RenderTask::RenderFrame {
                                // Calculate the time since the last frame and check if a new frame
                                // should be drawn or we still wait
                                let now = Instant::now();
                                let elapsed = now.duration_since(last_frame_time);
                                if elapsed.as_secs_f64() >= TARGET_FRAME_DURATION {
                                    last_frame_time = Instant::now();
                                    current_rendering_task = RenderTask::None;

                                    state.update();
                                    match state.render(&mut rust_boy.gpu) {
                                        Ok(_) => {}
                                        // Reconfigure the surface if it's lost or outdated
                                        Err(
                                            wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                        ) => {
                                            log::warn!("Surface is Lost or Outdated");
                                            state.resize(state.size)
                                        }
                                        // The system is out of memory, we should probably quit
                                        Err(
                                            wgpu::SurfaceError::OutOfMemory
                                            | wgpu::SurfaceError::Other,
                                        ) => {
                                            log::error!("OutOfMemory");
                                            control_flow.exit();
                                        }

                                        // This happens when a frame takes too long to present
                                        Err(wgpu::SurfaceError::Timeout) => {
                                            log::warn!("Surface timeout")
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        })
        .expect("Event loop should be able to run");
}

/// Set up the Rust Boy by initializing it with the given debugging flags and
/// loading the specified ROM file.
fn setup_rust_boy(debugging_flags: DebuggingFlags, rom_path: &str) -> RustBoy {
    // Initialize the logging for debug if compiling in debug mode
    #[cfg(debug_assertions)]
    setup_debugging_logs_files(debugging_flags, rom_path);

    // TODO: Handle header checksum (init of Registers f.H and f.C): https://gbdev.io/pandocs/Power_Up_Sequence.html#obp
    let mut rust_boy = RustBoy::new_after_boot(debugging_flags);

    rust_boy.load_program(rom_path);

    rust_boy
}

/// Run the emulator in headless mode. That is, without a window.
/// This is useful for (automated) testing and debugging purposes.
#[cfg(debug_assertions)]
fn run_headless(rust_boy: &mut RustBoy) {
    let mut current_rendering_task: RenderTask = RenderTask::None;
    // TODO: Check there is no overflow errors with total_num_cpu_cycles
    let mut total_num_cpu_cycles = 0;
    let mut last_frame_time = Instant::now();
    loop {
        // Make multiple steps per redraw request until something has to be rendered
        while current_rendering_task != RenderTask::RenderFrame {
            (total_num_cpu_cycles, current_rendering_task) =
                handle_no_rendering_task(rust_boy, total_num_cpu_cycles);
        }

        if current_rendering_task == RenderTask::RenderFrame {
            // Calculate the time since the last frame and check if a new frame
            // should be drawn or we still wait
            let now = Instant::now();
            let elapsed = now.duration_since(last_frame_time);
            if elapsed.as_secs_f64() >= TARGET_FRAME_DURATION {
                last_frame_time = Instant::now();
                current_rendering_task = RenderTask::None;
            }
        }
    }
}

/// Handle the case in the game boy loop, where we are not requesting a redraw.
fn handle_no_rendering_task(
    rust_boy: &mut RustBoy,
    total_num_cpu_cycles: u64,
) -> (u64, RenderTask) {
    // Fetch and execute next instruction with cpu_step().
    rust_boy.cpu_step();
    let last_num_of_cycles = rust_boy.cycle_counter - total_num_cpu_cycles;
    let total_num_cpu_cycles = rust_boy.cycle_counter;

    // Increment the timer and divider register according to the number of cycles that the
    // last instruction took
    rust_boy.handle_timer_and_divider(last_num_of_cycles as u32);

    // Convert m-cycles to dots (1 m-cycle = 4 dots)
    let last_num_of_dots = last_num_of_cycles * 4;

    // Check what has to be done for rendering and sync gpu with cpu with gpu_step()
    let new_rendering_task = rust_boy.gpu.gpu_step(
        &mut rust_boy.interrupt_flag_register,
        last_num_of_dots as u32,
    );

    // Return the new total number of cpu cycles and possible rendering tasks
    (total_num_cpu_cycles, new_rendering_task)
}
