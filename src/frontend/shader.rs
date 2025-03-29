use crate::{ORIGINAL_SCREEN_HEIGHT, ORIGINAL_SCREEN_WIDTH};
use bytemuck::cast;
use wgpu::util::DeviceExt;
use wgpu::{Device, SurfaceConfiguration};

/// Struct to represent vertices for the vertex buffers of the shader pipelines.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

/// The vertices of a rectangle consisting of two triangles. This is used to render the framebuffer
/// texture to the screen.
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
    /// Returns the vertex buffer layout for the vertex buffer of the shader pipeline.
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

/// A struct to pass the tile data to the scanline shader.  Each tile consists of 8 x 8 pixels.
/// Each pixel is represented by 2 bits, therefore each tile consists of 8 x 8 * 2 = 128 bits =
/// 16 bytes = 4 u32s. Furthermore, there are 16 x 16 = 256 tiles in the tilemap.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct TileData {
    pub tiles: [[u32; 4]; 256],
}

/// A struct to pack the tilemap data and then pass it to the scanline shader. We pass the tilemap as a list of
/// [PackedTilemapData] instances because the alignment has to be a multiple of 4 bytes.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PackedTilemapData {
    pub indices: [u32; 4],
}

/// A struct to pass the tilemap data to the scanline shader. The tilemap consists of 32 x 32 = 256
/// tiles.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct TilemapUniform {
    pub(super) tiles: [PackedTilemapData; 256], // 32x32 grid
}

/// Stores the objects that are in the current scanline in a format that can be passed to the
/// scanline shader.
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
pub struct BgAndWdViewportPosition {
    pub pos: [u32; 4],
}

/// Represents the current rendering line and the object size flag. Is a list of 4 elements just for alignment, we only use
/// the first and second entry. They are the current scanline and the object size flag (0 for 8x8, 1 for 8x16).
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderingLinePositionAndObjectSize {
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

/// Represents the palettes used for the background, window and objects. Is a list of 4 elements just
/// for alignment purposes. The last entry is not used. The first entry is the background and
/// window palette that corresponds to register 0xFF47. The second entry is the object
/// palette 0 that corresponds to register 0xFF48. The third entry is the object palette 1 that
/// corresponds to register 0xFF49. See https://gbdev.io/pandocs/Palettes.html#lcd-monochrome-palettes
/// for more information.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Palettes {
    pub values: [u32; 4],
}

