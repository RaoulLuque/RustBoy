mod shader;

use log::trace;
use winit::event::WindowEvent;
use winit::window::Window;

use crate::frontend::shader::{
    ATLAS_COLS, BackgroundViewportPosition, ObjectsInScanline, Palettes,
    RenderingLinePositionAndObjectSize, TILE_SIZE, TileData, TilemapUniform,
    setup_render_shader_pipeline, setup_scanline_buffer_pipeline,
};
use crate::gpu::object_handling::custom_ordering;
use crate::gpu::{ChangesToPropagateToShader, GPU};

/// TODO: Add docstring
pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub(super) size: winit::dpi::PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it, as the surface contains
    // unsafe references to the window's resources.
    pub(super) window: &'a Window,

    // The render pipeline is the pipeline that will be used to render the frames to the screen.
    render_pipeline: wgpu::RenderPipeline,
    // The vertex buffer is used to store the vertex data for the render pipeline (two triangles
    // that make up a rectangle).
    render_pipeline_vertex_buffer: wgpu::Buffer,
    // The screensize buffer is used to store the size of the screen (Width x Height in pixels).
    screensize_buffer: wgpu::Buffer,
    // Variable to keep track of whether the screen size has changed or not
    screensize_changed: bool,
    // The number of vertices in the vertex buffer (4).
    render_pipeline_num_vertices: u32,
    // The bind group corresponding to the render pipeline
    render_bind_group: wgpu::BindGroup,

    // The compute pipeline is the pipeline that will be used to run the compute shader. This
    // shader writes to the framebuffer texture for every RustBoy render line (that is 144 times
    // per frame).
    scanline_buffer_pipeline: wgpu::RenderPipeline,
    // The vertex buffer is used to store the vertex data for the render pipeline (two triangles
    // that make up a rectangle).
    scanline_buffer_pipeline_vertex_buffer: wgpu::Buffer,
    // The number of vertices in the vertex buffer (4).
    scanline_buffer_pipeline_num_vertices: u32,
    // The bind group corresponding to the compute pipeline
    scanline_buffer_bind_group: wgpu::BindGroup,
    // Tile atlas texture (128 x 128 rgba) to hold the (currently used) background tile data TODO: Update
    bg_and_wd_tile_data_buffer: wgpu::Buffer,
    // Tilemap buffer flattened 32x32 u8 array to hold the (currently used) tilemap data
    background_tile_map_buffer: wgpu::Buffer,
    // Buffer to hold the background viewport position (is a u32 array of 4 elements) where
    // the first two elements are the x and y position of the background viewport
    background_viewport_buffer: wgpu::Buffer,
    // Buffer to hold the palette data (is a u32 array of 4 elements) where only the first three are
    // used. They just mirror the registers FF47, FF48, FF49 as specified in the Pandocs
    // (https://gbdev.io/pandocs/Palettes.html) and make them available to the shader. The first
    // entry in the vec is the background palette (FF47), the second entry is the object palette 0
    // (FF48) and the third entry is the object palette 1 (FF49). The fourth entry is empty.
    palette_buffer: wgpu::Buffer,
    // Storage texture (160x144) to act as a framebuffer for the compute shader
    framebuffer_texture: wgpu::Texture,
    // Buffer to hold the current line to be rendered for the compute shader
    rendering_line_and_obj_size_buffer: wgpu::Buffer,

    // Storage texture (128 x 128 rgba) to hold the tiles used for objects/sprites TODO: Update
    object_tile_data_buffer: wgpu::Buffer,
    // Buffer to hold the information about the objects/sprites in the current scanline
    objects_in_scanline_buffer: wgpu::Buffer,
}

impl<'a> State<'a> {
    /// TODO: Add docstring
    pub async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                    memory_hints: Default::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let (
            scanline_buffer_pipeline,
            scanline_buffer_pipeline_vertex_buffer,
            scanline_buffer_pipeline_num_vertices,
            scanline_buffer_bind_group,
            tile_data_buffer,
            background_tilemap_buffer,
            background_viewport_buffer,
            palette_buffer,
            framebuffer_texture,
            rendering_line_and_obj_size_buffer,
            object_tile_data_buffer,
            objects_in_scanline_buffer,
        ) = setup_scanline_buffer_pipeline(&device);

