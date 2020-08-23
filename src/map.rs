use std::io;
use std::io::{Error, ErrorKind};

use byteorder::{LE, ReadBytesExt};

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<u8>,
    pub attrib: [u8; 0x100],
}

impl Map {
    pub fn load_from<R: io::Read>(mut map_data: R, mut attrib_data: R) -> io::Result<Self> {
        let mut magic = [0; 3];

        map_data.read_exact(&mut magic)?;

        if &magic != b"PXM" {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid magic"));
        }

        map_data.read_i8()?; // unused

        let width = map_data.read_u16::<LE>()? as usize;
        let height = map_data.read_u16::<LE>()? as usize;
        let mut tiles = vec![0u8; width * height];
        let mut attrib = [0u8; 0x100];

        map_data.read_exact(&mut tiles)?;
        attrib_data.read_exact(&mut attrib)?;

        let map = Map {
            width,
            height,
            tiles,
            attrib,
        };

        Ok(map)
    }

    pub fn get_attribute(&self, x: usize, y: usize) -> u8 {
        self.attrib[*self.tiles.get(self.width * y + x).unwrap_or_else(|| &0u8) as usize]
    }
}
