use wgpu::util::DeviceExt;
use wgpu::{Buffer, Device, SurfaceConfiguration};

pub(super) const TILE_SIZE: u32 = 8;
pub(super) const ATLAS_COLS: u32 = 16;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

pub(super) const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, 1.0, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
        color: [1.0, 0.0, 0.0],
    },
];

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PackedTileData {
    pub indices: [u32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct TilemapUniform {
    pub(super) tiles: [PackedTileData; 256], // 32x32 grid
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct ObjectsInScanline {
    pub(super) objects: [[u32; 4]; 10],
}

impl TilemapUniform {
    pub fn from_array(input: &[u8; 1024]) -> Self {
        let mut tiles = [PackedTileData { indices: [0; 4] }; 256];

        for i in 0..256 {
            tiles[i].indices = [
                input[i * 4] as u32,
                input[i * 4 + 1] as u32,
                input[i * 4 + 2] as u32,
                input[i * 4 + 3] as u32,
            ];
        }

        TilemapUniform { tiles }
    }
}

/// Represents the position of the viewport of the background in the tilemap. Is a list of 4 elements
/// just for alignment, we only use the first 2.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BackgroundViewportPosition {
    pub pos: [u32; 4],
}

/// Represents the current rendering line. Is a list of 4 elements just for alignment, we only use
/// the first entry.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderingLinePosition {
    pub pos: [u32; 4],
}

/// Sets up the render shader pipeline.
/// This pipeline is used to render the framebuffer texture to the screen. It is called in the
/// VBlank period of the RustBoy.
/// It uses very simple vertex and fragment shaders. The vertex shader simply creates two triangles
/// which form a rectangle of the desired size. The fragment shader simply takes the color of the
/// pixel from the framebuffer texture and writes it to the screen.
/// The framebuffer texture has to be provided as a parameter. It is created in the compute
/// shader pipeline and is used to buffer the rendered lines.
///
/// TODO: Add more docstring describing return values
pub fn setup_render_shader_pipeline(
    device: &Device,
    config: &SurfaceConfiguration,
    framebuffer_texture: &wgpu::Texture,
) -> (wgpu::RenderPipeline, Buffer, u32, wgpu::BindGroup) {
    // Configuration for the sampler
    let framebuffer_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Framebuffer Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest, // Critical for crisp pixels
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    // Create bind group layout for the framebuffer
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Framebuffer Bind Group Layout"),
        entries: &[
            // Framebuffer Texture (binding 0)
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // Framebuffer Sampler (binding 1)
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });

    // Create bind group for the framebuffer
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Framebuffer Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&framebuffer_texture.create_view(
                    &wgpu::TextureViewDescriptor {
                        format: Some(wgpu::TextureFormat::Rgba8Unorm),
                        ..Default::default()
                    },
                )),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&framebuffer_sampler),
            },
        ],
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let num_vertices = VERTICES.len() as u32;

    (render_pipeline, vertex_buffer, num_vertices, bind_group)
}

/// TODO: Add docstring
pub fn setup_compute_shader_pipeline(
    device: &Device,
) -> (
    wgpu::ComputePipeline,
    wgpu::BindGroup,
    wgpu::Texture,
    Buffer,
    Buffer,
    wgpu::Texture,
    Buffer,
    wgpu::Texture,
    Buffer,
) {
    // This holds the background and window tiles (16x16 tiles, 8x8 pixels each)
    let background_tile_atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Background Tile Atlas"),
        size: wgpu::Extent3d {
            // Each tile is 8x8 pixels, and we have 16x16 tiles
            width: TILE_SIZE * ATLAS_COLS,
            height: TILE_SIZE * ATLAS_COLS,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb, // Rust Boy uses 4 colors (RGBA for simplicity)
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    // Represents which tiles are displayed where (Rust Boy: 32x32 tile grid)
    // Initialize blank tilemap (0th tile always)
    let initial_tilemap_data = [0u8; 32 * 32];
    let initial_tilemap = TilemapUniform::from_array(&initial_tilemap_data);
    let tilemap_buffer: Buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Tilemap Buffer"),
        contents: bytemuck::cast_slice(&[initial_tilemap]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // Sets the position from where the background is drawn. Used for scrolling. Is given as pixel
    // shift-values in the tilemap.
    let initial_background_viewport_position = BackgroundViewportPosition { pos: [0, 0, 0, 0] };
    let background_viewport_buffer: Buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Background Viewport Buffer"),
            contents: bytemuck::cast_slice(&[initial_background_viewport_position]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    // Setup framebuffer texture. This buffers the frame to be sent to the fragment shader
    // which will transfer it to the screen. Therefore, it is as big as the screen.
    let framebuffer_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Framebuffer"),
        size: wgpu::Extent3d {
            width: 160,
            height: 144,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
    });

    // Sets the position from where the background is drawn. Used for scrolling. Is given as pixel
    // shift-values in the tilemap.
    let initial_rendering_line = RenderingLinePosition { pos: [0, 0, 0, 0] };
    let rendering_line_buffer: Buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rendering Line Buffer"),
            contents: bytemuck::cast_slice(&[initial_rendering_line]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    // This holds the background and window tiles (16x16 tiles, 8x8 pixels each)
    let object_tile_atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Object Tile Atlas"),
        size: wgpu::Extent3d {
            // Each tile is 8x8 pixels, and we have 16x16 tiles
            width: TILE_SIZE * ATLAS_COLS,
            height: TILE_SIZE * ATLAS_COLS,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb, // Rust Boy uses 4 colors (RGBA for simplicity)
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    // Represents the objects that
    let initial_objects_in_scanline = ObjectsInScanline {
        objects: [[0; 4]; 10],
    };
    let objects_in_scanline_buffer: Buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tilemap Buffer"),
            contents: bytemuck::cast_slice(&[initial_objects_in_scanline]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    // Create the bind group layout
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Compute Shader Bind Group Layout"),
        entries: &[
            // Tile Atlas Texture (binding 0)
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // Rendering Line Buffer (binding 1)
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Tilemap Uniform Buffer (binding 2)
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Background Viewport Position Uniform Buffer (binding 3)
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Framebuffer Texture (binding 4)
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            // Object Atlas Texture (binding 5)
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // Objects in scanline Uniform Buffer (binding 6)
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    // Create the bind group
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Main Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &background_tile_atlas_texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: rendering_line_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: tilemap_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: background_viewport_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(
                    &framebuffer_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(
                    &object_tile_atlas_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: objects_in_scanline_buffer.as_entire_binding(),
            },
        ],
    });

    let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Compute Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/compute_shader.wgsl").into()),
    });

    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Compute Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &compute_shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    (
        compute_pipeline,
        bind_group,
        background_tile_atlas_texture,
        tilemap_buffer,
        background_viewport_buffer,
        framebuffer_texture,
        rendering_line_buffer,
        object_tile_atlas_texture,
        objects_in_scanline_buffer,
    )
}
