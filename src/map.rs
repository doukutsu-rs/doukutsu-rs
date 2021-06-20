use std::collections::HashMap;
use std::io;
use std::io::{BufRead, BufReader, Error};
use std::sync::Arc;

use byteorder::{ReadBytesExt, LE};

use crate::common::{Color, Rect};
use crate::framework::error::GameError::ResourceLoadError;
use crate::framework::error::{GameError, GameResult};
use crate::str;

static SUPPORTED_PXM_VERSIONS: [u8; 1] = [0x10];
static SUPPORTED_PXE_VERSIONS: [u8; 2] = [0, 0x10];

pub struct Map {
    pub width: u16,
    pub height: u16,
    pub tiles: Vec<u8>,
    pub attrib: [u8; 0x100],
}

static SOLID_TILES: [u8; 8] = [0x05, 0x41, 0x43, 0x46, 0x54, 0x55, 0x56, 0x57];
static WATER_TILES: [u8; 16] =
    [0x02, 0x60, 0x61, 0x62, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0xa0, 0xa1, 0xa2, 0xa3];

#[derive(Copy, Clone, PartialEq)]
pub enum WaterRegionType {
    WaterLine,
    WaterDepth,
}

impl Map {
    pub fn load_from<R: io::Read>(mut map_data: R, mut attrib_data: R) -> GameResult<Map> {
        let mut magic = [0; 3];

        map_data.read_exact(&mut magic)?;

        if &magic != b"PXM" {
            return Err(ResourceLoadError(str!("Invalid magic")));
        }

        let version = map_data.read_u8()?;

        if !SUPPORTED_PXM_VERSIONS.contains(&version) {
            return Err(ResourceLoadError(format!("Unsupported PXM version: {:#x}", version)));
        }

        let width = map_data.read_u16::<LE>()?;
        let height = map_data.read_u16::<LE>()?;
        let mut tiles = vec![0u8; (width * height) as usize];
        let mut attrib = [0u8; 0x100];

        log::info!("Map size: {}x{}", width, height);

        map_data.read_exact(&mut tiles)?;
        if attrib_data.read_exact(&mut attrib).is_err() {
            log::warn!("Map attribute data is shorter than 256 bytes!");
        }

        Ok(Map { width, height, tiles, attrib })
    }

    pub fn get_attribute(&self, x: usize, y: usize) -> u8 {
        if x >= self.width as usize || y >= self.height as usize {
            return 0;
        }

        self.attrib[*self.tiles.get(self.width as usize * y + x).unwrap_or_else(|| &0u8) as usize]
    }

    pub fn find_water_regions(&self) -> Vec<(WaterRegionType, Rect<u16>)> {
        let mut result = Vec::new();

        if self.height == 0 || self.width == 0 {
            return result;
        }

        let mut walked = vec![false; self.width as usize * self.height as usize];
        let mut rects = Vec::<Rect<u16>>::new();

        for x in 0..self.width {
            for y in 0..self.height {
                let idx = self.width as usize * y as usize + x as usize;
                if walked[idx] {
                    continue;
                }

                let attr = self.get_attribute(x as usize, y as usize);

                if !WATER_TILES.contains(&attr) {
                    continue;
                }

                walked[idx] = true;
                let mut rect = Rect::new(x, y, x, y);
                let mut queue = Vec::new();
                queue.push((0b1100, x, y));

                while let Some((flow_flags, fx, fy)) = queue.pop() {
                    let idx = self.width as usize * fy as usize + fx as usize;

                    walked[idx] = true;

                    if fx < rect.left {
                        rect.left = fx;

                        for y in rect.top..rect.bottom {
                            walked[self.width as usize * y as usize + rect.left as usize] = true;
                        }
                    }

                    if fx > rect.right {
                        rect.right = fx;

                        for y in rect.top..rect.bottom {
                            walked[self.width as usize * y as usize + rect.right as usize] = true;
                        }
                    }

                    if fy < rect.top {
                        rect.top = fy;

                        for x in rect.left..rect.right {
                            walked[self.width as usize * rect.top as usize + x as usize] = true;
                        }
                    }

                    if fy > rect.bottom {
                        rect.bottom = fy;

                        for x in rect.left..rect.right {
                            walked[self.width as usize * rect.bottom as usize + x as usize] = true;
                        }
                    }

                    let mut check = |flow_flags: u8, ex: i32, ey: i32| {
                        if ex < 0 || ex >= self.width as i32 || ey < 0 || ey >= self.height as i32 {
                            return;
                        }

                        if walked[self.width as usize * ey as usize + ex as usize] {
                            return;
                        }

                        let attr = self.get_attribute(ex as usize, ey as usize);
                        if WATER_TILES.contains(&attr) {
                            queue.push((flow_flags, ex as u16, ey as u16));
                        }
                    };

                    if flow_flags & 0b0001 != 0 { check(0b1011, fx as i32 - 1, fy as i32); }
                    if flow_flags & 0b0100 != 0 { check(0b1110, fx as i32 + 1, fy as i32); }
                    if flow_flags & 0b0010 != 0 { check(0b0111, fx as i32, fy as i32 - 1); }
                    if flow_flags & 0b1000 != 0 { check(0b1101, fx as i32, fy as i32 + 1); }
                }

                rects.push(rect);
            }
        }

        walked.fill(false);

        for mut rect in rects {
            let line = rect.top;
            let line_up = rect.top - 1;
            let min_x = rect.left;
            let max_x = rect.right;

            rect.top += 1;
            result.push((WaterRegionType::WaterDepth, rect));

            let mut x = min_x;
            let mut length = 0;
            let mut make_water_line = false;

            loop {
                let idx = self.width as usize * line as usize + x as usize;
                let attr = self.get_attribute(x as usize, line as usize);
                let attr_up = if rect.top > 0 { self.get_attribute(x as usize, line_up as usize) } else { 0x41 };

                if !SOLID_TILES.contains(&attr_up) && !WATER_TILES.contains(&attr_up) {
                    make_water_line = true;
                }

                if !walked[idx] && WATER_TILES.contains(&attr) {
                    length += 1;
                } else if length != 0 {
                    let bounds = Rect::new(x - length, line, x, line);
                    result.push((
                        if make_water_line { WaterRegionType::WaterLine } else { WaterRegionType::WaterDepth },
                        bounds,
                    ));
                    length = 0;
                } else {
                    length = 0;
                }

                walked[idx] = true;
                x += 1;

                if x >= max_x {
                    if length != 0 {
                        let bounds = Rect::new(x - length, line, x, line);
                        result.push((
                            if make_water_line { WaterRegionType::WaterLine } else { WaterRegionType::WaterDepth },
                            bounds,
                        ));
                    }

                    break;
                }
            }
        }

        result
    }
}

