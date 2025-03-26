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
mod input;
mod interrupts;
mod memory_bus;
mod timer;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wasm_timer::Instant;

use cpu::registers::CPURegisters;
use debugging::DebugInfo;
#[cfg(debug_assertions)]
use debugging::setup_debugging_logs_files;
use frontend::State;
use gpu::GPU;
use gpu::RenderTask;
use input::Joypad;
use input::{handle_key_pressed_event, handle_key_released_event};
use interrupts::{InterruptEnableRegister, InterruptFlagRegister};
use timer::TimerInfo;

use winit::event_loop::EventLoopWindowTarget;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

const TARGET_FPS: f64 = 60.0;
const TARGET_FRAME_DURATION: f64 = 1.0 / TARGET_FPS;
pub(crate) const INITIAL_SCREEN_WIDTH: u32 = 160;
pub(crate) const INITIAL_SCREEN_HEIGHT: u32 = 144;
const M_CYCLES_PER_SECOND: u32 = 1_048_576;
const MEMORY_SIZE: usize = 65536;

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
/// Furthermore, there is a being_initialized flag to indicate if the memory bus is being
/// initialized. In this case, some writes should not cause side effects like DMA transfers being
/// scheduled.
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
    cycles_current_instruction: Option<u8>,
    ime: bool,
    ime_to_be_set: bool,
    halted: bool,
    just_entered_halt: bool,
    starting_up: bool,

    // Memory
    memory: [u8; MEMORY_SIZE],
    bios: [u8; 0x0100],
    being_initialized: bool,

    // GPU
    gpu: GPU,

    // Timers
    timer_info: TimerInfo,

    // Interrupts
    interrupt_enable_register: InterruptEnableRegister,
    interrupt_flag_register: InterruptFlagRegister,

    // Joypad
    joypad: Joypad,

    // Debugging Flags
    debugging_flags: DebugInfo,
}

impl RustBoy {
    /// Creates a new instance of the RustBoy struct.
    /// The registers and pointers are all set to their
    /// defaults, as they are before the boot rom has been executed. More specifically,
    /// The registers are set to 0, the program counter (PC) is set to 0x0000,
    /// the stack pointer (SP) is set to 0xFFFE, and the cycle counter is set to 0.
    /// The memory bus is also initialized.
    /// The GPU is initialized to an empty state.
    pub fn new_before_boot(debugging_flags: DebugInfo) -> RustBoy {
        RustBoy {
            registers: CPURegisters::new_zero(),
            pc: 0x0000,
            sp: 0xFFFE,
            cycle_counter: 0,
            cycles_current_instruction: None,
            memory: [0; 65536],
            bios: [0; 0x0100],
            starting_up: true,
            being_initialized: true,
            ime: false,
            ime_to_be_set: false,
            halted: false,
            just_entered_halt: false,
            gpu: GPU::new_empty(&debugging_flags),
            timer_info: TimerInfo::new(),
            interrupt_enable_register: InterruptEnableRegister::new(),
            interrupt_flag_register: InterruptFlagRegister::new(),
            joypad: Joypad::new_blank(),

            debugging_flags,
        }
    }

    /// Creates a new instance of the RustBoy struct.
    /// The registers and pointers are all set to their values which they would have after the
    /// boot rom has been executed. For reference, see in the
    /// [Pan Docs](https://gbdev.io/pandocs/Power_Up_Sequence.html#obp)
    pub fn new_after_boot(debugging_flags: DebugInfo) -> RustBoy {
        let mut rust_boy = RustBoy::new_before_boot(debugging_flags);
        rust_boy.registers = CPURegisters::new_after_boot();
        rust_boy.pc = 0x0100;
        rust_boy.starting_up = false;

        rust_boy.initialize_hardware_registers();
        rust_boy.being_initialized = false;
        rust_boy
    }
}

