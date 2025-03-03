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

// Struct to hold the tilemap. Ensures alignment that is multiple of 16 bytes.
struct TilemapUniform {
    tiles: array<vec4<u32>, 256>,
}

// Tile atlas
// Tile atlas is a 2D texture containing all the tiles used in the tilemap.
// The tiles here can be considered the building blocks used by the tilemap.
// Each tile is 8x8 pixels, with a total of 32 tiles, so the atlas is 256x256 pixels in total.
// It is encoded in Rgba8UnormSrgb format.
@group(0) @binding(0) var tileAtlas: texture_2d<f32>;
// Sampler for the tile atlas
@group(0) @binding(1) var atlasSampler: sampler;
// Tilemap
// Tilemap is a 32x32 array of u32s, the same size as the grid of tiles that is loaded in the Rust Boy.
// Each u32 is a tile index, which is used to look up the tile in the tile atlas.
@group(0) @binding(2) var<uniform> tilemap: TilemapUniform;

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Set the tile index of the top left tile from which we construct the
    // rust boy's screen. See https://gbdev.io/pandocs/Scrolling.html
    // This should be passed into the shader as a uniform buffer
    let top_left_tile_index = vec2<i32>(0, 0);

    // Set the size of the tile as a constant
    let tile_size = vec2<i32>(8, 8);

    let pixel_coords = in.clip_position.xy;

    // Calculate the index of the tile the pixel is in
    let tile_index_in_tilemap = (vec2<i32>(pixel_coords / vec2<f32>(tile_size)) + top_left_tile_index) % vec2<i32>(160, 144);
    // Calculate the index of the tile in the tile atlas
    let tilemap_flat_index = tile_index_in_tilemap.x + tile_index_in_tilemap.y * 32;
    let vec_index = tilemap_flat_index / 4;
    let comp_index = tilemap_flat_index % 4;

    var tile_index_in_atlas: u32;
    switch (comp_index) {
        case 0: { tile_index_in_atlas = tilemap.tiles[vec_index].x; break; }
        case 1: { tile_index_in_atlas = tilemap.tiles[vec_index].y; break; }
        case 2: { tile_index_in_atlas = tilemap.tiles[vec_index].z; break; }
        default: { tile_index_in_atlas = tilemap.tiles[vec_index].w; break; }
    }

//    Testing
//    if (comp_index == 0) {
//        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
//    } else if (comp_index == 1) {
//        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
//    } else if (comp_index == 2) {
//        return vec4<f32>(0.0, 1.0, 0.0, 1.0);
//    } else {
//        return vec4<f32>(0.0, 0.0, 1.0, 1.0);
//    }

//    Testing
//    if (tile_index_in_atlas == 0) {
//        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
//    } else {
//        return vec4<f32>(0.0, 1.0, 0.0, 1.0);
//    }

    // Calculate the coordinates of the pixel within the tile
    let pixel_index = vec2<i32>(pixel_coords) % vec2<i32>(tile_size);

    // Convert pixel position to normalized UV within the current 8x8 tile
    let tile_pixel_uv = vec2<f32>(pixel_index) / vec2<f32>(8.0, 8.0);

    // Calculate position in tile atlas (32x32 grid of tiles)
    let atlas_tile_x = f32(tile_index_in_atlas % 16);
    let atlas_tile_y = f32(tile_index_in_atlas / 16);

    // Calculate final UV coordinates in the atlas texture
    let atlas_uv = vec2<f32>(
        (atlas_tile_x + tile_pixel_uv.x) / 16,
        (atlas_tile_y + tile_pixel_uv.y) / 16
    );

    return textureSample(tileAtlas, atlasSampler, atlas_uv);
}



