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

pub fn setup_shader_pipeline(
    device: &Device,
    config: &SurfaceConfiguration,
) -> (
    wgpu::RenderPipeline,
    Buffer,
    u32,
    wgpu::BindGroup,
    Buffer,
    wgpu::Texture,
    Buffer,
) {
    // This holds all possible tiles (16x16 tiles, 8x8 pixels each)
    let tile_atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Tile Atlas"),
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

    // Configuration for the sampler
    let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Atlas Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest, // Critical for crisp pixels
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
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

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Main Bind Group Layout"),
        entries: &[
            // Tile Atlas Texture (binding 0)
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
            // Atlas Sampler (binding 1)
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            // Tilemap Uniform Buffer (binding 2)
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
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
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Main Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &tile_atlas_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&atlas_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: tilemap_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: background_viewport_buffer.as_entire_binding(),
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

    (
        render_pipeline,
        vertex_buffer,
        num_vertices,
        bind_group,
        tilemap_buffer,
        tile_atlas_texture,
        background_viewport_buffer,
    )
}
