use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{self, Cursor};
use std::io::{Read, Write};
use std::marker::Copy;
use std::path::PathBuf;
use std::str::{Chars, FromStr};

use byteorder::{BE, LE, ReadBytesExt, WriteBytesExt};
use num_traits::{clamp, FromPrimitive};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::bitfield;
use crate::common::{get_timestamp, Direction, FadeState, Version};
use crate::framework::context::Context;
use crate::framework::error::GameError::{self, ResourceLoadError};
use crate::framework::error::GameResult;
use crate::framework::filesystem::{self, user_read_dir};
use crate::game::player::{ControlMode, TargetPlayer};
use crate::game::profile::{ChallengeTime, GameProfile};
use crate::game::shared_game_state::{GameDifficulty, PlayerCount, SharedGameState, TimingMode};
use crate::game::weapon::{WeaponLevel, WeaponType};
use crate::game::inventory::{Inventory, Item};
use crate::mod_list::ModList;
use crate::mod_requirements::{self, ModRequirements};
use crate::scene::game_scene::GameScene;
use crate::util::rng::RNG;

pub const SWITCH_VER_1_2: Version = Version { major: 1, minor: 2, patch: 0 };
pub const SWITCH_VER_1_3: Version = Version { major: 1, minor: 3, patch: 0 };

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SaveSlot {
    MainGame(usize), // (save slot)
    CSPMod(u8, usize), // (mod save set, save_slot)
    Mod(String, usize), // (mod id, save slot)
}

impl FromStr for SaveSlot {
    type Err = GameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn extract_num(chars: &mut Chars, replace_emtpy: Option<usize>) -> Option<usize> {
            let mut buf = String::new();

            for char in chars {
                if char.is_digit(10) {
                    buf.push(char);
                } else if !buf.is_empty() {
                    break;
                }
            }

            if buf.is_empty() && replace_emtpy.is_some() {
                return replace_emtpy;
            }

            buf.parse::<usize>().ok()
        }

        if s.starts_with("Mod") {
            let mut chars = s.chars();

            let save_set = extract_num(&mut chars, None);
            let save_slot = extract_num(&mut chars, None);

            if let (Some(set), Some(slot)) = (save_set, save_slot) {
                return Ok(SaveSlot::CSPMod(set as u8, slot));
            }
        } else if s.starts_with("Profile") {
            let mut chars = s.chars();

            if let Some(slot) = extract_num(&mut chars, Some(1)) {
                return Ok(SaveSlot::MainGame(slot));
            }
        }

        Err(GameError::ParseError("Cannot parse save slot from the profile filename".to_owned()))
    }
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum SaveFormat {
    Freeware,
    Plus,
    Switch(Version),
    Generic,
}

impl SaveFormat {
    pub fn recognise(data: &[u8]) -> GameResult<Self> {
        let mut cur = Cursor::new(data);
        let magic = cur.read_u64::<BE>()?;

        fn recognise_switch_version(cur: &mut Cursor<&[u8]>, data: &[u8]) -> GameResult<Version> {
            // Set position to the start of the region with challenge times in v1.2.
            cur.set_position(0x20eb8);

            let shared_health = data[0x20eb4] == 1;
            let hell_time = cur.read_u32::<LE>()? == 0;
            let sanctuary_time = cur.read_u32::<LE>()? == 0;
            let boss_time = cur.read_u32::<LE>()? == 0;

            // v1.2 and v1.3 saves are the same size, and differ only in the shared health bar option and challenge times shift
            let version = if shared_health || hell_time {
                SWITCH_VER_1_2
            } else if sanctuary_time || boss_time {
                // If the time of the first challenge (assuming it's hell time from v1.2 save) is zero,
                // we check the next two challenges. They both have an RG requirement,
                // so if the assumed hell time is zero, then they should both be zero as well. 
                // If any of them is non-zero, then this is a v1.3 save,
                // and the `hell_time` variable actually contains a 4-byte offset,
                // while the `sanctuary_time` variable contains the real hell time.
                SWITCH_VER_1_3
            } else {
                // That's probably v1.2 save with a disabled shared health bar and unpassed hell.
                SWITCH_VER_1_2
            };

            Ok(version)
        }

        let original_format = match magic {
            // In CS+ a profile signature at the start of the save file is present only
            // if the first game slot profile exists. Otherwise it will be filled with zeros.
            0 =>
                match data.len() {
                    0x20020 => Some(Self::Plus),
                    0x20fb0 => Some(Self::Switch(recognise_switch_version(&mut cur, data)?)),
                    _ => None
                },
            SIG_Do041220 =>
                match data.len() {
                    0x604..=0x620 => Some(Self::Freeware),
                    0x20020 => Some(Self::Plus),
                    0x20fb0 => Some(Self::Switch(recognise_switch_version(&mut cur, data)?)),
                    _ => None
                },

            SIG_Do041115 => Some(Self::Freeware),
            _ => None
        };

        if let Some(format) = original_format {
            return Ok(format);
        }

        // Generic save is stored in JSON format, so it must start with '{' character
        if data[0] == '{' as u8 {
            cur.set_position(0);
            if serde_json::from_reader::<_, SaveContainer>(cur).is_ok() {
                return Ok(Self::Generic);
            }
        }

        Err(ResourceLoadError("Unsupported or invalid save file".to_owned()))
    }

