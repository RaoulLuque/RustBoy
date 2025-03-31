pub(crate) mod shader;

use winit::event::WindowEvent;
use winit::window::Window;

use super::{MEMORY_SIZE, MemoryBus, ORIGINAL_SCREEN_WIDTH};
use crate::frontend::shader::{
    ObjectsInScanline, TileData, TilemapUniform, setup_render_shader_pipeline,
    setup_scanline_shader_pipeline,
};
use crate::ppu::PPU;
use crate::ppu::information_for_shader::ChangesToPropagateToShader;
use crate::ppu::object_handling::custom_ordering;

/// Big struct capturing the current state of the window and shader pipeline, including its buffers.
pub struct State<'a> {
    /// The surface to render to (the window's screen).
    surface: wgpu::Surface<'a>,
    /// The device to use for rendering (the GPU).
    device: wgpu::Device,
    /// The queue to use for rendering (the command queue).
    queue: wgpu::Queue,
    /// The configuration for the surface.
    config: wgpu::SurfaceConfiguration,
    /// The size of the window.
    pub(super) size: winit::dpi::PhysicalSize<u32>,
    /// The window to render to, which "owns" the surface.
    pub(super) window: &'a Window,

    /// The render pipeline to use for rendering.
    render_pipeline: wgpu::RenderPipeline,
    /// The vertex buffer to use for rendering. Used to store the vertex data
    /// for the render pipeline (two triangles forming a rectangle).
    render_pipeline_vertex_buffer: wgpu::Buffer,
    /// The buffer to hold the screensize (width x height in pixels).
    screensize_buffer: wgpu::Buffer,
    /// A flag to indicate if the screensize has changed. Used to ensure the
    /// shader is informed of the new screensize.
    screensize_changed: bool,
    /// The number of vertices in the vertex buffer (4).
    render_pipeline_num_vertices: u32,
    /// The bind group corresponding to the render pipeline which renders the
    /// `framebuffer_texture` to the screen.
    render_bind_group: wgpu::BindGroup,

    /// The compute pipeline that runs the compute shader. This shader writes to the
    /// framebuffer texture for every RustBoy render line (144 times per frame).
    scanline_buffer_pipeline: wgpu::RenderPipeline,
    /// The vertex buffer used to store vertex data for the render pipeline (two
    /// triangles forming a rectangle).
    scanline_buffer_pipeline_vertex_buffer: wgpu::Buffer,
    /// The number of vertices in the vertex buffer (4).
    scanline_buffer_pipeline_num_vertices: u32,
    /// The bind group corresponding to the compute pipeline.
    scanline_buffer_bind_group: wgpu::BindGroup,

    /// The buffer to hold the background and window tile data. It consists of 16 x 16 tiles in a
    /// 2D grid, each of which is 8 x 8 pixels. Each pixel takes up two bits, which results in 16
    /// bytes per tile and a total size of 4096 bytes for the buffer.
    bg_and_wd_tile_data_buffer: wgpu::Buffer,
    /// Holds the background tilemap data. Is a flattened 32x32 u8 array to hold the (currently used)
    /// tilemap data for the background. The tilemap is used to look up the tiles to be drawn on the screen.
    background_tilemap_buffer: wgpu::Buffer,
    /// Holds the window tilemap data. Is a flattened 32x32 u8 array to hold the (currently used)
    /// tilemap data for the window. The tilemap is used to look up the tiles to be drawn on the screen.
    window_tilemap_buffer: wgpu::Buffer,
    /// Buffer to hold the background and window viewport position. The viewport position is used to
    /// calculate the position of the background and window on the screen. It is a list of four u32s
    /// where the first two are the x and y position of the background and the last two the x and y
    /// positions of the window. Note that for the background, this can be interpreted as the position
    /// of the screen within the background tilemap, whereas for the window it can be seen as the
    /// position of the window (tilemap) within the screen.
    bg_and_wd_viewport_buffer: wgpu::Buffer,
    /// The buffer to hold the object/sprite tile data. It consists of 16 x 16 tiles in a
    /// 2D grid, each of which is 8 x 8 pixels. Each pixel takes up two bits, which results in 16
    /// bytes per tile and a total size of 4096 bytes for the buffer.
    object_tile_data_buffer: wgpu::Buffer,
    /// This buffer contains the objects that should be drawn on the current scanline. It always
    /// has length 10, but the number of objects that are in the current scanline might be less.
    /// In that case, the rest of the buffer is filled with zeroes. Each object consists of four
    /// bytes which give the information about the object. The bytes are as follows:
    /// - Byte 0: The y coordinate of the object (with some extras, see [Pan Docs](https://gbdev.io/pandocs/OAM.html)).
    /// - Byte 1: The x coordinate of the object (with some extras, see [Pan Docs](https://gbdev.io/pandocs/OAM.html)).
    /// - Byte 2: The tile index of the object.
    /// - Byte 3: The attributes of the object.
    ///
    /// See also [Pan Docs](https://gbdev.io/pandocs/OAM.html).
    objects_in_scanline_buffer: wgpu::Buffer,
    /// Buffer to hold the palette data (a u32 array of 4 elements). The first three elements
    /// mirror the registers FF47, FF48, and FF49 as specified in the Pandocs
    /// (https://gbdev.io/pandocs/Palettes.html), making them available to the shader.
    /// - The first entry is the background palette (FF47).
    /// - The second entry is the object palette 0 (FF48).
    /// - The third entry is the object palette 1 (FF49).
    /// - The fourth entry is empty (zero).
    palette_buffer: wgpu::Buffer,
    /// Buffer to hold different rendering info.
    /// This includes the current scanline, the LCD control register, and the window
    /// internal line info. More precisely the entries are as follows:
    /// - The first entry is the current scanline index.
    /// - The second entry is the LCD control register (FF40).
    /// - The third entry is a flag whether the window is being drawn this scanline.
    /// - The fourth entry is the window internal line counter, that is, if the window is being
    /// drawn this scanline, the line that is taken from the window tilemap, see
    /// [Pan Docs](https://gbdev.io/pandocs/Scrolling.html#window).
    rendering_line_lcd_control_and_window_internal_line_info_buffer: wgpu::Buffer,

    /// This texture is used as an "offscreen" framebuffer of size 160 x 144 pixels. That is, the size
    /// of the original GameBoy screen. We use a framebuffer to render scanline by scanline to it
    /// and then render the entire framebuffer at once to the screen.
    /// Note that this is not a storage texture since WebGL2 and therefore WASM as a target does not
    /// support storage textures.
    framebuffer_texture: wgpu::Texture,
}

