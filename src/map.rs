use std::collections::HashMap;
use std::io;
use std::io::{BufRead, BufReader, Cursor, Read};
use std::sync::Arc;

use byteorder::{ReadBytesExt, LE};

use crate::common::{Color, Rect};
use crate::encoding::read_cur_shift_jis;
use crate::framework::context::Context;
use crate::framework::error::GameError::ResourceLoadError;
use crate::framework::error::{GameError, GameResult};
use crate::framework::filesystem;
use crate::shared_game_state::TileSize;
use crate::stage::{PxPackScroll, PxPackStageData, StageData};

static SUPPORTED_PXM_VERSIONS: [u8; 1] = [0x10];
static SUPPORTED_PXE_VERSIONS: [u8; 2] = [0, 0x10];

#[derive(Clone)]
pub struct Map {
    pub width: u16,
    pub height: u16,
    pub tiles: Vec<u8>,
    pub attrib: [u8; 0x100],
    pub tile_size: TileSize,
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
    pub fn load_pxm<R: io::Read>(mut map_data: R, mut attrib_data: R) -> GameResult<Map> {
        let mut magic = [0; 3];

        map_data.read_exact(&mut magic)?;

        if &magic != b"PXM" {
            return Err(ResourceLoadError("Invalid magic".to_owned()));
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

        Ok(Map { width, height, tiles, attrib, tile_size: TileSize::Tile16x16 })
    }

    pub fn load_pxpack<R: io::Read>(
        mut map_data: R,
        roots: &Vec<String>,
        data: &mut StageData,
        ctx: &mut Context,
    ) -> GameResult<Map> {
        let mut magic = [0u8; 16];

        map_data.read_exact(&mut magic)?;

        // based on https://github.com/tilderain/pxEdit/blob/kero/pxMap.py

        if &magic != b"PXPACK121127a**\0" {
            return Err(ResourceLoadError("Invalid magic".to_owned()));
        }

        fn read_string<R: io::Read>(map_data: &mut R) -> GameResult<String> {
            let mut bytes = map_data.read_u8()? as u32;
            let mut raw_chars = Vec::new();
            raw_chars.resize(bytes as usize, 0u8);
            map_data.read(&mut raw_chars)?;
            let mut raw_chars = Cursor::new(raw_chars);

            let mut chars = Vec::new();
            chars.reserve(bytes as usize);

            while bytes > 0 {
                let (consumed, chr) = read_cur_shift_jis(&mut raw_chars, bytes);
                chars.push(chr);
                bytes -= consumed;
            }

            Ok(chars.iter().collect())
        }

        fn skip_string<R: io::Read>(map_data: &mut R) -> GameResult {
            let bytes = map_data.read_u8()? as u32;
            for _ in 0..bytes {
                map_data.read_u8()?;
            }

            Ok(())
        }

        let map_name = read_string(&mut map_data)?;
        skip_string(&mut map_data)?; // left, right, up, down
        skip_string(&mut map_data)?;
        skip_string(&mut map_data)?;
        skip_string(&mut map_data)?;
        skip_string(&mut map_data)?; // spritesheet

        map_data.read_u16::<LE>()?;
        map_data.read_u16::<LE>()?;
        map_data.read_u8()?;

        let bg_color = Color::from_rgb(map_data.read_u8()?, map_data.read_u8()?, map_data.read_u8()?);

        let mut tileset_fg = read_string(&mut map_data)?;
        map_data.read_u8()?; // ignored
        let scroll_fg = PxPackScroll::from(map_data.read_u8()?);

        let mut tileset_mg = read_string(&mut map_data)?;
        map_data.read_u8()?; // ignored
        let scroll_mg = PxPackScroll::from(map_data.read_u8()?);

        let mut tileset_bg = read_string(&mut map_data)?;
        map_data.read_u8()?; // ignored
        let scroll_bg = PxPackScroll::from(map_data.read_u8()?);

        if tileset_fg.is_empty() {
            tileset_fg = data.tileset.filename()
        }
        if tileset_mg.is_empty() {
            tileset_mg = data.tileset.filename()
        }
        if tileset_bg.is_empty() {
            tileset_bg = data.tileset.filename()
        }

        let mut tiles = Vec::new();
        let mut attrib = [0u8; 0x100];

        let mut magic = [0u8; 8];
        map_data.read_exact(&mut magic)?;

        if &magic != b"pxMAP01\0" {
            return Err(ResourceLoadError("Invalid magic".to_owned()));
        }

        let width_fg = map_data.read_u16::<LE>()?;
        let height_fg = map_data.read_u16::<LE>()?;
        map_data.read_u8()?;

        log::info!("Foreground map size: {}x{}", width_fg, height_fg);

        let size_fg = width_fg as u32 * height_fg as u32;
        tiles.resize(size_fg as usize, 0u8);

        map_data.read_exact(&mut tiles[0..size_fg as usize])?;

        map_data.read_exact(&mut magic)?;

        if &magic != b"pxMAP01\0" {
            return Err(ResourceLoadError("Invalid magic".to_owned()));
        }

        let width_mg = map_data.read_u16::<LE>()?;
        let height_mg = map_data.read_u16::<LE>()?;

        log::info!("Middleground map size: {}x{}", width_mg, height_mg);

        let size_mg = width_mg as u32 * height_mg as u32;
        if size_mg != 0 {
            tiles.resize(size_fg as usize + size_mg as usize, 0u8);
            map_data.read_u8()?;
            map_data.read_exact(&mut tiles[size_fg as usize..(size_fg as usize + size_mg as usize)])?;
        }

        map_data.read_exact(&mut magic)?;

        if &magic != b"pxMAP01\0" {
            return Err(ResourceLoadError("Invalid magic".to_owned()));
        }

        let width_bg = map_data.read_u16::<LE>()?;
        let height_bg = map_data.read_u16::<LE>()?;

        log::info!("Background map size: {}x{}", width_bg, height_bg);

        let size_bg = width_bg as u32 * height_bg as u32;
        if size_bg != 0 {
            map_data.read_u8()?;
            tiles.resize(size_fg as usize + size_mg as usize + size_bg as usize, 0u8);
            map_data.read_exact(
                &mut tiles
                    [(size_fg as usize + size_mg as usize)..(size_fg as usize + size_mg as usize + size_bg as usize)],
            )?;
        }

        if let Ok(mut attrib_data) = filesystem::open_find(ctx, roots, ["/Stage/", &tileset_fg, ".pxa"].join("")) {
            if attrib_data.read_exact(&mut attrib).is_err() {
                log::warn!("Map attribute data is shorter than 256 bytes!");
            }
        } else if let Ok(mut attrib_data) =
            filesystem::open_find(ctx, roots, ["/Stage/", &tileset_fg, ".pxattr"].join(""))
        {
            attrib_data.read_exact(&mut magic)?;

            if &magic != b"pxMAP01\0" {
                return Err(ResourceLoadError("Invalid magic".to_owned()));
            }

            attrib_data.read_u16::<LE>()?;
            attrib_data.read_u16::<LE>()?;
            attrib_data.read_u8()?;

            if attrib_data.read_exact(&mut attrib).is_err() {
                log::warn!("Map attribute data is shorter than 256 bytes!");
            }

            for attr in &mut attrib {
                *attr = match *attr {
                    1 | 45 => 0x41,
                    2 | 66 => 0x44,
                    3 | 67 => 0x46,
                    4 | 68 => 0x43,
                    5 => 0x42,
                    7 => 0x4a,
                    8 => 0x50,
                    9 => 0x51,
                    10 => 0x52,
                    11 => 0x53,
                    12 => 0x54,
                    13 => 0x55,
                    14 => 0x56,
                    15 => 0x57,
                    40 => 0x5a,
                    41 => 0x5b,
                    42 => 0x5c,
                    43 => 0x5d,
                    64 => 0x60,
                    65 | 109 => 0x61,
                    69 => 0x62,
                    72 => 0x70,
                    73 => 0x71,
                    74 => 0x72,
                    75 => 0x73,
                    76 => 0x74,
                    77 => 0x75,
                    78 => 0x76,
                    79 => 0x77,
                    104 => 0x7a,
                    105 => 0x7b,
                    106 => 0x7c,
                    107 => 0x7d,
                    _ => 0,
                };
            }
        } else {
            log::warn!(
                "No tile attribute data found for foreground tileset {}, collision might be broken.",
                tileset_fg
            );
        }

        if !map_name.is_empty() {
            data.name = map_name;
        }

        data.background_color = bg_color;
        data.pxpack_data = Some(PxPackStageData {
            tileset_fg,
            tileset_mg,
            tileset_bg,
            scroll_fg,
            scroll_mg,
            scroll_bg,
            size_fg: (width_fg, height_fg),
            size_mg: (width_mg, height_mg),
            size_bg: (width_bg, height_bg),
            offset_mg: size_fg,
            offset_bg: size_fg + size_mg,
        });

        Ok(Map { width: width_fg, height: height_fg, tiles, attrib, tile_size: TileSize::Tile8x8 })
    }

    pub fn get_attribute(&self, x: usize, y: usize) -> u8 {
        if x >= self.width as usize || y >= self.height as usize {
            return 0;
        }

        self.attrib[*self.tiles.get(self.width as usize * y + x).unwrap_or(&0u8) as usize]
    }

    pub fn find_water_regions(&self, water_params: &WaterParams) -> Vec<(WaterRegionType, Rect<u16>, u8)> {
        let mut result = Vec::new();

        if self.height == 0 || self.width == 0 {
            return result;
        }

        let mut walked = vec![false; self.width as usize * self.height as usize];
        let mut rects = Vec::<(Rect<u16>, u8)>::new();

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

                let color_tile_idx = self.tiles[idx];
                let region_color = water_params.get_entry(color_tile_idx).color_middle;

                walked[idx] = true;
                let mut rect = Rect::new(x, y, x, y);
                let mut queue = vec![(0b1100, x, y)];

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

                        let idx = self.width as usize * ey as usize + ex as usize;
                        if walked[idx] {
                            return;
                        }

                        if water_params.get_entry(self.tiles[idx]).color_middle != region_color {
                            return;
                        }

                        let attr = self.get_attribute(ex as usize, ey as usize);
                        if WATER_TILES.contains(&attr) {
                            queue.push((flow_flags, ex as u16, ey as u16));
                        }
                    };

                    if flow_flags & 0b0001 != 0 {
                        check(0b1011, fx as i32 - 1, fy as i32);
                    }
                    if flow_flags & 0b0100 != 0 {
                        check(0b1110, fx as i32 + 1, fy as i32);
                    }
                    if flow_flags & 0b0010 != 0 {
                        check(0b0111, fx as i32, fy as i32 - 1);
                    }
                    if flow_flags & 0b1000 != 0 {
                        check(0b1101, fx as i32, fy as i32 + 1);
                    }
                }

