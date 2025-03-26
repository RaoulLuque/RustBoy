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

// Struct to hold tile data in a packed format. This just converts the Game Boys encoding of an array of u8s into an
// array of u32s with the same total size.
// Each tile consists of 8 x 8 pixels. Each pixel is represented by 2 bits, therefore each tile
// consists of 8 x 8 * 2 = 128 bits = 16 bytes = 4 u32s. Furthermore, there are 16 x 16 = 256
// tiles in the tilemap. Therefore, the size of the tiles array is 256 * 4 * 4 = 4096 bytes.
struct TileDataPacked {
    tiles: array<vec4<u32>, 256>,
}

// Struct to hold the tilemap. Ensures alignment that is multiple of 16 bytes.
struct TilemapUniform {
    indices: array<vec4<u32>, 256>,
}

// Struct to hold the possibly 10 objects/sprites in the current scanline
// If there are less than 10 objects, the rest of the array is filled with 0s.
struct ObjectsInScanline {
    objects: array<vec4<u32>, 10>,
}

const color_zero: vec4<f32> = vec4<f32>(0.836, 0.956, 0.726, 1.0);
const color_one: vec4<f32> = vec4<f32>(0.270, 0.527, 0.170, 1.0);
const color_two: vec4<f32> = vec4<f32>(0.0, 0.118, 0.0, 1.0);
const color_three: vec4<f32> = vec4<f32>(0.040, 0.118, 0.060, 1.0);

// Tile atlas is a 2D texture containing all the tiles used in the tilemap.
// The tiles here can be considered the building blocks used by the tilemap.
// Each tile is 8x8 pixels, with a total of 16 tiles per row/column, so the atlas is 128 x 128 pixels in total.
// It is encoded in Rgba8UnormSrgb format.
@group(0) @binding(0) var<uniform> bg_and_window_tile_data: TileDataPacked;
// We use only the first entry to store the current rendering line, the second entry is used to pass the object size
// flag (FF40 bit 2)
@group(0) @binding(1) var<uniform> current_line_and_obj_size: vec4<u32>;
// Tilemap
// Tilemap is a 32x32 array of u32s, the same size as the grid of tiles that is loaded in the Rust Boy.
// Each u32 is a tile index, which is used to look up the tile in the tile atlas. The tilemap is in row major,
// so the first 32 u32s are the first row of tiles, the next 32 u32s are the second row of tiles, and so on.
@group(0) @binding(2) var<uniform> background_tilemap: TilemapUniform;
// The viewport position is the position of the top left pixel of the visible screen within the tilemap.
// That is it is a vector with values between 0 and 255, since the tilemap is 256x256 pixels.
// We use the first two entries of the vector to store the x and y coordinates of the viewport position.
@group(0) @binding(3) var<uniform> background_viewport_position: vec4<u32>;
// The lcd monochrome palettes are just the registers FF47, FF48, FF49 as specified in the Pandocs
// (https://gbdev.io/pandocs/Palettes.html). The first entry in the vec is the background palette (FF47), the second
// entry is the object palette 0 (FF48) and the third entry is the object palette 1 (FF49). The fourth entry is empty.
@group(0) @binding(4) var<uniform> palettes: vec4<u32>;

// The sprite tile atlas is a 2D texture containing all the tiles used for the objects/sprites.
@group(0) @binding(5) var<uniform> object_tile_data: TileDataPacked;
// The objects in the current scnaline are the objects that are visible in the current line of the screen.
// The objects are stored in an array of 10 elements, each element is a vec4<u32>.
// If there are less than 10 objects, the rest of the array is filled with 0s.
@group(0) @binding(6) var<uniform> objects_in_scanline: ObjectsInScanline;



@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // The tilemap is a 32x32 grid of tiles, each tile is 8x8 pixels. That is 256x256 pixels. The following variable
    // represents the position of the top left pixel of the visible screen within the tilemap. That is it is a vector
    // with values between 0 and 255.
    let viewport_position_in_pixels = vec2<i32>(i32(background_viewport_position.x), i32(background_viewport_position.y));

    // Set the size of the tiles
    let tile_size = vec2<i32>(8, 8);

    // Retrieve the "position" of "the current pixel". That is, per workgroup, the y coordinate is fixed to the current
    // (rendering) line. The x coordinate on the other hand, is the local invocation id, which is an index iterating
    // between 0 and 159 Thus, each workgroup will render a line/row of 160 pixels.
    let x: u32 = u32(in.clip_position.x);
    let y: u32 = current_line_and_obj_size.x;

    var pixel_in_object = false;
    var object = vec4<u32>(0, 0, 0, 0);
    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    // We have to adjust for x_position = 0 being 8 pixels to the left of the left border of the screen
    let adjusted_x = x + 8;

    // Check if the current pixel is in an object in the objects_in_scanline
    for (var i = 0; i < 10; i = i + 1) {
        if (objects_in_scanline.objects[i].x == 0) {
            // objects_in_scanline.objects[i].x is the y coordinate of the object and if it is 0, it means that there are
            // no more objects in the current scanline. Because, no object with a y coordinate of 0 would be added to the
            // objects_in_scanline.
            break;
        }
        if (objects_in_scanline.objects[i].y <= adjusted_x && objects_in_scanline.objects[i].y + 8 > adjusted_x) {
            // objects_in_scanline.objects[i].y is the x coordinate of the object. With this, we check if the current pixel
            // lies within the object. For the y coordinate this is already guaranteed by objects_in_scanline
            object = objects_in_scanline.objects[i];
            color = compute_color_from_object(object, vec2<u32>(x, y));
            if (color.x == color_zero.x && color.y == color_zero.y && color.z == color_zero.z) {
                // If the color is transparent we can search the rest of the objects if they cover this pixel
                continue;
            } else {
                // If the color is not transparent, we have found the object that covers the pixel
                pixel_in_object = true;
                break;
            }
        }
    }

    if (!pixel_in_object) || ((object.w & 0x80u) != 0u) {
        color = compute_color_from_background(x, y, viewport_position_in_pixels, tile_size);
    }

    return color;
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
        case 0: { tile_index_in_atlas = background_tilemap.indices[vec_index].x; break; }
        case 1: { tile_index_in_atlas = background_tilemap.indices[vec_index].y; break; }
        case 2: { tile_index_in_atlas = background_tilemap.indices[vec_index].z; break; }
        default: { tile_index_in_atlas = background_tilemap.indices[vec_index].w; break; }
    }

    // Calculate the coordinates of the pixel within the tile
    let pixel_index = vec2<i32>(pixel_coords) % tile_size;

    return retrieve_color_from_tile_data_buffers(tile_index_in_atlas, vec2<u32>(pixel_index), 0);
}

