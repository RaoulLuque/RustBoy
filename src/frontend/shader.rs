use crate::{INITIAL_SCREEN_HEIGHT, INITIAL_SCREEN_WIDTH};
use bytemuck::cast;
use wgpu::util::DeviceExt;
use wgpu::{Device, SurfaceConfiguration};

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
pub(super) struct TileData {
    // Each tile consists of 8 x 8 pixels. Each pixel is represented by 2 bits, therefore each tile
    // consists of 8 x 8 * 2 = 128 bits = 16 bytes = 4 u32s. Furthermore, there are 16 x 16 = 256
    // tiles in the tilemap. Therefore, the size of the tiles array is 256 * 4 * 4 = 4096 bytes.
    pub tiles: [[u32; 4]; 256],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PackedTilemapData {
    pub indices: [u32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct TilemapUniform {
    pub(super) tiles: [PackedTilemapData; 256], // 32x32 grid
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct ObjectsInScanline {
    pub(super) objects: [[u32; 4]; 10],
}

impl TileData {
    /// Safely converts an input array of u8s of length 4096 to a TileData struct by using
    /// [bytemuck::cast].
    pub fn from_array(input: [u8; 4096]) -> Self {
        // This usage of cast is safe because we know that the size of the input array is 4096 bytes
        // and the size of the tile(s) array is 256 * 4 u32s = 4096 bytes.
        let tiles = cast(input);
        TileData { tiles }
    }
}

impl TilemapUniform {
    /// TODO: Change this to actually pack the data by packing 4 u8s into a u32 to save space.
    pub fn from_array(input: &[u8; 1024]) -> Self {
        let mut tiles = [PackedTilemapData { indices: [0; 4] }; 256];

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

/// Represents the current screensize of the window of the emulator. Is a list of 4 elements just for
/// alignment purposes. We only use the first two entries for the width and height of the screen in
/// pixels.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CurrentScreensize {
    pub size: [u32; 4],
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
) -> (
    wgpu::RenderPipeline,
    wgpu::Buffer,
    wgpu::Buffer,
    u32,
    wgpu::BindGroup,
) {
    // Configuration for the sampler
    let framebuffer_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Framebuffer Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest, // Critical for crisp pixels
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    // Sets the screensize for the rendering shader. Is a list of 4 elements for alignment purposes,
    // the first and two entries represent the width and height of the emulator window.
    let initial_screensize = CurrentScreensize {
        size: [INITIAL_SCREEN_WIDTH, INITIAL_SCREEN_HEIGHT, 0, 0],
    };
    let screensize_buffer: wgpu::Buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screensize Buffer"),
            contents: bytemuck::cast_slice(&[initial_screensize]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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
            // Screensize Buffer (binding 2)
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
            wgpu::BindGroupEntry {
                binding: 2,
                resource: screensize_buffer.as_entire_binding(),
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
        screensize_buffer,
        num_vertices,
        bind_group,
    )
}

/// TODO: Add docstring
pub fn setup_compute_shader_pipeline(
    device: &Device,
) -> (
    wgpu::ComputePipeline,
    wgpu::BindGroup,
    wgpu::Buffer,
    wgpu::Buffer,
    wgpu::Buffer,
    wgpu::Texture,
    wgpu::Buffer,
    wgpu::Buffer,
    wgpu::Buffer,
) {
    // This holds the background and window tiles (16x16 tiles, 16 bytes for each tile)
    let initial_tile_data_buffer_plain = [0u8; 16 * 16 * 16];
    let initial_tile_data_buffer = TileData::from_array(initial_tile_data_buffer_plain);
    let bg_and_wd_tile_data_buffer: wgpu::Buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tile Data Buffer"),
            contents: bytemuck::cast_slice(&[initial_tile_data_buffer]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

    // Represents which tiles are displayed where (Rust Boy: 32x32 tile grid)
    // Initialize blank tilemap (0th tile always)
    let initial_background_tilemap_plain = [0u8; 32 * 32];
    let initial_background_tilemap = TilemapUniform::from_array(&initial_background_tilemap_plain);
    let background_tilemap_buffer: wgpu::Buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tilemap Buffer"),
            contents: bytemuck::cast_slice(&[initial_background_tilemap]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    // Sets the position from where the background is drawn. Used for scrolling. Is given as pixel
    // shift-values in the tilemap.
    let initial_background_viewport_position = BackgroundViewportPosition { pos: [0, 0, 0, 0] };
    let background_viewport_buffer: wgpu::Buffer =
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
        view_formats: &[],
    });

    // Buffer to hold the current line to be rendered and whether the objects
    // are in size 8x8 or 8x16 mode for the compute shader
    let initial_rendering_line = RenderingLinePosition { pos: [0, 0, 0, 0] };
    let rendering_line_and_obj_size_buffer: wgpu::Buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rendering Line Buffer"),
            contents: bytemuck::cast_slice(&[initial_rendering_line]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    // This holds the background and window tiles (16x16 tiles, 16 bytes for each tile)
    let object_tile_data_buffer = [0u8; 16 * 16 * 16];
    let initial_object_tile_data_buffer = TileData::from_array(object_tile_data_buffer);
    let object_tile_data_buffer: wgpu::Buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Object Tile Data Buffer"),
            contents: bytemuck::cast_slice(&[initial_object_tile_data_buffer]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

    // Represents the objects that
    let initial_objects_in_scanline = ObjectsInScanline {
        objects: [[0; 4]; 10],
    };
    let objects_in_scanline_buffer: wgpu::Buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tilemap Buffer"),
            contents: bytemuck::cast_slice(&[initial_objects_in_scanline]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    // Create the bind group layout
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Compute Shader Bind Group Layout"),
        entries: &[
            // BG/Window Tile Data Buffer (binding 0)
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
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
            // Object Tile Data Buffer (binding 5)
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
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
                resource: bg_and_wd_tile_data_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: rendering_line_and_obj_size_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: background_tilemap_buffer.as_entire_binding(),
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
                resource: object_tile_data_buffer.as_entire_binding(),
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
        bg_and_wd_tile_data_buffer,
        background_tilemap_buffer,
        background_viewport_buffer,
        framebuffer_texture,
        rendering_line_and_obj_size_buffer,
        object_tile_data_buffer,
        objects_in_scanline_buffer,
    )
}
