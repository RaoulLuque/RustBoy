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

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.clip_position.xy / vec2<f32>(160.0, 144.0);
    return textureSample(frameBufferTexture, frameBufferSampler, uv);
}




