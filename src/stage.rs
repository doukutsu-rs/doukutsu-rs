use std::io::{Cursor, Read};
use std::str::from_utf8;

use byteorder::ReadBytesExt;
use byteorder::LE;
use log::info;

use crate::common::Color;
use crate::encoding::read_cur_shift_jis;
use crate::engine_constants::EngineConstants;
use crate::framework::context::Context;
use crate::framework::error::GameError::ResourceLoadError;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::map::{Map, NPCData};
use crate::scripting::tsc::text_script::TextScript;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct NpcType {
    name: String,
}

impl Clone for NpcType {
    fn clone(&self) -> Self {
        Self { name: self.name.clone() }
    }
}

impl NpcType {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_owned() }
    }

    pub fn filename(&self) -> String {
        ["Npc", &self.name].join("")
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Tileset {
    pub(crate) name: String,
}

impl Clone for Tileset {
    fn clone(&self) -> Self {
        Self { name: self.name.clone() }
    }
}

impl Tileset {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_owned() }
    }

    pub fn filename(&self) -> String {
        ["Prt", &self.name].join("")
    }

    pub fn orig_width(&self) -> usize {
        // todo: move to json or something?

        if self.name == "Labo" {
            return 128;
        }
        256
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Background {
    name: String,
}

impl Clone for Background {
    fn clone(&self) -> Self {
        Self { name: self.name.clone() }
    }
}

impl Background {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_owned() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn filename(&self) -> String {
        self.name.to_owned()
    }
}

#[derive(Debug, EnumIter, PartialEq, Eq, Hash, Copy, Clone)]
pub enum BackgroundType {
    TiledStatic,
    TiledParallax,
    Tiled,
    /// Used in Core room, renders water in front of tilemap which also affects player's physics.
    Water,
    Black,
    /// Used in Ironhead fight. Affects physics of XP/heart/missile drops.
    Scrolling,
    /// Same as Outside, except it affects physics of XP/heart/missile drops.
    OutsideWind,
    Outside,
    /// Present in CS+KAGE, Seems to be a clone of Outside, isn't used anywhere and has unknown purpose
    OutsideUnknown,
    /// Used by CS+KAGE in waterway, it's just TiledParallax with bkCircle2 drawn behind water
    Waterway,
}

impl From<u8> for BackgroundType {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::TiledStatic,
            1 => Self::TiledParallax,
            2 => Self::Tiled,
            3 => Self::Water,
            4 => Self::Black,
            5 => Self::Scrolling,
            6 => Self::OutsideWind,
            7 => Self::Outside,
            8 => Self::OutsideUnknown,
            9 => Self::Waterway,
            _ => {
                // log::warn!("Unknown background type: {}", val);
                Self::Black
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PxPackScroll {
    Normal,
    ThreeQuarters,
    Half,
    Quarter,
    Eighth,
    Zero,
    HThreeQuarters,
    HHalf,
    HQuarter,
    V0Half,
}

impl From<u8> for PxPackScroll {
    fn from(val: u8) -> Self {
        match val {
            0 => PxPackScroll::Normal,
            1 => PxPackScroll::ThreeQuarters,
            2 => PxPackScroll::Half,
            3 => PxPackScroll::Quarter,
            4 => PxPackScroll::Eighth,
            5 => PxPackScroll::Zero,
            6 => PxPackScroll::HThreeQuarters,
            7 => PxPackScroll::HHalf,
            8 => PxPackScroll::HQuarter,
            9 => PxPackScroll::V0Half,
            _ => PxPackScroll::Normal,
        }
    }
}

impl PxPackScroll {
    pub fn transform_camera_pos(self, x: f32, y: f32) -> (f32, f32) {
        match self {
            PxPackScroll::Normal => (x, y),
            PxPackScroll::ThreeQuarters => (x * 0.75, y * 0.75),
            PxPackScroll::Half => (x * 0.5, y * 0.5),
            PxPackScroll::Quarter => (x * 0.25, y * 0.25),
            PxPackScroll::Eighth => (x * 0.125, y * 0.125),
            PxPackScroll::Zero => (0.0, 0.0),
            PxPackScroll::HThreeQuarters => (x * 0.75, y),
            PxPackScroll::HHalf => (x * 0.5, y),
            PxPackScroll::HQuarter => (x * 0.25, y),
            PxPackScroll::V0Half => (x, y), // ???
        }
    }
}

#[derive(Debug, Clone)]
pub struct PxPackStageData {
    pub tileset_fg: String,
    pub tileset_mg: String,
    pub tileset_bg: String,
    pub scroll_fg: PxPackScroll,
    pub scroll_mg: PxPackScroll,
    pub scroll_bg: PxPackScroll,
    pub size_fg: (u16, u16),
    pub size_mg: (u16, u16),
    pub size_bg: (u16, u16),
    pub offset_mg: u32,
    pub offset_bg: u32,
}

#[derive(Debug)]
pub struct StageData {
    pub name: String,
    pub map: String,
    pub boss_no: u8,
    pub tileset: Tileset,
    pub pxpack_data: Option<PxPackStageData>,
    pub background: Background,
    pub background_type: BackgroundType,
    pub background_color: Color,
    pub npc1: NpcType,
    pub npc2: NpcType,
}

impl Clone for StageData {
    fn clone(&self) -> Self {
        StageData {
            name: self.name.clone(),
            map: self.map.clone(),
            boss_no: self.boss_no,
            tileset: self.tileset.clone(),
            pxpack_data: self.pxpack_data.clone(),
            background: self.background.clone(),
            background_type: self.background_type,
            background_color: self.background_color,
            npc1: self.npc1.clone(),
            npc2: self.npc2.clone(),
        }
    }
}

const NXENGINE_BACKDROPS: [&str; 15] = [
    "bk0",
    "bkBlue",
    "bkGreen",
    "bkBlack",
    "bkGard",
    "bkMaze",
    "bkGray",
    "bkRed",
    "bkWater",
    "bkMoon",
    "bkFog",
    "bkFall",
    "bkLight",
    "bkSunset",
    "bkHellish",
];

const NXENGINE_TILESETS: [&str; 22] = [
    "0", "Pens", "Eggs", "EggX", "EggIn", "Store", "Weed", "Barr", "Maze", "Sand", "Mimi", "Cave", "River", "Gard",
    "Almond", "Oside", "Cent", "Jail", "White", "Fall", "Hell", "Labo",
];

const NXENGINE_NPCS: [&str; 34] = [
    "Guest", "0", "Eggs1", "Ravil", "Weed", "Maze", "Sand", "Omg", "Cemet", "Bllg", "Plant", "Frog", "Curly", "Stream",
    "IronH", "Toro", "X", "Dark", "Almo1", "Eggs2", "TwinD", "Moon", "Cent", "Heri", "Red", "Miza", "Dr", "Almo2",
    "Kings", "Hell", "Press", "Priest", "Ballos", "Island",
];

fn zero_index(s: &[u8]) -> usize {
    s.iter().position(|&c| c == b'\0').unwrap_or(s.len())
}

fn from_shift_jis(s: &[u8]) -> String {
    let mut cursor = Cursor::new(s);
    let mut chars = Vec::new();
    let mut bytes = s.len() as u32;

    while bytes > 0 {
        let (consumed, chr) = read_cur_shift_jis(&mut cursor, bytes);
        chars.push(chr);
        bytes -= consumed;
    }

    chars.iter().collect()
}

impl StageData {
    pub fn load_stage_table(ctx: &mut Context, root: &str) -> GameResult<Vec<Self>> {
        let stage_tbl_path = [root, "stage.tbl"].join("");
        let stage_sect_path = [root, "stage.sect"].join("");
        let mrmap_bin_path = [root, "mrmap.bin"].join("");
        let stage_dat_path = [root, "stage.dat"].join("");

        if filesystem::exists(ctx, &stage_tbl_path) {
            // Cave Story+ stage table.
            let mut stages = Vec::new();

            info!("Loading Cave Story+ stage table from {}", &stage_tbl_path);

            let mut data = Vec::new();
            filesystem::open(ctx, stage_tbl_path)?.read_to_end(&mut data)?;

            let count = data.len() / 0xe5;
            let mut f = Cursor::new(data);
            for _ in 0..count {
                let mut ts_buf = vec![0u8; 0x20];
                let mut map_buf = vec![0u8; 0x20];
                let mut back_buf = vec![0u8; 0x20];
                let mut npc1_buf = vec![0u8; 0x20];
                let mut npc2_buf = vec![0u8; 0x20];
                let mut name_jap_buf = vec![0u8; 0x20];
                let mut name_buf = vec![0u8; 0x20];

                f.read_exact(&mut ts_buf)?;
                f.read_exact(&mut map_buf)?;
                let bg_type = f.read_u32::<LE>()? as u8;
                f.read_exact(&mut back_buf)?;
                f.read_exact(&mut npc1_buf)?;
                f.read_exact(&mut npc2_buf)?;
                let boss_no = f.read_u8()?;
                f.read_exact(&mut name_jap_buf)?;
                f.read_exact(&mut name_buf)?;

                let tileset = from_shift_jis(&ts_buf[0..zero_index(&ts_buf)]);
                let map = from_shift_jis(&map_buf[0..zero_index(&map_buf)]);
                let background = from_shift_jis(&back_buf[0..zero_index(&back_buf)]);
                let npc1 = from_shift_jis(&npc1_buf[0..zero_index(&npc1_buf)]);
                let npc2 = from_shift_jis(&npc2_buf[0..zero_index(&npc2_buf)]);
                let name = from_shift_jis(&name_buf[0..zero_index(&name_buf)]);

                let stage = StageData {
                    name: name.clone(),
                    map: map.clone(),
                    boss_no,
                    tileset: Tileset::new(&tileset),
                    pxpack_data: None,
                    background: Background::new(&background),
                    background_type: BackgroundType::from(bg_type),
                    background_color: Color::from_rgb(0, 0, 32),
                    npc1: NpcType::new(&npc1),
                    npc2: NpcType::new(&npc2),
                };
                stages.push(stage);
            }

            return Ok(stages);
        } else if filesystem::exists(ctx, &stage_sect_path) {
            // Cave Story freeware executable dump.
            let mut stages = Vec::new();

            info!("Loading Cave Story freeware exe dump stage table from {}", &stage_sect_path);

            let mut data = Vec::new();
            filesystem::open(ctx, stage_sect_path)?.read_to_end(&mut data)?;

            let count = data.len() / 0xc8;
            let mut f = Cursor::new(data);
            for _ in 0..count {
                let mut ts_buf = vec![0u8; 0x20];
                let mut map_buf = vec![0u8; 0x20];
                let mut back_buf = vec![0u8; 0x20];
                let mut npc1_buf = vec![0u8; 0x20];
                let mut npc2_buf = vec![0u8; 0x20];
                let mut name_buf = vec![0u8; 0x20];

                f.read_exact(&mut ts_buf)?;
                f.read_exact(&mut map_buf)?;
                let bg_type = f.read_u32::<LE>()? as u8;
                f.read_exact(&mut back_buf)?;
                f.read_exact(&mut npc1_buf)?;
                f.read_exact(&mut npc2_buf)?;
                let boss_no = f.read_u8()?;
                f.read_exact(&mut name_buf)?;
                // alignment
                {
                    let mut lol = [0u8; 3];
                    let _ = f.read(&mut lol)?;
                }

                let tileset = from_shift_jis(&ts_buf[0..zero_index(&ts_buf)]);
                let map = from_shift_jis(&map_buf[0..zero_index(&map_buf)]);
                let background = from_shift_jis(&back_buf[0..zero_index(&back_buf)]);
                let npc1 = from_shift_jis(&npc1_buf[0..zero_index(&npc1_buf)]);
                let npc2 = from_shift_jis(&npc2_buf[0..zero_index(&npc2_buf)]);
                let name = from_shift_jis(&name_buf[0..zero_index(&name_buf)]);

                let stage = StageData {
                    name: name.clone(),
                    map: map.clone(),
                    boss_no,
                    tileset: Tileset::new(&tileset),
                    pxpack_data: None,
                    background: Background::new(&background),
                    background_type: BackgroundType::from(bg_type),
                    background_color: Color::from_rgb(0, 0, 32),
                    npc1: NpcType::new(&npc1),
                    npc2: NpcType::new(&npc2),
                };
                stages.push(stage);
            }

            return Ok(stages);
        } else if filesystem::exists(ctx, &mrmap_bin_path) {
            // Moustache Rider stage table
            let mut stages = Vec::new();

            info!("Loading Moustache Rider stage table from {}", &mrmap_bin_path);

            let mut data = Vec::new();
            let mut fh = filesystem::open(ctx, &mrmap_bin_path)?;

            let count = fh.read_u32::<LE>()?;
            fh.read_to_end(&mut data)?;

            if data.len() < count as usize * 0x74 {
                return Err(ResourceLoadError(
                    "Specified stage table size is bigger than actual number of entries.".to_string(),
                ));
            }

            let mut f = Cursor::new(data);
            for _ in 0..count {
                let mut ts_buf = vec![0u8; 0x10];
                let mut map_buf = vec![0u8; 0x10];
                let mut back_buf = vec![0u8; 0x10];
                let mut npc1_buf = vec![0u8; 0x10];
                let mut npc2_buf = vec![0u8; 0x10];
                let mut name_buf = vec![0u8; 0x22];

                f.read_exact(&mut ts_buf)?;
                f.read_exact(&mut map_buf)?;
                let bg_type = f.read_u8()?;
                f.read_exact(&mut back_buf)?;
                f.read_exact(&mut npc1_buf)?;
                f.read_exact(&mut npc2_buf)?;
                let boss_no = f.read_u8()?;
                f.read_exact(&mut name_buf)?;

                let tileset = from_shift_jis(&ts_buf[0..zero_index(&ts_buf)]);
                let map = from_shift_jis(&map_buf[0..zero_index(&map_buf)]);
                let background = from_shift_jis(&back_buf[0..zero_index(&back_buf)]);
                let npc1 = from_shift_jis(&npc1_buf[0..zero_index(&npc1_buf)]);
                let npc2 = from_shift_jis(&npc2_buf[0..zero_index(&npc2_buf)]);
                let name = from_shift_jis(&name_buf[0..zero_index(&name_buf)]);

                let stage = StageData {
                    name: name.clone(),
                    map: map.clone(),
                    boss_no,
                    tileset: Tileset::new(&tileset),
                    pxpack_data: None,
                    background: Background::new(&background),
                    background_type: BackgroundType::from(bg_type),
                    background_color: Color::from_rgb(0, 0, 32),
                    npc1: NpcType::new(&npc1),
                    npc2: NpcType::new(&npc2),
                };
                stages.push(stage);
            }

            return Ok(stages);
        } else if filesystem::exists(ctx, &stage_dat_path) {
            let mut stages = Vec::new();

            info!("Loading NXEngine stage table from {}", &stage_dat_path);

            let mut data = Vec::new();
            let mut fh = filesystem::open(ctx, &stage_dat_path)?;

            let count = fh.read_u8()? as usize;
            fh.read_to_end(&mut data)?;

            if data.len() < count * 0x49 {
                return Err(ResourceLoadError(
                    "Specified stage table size is bigger than actual number of entries.".to_string(),
                ));
            }

            let mut f = Cursor::new(data);
            for _ in 0..count {
                let mut map_buf = vec![0u8; 0x20];
                let mut name_buf = vec![0u8; 0x23];

                f.read_exact(&mut map_buf)?;
                f.read_exact(&mut name_buf)?;

                let tileset_id = f.read_u8()? as usize;
                let bg_id = f.read_u8()? as usize;
                let bg_type = f.read_u8()?;
                let boss_no = f.read_u8()?;
                let npc1 = f.read_u8()? as usize;
                let npc2 = f.read_u8()? as usize;

                let map = from_utf8(&map_buf)
                    .map_err(|_| ResourceLoadError("UTF-8 error in map field".to_string()))?
                    .trim_matches('\0')
                    .to_owned();
                let name = from_utf8(&name_buf)
                    .map_err(|_| ResourceLoadError("UTF-8 error in name field".to_string()))?
                    .trim_matches('\0')
                    .to_owned();

                let stage = StageData {
                    name: name.clone(),
                    map: map.clone(),
                    boss_no,
                    tileset: Tileset::new(NXENGINE_TILESETS.get(tileset_id).unwrap_or(&"0")),
                    pxpack_data: None,
                    background: Background::new(NXENGINE_BACKDROPS.get(bg_id).unwrap_or(&"0")),
                    background_type: BackgroundType::from(bg_type),
                    background_color: Color::from_rgb(0, 0, 32),
                    npc1: NpcType::new(NXENGINE_NPCS.get(npc1).unwrap_or(&"0")),
                    npc2: NpcType::new(NXENGINE_NPCS.get(npc2).unwrap_or(&"0")),
                };
                stages.push(stage);
            }

            return Ok(stages);
        }

        Err(ResourceLoadError("No stage table found.".to_string()))
    }
}

#[derive(Clone)]
pub struct Stage {
    pub map: Map,
    pub data: StageData,
}

impl Stage {
    pub fn load(root: &str, data: &StageData, ctx: &mut Context) -> GameResult<Self> {
        let mut data = data.clone();

        if let Ok(pxpack_file) = filesystem::open(ctx, [root, "Stage/", &data.map, ".pxpack"].join("")) {
            let map = Map::load_pxpack(pxpack_file, root, &mut data, ctx)?;
            let stage = Self { map, data };

            Ok(stage)
        } else {
            let map_file = filesystem::open(ctx, [root, "Stage/", &data.map, ".pxm"].join(""))?;
            let attrib_file = filesystem::open(ctx, [root, "Stage/", &data.tileset.name, ".pxa"].join(""))?;

            let map = Map::load_pxm(map_file, attrib_file)?;

            let stage = Self { map, data };

            Ok(stage)
        }
    }

    pub fn load_text_script(
        &self,
        root: &str,
        constants: &EngineConstants,
        ctx: &mut Context,
    ) -> GameResult<TextScript> {
        let tsc_file = filesystem::open(ctx, [root, "Stage/", &self.data.map, ".tsc"].join(""))?;
        let text_script = TextScript::load_from(tsc_file, constants)?;

        Ok(text_script)
    }

    pub fn load_npcs(&self, root: &str, ctx: &mut Context) -> GameResult<Vec<NPCData>> {
        let pxe_file = filesystem::open(ctx, [root, "Stage/", &self.data.map, ".pxe"].join(""))?;
        let npc_data = NPCData::load_from(pxe_file)?;

        Ok(npc_data)
    }

    /// Returns map tile from foreground layer.
    pub fn tile_at(&self, x: usize, y: usize) -> u8 {
        if let Some(&tile) = self.map.tiles.get(y.wrapping_mul(self.map.width as usize).wrapping_add(x)) {
            tile
        } else {
            0
        }
    }

    /// Changes map tile on foreground layer. Returns true if smoke should be emitted
    pub fn change_tile(&mut self, x: usize, y: usize, tile_type: u8) -> bool {
        if let Some(ptr) = self.map.tiles.get_mut(y.wrapping_mul(self.map.width as usize).wrapping_add(x)) {
            if *ptr != tile_type {
                *ptr = tile_type;
                return true;
            }
        }

        false
    }
}

pub struct StageTexturePaths {
    /// Path to the stage's background texture.
    pub background: String,

    /// Path to the stage's foreground tileset texture.
    pub tileset_fg: String,

    /// Path to the stage's middleground tileset texture.
    pub tileset_mg: String,

    /// Path to the stage's background tileset texture.
    pub tileset_bg: String,

    /// Path to the stage's NPC spritesheet 1.
    pub npc1: String,

    /// Path to the stage's NPC spritesheet 2.
    pub npc2: String,
}

impl StageTexturePaths {
    pub fn new() -> StageTexturePaths {
        StageTexturePaths {
            background: "bk0".to_string(),
            tileset_fg: "Stage/Prt0".to_owned(),
            tileset_mg: "Stage/Prt0".to_owned(),
            tileset_bg: "Stage/Prt0".to_owned(),
            npc1: "Npc/Npc0".to_owned(),
            npc2: "Npc/Npc0".to_owned(),
        }
    }

    pub fn update(&mut self, stage: &Stage) {
        self.background = stage.data.background.filename();
        let (tileset_fg, tileset_mg, tileset_bg) = if let Some(pxpack_data) = stage.data.pxpack_data.as_ref() {
            let t_fg = ["Stage/", &pxpack_data.tileset_fg].join("");
            let t_mg = ["Stage/", &pxpack_data.tileset_mg].join("");
            let t_bg = ["Stage/", &pxpack_data.tileset_bg].join("");

            (t_fg, t_mg, t_bg)
        } else {
            let tex_tileset_name = ["Stage/", &stage.data.tileset.filename()].join("");

            (tex_tileset_name.clone(), tex_tileset_name.clone(), tex_tileset_name)
        };
        self.tileset_fg = tileset_fg;
        self.tileset_mg = tileset_mg;
        self.tileset_bg = tileset_bg;

        self.npc1 = ["Npc/", &stage.data.npc1.filename()].join("");
        self.npc2 = ["Npc/", &stage.data.npc2.filename()].join("");
    }
}