#[derive(Debug)]
pub struct NPCData {
    pub id: u16,
    pub x: i16,
    pub y: i16,
    pub flag_num: u16,
    pub event_num: u16,
    pub npc_type: u16,
    pub flags: u16,
    pub layer: u8,
}

impl NPCData {
    pub fn load_from<R: io::Read>(mut data: R) -> GameResult<Vec<NPCData>> {
        let mut magic = [0; 3];

        data.read_exact(&mut magic)?;

        if &magic != b"PXE" {
            return Err(ResourceLoadError(str!("Invalid magic")));
        }

        let version = data.read_u8()?;
        if !SUPPORTED_PXE_VERSIONS.contains(&version) {
            return Err(ResourceLoadError(format!("Unsupported PXE version: {:#x}", version)));
        }

        let count = data.read_u32::<LE>()? as usize;
        let mut npcs = Vec::with_capacity(count);

        for i in 0..count {
            let x = data.read_i16::<LE>()?;
            let y = data.read_i16::<LE>()?;
            let flag_num = data.read_u16::<LE>()?;
            let event_num = data.read_u16::<LE>()?;
            let npc_type = data.read_u16::<LE>()?;
            let flags = data.read_u16::<LE>()?;

            // booster's lab also specifies a layer field in version 0x10, prob for multi-layered maps
            let layer = if version == 0x10 { data.read_u8()? } else { 0 };

            npcs.push(NPCData { id: 170 + i as u16, x, y, flag_num, event_num, npc_type, flags, layer })
        }

        Ok(npcs)
    }
}

#[derive(Clone, Copy)]
pub struct WaterParamEntry {
    pub color_top: Color,
    pub color_middle: Color,
    pub color_bottom: Color,
}

pub struct WaterParams {
    entries: HashMap<u8, WaterParamEntry>,
}

impl WaterParams {
    pub fn new() -> WaterParams {
        WaterParams { entries: HashMap::new() }
    }

    pub fn load_from<R: io::Read>(&mut self, data: R) -> GameResult {
        fn next_u8(s: &mut std::str::Split<&str>, error_msg: &str) -> GameResult<u8> {
            match s.next() {
                None => Err(GameError::ParseError("Out of range.".to_string())),
                Some(v) => v.trim().parse::<u8>().map_err(|_| GameError::ParseError(error_msg.to_string())),
            }
        }

        for line in BufReader::new(data).lines() {
            match line {
                Ok(line) => {
                    let mut splits = line.split(":");

                    if splits.clone().count() != 5 {
                        return Err(GameError::ParseError("Invalid count of delimiters.".to_string()));
                    }

                    let tile_min = next_u8(&mut splits, "Invalid minimum tile value.")?;
                    let tile_max = next_u8(&mut splits, "Invalid maximum tile value.")?;

                    if tile_min > tile_max {
                        return Err(GameError::ParseError("tile_min > tile_max".to_string()));
                    }

                    let mut read_color = || -> GameResult<Color> {
                        let cstr = splits.next().unwrap().trim();
                        if !cstr.starts_with("[") || !cstr.ends_with("]") {
                            return Err(GameError::ParseError("Invalid format of color value.".to_string()));
                        }

                        let mut csplits = cstr[1..cstr.len() - 1].split(",");

                        if csplits.clone().count() != 4 {
                            return Err(GameError::ParseError("Invalid count of delimiters.".to_string()));
                        }

                        let r = next_u8(&mut csplits, "Invalid red value.")?;
                        let g = next_u8(&mut csplits, "Invalid green value.")?;
                        let b = next_u8(&mut csplits, "Invalid blue value.")?;
                        let a = next_u8(&mut csplits, "Invalid alpha value.")?;

                        Ok(Color::from_rgba(r, g, b, a))
                    };

                    let color_top = read_color()?;
                    let color_middle = read_color()?;
                    let color_bottom = read_color()?;

                    let entry = WaterParamEntry { color_top, color_middle, color_bottom };

                    for i in tile_min..=tile_max {
                        self.entries.insert(i, entry);
                    }
                }
                Err(e) => return Err(GameError::IOError(Arc::new(e))),
            }
        }

        Ok(())
    }

    #[inline]
    pub fn loaded(&self) -> bool {
        !self.entries.is_empty()
    }

    pub fn get_entry(&self, tile: u8) -> &WaterParamEntry {
        static DEFAULT_ENTRY: WaterParamEntry = WaterParamEntry {
            color_top: Color::new(1.0, 1.0, 1.0, 1.0),
            color_middle: Color::new(1.0, 1.0, 1.0, 1.0),
            color_bottom: Color::new(1.0, 1.0, 1.0, 1.0),
        };

        self.entries.get(&tile).unwrap_or(&DEFAULT_ENTRY)
    }
}
