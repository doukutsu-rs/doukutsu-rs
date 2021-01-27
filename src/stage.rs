use std::io::{Cursor, Read};
use std::str::from_utf8;

use byteorder::LE;
use byteorder::ReadBytesExt;
use log::info;

use crate::encoding::read_cur_shift_jis;
use crate::engine_constants::EngineConstants;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::map::{Map, NPCData};
use crate::text_script::TextScript;
use crate::framework::error::GameError::ResourceLoadError;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct NpcType {
    name: String,
}

impl Clone for NpcType {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
        }
    }
}

impl NpcType {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
    }

    pub fn filename(&self) -> String {
        ["Npc", &self.name].join("")
    }
}


#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Tileset {
    name: String,
}

impl Clone for Tileset {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
        }
    }
}

impl Tileset {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
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
        Self {
            name: self.name.clone(),
        }
    }
}

impl Background {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
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
    Stationary,
    MoveDistant,
    MoveNear,
    Water,
    Black,
    Autoscroll,
    OutsideWind,
    Outside,
}

impl BackgroundType {
    pub fn new(id: usize) -> Self {
        match id {
            0 => { Self::Stationary }
            1 => { Self::MoveDistant }
            2 => { Self::MoveNear }
            3 => { Self::Water }
            4 => { Self::Black }
            5 => { Self::Autoscroll }
            6 => { Self::OutsideWind }
            7 => { Self::Outside }
            _ => { Self::Black }
        }
    }
}

#[derive(Debug)]
pub struct StageData {
    pub name: String,
    pub map: String,
    pub boss_no: usize,
    pub tileset: Tileset,
    pub background: Background,
    pub background_type: BackgroundType,
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
            background: self.background.clone(),
            background_type: self.background_type,
            npc1: self.npc1.clone(),
            npc2: self.npc2.clone(),
        }
    }
}

const NXENGINE_BACKDROPS: [&str; 15] = [
    "bk0", "bkBlue", "bkGreen", "bkBlack", "bkGard", "bkMaze",
    "bkGray", "bkRed", "bkWater", "bkMoon", "bkFog", "bkFall",
    "bkLight", "bkSunset", "bkHellish"
];

const NXENGINE_TILESETS: [&str; 22] = [
    "0", "Pens", "Eggs", "EggX", "EggIn", "Store", "Weed",
    "Barr", "Maze", "Sand", "Mimi", "Cave", "River",
    "Gard", "Almond", "Oside", "Cent", "Jail", "White",
    "Fall", "Hell", "Labo"
];

