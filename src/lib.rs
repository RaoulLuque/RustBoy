#![warn(missing_docs)] // Warn if public items lack documentation
#![warn(unused_imports)] // Warn about unused imports
#![warn(unused_variables)] // Warn about unused variables
#![warn(unreachable_code)] // Warn if there is unreachable code
#![warn(dead_code)] // Warn about unused functions, structs, etc.
#![warn(non_snake_case)] // Warn about non-standard naming conventions
#![warn(rust_2021_compatibility)] // Warn about issues with Rust 2021 edition
#![warn(clippy::all)] // Enable all Clippy lints
//! This crate provides the methods used to run a Game Boy emulator written in Rust, so it can be run both natively and on the web using WebAssembly.

mod cpu;
mod frontend;

use std::path::Path;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wasm_timer::Instant;

use crate::cpu::CPU;
use frontend::State;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::ControlFlow,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

const TARGET_FPS: u32 = 1;
const TARGET_FRAME_DURATION: f64 = 1.0 / TARGET_FPS as f64;
const SCREEN_WIDTH: u32 = 160;
const SCREEN_HEIGHT: u32 = 144;

/// Run the emulator.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    // Initialize logger according to the target architecture
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).ok();
        } else {
            env_logger::init();
        }
    }
    log::info!("Logger initialized");

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("RustBoy");
    let _ = window.request_inner_size(PhysicalSize::new(SCREEN_WIDTH, SCREEN_HEIGHT));

    // Add a canvas to the HTML document
    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        let _ = window.request_inner_size(PhysicalSize::new(SCREEN_WIDTH, SCREEN_HEIGHT));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("emulator-body")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = State::new(&window).await;
    let mut surface_configured = false;

    let mut cpu = setup_cpu();

    let mut last_frame_time = Instant::now();

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

                            // Calculate the time since the last frame and check if a new frame
                            // should be drawn or we still wait
                            let now = Instant::now();
                            let elapsed = now.duration_since(last_frame_time);
                            if elapsed.as_secs_f64() >= TARGET_FRAME_DURATION {
                                last_frame_time = Instant::now();
                                cpu.step();

                                state.update();
                                match state.render() {
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
                                        wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other,
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
                        _ => {}
                    }
                }
            }
            _ => {}
        })
        .expect("Event loop should be able to run");
}

fn setup_cpu() -> CPU {
    let mut cpu = CPU::new_after_boot();
    log::trace!("CPU Bus initial state: {}", cpu.bus);

    match Path::new("/etc/hosts").exists() {
        true => {
            cpu.load_program("roms/tetris.gb");
            // TODO: Handle header checksum (init of Registers f.H and f.C): https://gbdev.io/pandocs/Power_Up_Sequence.html#obp
            log::trace!("CPU Bus after loading program: {}", cpu.bus);
        }
        false => log::warn!("No rom found"),
    };

    cpu
}