/// Sets up the render shader pipeline.
/// This pipeline is used to render the framebuffer texture to the screen. It is called in the
/// VBlank period of the RustBoy.
/// It uses very simple vertex and fragment shaders. The vertex shader simply creates two triangles
/// which form a rectangle of the desired size. The fragment shader simply takes the color of the
/// pixel from the framebuffer texture and writes it to the screen.
///
/// The return values are as follows:
/// - `wgpu::RenderPipeline` The render pipeline.
/// - `wgpu::Buffer` The vertex buffer.
/// - `wgpu::Buffer` The screensize buffer.
/// - `u32` The number of vertices.
/// - `wgpu::BindGroup` The bind group.
///
/// For their details, see the documentation of the fields of [crate::frontend::State] struct.
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
    // Configuration for the sampler that is used to sample the framebuffer texture.
    let framebuffer_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Framebuffer Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest, // Critical for crisp pixels
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    // Sets the screensize for the rendering shader. See the [screensize_buffer] field of the
    // [crate::frontend::State] struct for more details.
    let initial_screensize = CurrentScreensize {
        size: [ORIGINAL_SCREEN_WIDTH, ORIGINAL_SCREEN_HEIGHT, 0, 0],
    };
    let screensize_buffer: wgpu::Buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screensize Buffer"),
            contents: bytemuck::cast_slice(&[initial_screensize]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    // Create bind group layout for the render pipeline
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Render Pipeline Bind Group Layout"),
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
        label: Some("Render Pipeline Bind Group"),
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
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/render_to_screen.wgsl").into()),
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

/// Sets up the scanline shader pipeline.
/// This pipeline is used to render scanline by scanline to the buffer texture. It is called in the
/// after each HBlank period of the RustBoy, that is, when a scanline is finished.
///
/// The return values are as follows:
/// - `wgpu::RenderPipeline` The render pipeline.
/// - `wgpu::Buffer` The vertex buffer.
/// - `u32` The number of vertices.
/// - `wgpu::BindGroup` The bind group.
/// - `wgpu::Buffer` The tile data buffer.
/// - `wgpu::Buffer` The background tilemap buffer.
/// - `wgpu::Buffer` The window tilemap buffer.
/// - `wgpu::Buffer` The background and window viewport position buffer.
/// - `wgpu::Buffer` The palette buffer.
/// - `wgpu::Texture` The framebuffer texture.
/// - `wgpu::Buffer` The rendering line and object size buffer.
/// - `wgpu::Buffer` The object tile data buffer.
/// - `wgpu::Buffer` The objects in scanline buffer.
///
/// For their details, see the documentation of the fields of [crate::frontend::State] struct.
pub fn setup_scanline_shader_pipeline(
    device: &Device,
) -> (
    wgpu::RenderPipeline,
    wgpu::Buffer,
    u32,
    wgpu::BindGroup,
    wgpu::Buffer,
    wgpu::Buffer,
    wgpu::Buffer,
    wgpu::Buffer,
    wgpu::Buffer,
    wgpu::Texture,
    wgpu::Buffer,
    wgpu::Buffer,
    wgpu::Buffer,
) {
    // This holds the background and window tiles. 
    // For more details see the [bg_and_wd_tile_data_buffer] field of the [crate::frontend::State] struct.
    let initial_tile_data_buffer_plain = [0u8; 16 * 16 * 16];
    let initial_tile_data_buffer = TileData::from_array(initial_tile_data_buffer_plain);
    let bg_and_wd_tile_data_buffer: wgpu::Buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tile Data Buffer"),
            contents: bytemuck::cast_slice(&[initial_tile_data_buffer]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    // Represents which background tiles are displayed where.
    // For more details see the [background_tilemap_buffer] field of the [crate::frontend::State] struct.
    let initial_background_tilemap_plain = [0u8; 32 * 32];
    let initial_background_tilemap = TilemapUniform::from_array(&initial_background_tilemap_plain);
    let background_tilemap_buffer: wgpu::Buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tilemap Buffer"),
            contents: bytemuck::cast_slice(&[initial_background_tilemap]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    // Represents which window tiles are displayed where.
    // For more details see the [window_tilemap_buffer] field of the [crate::frontend::State] struct.
    let initial_window_tilemap_plain = [0u8; 32 * 32];
    let initial_window_tilemap = TilemapUniform::from_array(&initial_window_tilemap_plain);
    let window_tilemap_buffer: wgpu::Buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tilemap Buffer"),
            contents: bytemuck::cast_slice(&[initial_window_tilemap]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    // Sets the positions from where the background and the window are drawn.
    // For more details see the [bg_and_wd_viewport_buffer] field of the [crate::frontend::State] struct.
    let initial_bg_and_wd_viewport_position = BgAndWdViewportPosition { pos: [0, 0, 0, 0] };
    let bg_and_wd_viewport_buffer: wgpu::Buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Background Viewport Buffer"),
            contents: bytemuck::cast_slice(&[initial_bg_and_wd_viewport_position]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    // Setup framebuffer texture.
    // For more details see the [framebuffer_texture] field of the [crate::frontend::State] struct.
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
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    // Buffer to hold the current line to be rendered, the lcd control register and some
    // window internal line counter information.
    // For more details see the [rendering_line_lcd_control_and_window_internal_line_info_buffer] field of the [crate::frontend::State] struct.
    let initial_rendering_line_lcd_control_and_window_internal_line_info =
        RenderingLinePositionAndObjectSize { pos: [0, 0, 0, 0] };
    let rendering_line_lcd_control_and_window_internal_line_info_buffer: wgpu::Buffer = device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rendering Line and Object Size Buffer"),
            contents: bytemuck::cast_slice(&[
                initial_rendering_line_lcd_control_and_window_internal_line_info,
            ]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    // This holds the palettes used for the background, window and objects.
    // For more details see the [palette_buffer] field of the [crate::frontend::State] struct.
    let initial_palette = Palettes {
        values: [0, 0, 0, 0],
    };
    let palette_buffer: wgpu::Buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Palette Buffer"),
            contents: bytemuck::cast_slice(&[initial_palette]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    // This holds the object tile data.
    // For more details see the [object_tile_data_buffer] field of the [crate::frontend::State] struct.
    let object_tile_data_buffer = [0u8; 16 * 16 * 16];
    let initial_object_tile_data_buffer = TileData::from_array(object_tile_data_buffer);
    let object_tile_data_buffer: wgpu::Buffer =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Object Tile Data Buffer"),
            contents: bytemuck::cast_slice(&[initial_object_tile_data_buffer]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    // Represents the objects that are visible in the current scanline.
    // For more details see the [objects_in_scanline_buffer] field of the [crate::frontend::State] struct.
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
        label: Some("Scanline Shader Bind Group Layout"),
        entries: &[
            // BG/Window Tile Data Buffer
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Rendering Line Buffer
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Background Tilemap Uniform Buffer
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
            // Window Tilemap Uniform Buffer
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
            // Background and Window Viewport Position Uniform Buffer
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Palettes Uniform Buffer
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Object Tile Data Buffer
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Objects in scanline Uniform Buffer
            wgpu::BindGroupLayoutEntry {
                binding: 7,
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

    // Create the bind group
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Scanline Buffer Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: bg_and_wd_tile_data_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: rendering_line_lcd_control_and_window_internal_line_info_buffer
                    .as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: background_tilemap_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: window_tilemap_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: bg_and_wd_viewport_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: palette_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: object_tile_data_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: objects_in_scanline_buffer.as_entire_binding(),
            },
        ],
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/scanline_shader.wgsl").into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Scanline Shader Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let scanline_buffer_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Scanline Render Pipeline"),
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
                format: wgpu::TextureFormat::Rgba8Unorm,
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
        scanline_buffer_pipeline,
        vertex_buffer,
        num_vertices,
        bind_group,
        bg_and_wd_tile_data_buffer,
        background_tilemap_buffer,
        window_tilemap_buffer,
        bg_and_wd_viewport_buffer,
        palette_buffer,
        framebuffer_texture,
        rendering_line_lcd_control_and_window_internal_line_info_buffer,
        object_tile_data_buffer,
        objects_in_scanline_buffer,
    )
}