const NXENGINE_NPCS: [&str; 34] = [
    "Guest", "0", "Eggs1", "Ravil", "Weed", "Maze",
    "Sand", "Omg", "Cemet", "Bllg", "Plant", "Frog",
    "Curly", "Stream", "IronH", "Toro", "X", "Dark",
    "Almo1", "Eggs2", "TwinD", "Moon", "Cent", "Heri",
    "Red", "Miza", "Dr", "Almo2", "Kings", "Hell",
    "Press", "Priest", "Ballos", "Island"
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
    // todo: refactor to make it less repetitive.
    pub fn load_stage_table(ctx: &mut Context, root: &str) -> GameResult<Vec<Self>> {
        let stage_tbl_path = [root, "stage.tbl"].join("");
        let stage_sect_path = [root, "stage.sect"].join("");
        let mrmap_bin_path = [root, "mrmap.bin"].join("");
        let stage_dat_path = [root, "stage.dat"].join("");

        if filesystem::exists(ctx, &stage_tbl_path) {
            // Cave Story+ stage table.
            let mut stages = Vec::new();

            info!("Loading Cave Story+/Booster's Lab style stage table from {}", &stage_tbl_path);

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
                let bg_type = f.read_u32::<LE>()? as usize;
                f.read_exact(&mut back_buf)?;
                f.read_exact(&mut npc1_buf)?;
                f.read_exact(&mut npc2_buf)?;
                let boss_no = f.read_u8()? as usize;
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
                    background: Background::new(&background),
                    background_type: BackgroundType::new(bg_type),
                    npc1: NpcType::new(&npc1),
                    npc2: NpcType::new(&npc2),
                };
                stages.push(stage);
            }

            return Ok(stages);
        } else if filesystem::exists(ctx, &stage_sect_path) {
            // Cave Story freeware executable dump.
            let mut stages = Vec::new();

            info!("Loading Cave Story freeware exe dump style stage table from {}", &stage_sect_path);

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
                let bg_type = f.read_u32::<LE>()? as usize;
                f.read_exact(&mut back_buf)?;
                f.read_exact(&mut npc1_buf)?;
                f.read_exact(&mut npc2_buf)?;
                let boss_no = f.read_u8()? as usize;
                f.read_exact(&mut name_buf)?;
                // alignment
                let _ = f.read_u8()?;
                let _ = f.read_u8()?;
                let _ = f.read_u8()?;

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
                    background: Background::new(&background),
                    background_type: BackgroundType::new(bg_type),
                    npc1: NpcType::new(&npc1),
                    npc2: NpcType::new(&npc2),
                };
                stages.push(stage);
            }

            return Ok(stages);
        } else if filesystem::exists(ctx, &mrmap_bin_path) {
            // CSE2E stage table
            let mut stages = Vec::new();

            info!("Loading CSE2E style stage table from {}", &mrmap_bin_path);

            let mut data = Vec::new();
            let mut fh = filesystem::open(ctx, &mrmap_bin_path)?;

            let count = fh.read_u32::<LE>()?;
            fh.read_to_end(&mut data)?;

            if data.len() < count as usize * 0x74 {
                return Err(ResourceLoadError("Specified stage table size is bigger than actual number of entries.".to_string()));
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
                let bg_type = f.read_u8()? as usize;
                f.read_exact(&mut back_buf)?;
                f.read_exact(&mut npc1_buf)?;
                f.read_exact(&mut npc2_buf)?;
                let boss_no = f.read_u8()? as usize;
                f.read_exact(&mut name_buf)?;


                let tileset = from_shift_jis(&ts_buf[0..zero_index(&ts_buf)]);
                let map = from_shift_jis(&map_buf[0..zero_index(&map_buf)]);
                let background = from_shift_jis(&back_buf[0..zero_index(&back_buf)]);
                let npc1 = from_shift_jis(&npc1_buf[0..zero_index(&npc1_buf)]);
                let npc2 = from_shift_jis(&npc2_buf[0..zero_index(&npc2_buf)]);
                let name = from_shift_jis(&name_buf[0..zero_index(&name_buf)]);

                println!("bg type: {}", bg_type);

                let stage = StageData {
                    name: name.clone(),
                    map: map.clone(),
                    boss_no,
                    tileset: Tileset::new(&tileset),
                    background: Background::new(&background),
                    background_type: BackgroundType::new(bg_type),
                    npc1: NpcType::new(&npc1),
                    npc2: NpcType::new(&npc2),
                };
                stages.push(stage);
            }

            return Ok(stages);
        } else if filesystem::exists(ctx, &stage_dat_path) {
            let mut stages = Vec::new();

            info!("Loading NXEngine style stage table from {}", &stage_dat_path);

            let mut data = Vec::new();
            let mut fh = filesystem::open(ctx, &stage_dat_path)?;

            let count = fh.read_u8()? as usize;
            fh.read_to_end(&mut data)?;

            if data.len() < count * 0x49 {
                return Err(ResourceLoadError("Specified stage table size is bigger than actual number of entries.".to_string()));
            }

            let mut f = Cursor::new(data);
            for _ in 0..count {
                let mut map_buf = vec![0u8; 0x20];
                let mut name_buf = vec![0u8; 0x23];

                f.read_exact(&mut map_buf)?;
                f.read_exact(&mut name_buf)?;

                let tileset_id = f.read_u8()? as usize;
                let bg_id = f.read_u8()? as usize;
                let bg_type = f.read_u8()? as usize;
                let boss_no = f.read_u8()? as usize;
                let npc1 = f.read_u8()? as usize;
                let npc2 = f.read_u8()? as usize;

                let map = from_utf8(&map_buf)
                    .map_err(|_| ResourceLoadError("UTF-8 error in map field".to_string()))?
                    .trim_matches('\0').to_owned();
                let name = from_utf8(&name_buf)
                    .map_err(|_| ResourceLoadError("UTF-8 error in name field".to_string()))?
                    .trim_matches('\0').to_owned();

                let stage = StageData {
                    name: name.clone(),
                    map: map.clone(),
                    boss_no,
                    tileset: Tileset::new(NXENGINE_TILESETS.get(tileset_id).unwrap_or(&"0")),
                    background: Background::new(NXENGINE_BACKDROPS.get(bg_id).unwrap_or(&"0")),
                    background_type: BackgroundType::new(bg_type),
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

pub struct Stage {
    pub map: Map,
    pub data: StageData,
}

impl Stage {
    pub fn load(root: &str, data: &StageData, ctx: &mut Context) -> GameResult<Self> {
        let map_file = filesystem::open(ctx, [root, "Stage/", &data.map, ".pxm"].join(""))?;
        let attrib_file = filesystem::open(ctx, [root, "Stage/", &data.tileset.name, ".pxa"].join(""))?;

        let map = Map::load_from(map_file, attrib_file)?;

        let stage = Self {
            map,
            data: data.clone(),
        };

        Ok(stage)
    }

    pub fn load_text_script(&self, root: &str, constants: &EngineConstants, ctx: &mut Context) -> GameResult<TextScript> {
        let tsc_file = filesystem::open(ctx, [root, "Stage/", &self.data.map, ".tsc"].join(""))?;
        let text_script = TextScript::load_from(tsc_file, constants)?;

        Ok(text_script)
    }

    pub fn load_npcs(&self, root: &str, ctx: &mut Context) -> GameResult<Vec<NPCData>> {
        let pxe_file = filesystem::open(ctx, [root, "Stage/", &self.data.map, ".pxe"].join(""))?;
        let npc_data = NPCData::load_from(pxe_file)?;

        Ok(npc_data)
    }

    pub fn tile_at(&self, x: usize, y: usize) -> u8 {
        if let Some(&tile) = self.map.tiles.get(y * self.map.width as usize + x) {
            tile
        } else {
            0
        }
    }

    /// Changes map tile. Returns true if smoke should be emitted
    pub fn change_tile(&mut self, x: usize, y: usize, tile_type: u8) -> bool {
        if let Some(ptr) = self.map.tiles.get_mut(y * self.map.width as usize + x) {
            if *ptr != tile_type {
                *ptr = tile_type;
                return true;
            }
        }

        false
    }
}