/// Run the emulator.
/// This function is the entry point for the emulator. The parameters are as follows:
/// - `headless`: If true, the emulator runs in headless mode. That is, without opening a window
/// and therefore not showing the graphics
/// - `game_boy_doctor_mode`, `file_logs`, `binjgb_mode`, `timing_mode`, `print_serial_output_to_terminal`:
/// See [debugging::DebugInfo] for more information.
/// - `rom_data`: The ROM data to be loaded into the emulator.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn run(
    headless: bool,
    game_boy_doctor_mode: bool,
    file_logs: bool,
    binjgb_mode: bool,
    timing_mode: bool,
    print_serial_output_to_terminal: bool,
    rom_data: &[u8],
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

    let debugging_flags = DebugInfo {
        file_handle_doctor_logs: None,
        file_handle_extensive_logs: None,
        log_file_index: 0,
        current_number_of_lines_in_log_file: 0,
        instruction_was_logged: true,
        doctor: game_boy_doctor_mode,
        file_logs,
        binjgb_mode,
        timing_mode,
        start_time: if timing_mode {
            Some(Instant::now())
        } else {
            None
        },
        sb_to_terminal: print_serial_output_to_terminal,
    };

    let mut rust_boy = setup_rust_boy(debugging_flags, rom_data);

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
    let _ = window.request_inner_size(PhysicalSize::new(
        crate::INITIAL_SCREEN_WIDTH,
        crate::INITIAL_SCREEN_HEIGHT,
    ));

    let mut state = State::new(&window).await;
    let mut surface_configured = false;

    // Variable to keep track of the current [gpu::RenderTask] to be executed
    let mut current_rendering_task: RenderTask = RenderTask::None;

    let mut last_frame_time: Instant = Instant::now();
    log::info!("Starting event loop");

    // Variables to estimate FPS
    let mut running_frame_counter = 0;
    let mut time_of_last_fps_calculation = Instant::now();

    // Variable to track if emulator is paused
    let mut paused = false;

    event_loop
        .run(move |event, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => handle_close_event(control_flow),
                        WindowEvent::KeyboardInput { .. } => {
                            handle_keyboard_input(event, control_flow, &mut rust_boy, &mut paused)
                        }
                        WindowEvent::Resized(physical_size) => {
                            log::info!("physical_size: {physical_size:?}");
                            surface_configured = true;
                            state.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            handle_redraw_requested_event(
                                &mut state,
                                control_flow,
                                &mut rust_boy,
                                &mut current_rendering_task,
                                &mut last_frame_time,
                                &mut time_of_last_fps_calculation,
                                &mut running_frame_counter,
                                surface_configured,
                                paused,
                            );
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
fn setup_rust_boy(mut debugging_flags: DebugInfo, rom_data: &[u8]) -> RustBoy {
    // Initialize the logging for debug if compiling in debug mode
    #[cfg(debug_assertions)]
    setup_debugging_logs_files(&mut debugging_flags);

    // TODO: Handle header checksum (init of Registers f.H and f.C): https://gbdev.io/pandocs/Power_Up_Sequence.html#obp
    let mut rust_boy = RustBoy::new_after_boot(debugging_flags);

    rust_boy.load_program(rom_data);

    rust_boy
}

/// Run the emulator in headless mode. That is, without a window.
/// This is useful for (automated) testing and debugging purposes.
#[cfg(debug_assertions)]
fn run_headless(rust_boy: &mut RustBoy) {
    let mut current_rendering_task: RenderTask = RenderTask::None;
    let mut last_frame_time = Instant::now();
    loop {
        // Make multiple steps per redraw request until something has to be rendered
        while current_rendering_task != RenderTask::RenderFrame {
            current_rendering_task = handle_no_rendering_task(rust_boy);
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

/// Handle the redraw requested event.
///
/// This function is called whenever the window requests a redraw. That is, [TARGET_FPS] times per
/// second (if there are no dropped frames). It handles the stepping of the CPU and GPU, therefore
/// keeping them in sync and providing a "runtime" for the entire emulator.
fn handle_redraw_requested_event(
    state: &mut State,
    control_flow: &EventLoopWindowTarget<()>,
    rust_boy: &mut RustBoy,
    current_rendering_task: &mut RenderTask,
    last_frame_time: &mut Instant,
    time_of_last_fps_calculation: &mut Instant,
    running_frame_counter: &mut u32,
    surface_configured: bool,
    paused: bool,
) {
    // This tells winit that we want another frame after this one
    state.window().request_redraw();

    if !surface_configured {
        log::warn!("Surface not configured");
        return;
    }

    // If the emulator is paused, we don't want to run any cycles
    if paused {
        return;
    }

    // Make multiple steps per redraw request until something has to be rendered
    while *current_rendering_task != RenderTask::RenderFrame {
        *current_rendering_task = handle_no_rendering_task(rust_boy);

        // We draw a new line to the framebuffer whenever the gpu requests a new line or when it requests a
        // new frame, since in the latter case, the last line is still missing
        if *current_rendering_task != RenderTask::None {
            state.update();

            if let RenderTask::WriteLineToBuffer(current_scanline) = *current_rendering_task {
                // If the current rendering task was to render a line, we need to reset it to none,
                // since we have just written a line to the framebuffer. If it was to render a frame,
                // it has to stay as is, since we still need to render the frame
                *current_rendering_task = RenderTask::None;
                state.render_scanline(&mut rust_boy.gpu, current_scanline);
            } else {
                // Otherwise, the current rendering task was to render a frame, and we still need to
                // write the last line to the framebuffer
                state.render_scanline(&mut rust_boy.gpu, 143);
            }
        }
    }

    if *current_rendering_task == RenderTask::RenderFrame {
        // Calculate the time since the last frame and check if a new frame
        // should be drawn or we still wait
        let now = Instant::now();
        let elapsed = now.duration_since(*last_frame_time);
        if elapsed.as_secs_f64() >= TARGET_FRAME_DURATION {
            *last_frame_time = Instant::now();
            *current_rendering_task = RenderTask::None;

            // Estimate FPS
            *running_frame_counter += 1;

            if time_of_last_fps_calculation.elapsed().as_secs() > 5 {
                let elapsed_time = time_of_last_fps_calculation.elapsed();
                let fps = *running_frame_counter as f64 / elapsed_time.as_secs_f64();
                log::debug!("FPS: {}", fps);
                *running_frame_counter = 0;
                *time_of_last_fps_calculation = now;
            }

            state.update();
            match state.render_screen() {
                Ok(_) => {}
                // Reconfigure the surface if it's lost or outdated
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    log::warn!("Surface is Lost or Outdated");
                    state.resize(state.size)
                }
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
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

/// Handle the case in the game boy loop, where we are not requesting a redraw.
fn handle_no_rendering_task(rust_boy: &mut RustBoy) -> RenderTask {
    // Fetch and execute next instruction with cpu_step().
    rust_boy.cpu_step();
    let last_num_of_cycles = rust_boy
        .cycles_current_instruction
        .expect("Cycles should be set by cpu_step()");

    // Increment the timer and divider register according to the number of cycles that the
    // last instruction took
    rust_boy.handle_timer_and_divider(last_num_of_cycles as u32);

    // Convert m-cycles to dots (1 m-cycle = 4 dots)
    let last_num_of_dots = last_num_of_cycles as u32 * 4;

    // Check what has to be done for rendering and sync gpu with cpu with gpu_step()
    let new_rendering_task = rust_boy
        .gpu
        .gpu_step(&mut rust_boy.interrupt_flag_register, last_num_of_dots);

    // Reset the cycles of the current instruction
    rust_boy.cycles_current_instruction = None;

    // Return the new total number of cpu cycles and possible rendering tasks
    new_rendering_task
}

/// Handles the close event of the window by exiting the event loop.
fn handle_close_event(control_flow: &EventLoopWindowTarget<()>) {
    control_flow.exit();
}

/// Handles the keyboard input events.
///
/// That is, control flow inputs like ESCAPE to exit the emulator, or P to pause the emulator but
/// also inputs for the emulator itself.
fn handle_keyboard_input(
    event: &WindowEvent,
    control_flow: &EventLoopWindowTarget<()>,
    rust_boy: &mut RustBoy,
    paused: &mut bool,
) {
    match event {
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                    ..
                },
            ..
        } => control_flow.exit(),
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: key,
                    ..
                },
            ..
        } => handle_key_pressed_event(rust_boy, key, paused),
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state: ElementState::Released,
                    physical_key: key,
                    ..
                },
            ..
        } => handle_key_released_event(rust_boy, key),
        _ => {}
    }
}