fn compute_color_from_object(object: vec4<u32>, pixel_coords: vec2<u32>) -> vec4<f32> {
    let object_size_flag = (current_line_and_obj_size.y & 0x1) != 0;

    // These are the x and y coordinates of the top left corner of the object
    let object_coordinates = vec2<u32>(object.y - 8, object.x - 16);

    // These are the x and y coordinate of the pixel within the object
    var within_object_pixel_coordinates: vec2<u32> = pixel_coords - object_coordinates;

    // Check for x or y flip
    if (object.w & 0x20) != 0 {
        // x flip
        within_object_pixel_coordinates.x = 7 - within_object_pixel_coordinates.x;
    }
    if (object.w & 0x40) != 0 {
        // y flip
        if object_size_flag {
            // Object_size_flag is set, therefore objects are 16 pixels high
            within_object_pixel_coordinates.y = 15 - within_object_pixel_coordinates.y;
        } else {
            // Object_size_flag is not set, therefore objects are 8 pixels high
            within_object_pixel_coordinates.y = 7 - within_object_pixel_coordinates.y;
        }
    }

    // The tile index is given as the third entry in the object vector
    var tile_index_in_atlas = object.z;

    // If the object is 16 pixels high, we need to adjust the tile_index_in_atlas and the pixel coordinates
    // such that we are compatible with retrieve_color_from_tile_data_buffers
    if object_size_flag {
        if within_object_pixel_coordinates.y > 7 {
            // The pixel lies within the bottom part of the object, therefore we need to adjust the tile index
            // and the pixel coordinates
            tile_index_in_atlas = tile_index_in_atlas + 1;
            within_object_pixel_coordinates.y = within_object_pixel_coordinates.y - 8;
        }
    }

    return retrieve_color_from_tile_data_buffers(tile_index_in_atlas, within_object_pixel_coordinates, 1);
}

/// Given the tile_index_in_buffer, the pixel coordinates within the tile, computes the color a pixel should have.
/// To distinguish between the background and window tile data buffer and the object tile data buffer, the tile_data_flag
/// can be set to = 0 for background and window tile data buffer and = 1 (and else) for object tile data buffer.
fn retrieve_color_from_tile_data_buffers(tile_index_in_buffer: u32, within_tile_pixel_coords: vec2<u32>, tile_data_flag: u32) -> vec4<f32> {
    // Get the correct tile based on whether we are using the background or object tile data
    var tile_containing_pixel: vec4<u32>;
    if tile_data_flag == 0 {
        tile_containing_pixel = bg_and_window_tile_data.tiles[tile_index_in_buffer];
    } else {
        tile_containing_pixel = object_tile_data.tiles[tile_index_in_buffer];
    }

    // Find the encoded color value of the pixel in the tile. This is quite obscure due to the Game Boys' tile data
    // encoding scheme, see: https://gbdev.io/pandocs/Tile_Data.html#data-format
    var bytes_containing_color_code: u32;
    let in_tile_index = within_tile_pixel_coords.y / 2;
    switch (in_tile_index) {
            case 0u: { bytes_containing_color_code = tile_containing_pixel.x; break; }
            case 1u: { bytes_containing_color_code = tile_containing_pixel.y; break; }
            case 2u: { bytes_containing_color_code = tile_containing_pixel.z; break; }
            default: { bytes_containing_color_code = tile_containing_pixel.w; break; }
    }
    let mask_lower_bit: u32 = 1u << (15u - within_tile_pixel_coords.x + ((within_tile_pixel_coords.y % 2) * 16u) - 8u);
    let mask_upper_bit: u32 = 1u << (15u - within_tile_pixel_coords.x + ((within_tile_pixel_coords.y % 2) * 16u));
    let color_code = u32((bytes_containing_color_code & mask_lower_bit) != 0) | (u32((bytes_containing_color_code & mask_upper_bit) != 0) << 1u);
    let color = convert_color_code_to_rgba8_color(color_code);

    return color;
}

fn convert_color_code_to_rgba8_color(color_code: u32) -> vec4<f32> {
    // The color code is a 2-bit value, where each bit represents a color
    // 0 = white, 1 = light green, 2 = dark green, 3 = very dark green/black
    switch (color_code) {
        case 0u: { return color_zero; }
        case 1u: { return color_one; }
        case 2u: { return color_two; }
        default: { return color_three; }
    }
}