impl<'a> State<'a> {
    /// Creates a new instance of [State]. This function is called once at the beginning of the
    /// program to set up the GPU (of the Host) and the window.
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
            bg_and_wd_tile_data_buffer,
            background_tilemap_buffer,
            window_tilemap_buffer,
            bg_and_wd_viewport_buffer,
            palette_buffer,
            framebuffer_texture,
            rendering_line_lcd_control_and_window_internal_line_info_buffer,
            object_tile_data_buffer,
            objects_in_scanline_buffer,
        ) = setup_scanline_shader_pipeline(&device);

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
            bg_and_wd_tile_data_buffer,
            background_tilemap_buffer,
            window_tilemap_buffer,
            bg_and_wd_viewport_buffer,
            palette_buffer,
            framebuffer_texture,
            rendering_line_lcd_control_and_window_internal_line_info_buffer,
            object_tile_data_buffer,
            objects_in_scanline_buffer,
        }
    }

    /// Get a reference to the window.
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Resize the window to the provided new_size.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.screensize_changed = true;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Check if an event is a valid input event.
    pub fn input(&mut self, _: &WindowEvent) -> bool {
        false
    }

    /// Render the screen. This function is called once per frame to render the
    /// current framebuffer to the screen using the render shader pipeline.
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

    /// Render the provided `current_scanline` scanline to the framebuffer texture.
    /// This function is called once per frame to render the current scanline to the screen using
    /// the scanline shader pipeline.
    pub fn render_scanline(
        &mut self,
        rust_boy_ppu: &mut PPU,
        memory_bus: &mut MemoryBus,
        current_scanline: u8,
    ) {
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
            render_pass.set_scissor_rect(0, current_scanline as u32, ORIGINAL_SCREEN_WIDTH, 1);

            render_pass.draw(0..self.scanline_buffer_pipeline_num_vertices, 0..1);
        }

        // Update the background tilemap if the tilemap currently in use changed or if we switched
        // the tilemap we are using since the last scanline
        if PPU::current_background_tile_map_changed(memory_bus)
            | memory_bus.memory_changed.background_tile_map_flag_changed
        {
            // trace!("Updating tilemap");
            // trace!(
            //     "Current Scrolling: x: {} y: {}",
            //     rust_boy_gpu.gpu_registers.get_bg_scroll_x() as u32,
            //     rust_boy_gpu.gpu_registers.get_bg_scroll_y() as u32,
            // );
            // trace!(
            //     "New Tilemap (in use) \n {} \n \n",
            //     tile_map_to_string(rust_boy_gpu.get_background_tile_map())
            // );

            // Update tilemap and tile atlas (e.g., VRAM changes)
            let new_tilemap_data = rust_boy_ppu.buffers_for_rendering.background_tile_map;
            let tilemap = TilemapUniform::from_array(&new_tilemap_data);
            self.queue.write_buffer(
                &self.background_tilemap_buffer,
                0,
                bytemuck::cast_slice(&[tilemap]),
            );
        }

        // Update the background tilemap if the tilemap currently in use changed or if we switched
        // the tilemap we are using since the last scanline
        if PPU::current_window_tile_map_changed(memory_bus)
            | memory_bus.memory_changed.window_tile_map_flag_changed
        {
            // Update tilemap and tile atlas (e.g., VRAM changes)
            let new_tilemap_data = rust_boy_ppu.buffers_for_rendering.window_tile_map;
            let tilemap = TilemapUniform::from_array(&new_tilemap_data);
            self.queue.write_buffer(
                &self.window_tilemap_buffer,
                0,
                bytemuck::cast_slice(&[tilemap]),
            );
        }

        // Update the tile data if the tile data currently in use changed or if we switched
        // the tilemap we are using since the last scanline
        if PPU::current_bg_and_wd_tile_data_changed(memory_bus)
            | memory_bus.memory_changed.tile_data_flag_changed
        {
            // DEBUG
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

            let new_background_tile_data_plain =
                rust_boy_ppu.buffers_for_rendering.bg_and_wd_tile_data;
            self.queue.write_buffer(
                &self.bg_and_wd_tile_data_buffer,
                0,
                bytemuck::cast_slice(&[TileData::from_array(new_background_tile_data_plain)]),
            );
        }

        // Update the background and window viewport position if either of them changed since the last scanline
        if memory_bus
            .memory_changed
            .background_viewport_position_changed
            || memory_bus.memory_changed.window_viewport_position_changed
        {
            let updated_bg_and_wd_viewport_position = rust_boy_ppu
                .buffers_for_rendering
                .bg_and_wd_viewport_position;
            self.queue.write_buffer(
                &self.bg_and_wd_viewport_buffer,
                0,
                bytemuck::cast_slice(&[updated_bg_and_wd_viewport_position]),
            );
        }

        // Update the palette buffer if the palettes have changed
        let updated_palettes = rust_boy_ppu.buffers_for_rendering.palettes;
        self.queue.write_buffer(
            &self.palette_buffer,
            0,
            bytemuck::cast_slice(&[updated_palettes]),
        );

        // Update the current scanline and object size uniform buffer
        let updated_current_scanline_lcd_control_and_window_internal_line_info = rust_boy_ppu
            .buffers_for_rendering
            .rendering_line_lcd_control_and_window_internal_line_info;
        // DEBUG
        log::trace!(
            "Updated rendering_line_lcd_control_and_window_internal_line_info: {:?}",
            updated_current_scanline_lcd_control_and_window_internal_line_info
        );
        self.queue.write_buffer(
            &self.rendering_line_lcd_control_and_window_internal_line_info_buffer,
            0,
            bytemuck::cast_slice(&[
                updated_current_scanline_lcd_control_and_window_internal_line_info,
            ]),
        );

        // Update the object tile data buffer if it changed since the last scanline
        if memory_bus.memory_changed.tile_data_block_0_1_changed {
            let new_object_tile_data = rust_boy_ppu.buffers_for_rendering.object_tile_data;
            self.queue.write_buffer(
                &self.object_tile_data_buffer,
                0,
                bytemuck::cast_slice(&[TileData::from_array(new_object_tile_data)]),
            );
        }

        // Update the objects in scanline buffer
        let mut objects_in_scanline = rust_boy_ppu
            .buffers_for_rendering
            .objects_in_scanline_buffer;
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
        memory_bus.memory_changed = ChangesToPropagateToShader::new_false();

        // Submit the compute commands to the GPU
        // Submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
