// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@group(0) @binding(0) var tileAtlas: texture_2d<f32>;
@group(0) @binding(1) var atlasSampler: sampler;
@group(0) @binding(2) var<storage> tilemap: array<u32>;

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate the index of the tile the pixel is in
    let tile_size = 8;
    let tile_index_x = i32(in.clip_position.x / f32(tile_size));
    let tile_index_y = i32(in.clip_position.y / f32(tile_size));

    // Calculate the coordinates of the pixel within the tile
    let pixel_x = i32(in.clip_position.x) % i32(tile_size);
    let pixel_y = i32(in.clip_position.y) % i32(tile_size);

    let normalized = in.clip_position.xy / vec2<f32>(160.0, 144.0);
    let color_linear = vec3<f32>(normalized, 0.0);
    // Apply gamma correction
    let gamma = 2.2;
    let color_gamma_corrected = pow(color_linear, vec3<f32>(1.0 / gamma));
    return vec4<f32>(color_gamma_corrected, 1.0);
}