    pub fn is_csp(&self) -> bool {
        match self {
            Self::Plus | Self::Switch(_) => true,
            _ => false
        }
    }

    pub fn is_switch(&self) -> bool {
        if let Self::Switch(_) = self {
            return true;
        }

        false
    }


    pub fn profile_size(&self) -> usize {
        match self {
            SaveFormat::Freeware | SaveFormat::Plus => 0x620,
            SaveFormat::Switch(_) => 0x680,
            SaveFormat::Generic => 0,
        }
    }

    // TODO: compatibility warnings
}


#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CSPModProfile {
    pub profiles: HashMap<usize, GameProfile>,
}

impl CSPModProfile {
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum PatchSlot {
    Profile(SaveSlot),
    BestTime(u8)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum SavePatch {
    Added,
    Modified,
    Deleted
}

#[derive(Clone, Debug)]
pub struct SaveParams {
    pub slots: Vec<SaveSlot>,
    pub settings: bool,
}

impl Default for SaveParams {
    fn default() -> Self {
        Self {
            // Import/export all slots by default
            slots: vec![],
            settings: true
        }
    }
}

// Generic container to store all possible info from original game saves
#[derive(Clone, Deserialize, Serialize)]
pub struct SaveContainer {
    pub version: usize,
    pub game_profiles: HashMap<usize, GameProfile>,
    pub csp_mods: HashMap<u8, CSPModProfile>, // save_set number -> saves & time

    // TODO: use this field instead of 290 records
    pub csp_best_times: HashMap<u8, ChallengeTime>, // mod_id -> time info

    #[serde(skip)]
    patchset: HashMap<PatchSlot, SavePatch>,
    // TODO: engine and mods specific fields
}

impl Default for SaveContainer {
    fn default() -> SaveContainer {
        SaveContainer {
            version: 1,

            game_profiles: HashMap::new(),
            csp_mods: HashMap::new(),
            csp_best_times: HashMap::new(),

            patchset: HashMap::new(),
        }
    }
}

impl SaveContainer {
    pub fn load(ctx: &mut Context, state: &mut SharedGameState) -> GameResult<SaveContainer> {
        log::debug!("DEBUG LOAD SAVE");

        let filename = Self::get_save_filename(SaveFormat::Generic, None);
        let container_opt = if let Ok(mut file) = filesystem::user_open(ctx, filename.clone()) {
            log::debug!("DEBUG LOAD SAVE - FILE EXISTS");
            // Using of buf significantly speed up the deserialization.
            let mut buf: Vec<u8> = Vec::new();
            file.read_to_end(&mut buf)?;

            // Try to deserialize the file and create a backup if it's invalid
            let save = Self::load_from_buf(buf.as_slice());
            if save.is_err() {
                let filename_bad = &(1..u64::MAX)
                    .find_map(|i| {
                        let fname_bad = format!("{}.bad.{}", filename.clone(), i);
                        if !filesystem::user_exists(ctx, fname_bad.clone()) {
                            return Some(fname_bad);
                        }

                        None
                    }).unwrap();

                log::error!("Save file is corrupted. Trying to move its contents to {filename_bad}");
                filesystem::user_create(ctx, filename_bad)
                    .and_then(|mut f| {
                        let _ = f.write_all(&buf)?;
                        Ok(())
                    })
                    .map_err(|e| GameError::ResourceLoadError(
                        format!("Failed to create a backup of the invalid save file: {:?}.", e)
                    ))?;

                log::info!("A backup of the corrupted save file has been created and written in {filename_bad}");
            }

            save.ok()
        } else { None };

        if let Some(mut container) = container_opt {
            container.load_times(ctx, state);
            return Ok(container);
        }
        
        log::info!("No save file is found, creating a new one.");

        let mut save = Self::default();
        save.load_profiles(ctx, state);
        save.load_times(ctx, state);

        let _ = save.write_save(ctx, state, SaveFormat::Generic, None, None, &SaveParams::default())?;

        Ok(save)
    }

    fn load_from_buf(buf: &[u8]) -> GameResult<SaveContainer> {
        serde_json::from_slice::<SaveContainer>(buf)
            .map_err(|err| GameError::ResourceLoadError(format!("Failed to deserialize a generic save. {}", err.to_string())))
            .map(|container| container.upgrade())
    }

    fn load_profiles(&mut self, ctx: &mut Context, state: &mut SharedGameState) {
        let filename = Self::get_save_filename(SaveFormat::Plus, None);
        let format = if let Ok(mut file) = filesystem::user_open(ctx, filename.clone()) {
            let mut buf: Vec<u8> = Vec::new();
            let _ = file.read_to_end(&mut buf);

            SaveFormat::recognise(&buf).unwrap_or(SaveFormat::Freeware)
        } else  {
            SaveFormat::Freeware
        };

        log::debug!("Auto-recognised format is: {:?}", format);
        if let Some(fs_container) = &state.fs_container {
            let path = fs_container.user_path.clone();
            if format != SaveFormat::Freeware {        
                let save_path = path.join(filename.clone().split_off(1));
                let _ = self.import(state, ctx, Some(format), SaveParams::default(), save_path);
            } else if let Ok(iter) = user_read_dir(ctx, "/") {
                let ext = OsString::from("dat");
                let save_path = path;

                for entry in iter {
                    log::debug!("Trying file: {:?} - {} - {} - {:?} - {}", entry.clone(), entry.starts_with("Profile"), entry.starts_with("Mod"), entry.extension(), entry.extension() == Some(&ext));
                    if (entry.starts_with("/Profile") || entry.starts_with("/Mod")) && entry.extension() == Some(&ext) {
                        if let Some(entry_fname) = entry.clone().file_name().and_then(|s| s.to_os_string().into_string().ok()) {
                            let save_path = save_path.join(entry_fname);
                            let _ = self.import(state, ctx, Some(format), SaveParams::default(), save_path);
                        }
                    }
                }
            }
        }
    }

    // TODO: add import interface for challenge times?
    fn load_times(&mut self, ctx: &Context, state: &SharedGameState) {
        let hell_time = ChallengeTime::load(ctx, state, Self::get_rec_filename(state, 0, true));
        if let Ok(time) = hell_time {
            self.set_best_time(0, time, true);
        }

        for mod_info in state.mod_list.mods.iter().filter(|m| m.get_csp_id().is_some()) {
            let filename = ["/".to_owned(), mod_info.get_rec_filename(".rec".to_owned())].join("");

            if let Ok(time) = ChallengeTime::load(ctx, state, filename) {
                let id = mod_info.get_csp_id().unwrap();
                self.set_best_time(id, time, true);
            }
        }
    }

    pub fn write_times(&self, ctx: &Context, state: &SharedGameState) -> GameResult {
        for (mod_id, time) in self.csp_best_times.iter() {
            let filename = Self::get_rec_filename(state, *mod_id, true);
            time.write(ctx, state, filename)?;
        }

        Ok(())
    }

    pub fn write_save(&self, ctx: &mut Context, state: &mut SharedGameState, format: SaveFormat, slot: Option<SaveSlot>, mut out_path: Option<PathBuf>, params: &SaveParams) -> GameResult {
        log::debug!("DEBUG WRITE SAVE");

        let save_path = Self::get_save_filename(format, slot.clone());

        match format {
            SaveFormat::Generic => {
                let mut file = filesystem::user_create(ctx, &save_path)?;

                // Using of buf significantly speed up the serializing.
                let buf = serde_json::to_vec(&self)?;
                file.write_all(&buf)?;
                file.flush()?;
            },
            SaveFormat::Freeware => {
                if let Some(path) = &mut out_path {
                    if path.is_dir() {
                        path.push("Profile.dat");
                    }
                }

                for (save_slot, profile) in &self.game_profiles {
                    if params.slots.is_empty() || params.slots.contains(&SaveSlot::MainGame(*save_slot)) {
                        let mut buf = Vec::new();
                        let mut cur = Cursor::new(&mut buf);
                        profile.write_save(&mut cur, &format)?;

                        let mut filename = Self::get_save_filename(format, Some(SaveSlot::MainGame(*save_slot)));
                        if let Some(path) = &mut out_path {
                            let os_filename = OsString::from_str(filename.split_off(1).as_str()).unwrap();
                            let _ = path.set_file_name(os_filename);

                            let mut file = File::create(path)?;
                            file.write_all(&buf)?;
                        } else {
                            let mut file = filesystem::user_create(ctx, filename)?;
                            file.write_all(&buf)?;
                        }
                    }
                }

                for (save_set, csp_mod) in &self.csp_mods {
                    for (slot, profile) in &csp_mod.profiles {
                        if params.slots.is_empty() || params.slots.contains(&SaveSlot::CSPMod(*save_set, *slot)) {
                            let mut buf = Vec::new();
                            let mut cur = Cursor::new(&mut buf);
                            profile.write_save(&mut cur, &format)?;

                            let mut filename = Self::get_save_filename(format, Some(SaveSlot::CSPMod(*save_set, *slot)));
                            if let Some(path) = &mut out_path {
                                let os_filename = OsString::from_str(filename.split_off(1).as_str()).unwrap();
                                let _ = path.set_file_name(os_filename);

                                let mut file = File::create(path)?;
                                file.write_all(&buf)?;
                            } else {
                                let mut file = filesystem::user_create(ctx, filename)?;
                                file.write_all(&buf)?;
                            }
                        }
                    }
                }

                
                for (mod_id, best_time) in &self.csp_best_times {
                    if let Some(path) = &mut out_path {
                        let filename = Self::get_rec_filename(state, *mod_id, false);

                        let os_filename = OsString::from_str(filename.as_str()).unwrap();
                        let _ = path.set_file_name(os_filename);

                        let file = File::create(path)?;
                        best_time.write_time(file, state, format)?;
                    }
                }
                
                if out_path.is_none() {
                    self.write_times(ctx, state)?;
                }
                
                // for (mod_id, best_time) in &self.csp_best_times {
                //     let filename = Self::get_rec_filename(state, *mod_id, false);
                //     if let Some(path) = &mut out_path {
                //         let os_filename = OsString::from_str(filename.as_str()).unwrap();
                //         let _ = path.set_file_name(os_filename);

                //         let file = File::create(path)?;
                //         best_time.write_time(file, state, format)?;
                //     } else {
                //         let file = filesystem::user_create(ctx, ["/", filename.as_str()].join(""))?;
                //         best_time.write_time(file, state, format)?;
                //     }
                // }

                for (patch_slot, patch_state) in self.patchset.iter() {
                    if *patch_state == SavePatch::Deleted {
                        let filename = match patch_slot {
                            PatchSlot::Profile(save_slot) => Self::get_save_filename(format, Some(save_slot.clone())),
                            PatchSlot::BestTime(mod_id) => Self::get_rec_filename(state, *mod_id, true)
                        };

                        // Since the file can miss, we aren't unwrap this call
                        let _ = filesystem::user_delete(ctx, filename);
                    }
                }
            },
            SaveFormat::Plus | SaveFormat::Switch(_) => {
                let mut active_slots = [0u8; 32];

                // Settings
                let bgm_volume = ((state.settings.bgm_volume * 10.0) as u32).min(10);
                let sfx_volume = ((state.settings.bgm_volume * 10.0) as u32).min(10);
                let seasonal_textures = state.settings.seasonal_textures as u8;
                let soundtrack: u8 = match state.settings.soundtrack.as_str() {
                    "organya" => 2,
                    "new" => 3,
                    "remastered" => 4,
                    "famitracks" => 5,
                    "ridiculon" => 6,
                    _ => 2 // Fallback to Organya
                };
                let graphics = state.settings.original_textures as u8;
                let language = (state.settings.locale == "jp") as u8;
                let beaten_hell = state.mod_requirements.beat_hell as u8;
                let jukebox = [0xff as u8; 6]; // TODO: implement storing ids of played songs for jukebox
                let mut eggfish_killed = [0u8; 3];

                let mut buf = Vec::new();
                let mut cur = Cursor::new(&mut buf);

                let default_profile = GameProfile::default();
                for save_slot in 1..=3 {
                    let profile = if params.slots.is_empty() || params.slots.contains(&SaveSlot::MainGame(save_slot)) {
                        log::debug!("Writing Game profile: {}", save_slot);
                        if let Some(game_profile) = self.game_profiles.get(&save_slot) {
                            active_slots[0] |= 1u8 << (save_slot - 1);
                            game_profile
                        } else {
                            &default_profile
                        }
                    } else {
                        &default_profile
                    };

                    profile.write_save(&mut cur, &format)?;

                    eggfish_killed[save_slot - 1] = profile.eggfish_killed as u8;
                }

                let default_csp_profile = CSPModProfile::default();
                for save_set in 1..=26 {
                    let csp_mod = self.csp_mods.get(&save_set).unwrap_or(&default_csp_profile);
                    for save_slot in 1..=3 {
                        let profile = if params.slots.is_empty() || params.slots.contains(&SaveSlot::CSPMod(save_set, save_slot)) {
                            log::debug!("Writing CSP profile: {} - {}", save_set, save_slot);
                            if let Some(game_profile) = csp_mod.profiles.get(&save_slot) {
                                active_slots[save_set as usize] |= 1u8 << (save_slot - 1);
                                game_profile
                            } else {
                                &default_profile
                            }
                        } else {
                            &default_profile
                        };

                        profile.write_save(&mut cur, &format)?;
                    }
                }

                cur.write(&active_slots)?;

                // Settings
                cur.write_u32::<LE>(bgm_volume)?;
                cur.write_u32::<LE>(sfx_volume)?;
                cur.write_u8(seasonal_textures)?;
                cur.write_u8(soundtrack)?;
                cur.write_u8(graphics)?;
                cur.write_u8(language)?;
                cur.write_u8(beaten_hell)?;

                if let SaveFormat::Switch(_) = format {
                    cur.write(&jukebox)?;
                    cur.write_u8(0)?; // Unlock notifications

                    cur.write_u8(0)?; // Shared health bar
                    cur.write_u24::<LE>(0)?; // Something
                } else {
                    let zeros = [0u8; 7];
                    cur.write(&zeros)?;
                }

                // Challenge best times
                let best_times = match format {
                    SaveFormat::Switch(SWITCH_VER_1_2) => 29,
                    SaveFormat::Switch(SWITCH_VER_1_3) => {
                        // In v1.3, challenge times are shifted by 4 bytes
                        cur.write_u32::<LE>(0)?;
                        28
                    },
                    _ => 26
                };

                let default_best_time = ChallengeTime::new();
                for mod_id in 0..=best_times {
                    let mod_time = self.csp_best_times.get(&mod_id).unwrap_or(&default_best_time).clone();

                    let csp_time = mod_time.with_timing(TimingMode::_60Hz);
                    csp_time.write_time(&mut cur, state, format)?;
                }

                if format.is_switch() {
                    let challenge_unlocks = ChallengeUnlocks::dump(&state.mod_requirements);
                    cur.write_u8(challenge_unlocks.0)?;

                    let something = [0u8; 0x77];
                    let something2 = [0u8; 6];

                    cur.write(&something)?;
                    cur.write_u16::<LE>(0)?; // P2 character unlocks
                    cur.write(&something2)?;
                } else {
                    cur.write(&eggfish_killed)?;

                    let something = [0u8; 0x3d];
                    cur.write(&something)?;

                    let something2 = [0u8; 0xf20];
                    cur.write(&something2)?;
                }

                if let Some(path) = out_path {
                    // TODO
                    File::create(path)?.write(&buf)?;
                } else {
                    filesystem::user_create(ctx, save_path)?.write(&buf)?;
                }
            }
        }

        Ok(())
    }

    pub fn save(&mut self, ctx: &mut Context, state: &mut SharedGameState, params: SaveParams) -> GameResult {
        self.write_save(ctx, state, SaveFormat::Generic, None, None, &params)?;
        self.write_times(ctx, state)?;
        self.write_save(ctx, state, state.settings.save_format, None, None, &params)?;
        self.patchset.clear();
        Ok(())
    }

    pub fn upgrade(self) -> Self {
        log::debug!("DEBUG UPGRADE SAVE");
        let initial_version = self.version;

        if self.version != initial_version {
            log::info!("Upgraded generic save from version {} to {}.", initial_version, self.version);
        }

        self
    }


    pub fn get_save_filename(format: SaveFormat, slot: Option<SaveSlot>) -> String {
        match format {
            SaveFormat::Generic => "/save.json".to_owned(),
            SaveFormat::Plus => "/Profile.dat".to_owned(),
            SaveFormat::Switch(_) => "/profile.dat".to_owned(),
            SaveFormat::Freeware => {
                match slot {
                    Some(SaveSlot::MainGame(save_slot)) => if save_slot == 1 {
                        "/Profile.dat".to_owned()
                    } else {
                        format!("/Profile{}.dat", save_slot)
                    },
                    Some(SaveSlot::CSPMod(save_set, save_slot)) => format!("/Mod{}_Profile{}.dat", save_set, save_slot),
                    Some(SaveSlot::Mod(_mod_id, _save_slot)) => unimplemented!(),
                    _ => "/Profile.dat".to_owned()
                }
            }
        }
    }

    pub fn get_rec_filename(state: &SharedGameState, mod_id: u8, is_absolute: bool) -> String {
        let prefix = if is_absolute { "/" } else { "" };

        let name = if mod_id == 0 {
            "290".to_owned()
        } else {
            let id = format!("cspmod_{mod_id:02}");
            state
                .mod_list
                .get_info_from_id(id.clone())
                .and_then(|mod_info| Some(mod_info.get_rec_filename("".to_string())))
                .unwrap_or(id)
        };

        [prefix.to_owned(), name, ".rec".to_string()].join("")
    }


    pub fn set_profile(&mut self, slot: SaveSlot, profile: GameProfile) {
        log::debug!("Debug profile set: {:?}; {}", slot, profile.timestamp);

        let prev_save: Option<GameProfile> = match slot {
            SaveSlot::MainGame(save_slot) => {
                self.game_profiles.insert(save_slot, profile)
            },
            SaveSlot::CSPMod(save_set, save_slot) => {
                self.csp_mods.entry(save_set)
                    .or_insert(CSPModProfile::default())
                    .profiles
                    .insert(save_slot, profile)
            },
            SaveSlot::Mod(_mod_id, _save_slot) => {
                // TODO
                unimplemented!();
            }
        };

        if prev_save.is_none() {
            self.patchset.insert(PatchSlot::Profile(slot), SavePatch::Added);
        } else {
            self.patchset.insert(PatchSlot::Profile(slot), SavePatch::Modified);
        }
    }

    pub fn get_profile(&self, slot: SaveSlot) -> Option<&GameProfile> {
        log::debug!("Debug profile get: {:?}", slot);
        match slot {
            SaveSlot::MainGame(save_slot) => self.game_profiles.get(&save_slot),
            SaveSlot::CSPMod(save_set, save_slot) => {
                if let Some(csp_mod) = self.csp_mods.get(&save_set) {
                    return csp_mod.profiles.get(&save_slot);
                }

                None
            },
            SaveSlot::Mod(_mod_id, _save_slot) => unimplemented!()
        }
    }

    pub fn delete_profile(&mut self, ctx: &Context, slot: SaveSlot) {
        log::debug!("Debug profile delete: {:?}", slot);
        match slot {
            SaveSlot::MainGame(save_slot) => {
                let _ = self.game_profiles.remove(&save_slot);
            },
            SaveSlot::CSPMod(save_set, save_slot) => {
                if let Some(csp_mod) = self.csp_mods.get_mut(&save_set) {
                    csp_mod.profiles.remove(&save_slot);

                    if csp_mod.is_empty() {
                        self.csp_mods.remove(&save_set);
                    }
                }
            },
            SaveSlot::Mod(_mod_id, _save_slot) => unimplemented!()
        }

        self.patchset.insert(PatchSlot::Profile(slot), SavePatch::Deleted);
    }


    pub fn set_best_time(&mut self, mod_id: u8, time: ChallengeTime, no_patch: bool) {
        let mut modified = false;
        self.csp_best_times
            .entry(mod_id)
            .and_modify(|t| {
                if *t > time {
                    *t = time;
                    modified = true;
                }
            })
            .or_insert(time);

        if !no_patch && modified {
            self.patchset.insert(PatchSlot::BestTime(mod_id), SavePatch::Modified);
        }
    }

    pub fn get_best_time(&self, mod_id: u8) -> Option<&ChallengeTime> {
        self.csp_best_times.get(&mod_id)
    }


    pub fn is_empty(&self) -> bool {
        if !self.game_profiles.is_empty() {
            return false;
        }

        for (_, csp_mod) in self.csp_mods.iter() {
            if !csp_mod.is_empty() {
                return false;
            }
        }

        true
    }


    fn merge(&mut self, b: &Self) {
        self.game_profiles.extend(b.game_profiles.iter());
        for (b_set, b_csp_mod) in &b.csp_mods {
            let _ = self.csp_mods.entry(*b_set)
                .and_modify(|csp_mod| csp_mod.profiles.extend(b_csp_mod.profiles.iter()))
                .or_insert(b_csp_mod.clone());
        }

        for (b_mod_id, b_best_time) in &b.csp_best_times {
            self.set_best_time(*b_mod_id, *b_best_time, true);
        }
    }

    pub fn import(
        &mut self,
        state: &mut SharedGameState,
        ctx: &mut Context,
        format: Option<SaveFormat>,
        params: SaveParams,
        save_path: PathBuf
    ) -> GameResult {
        let path = save_path.clone().into_boxed_path();

        // Working with data in memory is much faster than reading every field from drive
        let data = std::fs::read(path)?;
        let format = format.unwrap_or(SaveFormat::recognise(data.as_slice())?);

        log::trace!("Import format: {:?}.", format);
        log::trace!("Import params: {:?}.", params);
        log::trace!("Import path: {:?}.", save_path);

        match format {
            SaveFormat::Generic => {
                *self = Self::load_from_buf(data.as_slice())?;
            },
            SaveFormat::Freeware => {
                let filename = save_path.file_name().and_then(|s| s.to_os_string().into_string().ok()).unwrap();

                let mut cur = Cursor::new(data);
                let profile = GameProfile::load_from_save(&mut cur, format)?;

                let save_slot = params.slots.first().cloned().or(filename.parse::<SaveSlot>().ok());
                if let Some(slot) = save_slot {
                    self.set_profile(slot, profile);
                } else {
                    return Err(ResourceLoadError("Cannot parse save slot from the profile filename.".to_owned()));
                }
            },
            SaveFormat::Plus | SaveFormat::Switch(_) => {
                let mut container = Self::default();

                let mut cur = Cursor::new(data);
                let mut active_slots = [0u8; 32];

                for save_set in 0..=26 {
                    for save_slot in 1..=3 {
                        let slot = if save_set == 0 { SaveSlot::MainGame(save_slot) } else { SaveSlot::CSPMod(save_set, save_slot) };
                        let profile = GameProfile::load_from_save(&mut cur, format)?;
                        container.set_profile(slot, profile);
                    }
                }

                cur.read_exact(&mut active_slots)?;

                for save_set in 0..28 {
                    for save_slot in 1..=3 {
                        let is_inactive = active_slots[save_set] & (1u8 << (save_slot - 1)) != 0;
                        if is_inactive {
                            continue;
                        }

                        let slot = if save_set == 0 { SaveSlot::MainGame(save_slot) } else { SaveSlot::CSPMod(save_set as u8, save_slot) };
                        container.delete_profile(ctx, slot);
                    }
                }

                let bgm_volume = cur.read_u32::<LE>()?;
                let sfx_volume = cur.read_u32::<LE>()?;
                let seasonal_textures = cur.read_u8()? == 1;
                let soundtrack = cur.read_u8()?;
                let original_textures = cur.read_u8()? == 1;
                let _locale = cur.read_u8()?;

                if params.settings {
                    // TODO: change settings in the menu
                    state.settings.bgm_volume = bgm_volume as f32 / 10.0;
                    state.settings.sfx_volume = sfx_volume as f32 / 10.0;
                    state.settings.seasonal_textures = seasonal_textures;
                    state.settings.soundtrack = match soundtrack {
                        2 => "organya",
                        3 => "new",
                        4 => "remastered",
                        5 => "famitracks",
                        6 => "ridiculon",
                        _ => state.settings.soundtrack.as_str()
                    }.to_owned();

                    if original_textures != state.settings.original_textures {
                        state.settings.original_textures = original_textures;
                        state.reload_resources(ctx)?;
                    }

                    // TODO: should we import locale?

                    state.settings.save(ctx)?;
                }

                let beat_hell = cur.read_u8()? == 1;
                state.mod_requirements.beat_hell = state.mod_requirements.beat_hell || beat_hell;

                if format.is_switch() {
                    // TODO: jukebox
                    let mut jukebox = [0u8; 6];
                    cur.read_exact(&mut jukebox)?;

                    // TODO
                    let _ = cur.read_u8()?; // Unlock notifications
                    let _ = cur.read_u8()?; // Shared healthbar

                    // unused
                    let _ = cur.read_u24::<LE>()?;
                } else {
                    let mut unused = [0u8; 7];
                    cur.read_exact(&mut unused)?;
                }

                let best_times = match format {
                    SaveFormat::Switch(SWITCH_VER_1_2) => 29,
                    SaveFormat::Switch(SWITCH_VER_1_3) => {
                        // In v1.3, challenge times are shifted by 4 bytes
                        let _ = cur.read_u32::<LE>()?;
                        28
                    },
                    _ => 26
                };

                for mod_id in 0..=best_times {
                    let mut best_time = ChallengeTime::new();
                    best_time.load_time(&mut cur, format)?;

                    if best_time.ticks != 0 {
                        //container.csp_best_times.insert(mod_id, best_time);
                        container.set_best_time(mod_id, best_time, false);
                    }
                }

                if format == SaveFormat::Plus {
                    for save_slot in 1..=3 as usize {
                        let eggfish_killed = cur.read_u8()?;
                        if let Some(profile) = self.game_profiles.get_mut(&save_slot) {
                            profile.eggfish_killed = eggfish_killed != 0;
                        }
                    }

                    let mut something = [0u8; 61];
                    cur.read_exact(&mut something)?;

                    let mut unused = [0u8; 32];
                    cur.read_exact(&mut unused)?;
                } else {
                    let challenge_unlocks = ChallengeUnlocks(cur.read_u8()?);
                    challenge_unlocks.load(ctx, &mut state.mod_requirements)?;

                    let mut something: [u8; 119] = [0u8; 119];
                    cur.read_exact(&mut something)?;

                    // TODO
                    let _ = cur.read_u16::<LE>()?; // P2 Skins

                    let mut something2 = [0u8; 6];
                    cur.read_exact(&mut something2)?;
                }

                self.merge(&container);
            }
        }

        Ok(())
    }

    pub fn export(
        &mut self,
        state: &mut SharedGameState,
        ctx: &mut Context,
        format: SaveFormat,
        params: SaveParams,
        out_path: PathBuf
    ) -> GameResult {
        self.write_save(ctx, state, format, None, Some(out_path.clone()), &params)?;

        log::trace!("Export format: {:?}.", format.clone());
        log::trace!("Export params: {:?}.", params);
        log::trace!("Export path: {:?}.", out_path);
        Ok(())
    }
}

bitfield! {
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ChallengeUnlocks(u8);
    impl Debug;

    pub main_game, set_main_game: 0; // 0x1
    pub sanctuary_time, set_sanctuary_time: 1; // 0x2
    pub boss_attack, set_boss_attack: 2; // 0x4
    pub curly_story, set_curly_story: 3; // 0x8
    pub wind_fortress, set_wind_fortress: 4; // 0x10
    pub nemesis, set_nemesis: 5; // 0x20
    pub machine_gun, set_machine_gun: 6; // 0x40
    pub sand_pit, set_sand_pit: 7; // 0x80
}

impl ChallengeUnlocks {
    pub fn new() -> Self {
        Self(1)
    }

    pub fn dump(mod_requirements: &ModRequirements) -> Self {
        let mut challenge_unlocks = Self::new();

        // Mods with RG requirement
        if mod_requirements.beat_hell {
            challenge_unlocks.set_sanctuary_time(true);
            challenge_unlocks.set_boss_attack(true);
            challenge_unlocks.set_wind_fortress(true);
        }

        // Mods with RA requirement

        // Mods with RA12 requirement.
        if mod_requirements.has_weapon(12) {
            challenge_unlocks.set_nemesis(true);
        }

        // Mods with RA3 requirement.
        if mod_requirements.has_weapon(3) {
            challenge_unlocks.set_sand_pit(true);
        }

        // Mods with RI requirement

        // Mods with RI35 requirement.
        if mod_requirements.has_item(35) {
           challenge_unlocks.set_curly_story(true);
           challenge_unlocks.set_machine_gun(true);
        }

        challenge_unlocks
    }

    pub fn load(&self, ctx: &Context, mod_requirements: &mut ModRequirements) -> GameResult {
        if self.sanctuary_time() || self.boss_attack() || self.wind_fortress() {
            mod_requirements.beat_hell = true;
        }

        if self.nemesis() {
            mod_requirements.append_weapon(ctx, 12)?;
        }

        if self.sand_pit() {
            mod_requirements.append_weapon(ctx, 3)?;
        }

        if self.curly_story() || self.machine_gun() {
            mod_requirements.append_item(ctx, 35)?;
        }

        Ok(())
    }
}