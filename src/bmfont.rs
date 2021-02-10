use std::collections::HashMap;
use std::io;

use byteorder::{LE, ReadBytesExt};

use crate::framework::context::Context;
use crate::framework::error::GameError::ResourceLoadError;
use crate::framework::error::GameResult;
use crate::str;

#[derive(Debug)]
pub struct BmChar {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub xoffset: i16,
    pub yoffset: i16,
    pub xadvance: i16,
    pub page: u8,
    pub chnl: u8,
}

#[derive(Debug)]
pub struct BMFont {
    pub pages: u16,
    pub font_size: i16,
    pub line_height: u16,
    pub base: u16,
    pub chars: HashMap<char, BmChar>,
}

const MAGIC: [u8; 4] = [b'B', b'M', b'F', 3];

impl BMFont {
    pub fn load_from<R: io::Read + io::Seek>(mut data: R) -> GameResult<Self> {
        let mut magic = [0u8; 4];
        let mut pages = 0u16;
        let mut chars = HashMap::with_capacity(128);
        let mut font_size = 0i16;
        let mut line_height = 0u16;
        let mut base = 0u16;

        data.read_exact(&mut magic)?;

        if magic != MAGIC {
            return Err(ResourceLoadError(str!( "Invalid magic")));
        }

        while let Ok(block_type) = data.read_u8() {
            let length = data.read_u32::<LE>()?;
            match block_type {
                1 => {
                    font_size = data.read_i16::<LE>()?;

                    data.seek(io::SeekFrom::Current(length as i64 - 2))?;
                }
                2 => {
                    line_height = data.read_u16::<LE>()?;
                    base = data.read_u16::<LE>()?;
                    data.seek(io::SeekFrom::Current(4))?;
                    pages = data.read_u16::<LE>()?;

                    data.seek(io::SeekFrom::Current(length as i64 - 10))?;
                }
                3 | 5 => {
                    data.seek(io::SeekFrom::Current(length as i64))?;
                }
                4 => {
                    let count = length / 20;
                    for _ in 0..count {
                        let id = data.read_u32::<LE>()?;
                        let x = data.read_u16::<LE>()?;
                        let y = data.read_u16::<LE>()?;
                        let width = data.read_u16::<LE>()?;
                        let height = data.read_u16::<LE>()?;
                        let xoffset = data.read_i16::<LE>()?;
                        let yoffset = data.read_i16::<LE>()?;
                        let xadvance = data.read_i16::<LE>()?;
                        let page = data.read_u8()?;
                        let chnl = data.read_u8()?;

                        if let Some(chr) = std::char::from_u32(id) {
                            chars.insert(chr, BmChar {
                                x,
                                y,
                                width,
                                height,
                                xoffset,
                                yoffset,
                                xadvance,
                                page,
                                chnl,
                            });
                        }
                    }
                }
                _ => { return Err(ResourceLoadError(str!( "Unknown block type."))); }
            }
        }

        Ok(Self {
            pages,
            font_size,
            line_height,
            base,
            chars,
        })
    }
}
