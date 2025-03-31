use crate::MemoryBus;
use crate::PPU;
use crate::memory_bus::{OAM_END, OAM_START};
use crate::ppu::registers::LCDCRegister;
use bytemuck::cast_ref;

/// Represents an object/sprite in the GPU's object attribute memory. These structs are used to
/// more accessibly represent the data in the OAM (Object Attribute Memory).
/// The 4 u8 (byte sized) fields represent the 4 bytes each OAM entry has. Their definitions are
/// as follows (see https://gbdev.io/pandocs/OAM.html):
/// - y_position: The y position of the object on the screen. Note that y = 0 means that the top
/// edge of the object is 16 pixels above the top of the screen.
/// - x_position: The x position of the object on the screen. Note that x = 0 means that the left
/// edge of the object is 8 pixels to the left of the left edge of the screen.
/// - tile_index: The index of the tile in the tile set that represents the object.
/// - attributes: The attributes of the object.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Object {
    pub y_position: u8,
    pub x_position: u8,
    pub tile_index: u8,
    pub attributes: u8,
}

impl Default for Object {
    /// Creates a new instance of object with all zero values.
    fn default() -> Self {
        Object {
            y_position: 0,
            x_position: 0,
            tile_index: 0,
            attributes: 0,
        }
    }
}

impl Object {
    pub fn to_bytes(&self) -> [u32; 4] {
        [
            self.y_position as u32,
            self.x_position as u32,
            self.tile_index as u32,
            self.attributes as u32,
        ]
    }
}

impl PPU {
    /// TODO: Write docstring
    pub fn get_objects_for_current_scanline(
        &self,
        memory_bus: &MemoryBus,
        scanline: u8,
    ) -> [[u32; 4]; 10] {
        let oam_as_objects: &[Object; 40] =
            cast_ref::<[u8; (OAM_END - OAM_START) as usize], [Object; 40]>(
                memory_bus.memory[OAM_START as usize..OAM_END as usize]
                    .as_ref()
                    .try_into()
                    .expect(
                        "Slice should be of correct length, work with me here compiler:\
                    40 objects * 4 bytes each = 160 bytes = 0xA0 bytes = 0xFEA0 bytes - 0xFE00 bytes",
                    ),
            );

        let mut objects: [[u32; 4]; 10] = Default::default();
        let mut count = 0;
        // Adjust for y_position = 0 being 16 pixels above the top of the screen
        let adjusted_scanline = scanline + 16;

        for i in 0..oam_as_objects.len() {
            let object = oam_as_objects[i];
            // Set object height according to the flag in the LCD control register
            let object_height = if LCDCRegister::get_sprite_size_flag(memory_bus) {
                16
            } else {
                8
            };
            // We have to adjust for y_position = 0 being 16 pixels above the top of the screen
            if object.y_position <= adjusted_scanline
                && object.y_position + object_height > adjusted_scanline
            {
                objects[count] = object.to_bytes();
                count += 1;
                if count == 10 {
                    break;
                }
            }
        }

        objects
    }
}

pub fn custom_ordering(a: u32, b: u32) -> std::cmp::Ordering {
    if a == b {
        std::cmp::Ordering::Equal
    } else if a == 0 {
        std::cmp::Ordering::Greater
    } else if b == 0 {
        std::cmp::Ordering::Less
    } else {
        a.cmp(&b)
    }
}
