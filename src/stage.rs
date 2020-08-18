use std::io::{Cursor, Read};
use std::str::from_utf8;

use byteorder::{LE, ReadBytesExt};
use ggez::{Context, filesystem, GameResult};
use ggez::GameError::ResourceLoadError;
use log::info;
use strum::AsStaticRef;

use crate::map::Map;

#[derive(Debug, EnumIter, AsStaticStr, PartialEq, Eq, Hash, Copy, Clone)]
pub enum NpcType {
    #[strum(serialize = "0")]
    Zero,
    Almo1,
    Almo2,
    Ballos,
    Bllg,
    Cemet,
    Cent,
    Curly,
    Dark,
    Dr,
    Eggs1,
    Eggs2,
    Frog,
    Guest,
    Hell,
    Heri,
    IronH,
    Island,
    Kings,
    Maze,
    Miza,
    Moon,
    Omg,
    Plant,
    Press,
    Priest,
    Ravil,
    Red,
    Regu,
    Sand,
    Stream,
    Sym,
    Toro,
    TwinD,
    Weed,
    X,
}

impl NpcType {
    pub fn filename(&self) -> String {
        ["Npc", self.as_static()].join("")
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Tileset {
    name: String,
}

impl Clone for Tileset {
    fn clone(&self) -> Self {
        Tileset {
            name: self.name.clone(),
        }
    }
}

impl Tileset {
    pub fn new(name: &str) -> Tileset {
        Tileset {
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
        Background {
            name: self.name.clone(),
        }
    }
}

impl Background {
    pub fn new(name: &str) -> Background {
        Background {
            name: name.to_owned(),
        }
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
    pub fn new(id: u8) -> BackgroundType {
        match id {
            0 => { BackgroundType::Stationary }
            1 => { BackgroundType::MoveDistant }
            2 => { BackgroundType::MoveNear }
            3 => { BackgroundType::Water }
            4 => { BackgroundType::Black }
            5 => { BackgroundType::Autoscroll }
            6 => { BackgroundType::OutsideWind }
            7 => { BackgroundType::Outside }
            _ => { BackgroundType::Stationary }
        }
    }
}

#[derive(Debug)]
pub struct StageData {
    pub name: String,
    pub map: String,
    pub boss: String,
    pub boss_no: usize,
    pub tileset: Tileset,
    pub background: Background,
    pub background_type: BackgroundType,
    pub npc: NpcType,
}

impl Clone for StageData {
    fn clone(&self) -> Self {
        StageData {
            name: self.name.clone(),
            map: self.map.clone(),
            boss: self.boss.clone(),
            boss_no: self.boss_no,
            tileset: self.tileset.clone(),
            background: self.background.clone(),
            background_type: self.background_type,
            npc: self.npc,
        }
    }
}

impl StageData {
    pub fn load_stage_table(ctx: &mut Context, root: &str) -> GameResult<Vec<StageData>> {
        let stage_tbl_path = [root, "stage.tbl"].join("");
        let mrmap_bin_path = [root, "mrmap.bin"].join("");

        if filesystem::exists(ctx, &stage_tbl_path) {
            let mut stages = Vec::new();

            info!("Loading stage table from {}", &stage_tbl_path);

            let mut data = Vec::new();
            filesystem::open(ctx, stage_tbl_path)?.read_to_end(&mut data)?;

            let count = data.len() / 0xe5;
            let mut f = Cursor::new(data);
            for i in 0..count {}

            return Ok(stages);
        } else if filesystem::exists(ctx, &mrmap_bin_path) {
            let mut stages = Vec::new();

            info!("Loading stage table from {}", &mrmap_bin_path);

            let mut data = Vec::new();
            let mut fh = filesystem::open(ctx, &mrmap_bin_path)?;

            let count = fh.read_u32::<LE>()?;
            fh.read_to_end(&mut data)?;

            if data.len() < count as usize * 0x74 {
                return Err(ResourceLoadError("Specified stage table size is bigger than actual number of entries.".to_string()));
            }

            let mut f = Cursor::new(data);

            for _ in 0..count {
                let mut map_buf = Box::new(vec![0u8; 0x10]);
                let mut boss_buf = Box::new(vec![0u8; 0x10]);
                let mut name_buf = Box::new(vec![0u8; 0x22]);
                let mut ts_buf = vec![0u8; 0x10];
                let mut back_buf = vec![0u8; 0x10];
                let mut npc_buf = vec![0u8; 0x10];

                f.read_exact(&mut ts_buf)?;
                f.read_exact(&mut map_buf)?;
                let bg_type = f.read_u8()?;
                f.read_exact(&mut back_buf)?;
                f.read_exact(&mut npc_buf)?;
                f.read_exact(&mut boss_buf)?;
                let boss_no = f.read_u8()? as usize;
                f.read_exact(&mut name_buf)?;

                let tileset = from_utf8(&ts_buf)
                    .map_err(|_| ResourceLoadError("UTF-8 error in tileset field".to_string()))?
                    .trim_matches('\0').to_owned();
                let map = from_utf8(&map_buf)
                    .map_err(|_| ResourceLoadError("UTF-8 error in map field".to_string()))?
                    .trim_matches('\0').to_owned();
                let background = from_utf8(&back_buf)
                    .map_err(|_| ResourceLoadError("UTF-8 error in background field".to_string()))?
                    .trim_matches('\0').to_owned();
                let boss = from_utf8(&boss_buf)
                    .map_err(|_| ResourceLoadError("UTF-8 error in boss field".to_string()))?
                    .trim_matches('\0').to_owned();
                let name = from_utf8(&name_buf)
                    .map_err(|_| ResourceLoadError("UTF-8 error in name field".to_string()))?
                    .trim_matches('\0').to_owned();

                let stage = StageData {
                    name: name.clone(),
                    map: map.clone(),
                    boss: boss.clone(),
                    boss_no,
                    tileset: Tileset::new(&tileset),
                    background: Background::new(&background),
                    background_type: BackgroundType::new(bg_type),
                    npc: NpcType::Zero,
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
    pub fn load(ctx: &mut Context, data: &StageData) -> GameResult<Stage> {
        let map_file = filesystem::open(ctx, ["/Stage/", &data.map, ".pxm"].join(""))?;
        let attrib_file = filesystem::open(ctx, ["/Stage/", &data.tileset.name, ".pxa"].join(""))?;
        let map = Map::load_from(map_file, attrib_file)?;

        let stage = Stage {
            map,
            data: data.clone(),
        };

        Ok(stage)
    }
}
