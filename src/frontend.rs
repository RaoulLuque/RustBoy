mod shader;

use log::trace;
use winit::event::WindowEvent;
use winit::window::Window;

use crate::frontend::shader::{
    ATLAS_COLS, BackgroundViewportPosition, ObjectsInScanline, RenderingLinePosition, TILE_SIZE,
    TilemapUniform, setup_compute_shader_pipeline, setup_render_shader_pipeline,
};
use crate::gpu::GPU;
use crate::gpu::tile_handling::{
    Tile, tile_array_to_rgba_array, tile_data_to_string, tile_map_to_string,
};

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
    vertex_buffer: wgpu::Buffer,
    // The screensize buffer is used to store the size of the screen (Width x Height in pixels).
    screensize_buffer: wgpu::Buffer,
    // Variable to keep track of whether the screen size has changed or not
    screensize_changed: bool,
    // The number of vertices in the vertex buffer (4).
    num_vertices: u32,
    // The bind group corresponding to the render pipeline
    render_bind_group: wgpu::BindGroup,

    // The compute pipeline is the pipeline that will be used to run the compute shader. This
    // shader writes to the framebuffer texture for every RustBoy render line (that is 144 times
    // per frame).
    compute_pipeline: wgpu::ComputePipeline,
    // The bind group corresponding to the compute pipeline
    compute_bind_group: wgpu::BindGroup,
    // Tile atlas texture (128 x 128 rgba) to hold the (currently used) background tile data
    tile_atlas_texture: wgpu::Texture,
    // Tilemap buffer flattened 32x32 u8 array to hold the (currently used) tilemap data
    tilemap_buffer: wgpu::Buffer,
    // Buffer to hold the background viewport position (is a u32 array of 4 elements) where
    // the first two elements are the x and y position of the background viewport
    background_viewport_buffer: wgpu::Buffer,
    // Storage texture (160x144) to act as a framebuffer for the compute shader
    framebuffer_texture: wgpu::Texture,
    // Buffer to hold the current line to be rendered for the compute shader
    rendering_line_and_obj_size_buffer: wgpu::Buffer,

    // Storage texture (128 x 128 rgba) to hold the tiles used for objects/sprites
    object_tile_atlas_texture: wgpu::Texture,
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
            compute_pipeline,
            compute_bind_group,
            tile_atlas_texture,
            tilemap_buffer,
            background_viewport_buffer,
            framebuffer_texture,
            rendering_line_and_obj_size_buffer,
            object_tile_atlas_texture,
            objects_in_scanline_buffer,
        ) = setup_compute_shader_pipeline(&device);

        let (render_pipeline, vertex_buffer, screensize_buffer, num_vertices, render_bind_group) =
            setup_render_shader_pipeline(&device, &config, &framebuffer_texture);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            vertex_buffer,
            screensize_buffer,
            screensize_changed: false,
            num_vertices,
            render_bind_group,
            compute_pipeline,
            compute_bind_group,
            tile_atlas_texture,
            tilemap_buffer,
            background_viewport_buffer,
            framebuffer_texture,
            rendering_line_and_obj_size_buffer,
            object_tile_atlas_texture,
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
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
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

    pub fn render_compute(&mut self, rust_boy_gpu: &mut GPU, current_scanline: u8) {
        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        // Begin compute pass
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Line Render Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);

            // Dispatch 1 workgroup per scanline (160 pixels wide)
            // Workgroup size is defined as @workgroup_size(160, 1, 1)
            compute_pass.dispatch_workgroups(1, 1, 1);
        }

        if rust_boy_gpu.tile_map_changed() {
            trace!("Updating tilemap");
            trace!(
                "Current Scrolling: x: {} y: {}",
                rust_boy_gpu.gpu_registers.get_scroll_x() as u32,
                rust_boy_gpu.gpu_registers.get_scroll_y() as u32,
            );
            trace!(
                "New Tilemap (in use) \n {} \n \n",
                tile_map_to_string(rust_boy_gpu.get_background_tile_map())
            );
            // Update tilemap and tile atlas (e.g., VRAM changes)
            let new_tilemap_data = rust_boy_gpu.get_background_tile_map();
            let tilemap = TilemapUniform::from_array(new_tilemap_data);
            self.queue
                .write_buffer(&self.tilemap_buffer, 0, bytemuck::cast_slice(&[tilemap]));
        }

        let tile = [
            [crate::gpu::tile_handling::TilePixelValue::Three; 8],
            [crate::gpu::tile_handling::TilePixelValue::Zero; 8],
            [crate::gpu::tile_handling::TilePixelValue::Three; 8],
            [crate::gpu::tile_handling::TilePixelValue::Zero; 8],
            [crate::gpu::tile_handling::TilePixelValue::Three; 8],
            [crate::gpu::tile_handling::TilePixelValue::Zero; 8],
            [crate::gpu::tile_handling::TilePixelValue::Three; 8],
            [crate::gpu::tile_handling::TilePixelValue::Zero; 8],
        ];

        let mut empty_tiles = [[[crate::gpu::tile_handling::TilePixelValue::Zero; 8]; 8]; 256];
        empty_tiles[0] = tile;

        if rust_boy_gpu.tile_data_changed() {
            trace!("Updating tile data");
            let tile_data_as_tiles = rust_boy_gpu.get_background_and_window_tile_data();
            trace!("Tile data: \n {}", tile_data_to_string(&tile_data_as_tiles));
            trace!(
                "Tile data Block 0 and 1: \n {}",
                tile_data_to_string(
                    &rust_boy_gpu.get_background_and_window_tile_data_block_0_and_1()
                )
            );
            trace!(
                "Tile data Block 2 and 1: \n {}",
                tile_data_to_string(
                    &rust_boy_gpu.get_background_and_window_tile_data_block_2_and_1()
                )
            );
            let new_tile_data = tile_array_to_rgba_array(
                <&[Tile; 256]>::try_from(&rust_boy_gpu.get_background_and_window_tile_data())
                    .unwrap(),
            );
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.tile_atlas_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &new_tile_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * TILE_SIZE * ATLAS_COLS),
                    rows_per_image: None,
                },
                self.tile_atlas_texture.size(),
            );
        }

        // TODO: Update this once per frame only (or whenever it is supposed to be updated)
        // Update the background viewport position
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

        // Update the current scanline uniform buffer
        let updated_current_scanline = RenderingLinePosition {
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

        // Update the object tile atlas
        // TODO: Update this only when necessary
        let new_object_tile_data = tile_array_to_rgba_array(
            <&[Tile; 256]>::try_from(&rust_boy_gpu.get_object_tile_data()).unwrap(),
        );
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.object_tile_atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &new_object_tile_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * TILE_SIZE * ATLAS_COLS),
                rows_per_image: None,
            },
            self.object_tile_atlas_texture.size(),
        );

        // Update the objects in scanline buffer
        // TODO: Update this only when necessary
        let objects_in_scanline = rust_boy_gpu.get_objects_for_current_scanline(current_scanline);
        // let oam = rust_boy_gpu.oam;
        // if objects_in_scanline[0][0] != 0 {
        //     println!("Current scanline: {}", current_scanline);
        //     println!("Objects in scanline: {:?}", objects_in_scanline);
        //     println!("OAM: {:?}", oam);
        // }
        let new_objects_in_scanline = ObjectsInScanline {
            objects: objects_in_scanline,
        };
        self.queue.write_buffer(
            &self.objects_in_scanline_buffer,
            0,
            bytemuck::cast_slice(&[new_objects_in_scanline]),
        );
        // Submit the compute commands to the GPU
        // Submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}