        let (
            render_pipeline,
            render_pipeline_vertex_buffer,
            screensize_buffer,
            render_pipeline_num_vertices,
            render_bind_group,
        ) = setup_render_shader_pipeline(&device, &config, &framebuffer_texture);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            render_pipeline_vertex_buffer,
            screensize_buffer,
            screensize_changed: false,
            render_pipeline_num_vertices,
            render_bind_group,
            scanline_buffer_pipeline,
            scanline_buffer_pipeline_vertex_buffer,
            scanline_buffer_pipeline_num_vertices,
            scanline_buffer_bind_group,
            bg_and_wd_tile_data_buffer: tile_data_buffer,
            background_tile_map_buffer: background_tilemap_buffer,
            background_viewport_buffer,
            palette_buffer,
            framebuffer_texture,
            rendering_line_and_obj_size_buffer,
            object_tile_data_buffer,
            objects_in_scanline_buffer,
        }
    }

    /// TODO: Add docstring
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// TODO: Add docstring
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.screensize_changed = true;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// TODO: Add docstring
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    /// TODO: Add docstring
    pub fn update(&mut self) {}

    /// TODO: Add docstring
    pub fn render_screen(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.render_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.render_pipeline_vertex_buffer.slice(..));
            render_pass.draw(0..self.render_pipeline_num_vertices, 0..1);
        }

        // Update the screensize for the fragment shader, if the size has changed
        if self.screensize_changed {
            // Update the screensize buffer with the new size
            let screensize = [self.size.width, self.size.height, 0, 0];
            self.queue.write_buffer(
                &self.screensize_buffer,
                0,
                bytemuck::cast_slice(&screensize),
            );
            self.screensize_changed = false;
        }

        // Submit the rendering commands to the GPU
        // Submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn render_scanline(&mut self, rust_boy_gpu: &mut GPU, current_scanline: u8) {
        // Create a view of the offscreen texture.
        let framebuffer_view = self
            .framebuffer_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Scanline Encoder"),
            });

        // Begin a render pass that writes to the framebuffer texture ("offscreen texture")
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Offscreen Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &framebuffer_view,
                    resolve_target: None,
                    // Use LoadOp::Load to preserve previously rendered scanlines
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.scanline_buffer_pipeline);
            render_pass.set_bind_group(0, &self.scanline_buffer_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.scanline_buffer_pipeline_vertex_buffer.slice(..));

            // Set the scissor rect to only update the current scanline.
            render_pass.set_scissor_rect(0, current_scanline as u32, self.size.width, 1);

            render_pass.draw(0..self.scanline_buffer_pipeline_num_vertices, 0..1);
        }

        // Update the tile map if the tile map currently in use changed or if we switched
        // the tile map we are using since the last scanline
        if rust_boy_gpu.current_background_tile_map_changed()
            | rust_boy_gpu.memory_changed.background_tile_map_flag_changed
        {
            // trace!("Updating tilemap");
            // trace!(
            //     "Current Scrolling: x: {} y: {}",
            //     rust_boy_gpu.gpu_registers.get_scroll_x() as u32,
            //     rust_boy_gpu.gpu_registers.get_scroll_y() as u32,
            // );
            // trace!(
            //     "New Tilemap (in use) \n {} \n \n",
            //     tile_map_to_string(rust_boy_gpu.get_background_tile_map())
            // );

            // Update tilemap and tile atlas (e.g., VRAM changes)
            let new_tilemap_data = rust_boy_gpu.get_background_tile_map();
            let tilemap = TilemapUniform::from_array(new_tilemap_data);
            self.queue.write_buffer(
                &self.background_tile_map_buffer,
                0,
                bytemuck::cast_slice(&[tilemap]),
            );
        }

        // Update the tile data if the tile data currently in use changed or if we switched
        // the tile map we are using since the last scanline
        if rust_boy_gpu.current_tile_data_changed()
            | rust_boy_gpu.memory_changed.tile_data_flag_changed
        {
            #[cfg(debug_assertions)]
            {
                // For debug
                // trace!("Updating tile data");
                // let tile_data_as_tiles = rust_boy_gpu.get_background_and_window_tile_data_debug();
                // trace!("Tile data: \n {}", tile_data_to_string(&tile_data_as_tiles));
                // trace!(
                //     "Tile data Block 0 and 1: \n {}",
                //     tile_data_to_string(
                //         &rust_boy_gpu.get_background_and_window_tile_data_block_0_and_1_debug()
                //     )
                // );
                // trace!(
                //     "Tile data Block 2 and 1: \n {}",
                //     tile_data_to_string(
                //         &rust_boy_gpu.get_background_and_window_tile_data_block_2_and_1_debug()
                //     )
                // );
            }
            let new_background_tile_data_plain = rust_boy_gpu.get_background_and_window_tile_data();
            self.queue.write_buffer(
                &self.bg_and_wd_tile_data_buffer,
                0,
                bytemuck::cast_slice(&[TileData::from_array(new_background_tile_data_plain)]),
            );
        }

        // Update the background viewport position if it changed since the last scanline
        if rust_boy_gpu
            .memory_changed
            .background_viewport_position_changed
        {
            let updated_background_viewport_position = BackgroundViewportPosition {
                pos: [
                    rust_boy_gpu.gpu_registers.get_scroll_x() as u32,
                    rust_boy_gpu.gpu_registers.get_scroll_y() as u32,
                    0,
                    0,
                ],
            };
            self.queue.write_buffer(
                &self.background_viewport_buffer,
                0,
                bytemuck::cast_slice(&[updated_background_viewport_position]),
            );
        }

        // Update the palette buffer if the palettes have changed
        let updated_palettes = Palettes {
            values: [
                rust_boy_gpu.gpu_registers.get_background_palette() as u32,
                rust_boy_gpu.gpu_registers.get_object_palette_zero() as u32,
                rust_boy_gpu.gpu_registers.get_object_palette_one() as u32,
                0,
            ],
        };
        self.queue.write_buffer(
            &self.palette_buffer,
            0,
            bytemuck::cast_slice(&[updated_palettes]),
        );

        // Update the current scanline and object size uniform buffer
        let updated_current_scanline = RenderingLinePositionAndObjectSize {
            pos: [
                current_scanline as u32,
                rust_boy_gpu.gpu_registers.get_sprite_size_flag() as u32,
                0,
                0,
            ],
        };
        self.queue.write_buffer(
            &self.rendering_line_and_obj_size_buffer,
            0,
            bytemuck::cast_slice(&[updated_current_scanline]),
        );

        // Update the object tile data buffer if it changed since the last scanline
        if rust_boy_gpu.memory_changed.tile_data_block_0_1_changed {
            let new_object_tile_data = rust_boy_gpu.get_object_tile_data();
            self.queue.write_buffer(
                &self.object_tile_data_buffer,
                0,
                bytemuck::cast_slice(&[TileData::from_array(new_object_tile_data)]),
            );
        }

        // Update the objects in scanline buffer
        let mut objects_in_scanline =
            rust_boy_gpu.get_objects_for_current_scanline(current_scanline);
        // Sort objects in scanline by their x coordinate, see https://gbdev.io/pandocs/OAM.html#drawing-priority
        objects_in_scanline.sort_by(|v, w| custom_ordering(v[1], w[1]));
        let new_objects_in_scanline = ObjectsInScanline {
            objects: objects_in_scanline,
        };
        self.queue.write_buffer(
            &self.objects_in_scanline_buffer,
            0,
            bytemuck::cast_slice(&[new_objects_in_scanline]),
        );
        // DEBUG
        #[cfg(debug_assertions)]
        {
            // let objects_tile_data = rust_boy_gpu.get_object_tile_data();
            // let objects_tile_data = TileData::from_array(objects_tile_data);
            // trace!(
            //     "{:?}",
            //     objects_tile_data.tiles[objects_in_scanline[0][3] as usize]
            // );
        }

        // Reset the changed flags so on the next scanline only buffers are updated which need to be
        rust_boy_gpu.memory_changed = ChangesToPropagateToShader::new();

        // Submit the compute commands to the GPU
        // Submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
