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

// The frame buffer stores the current state of the frame and is received from the compute shader which writes line by
// line to it until the entire frame is ready (144 lines). Therefore, the texture is 160x144 pixels, which is the size of
// the screen.
@group(0) @binding(0) var frameBufferTexture: texture_2d<f32>;
// Sampler for the frame buffer
@group(0) @binding(1) var frameBufferSampler: sampler;
// The current screensize in pixels (x,y) are the first two components of the vector. The last two are unused and just
// serve as padding.
@group(0) @binding(2) var<uniform> current_screensize: vec4<u32>;

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert the uniform screen size to f32.
    let screensize = vec2<f32>(f32(current_screensize.x), f32(current_screensize.y));

    // Use the vertex's clip_position directly as pixel coordinates.
    let pixel_coord = in.clip_position.xy;

    // Determine how many screen pixels correspond to one Game Boy pixel.
    let scale = screensize / vec2<f32>(160.0, 144.0);

    // Calculate the corresponding original pixel index.
    let original_pixel = floor(pixel_coord / scale);

    // Clamp the index to valid range (texture pixel indices run from 0 to 159 in x, and 0 to 143 in y).
    let clamped_pixel = clamp(original_pixel, vec2<f32>(0.0), vec2<f32>(159.0, 143.0));

    // Compute UV coordinates by sampling at the center of the texel.
    let uv = (clamped_pixel + vec2<f32>(0.5)) / vec2<f32>(160.0, 144.0);

    return textureSample(frameBufferTexture, frameBufferSampler, uv);
}