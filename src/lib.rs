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
mod frontend;
mod gpu;
mod memory_bus;

use std::path::Path;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wasm_timer::Instant;

use crate::gpu::RenderTask;
use cpu::registers::CPURegisters;
use frontend::State;
use gpu::GPU;
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
/// At last the CPU has a starting_up flag to indicate if the CPU/System is starting up.
///
/// For implementations of the CPU instructions please see [instructions].
///
/// The memory bus is a struct that represents the memory of the Rust Boy.
/// It is an array that represents the memory of the RustBoy.
/// 65536 is the size of the memory in bytes
/// The memory bus also has a bios array that represents the BIOS of the RustBoy which is used
/// during startup instead of the first 0x0100 bytes of the memory.
pub struct RustBoy {
    // CPU
    registers: CPURegisters,
    pc: u16,
    sp: u16,
    cycle_counter: u64,
    ime: bool,
    ime_to_be_set: bool,
    starting_up: bool,

    // Memory
    memory: [u8; 65536],
    bios: [u8; 0x0100],

    // GPU
    gpu: GPU,

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
            gpu: GPU::new_empty(debugging_flags),

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
            gpu: GPU::new_empty(debugging_flags),

            debugging_flags,
        };

        cpu.initialize_hardware_registers();
        cpu
    }
}

/// Run the emulator.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn run(game_boy_doctor_mode: bool) {
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

    let debugging_flags = DebuggingFlags {
        doctor: game_boy_doctor_mode,
    };

    let mut rust_boy = setup_rust_boy(debugging_flags);

    // Track the cpu cycles
    let mut total_num_cpu_cycles = 0;

    // Flag if we are idling to wait for the next frame
    let mut redraw_request = false;

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
                            while !redraw_request {
                                rust_boy.step();
                                let last_num_of_cycles =
                                    rust_boy.cycle_counter - total_num_cpu_cycles;
                                total_num_cpu_cycles = rust_boy.cycle_counter;

                                match rust_boy.gpu.step(last_num_of_cycles as u32) {
                                    RenderTask::None => {}
                                    RenderTask::Render => {
                                        redraw_request = true;
                                    }
                                };
                            }

                            if redraw_request {
                                // Calculate the time since the last frame and check if a new frame
                                // should be drawn or we still wait
                                let now = Instant::now();
                                let elapsed = now.duration_since(last_frame_time);
                                if elapsed.as_secs_f64() >= TARGET_FRAME_DURATION {
                                    last_frame_time = Instant::now();
                                    redraw_request = false;

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

fn setup_rust_boy(debugging_flags: DebuggingFlags) -> RustBoy {
    let mut rust_boy = RustBoy::new_after_boot(debugging_flags);
    log::trace!("CPU Bus initial state: {}", rust_boy.memory_to_string());

    // TODO: Handle WASM, where the rom cannot be loaded from the filesystem and instead served by the webserver
    match Path::new("roms/").exists() {
        true => {
            rust_boy.load_program("roms/tetris.gb");
            // TODO: Handle header checksum (init of Registers f.H and f.C): https://gbdev.io/pandocs/Power_Up_Sequence.html#obp
        }
        false => log::warn!("No rom found"),
    };

    rust_boy
}

/// Struct to represent the debugging flags.
/// The flags are:
/// - 'doctor': If true, the emulator runs in game boy doctor compatible mode,
/// see https://github.com/robert/gameboy-doctor

#[derive(Copy, Clone, Debug)]
pub struct DebuggingFlags {
    doctor: bool,
}
