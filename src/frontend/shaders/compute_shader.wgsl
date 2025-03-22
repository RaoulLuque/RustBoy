// Struct to hold the tilemap. Ensures alignment that is multiple of 16 bytes.
struct TilemapUniform {
    tiles: array<vec4<u32>, 256>,
}

// Struct to hold the possibly 10 objects/sprites in the current scanline
// If there are less than 10 objects, the rest of the array is filled with 0s.
struct ObjectsInScanline {
    objects: array<vec4<u32>, 10>,
}

// Tile atlas is a 2D texture containing all the tiles used in the tilemap.
// The tiles here can be considered the building blocks used by the tilemap.
// Each tile is 8x8 pixels, with a total of 16 tiles per row/column, so the atlas is 128 x 128 pixels in total.
// It is encoded in Rgba8UnormSrgb format.
@group(0) @binding(0) var background_tile_atlas: texture_2d<f32>;
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
// The framebuffer stores the current state of the frame and is transferred to the fragment shader to render
// the final image. It is a 2D texture with the same size as the screen (160 x 144)
@group(0) @binding(4) var framebuffer: texture_storage_2d<rgba8unorm, write>;

// The sprite tile atlas is a 2D texture containing all the tiles used for the objects/sprites.
@group(0) @binding(5) var object_tile_atlas: texture_2d<f32>;
// The objects in the current scnaline are the objects that are visible in the current line of the screen.
// The objects are stored in an array of 10 elements, each element is a vec4<u32>.
// If there are less than 10 objects, the rest of the array is filled with 0s.
@group(0) @binding(6) var<uniform> objects_in_scanline: ObjectsInScanline;

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
    let x: u32 = local_id.x;
    let y: u32 = current_line_and_obj_size.x;

    var pixel_in_object = false;
    var object = vec4<u32>(0, 0, 0, 0);
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
            // TODO: Handle, if the object actually covers the background (Byte 3 Priority)
            pixel_in_object = true;
            object = objects_in_scanline.objects[i];
        }
    }

    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    if (pixel_in_object) {
        color = compute_color_from_object(object, vec2<u32>(x, y));
        if (color.x == 1.0 && color.y == 1.0 && color.z == 1.0) {
            // If the color is white, it means that the pixel is transparent and we should use the background color(?)
            // TODO: Check if this is correct
            color = compute_color_from_background(x, y, viewport_position_in_pixels, tile_size);
        }
    } else {
        color = compute_color_from_background(x, y, viewport_position_in_pixels, tile_size);
    }

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

    return retrieve_color_from_atlas_texture(tile_index_in_atlas, tile_pixel_uv);
}

fn compute_color_from_object(object: vec4<u32>, pixel_coords: vec2<u32>) -> vec4<f32> {
    let object_size_flag = (current_line_and_obj_size.y & 0x1) != 0;

    // These are the x and y coordinates of the top left corner of the object
    let object_coordinates = vec2<u32>(object.y - 8, object.x - 16);

    // These are the x and y coordinate of the pixel within the object
    var within_object_pixel_coordinates: vec2<u32> = pixel_coords - object_coordinates;

    // Check for x or y flip
    if (object.z & 0x20) != 0 {
        // x flip
        within_object_pixel_coordinates.x = 7 - within_object_pixel_coordinates.x;
    }
    if (object.z & 0x40) != 0 {
        // y flip
        if object_size_flag {
            // Object_size_flag is set, therefore objects are 16 pixels high
            within_object_pixel_coordinates.y = 15 - within_object_pixel_coordinates.y;
        } else {
            // Object_size_flag is not set, therefore objects are 8 pixels high
            within_object_pixel_coordinates.y = 7 - within_object_pixel_coordinates.y;
        }
    }

    // Convert pixel position to normalized UV (0.0 - 1.0) within the current tile
    var tile_pixel_uv: vec2<f32>;
    if object_size_flag {
        // Object_size_flag is set, therefore objects are 16 pixels high and we have to normalize y coordinate by
        // dividing by 16
        tile_pixel_uv = vec2<f32>(within_object_pixel_coordinates) / vec2<f32>(8.0, 16.0);
    } else {
        // Object_size_flag is not set, therefore objects are 8 pixels high and we have to normalize y coordinate by
        // dividing by 8
        tile_pixel_uv = vec2<f32>(within_object_pixel_coordinates) / vec2<f32>(8.0, 8.0);
    }

    // The tile index is given as the third entry in the object vector
    let tile_index_in_atlas = object.z;

    return retrieve_color_from_atlas_texture(tile_index_in_atlas, tile_pixel_uv);
}

fn compute_color_from_object_size_8(object: vec4<u32>, pixel_coords: vec2<u32>) -> vec4<f32> {
    // These are the x and y coordinates of the top left corner of the object
    let object_coordinates = vec2<u32>(object.y - 8, object.x - 16);

    // These are the x and y coordinate of the pixel within the object
    var within_object_pixel_coordinates: vec2<u32> = pixel_coords - object_coordinates;

    // Check for x or y flip
    if (object.z & 0x20) != 0 {
        // x flip
        within_object_pixel_coordinates.x = 7 - within_object_pixel_coordinates.x;
    }
    if (object.z & 0x40) != 0 {
        // y flip
        within_object_pixel_coordinates.y = 7 - within_object_pixel_coordinates.y;
    }

    // Convert pixel position to normalized UV (0.0 - 1.0) within the current tile
    let tile_pixel_uv = vec2<f32>(within_object_pixel_coordinates) / vec2<f32>(8.0, 8.0);

    // The tile index is given as the third entry in the object vector
    let tile_index_in_atlas = object.z;

    return retrieve_color_from_atlas_texture(tile_index_in_atlas, tile_pixel_uv);
}

fn compute_color_from_object_size_16(object: vec4<u32>, pixel_coords: vec2<u32>) -> vec4<f32> {
    // These are the x and y coordinates of the top left corner of the object
    let object_x_coordinate = object.y - 8;
    let object_y_coordinate = object.x - 16;

    // TODO: Implement this function
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}

fn retrieve_color_from_atlas_texture(tile_index_in_atlas: u32, within_tile_pixel_uv_coords: vec2<f32>) -> vec4<f32> {
    // Calculate position in tile atlas (flattened 16x16 grid of tiles)
    let atlas_tile_x = f32(tile_index_in_atlas % 16);
    let atlas_tile_y = f32(u32(tile_index_in_atlas / 16));

    // Calculate final UV coordinates in the atlas texture
    let atlas_uv = vec2<f32>(
        (atlas_tile_x + within_tile_pixel_uv_coords.x) / 16,
        (atlas_tile_y + within_tile_pixel_uv_coords.y) / 16
    );

    // Get the atlas texture dimensions
    let atlasSize = textureDimensions(background_tile_atlas);

    // Convert the normalized UV to integer texel coordinates
    let atlas_texel_coord = vec2<i32>(atlas_uv * vec2<f32>(atlasSize));

    // Load the color from the tile atlas at mip level 0
    let color = textureLoad(background_tile_atlas, atlas_texel_coord, 0);

    return color;
}