                rects.push((rect, color_tile_idx));
            }
        }

        walked.fill(false);

        for (mut rect, color_idx) in rects {
            let line = rect.top;
            let line_up = rect.top - 1;
            let min_x = rect.left;
            let max_x = rect.right;

            rect.top += 1;
            result.push((WaterRegionType::WaterDepth, rect, color_idx));

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
                        color_idx,
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
                            color_idx,
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
            return Err(ResourceLoadError("Invalid magic".to_owned()));
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

impl Default for WaterParamEntry {
    fn default() -> Self {
        Self {
            color_top: Color::from_rgba(102, 153, 204, 150),
            color_middle: Color::from_rgba(102, 153, 204, 75),
            color_bottom: Color::from_rgba(102, 153, 204, 75),
        }
    }
}

pub struct WaterParams {
    entries: HashMap<u8, WaterParamEntry>,
}

impl WaterParams {
    pub fn new() -> WaterParams {
        WaterParams { entries: HashMap::new() }
    }

    pub fn load_from<R: io::Read>(&mut self, data: R) -> GameResult {
        fn next_u8<'a>(s: &mut impl Iterator<Item = &'a str>, error_msg: &str) -> GameResult<u8> {
            match s.next() {
                None => Err(GameError::ParseError("Out of range.".to_string())),
                Some(v) => v.trim().parse::<u8>().map_err(|_| GameError::ParseError(error_msg.to_string())),
            }
        }

        for line in BufReader::new(data).lines() {
            match line {
                Ok(line) => {
                    let mut splits = line.split(':');

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
                        if !cstr.starts_with('[') || !cstr.ends_with(']') {
                            return Err(GameError::ParseError("Invalid format of color value.".to_string()));
                        }

                        let mut csplits = cstr[1..cstr.len() - 1].split(',');

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
                        let e = self.entries.entry(i);
                        e.or_insert(entry);
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
