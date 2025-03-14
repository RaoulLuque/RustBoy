// Struct to hold the tilemap. Ensures alignment that is multiple of 16 bytes.
struct TilemapUniform {
    tiles: array<vec4<u32>, 256>,
}

//// Struct to hold the possibly 10 objects/sprites in the current scanline
//// If there are less than 10 objects, the rest of the array is filled with 0s.
//struct ObjectsInScanline {
//    objects: array<vec4<u32>, 10>,
//}

// Tile atlas is a 2D texture containing all the tiles used in the tilemap.
// The tiles here can be considered the building blocks used by the tilemap.
// Each tile is 8x8 pixels, with a total of 16 tiles per row/column, so the atlas is 128 x 128 pixels in total.
// It is encoded in Rgba8UnormSrgb format.
@group(0) @binding(0) var background_tile_atlas: texture_2d<f32>;
// We use only the first entry to store the current rendering line
@group(0) @binding(1) var<uniform> current_line: vec4<u32>;
// Tilemap
// Tilemap is a 32x32 array of u32s, the same size as the grid of tiles that is loaded in the Rust Boy.
// Each u32 is a tile index, which is used to look up the tile in the tile atlas. The tilemap is in row major,
// so the first 32 u32s are the first row of tiles, the next 32 u32s are the second row of tiles, and so on.
@group(0) @binding(2) var<uniform> background_tilemap: TilemapUniform;
// The viewport position is the position of the top left pixel of the visible screen within the tilemap.
// That is it is a vector with values between 0 and 255, since the tilemap is 256x256 pixels.
// We use the first two entries of the vector to store the x and y coordinates of the viewport position.
@group(0) @binding(3) var<uniform> background_viewport_position: vec4<u32>;
// The framebuffer stores the current state of the frame and is transferred to the fragment shader to render
// the final image. It is a 2D texture with the same size as the screen (160 x 144)
@group(0) @binding(4) var framebuffer: texture_storage_2d<rgba8unorm, write>;

//// The sprite tile atlas is a 2D texture containing all the tiles used for the objects/sprites.
//@group(0) @binding(5) var object_tile_atlas: texture_2d<f32>;
//// The objects in the current scnaline are the objects that are visible in the current line of the screen.
//// The objects are stored in an array of 10 elements, each element is a vec4<u32>.
//// If there are less than 10 objects, the rest of the array is filled with 0s.
//@group(0) @binding(6) var objects_in_scanline: ObjectsInScanline;

@compute @workgroup_size(160, 1, 1)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {
    // The tilemap is a 32x32 grid of tiles, each tile is 8x8 pixels. That is 256x256 pixels. The following variable
    // represents the position of the top left pixel of the visible screen within the tilemap. That is it is a vector
    // with values between 0 and 255.
    let viewport_position_in_pixels = vec2<i32>(i32(background_viewport_position.x), i32(background_viewport_position.y));

    // Set the size of the tiles
    let tile_size = vec2<i32>(8, 8);

    // Retrieve the "position" of "the current pixel". That is, per workgroup, the y coordinate is fixed to the current
    // (rendering) line. The x coordinate on the other hand, is the local invocation id, which is an index iterating
    // between 0 and 159 Thus, each workgroup will render a line/row of 160 pixels.
    let x = local_id.x;
    let y = current_line.x;

    let color = compute_color_from_background(x, y, viewport_position_in_pixels, tile_size);

    textureStore(framebuffer, vec2<i32>(i32(x), i32(y)), color);
}

fn compute_color_from_background(x: u32, y: u32, viewport_position_in_pixels: vec2<i32>, tile_size: vec2<i32>) -> vec4<f32> {
    // This is the position of the current pixel in the screen in the tilemap taking into consideration the viewport
    // position.
    let pixel_coords = vec2<f32>(f32(x), f32(y)) + vec2<f32>(viewport_position_in_pixels);

    // Calculate the index (vector of x and y indeces) of the tile the pixel is in
    let tile_index_in_tilemap = (vec2<i32>(pixel_coords / vec2<f32>(tile_size))) % vec2<i32>(32, 32);
    // Calculate the flattened index
    let tilemap_flat_index = tile_index_in_tilemap.x + tile_index_in_tilemap.y * 32;
    let vec_index = tilemap_flat_index / 4;
    let comp_index = tilemap_flat_index % 4;

    // Retrieve the tile index in the tile atlas from the tilemap
    var tile_index_in_atlas: u32;
    switch (comp_index) {
        case 0: { tile_index_in_atlas = background_tilemap.tiles[vec_index].x; break; }
        case 1: { tile_index_in_atlas = background_tilemap.tiles[vec_index].y; break; }
        case 2: { tile_index_in_atlas = background_tilemap.tiles[vec_index].z; break; }
        default: { tile_index_in_atlas = background_tilemap.tiles[vec_index].w; break; }
    }

    // Calculate the coordinates of the pixel within the tile
    let pixel_index = vec2<i32>(pixel_coords) % tile_size;

    // Convert pixel position to normalized UV (0.0 - 1.0) within the current 8x8 tile
    let tile_pixel_uv = vec2<f32>(pixel_index) / vec2<f32>(8.0, 8.0);

    // Calculate position in tile atlas (16x16 grid of tiles)
    let atlas_tile_x = f32(tile_index_in_atlas % 16);
    let atlas_tile_y = f32(u32(tile_index_in_atlas / 16));

    // Calculate final UV coordinates in the atlas texture
    let atlas_uv = vec2<f32>(
        (atlas_tile_x + tile_pixel_uv.x) / 16,
        (atlas_tile_y + tile_pixel_uv.y) / 16
    );

    // Get the atlas texture dimensions
    let atlasSize = textureDimensions(background_tile_atlas);

    // Convert the normalized UV to integer texel coordinates
    let atlas_texel_coord = vec2<i32>(atlas_uv * vec2<f32>(atlasSize));

    // Load the color from the tile atlas at mip level 0
    let color = textureLoad(background_tile_atlas, atlas_texel_coord, 0);

    return color;
}