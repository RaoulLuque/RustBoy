use super::GPU;
use crate::MEMORY_SIZE;
use crate::gpu::registers::LCDCRegister;
use crate::memory_bus::OAM_START;

/// Represents an object/sprite in the GPU's object attribute memory. These structs are used to
/// more accessibly represent the data in the OAM (Object Attribute Memory).
/// The 4 u8 (byte sized) fields represent the 4 bytes each OAM entry has. Their definitions are
/// as follows (see https://gbdev.io/pandocs/OAM.html):
/// - y_position: The y position of the object on the screen. Note that y = 0 means that the top
/// edge of the object is 16 pixels above the top of the screen.
/// - x_position: The x position of the object on the screen. Note that x = 0 means that the left
/// edge of the object is 8 pixels to the left of the left edge of the screen.
/// - tile_index: The index of the tile in the tile set that represents the object.
///
#[derive(Copy, Clone, Debug)]
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

impl GPU {
    /// TODO: Write docstring
    pub(crate) fn handle_oam_read(&self, address: u16) -> u8 {
        let index = (address - OAM_START) as usize;
        if index >= self.oam.len() * 4 {
            panic!("OAM read out of bounds");
        }
        let object_index = index / 4;
        let attribute_index = index % 4;
        match attribute_index {
            0 => self.oam[object_index].y_position,
            1 => self.oam[object_index].x_position,
            2 => self.oam[object_index].tile_index,
            3 => self.oam[object_index].attributes,
            _ => unreachable!(),
        }
    }

    /// TODO: Write docstring
    pub(crate) fn handle_oam_write(&mut self, address: u16, value: u8) {
        let index = (address - OAM_START) as usize;
        if index >= self.oam.len() * 4 {
            panic!("OAM write out of bounds");
        }
        let object_index = index / 4;
        let attribute_index = index % 4;
        match attribute_index {
            0 => self.oam[object_index].y_position = value,
            1 => self.oam[object_index].x_position = value,
            2 => self.oam[object_index].tile_index = value,
            3 => self.oam[object_index].attributes = value,
            _ => unreachable!(),
        }
    }

    /// TODO: Write docstring
    pub fn get_objects_for_current_scanline(
        &self,
        memory: &[u8; MEMORY_SIZE],
        scanline: u8,
    ) -> [[u32; 4]; 10] {
        let mut objects: [[u32; 4]; 10] = Default::default();
        let mut count = 0;
        let adjusted_scanline = scanline + 16; // Adjust for y_position = 0 being 16 pixels above the top of the screen

        for i in 0..self.oam.len() {
            let object = self.oam[i];
            // Set object height according to the flag in the LCD control register
            let object_height = if LCDCRegister::get_sprite_size_flag(memory) {
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
