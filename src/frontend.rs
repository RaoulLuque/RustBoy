mod shader;

use winit::event::WindowEvent;
use winit::window::Window;

use crate::frontend::shader::{setup_shader_pipeline, TilemapUniform, ATLAS_COLS, TILE_SIZE};
use crate::gpu::tile_handling::{tile_array_to_rgba_array, Tile};
use crate::gpu::GPU;

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
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    bind_group: wgpu::BindGroup,
    tilemap_buffer: wgpu::Buffer,
    tile_atlas_texture: wgpu::Texture,
}

impl<'a> State<'a> {
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
            render_pipeline,
            vertex_buffer,
            num_vertices,
            bind_group,
            tilemap_buffer,
            tile_atlas_texture,
        ) = setup_shader_pipeline(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            vertex_buffer,
            num_vertices,
            bind_group,
            tilemap_buffer,
            tile_atlas_texture,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self, rust_boy_gpu: &mut GPU) -> Result<(), wgpu::SurfaceError> {
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
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
        }
        if rust_boy_gpu.tile_map_changed() {
            println!("Updating tilemap");
            // Update tilemap and tile atlas (e.g., VRAM changes)
            let new_tilemap_data = [0u32; 32 * 32];
            let tilemap = TilemapUniform::from_array(&new_tilemap_data);
            self.queue
                .write_buffer(&self.tilemap_buffer, 0, bytemuck::cast_slice(&[tilemap]));
        }

        // Each pixel is 4 bytes (RGBA)
        // Each tile is 8x8 pixels, and we have 16x16 tiles
        // So we need (8 * 4) * 16 per row and 8 * 16 of these rows
        // let mut new_tile_data = [0u8; TILE_SIZE as usize
        //     * ATLAS_COLS as usize
        //     * TILE_SIZE as usize
        //     * ATLAS_COLS as usize
        //     * 4];
        //
        // new_tile_data[0..32].copy_from_slice(&[
        //     0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
        //     0xFF, 0xFF, 0xFF, 0xFF,
        // ]);
        // new_tile_data[(0 + (512))..(32 + (512))].copy_from_slice(&[
        //     0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00,
        //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        //     0x00, 0x00, 0x00, 0x00,
        // ]);
        // new_tile_data[(0 + 512 * 2)..(32 + 512 * 2)].copy_from_slice(&[
        //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
        //     0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
        //     0x00, 0x00, 0x00, 0x00,
        // ]);
        // new_tile_data[(0 + 512 * 3)..(32 + 512 * 3)].copy_from_slice(&[
        //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
        //     0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
        //     0x00, 0x00, 0x00, 0x00,
        // ]);
        // new_tile_data[(0 + 512 * 4)..(32 + 512 * 4)].copy_from_slice(&[
        //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
        //     0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
        //     0x00, 0x00, 0x00, 0x00,
        // ]);
        // new_tile_data[(0 + 512 * 5)..(32 + 512 * 5)].copy_from_slice(&[
        //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
        //     0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
        //     0x00, 0x00, 0x00, 0x00,
        // ]);
        // new_tile_data[(0 + 512 * 6)..(32 + 512 * 6)].copy_from_slice(&[
        //     0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00,
        //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        //     0x00, 0x00, 0x00, 0x00,
        // ]);
        // new_tile_data[(0 + 512 * 7)..(32 + 512 * 7)].copy_from_slice(&[
        //     0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
        //     0xFF, 0xFF, 0xFF, 0xFF,
        // ]);

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
            println!("Updating tile data");
            let new_tile_data = tile_array_to_rgba_array(
                <&[Tile; 256]>::try_from(&rust_boy_gpu.get_window_and_tile_data()).unwrap(),
            );
            // let new_tile_data = tile_array_to_rgba_array(&empty_tiles);
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

        // Submit the rendering commands to the GPU
        // Submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
