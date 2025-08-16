use std::collections::{HashMap, hash_map::Entry};
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Cursor};
use std::io::{Read, Write};
use std::marker::Copy;
use std::path::{Path, PathBuf};
use std::str::{Chars, FromStr};

use byteorder::{BE, LE, ReadBytesExt, WriteBytesExt};
use num_traits::{clamp, FromPrimitive};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::common::{Direction, FadeState, get_timestamp};
use crate::components::nikumaru::NikumaruCounter;
use crate::framework::context::Context;
use crate::framework::error::GameError::{self, ResourceLoadError};
use crate::framework::error::GameResult;
use crate::framework::filesystem::{user_create, user_delete, user_exists, user_open};
use crate::game::player::{ControlMode, TargetPlayer};
use crate::game::shared_game_state::{GameDifficulty, PlayerCount, SharedGameState};
use crate::game::weapon::{WeaponLevel, WeaponType};
use crate::game::inventory::Inventory;
use crate::scene::game_scene::GameScene;

const SIG_Do041115: u64 = 0x446f303431313135;
const SIG_Do041220: u64 = 0x446f303431323230;
const SIG_FLAG: u32 = 0x464c4147;


#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct WeaponData {
    pub weapon_id: u32,
    pub level: u32,
    pub exp: u32,
    pub max_ammo: u32,
    pub ammo: u32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct TeleporterSlotData {
    pub index: u32,
    pub event_num: u32,
}


trait SaveProfile {
    fn apply(&self, state: &mut SharedGameState, game_scene: &mut GameScene, ctx: &mut Context);
    fn dump(state: &mut SharedGameState, game_scene: &mut GameScene, target_player: Option<TargetPlayer>) -> Self;
    fn write_save<W: io::Write>(&self, data: W, format: SaveFormat) -> GameResult;
    fn load_from_save<R: io::Read>(data: R, format: SaveFormat) -> GameResult<GameProfile>;
    fn is_empty(&self) -> bool;
}

#[serde_as]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct GameProfile {
    pub current_map: u32,
    pub current_song: u32,
    pub pos_x: i32,
    pub pos_y: i32,
    pub direction: Direction,
    pub max_life: u16,
    pub max_life_p2: u16,
    pub stars: u16,
    pub life: u16,
    pub life_p2: u16,
    pub current_weapon: u32,
    pub current_item: u32,
    pub equipment: u32,
    pub control_mode: u32,
    pub counter: u32,
    pub weapon_data: [WeaponData; 8],
    pub weapon_data_p2: Option<[WeaponData; 8]>,
    pub items: [u32; 32],
    pub teleporter_slots: [TeleporterSlotData; 8],
    #[serde_as(as = "[_; 128]")]
    pub map_flags: [u8; 128],
    #[serde_as(as = "[_; 1000]")]
    pub flags: [u8; 1000],
    pub timestamp: u64,
    pub difficulty: u8,

    // CS+ fields
    pub eggfish_killed: bool,
}

impl Default for GameProfile {
    fn default() -> GameProfile {
        GameProfile {
            current_map: 0,
            current_song: 0,
            pos_x: 0,
            pos_y: 0,
            direction: Direction::Left,
            max_life: 0,
            max_life_p2: 0,
            stars: 0,
            life: 0,
            life_p2: 0,
            current_weapon: 0,
            current_item: 0,
            equipment: 0,
            control_mode: 0,
            counter: 0,
            weapon_data: [WeaponData::default(); 8],
            weapon_data_p2: None,
            items: [0u32; 32],
            teleporter_slots: [TeleporterSlotData::default(); 8],
            map_flags: [0u8; 128],
            flags: [0u8; 1000],
            timestamp: 0,
            difficulty: 0,

            eggfish_killed: false,
        }
    }
}

impl GameProfile {
    pub fn apply(&self, state: &mut SharedGameState, game_scene: &mut GameScene, ctx: &mut Context) {
        state.fade_state = FadeState::Visible;
        state.control_flags.set_tick_world(true);
        state.control_flags.set_control_enabled(true);

        let _ = state.sound_manager.play_song(self.current_song as usize, &state.constants, &state.settings, ctx, false);

        game_scene.inventory_player1.current_weapon = self.current_weapon as u16;
        game_scene.inventory_player1.current_item = self.current_item as u16;

        fn apply_weapons(ctx: &mut Context, state: &mut SharedGameState, weapons: &[WeaponData], inventory: &mut Inventory) {
            for weapon in weapons {
                if weapon.weapon_id == 0 {
                    continue;
                }

                let _ = state.mod_requirements.append_weapon(ctx, weapon.weapon_id as u16);
                let weapon_type: Option<WeaponType> = FromPrimitive::from_u8(weapon.weapon_id as u8);

                if let Some(wtype) = weapon_type {
                    inventory.add_weapon_data(
                        wtype,
                        weapon.ammo as u16,
                        weapon.max_ammo as u16,
                        weapon.exp as u16,
                        match weapon.level {
                            2 => WeaponLevel::Level2,
                            3 => WeaponLevel::Level3,
                            _ => WeaponLevel::Level1,
                        },
                    );
                }
            }
        }

        apply_weapons(ctx, state, &self.weapon_data, &mut game_scene.inventory_player1);
        if let Some(weapons) = self.weapon_data_p2.as_ref() {
            apply_weapons(ctx, state, weapons, &mut game_scene.inventory_player2);
        }

        for item in self.items.iter().copied() {
            let item_id = item as u16;
            let _ = state.mod_requirements.append_item(ctx, item_id);

            let amount = (item >> 16) as u16;
            if item_id == 0 {
                break;
            }

            // TODO: original save formats can't store inventory for player 2, but we can, so should we save it,
            // even if it's equal to the inventroy of player 1?
            game_scene.inventory_player1.add_item_amount(item_id, amount + 1);
            game_scene.inventory_player2.add_item_amount(item_id, amount + 1);
        }

        for slot in &self.teleporter_slots {
            if slot.event_num == 0 {
                break;
            }

            state.teleporter_slots.push((slot.index as u16, slot.event_num as u16));
        }

        for (idx, &flag) in self.map_flags.iter().enumerate() {
            state.set_map_flag(idx, flag != 0);
        }

        for (idx, &flags) in self.flags.iter().enumerate() {
            if flags & 0b00000001 != 0 {
                state.set_flag(idx * 8, true);
            }
            if flags & 0b00000010 != 0 {
                state.set_flag(idx * 8 + 1, true);
            }
            if flags & 0b00000100 != 0 {
                state.set_flag(idx * 8 + 2, true);
            }
            if flags & 0b00001000 != 0 {
                state.set_flag(idx * 8 + 3, true);
            }
            if flags & 0b00010000 != 0 {
                state.set_flag(idx * 8 + 4, true);
            }
            if flags & 0b00100000 != 0 {
                state.set_flag(idx * 8 + 5, true);
            }
            if flags & 0b01000000 != 0 {
                state.set_flag(idx * 8 + 6, true);
            }
            if flags & 0b10000000 != 0 {
                state.set_flag(idx * 8 + 7, true);
            }
        }

        state.textscript_vm.start_script(0);

        game_scene.player1.equip.0 = self.equipment as u16;

        game_scene.player1.x = self.pos_x;
        game_scene.player1.y = self.pos_y;

        game_scene.player1.control_mode =
            if self.control_mode == 1 { ControlMode::IronHead } else { ControlMode::Normal };
        game_scene.player1.direction = self.direction;
        game_scene.player1.life = self.life;
        game_scene.player1.max_life = self.max_life;
        game_scene.player1.stars = clamp(self.stars, 0, 3) as u8;

        game_scene.player2 = game_scene.player1.clone();
        if self.max_life_p2 != 0 && self.life_p2 != 0 {
            game_scene.player2.max_life = self.max_life_p2;
            game_scene.player2.life = self.life_p2;
        }
        // game_scene.inventory_player2 = game_scene.inventory_player1.clone();

        game_scene.player1.cond.0 = 0x80;

        state.difficulty = GameDifficulty::from_primitive(self.difficulty);

        game_scene.player1.skin.apply_gamestate(state);
        game_scene.player2.skin.apply_gamestate(state);
    }

    pub fn dump(state: &mut SharedGameState, game_scene: &mut GameScene, target_player: Option<TargetPlayer>) -> GameProfile {
        let player = match target_player.unwrap_or(TargetPlayer::Player1) {
            TargetPlayer::Player1 => &game_scene.player1,
            TargetPlayer::Player2 => &game_scene.player2,
        };

        // TODO: should we store inventory of player 2?
        let inventory_player = match target_player.unwrap_or(TargetPlayer::Player1) {
            TargetPlayer::Player1 => &game_scene.inventory_player1,
            TargetPlayer::Player2 => &game_scene.inventory_player2,
        };

        let current_map = game_scene.stage_id as u32;
        let current_song = state.sound_manager.current_song() as u32;
        let pos_x = player.x as i32;
        let pos_y = player.y as i32;
        let direction = player.direction;
        let max_life = game_scene.player1.max_life;
        let max_life_p2 = game_scene.player1.max_life;
        let stars = player.stars as u16;
        let life = game_scene.player1.life;
        let life_p2 = game_scene.player2.life;
        let current_weapon = inventory_player.current_weapon as u32;
        let current_item = inventory_player.current_item as u32;
        let equipment = player.equip.0 as u32;
        let control_mode = player.control_mode as u32;
        let counter = 0; // TODO
        let mut weapon_data = [WeaponData::default(); 8];
        let mut weapon_data_p2 = None;
        let mut items = [0u32; 32];
        let mut teleporter_slots = [TeleporterSlotData::default(); 8];

        fn dump_weapons(weapons: &mut [WeaponData], inventory: &Inventory) {
            for (idx, weap) in weapons.iter_mut().enumerate() {
                if let Some(weapon) = inventory.get_weapon(idx) {
                    weap.weapon_id = weapon.wtype as u32;
                    weap.level = weapon.level as u32;
                    weap.exp = weapon.experience as u32;
                    weap.max_ammo = weapon.max_ammo as u32;
                    weap.ammo = weapon.ammo as u32;
                }
            }
        }

        dump_weapons(&mut weapon_data, &game_scene.inventory_player1);
        if state.player_count == PlayerCount::Two {
            let mut weapons_p2 = [WeaponData::default(); 8];
            dump_weapons(&mut weapons_p2, &game_scene.inventory_player2);

            weapon_data_p2 = Some(weapons_p2);
        }

        for (idx, item) in items.iter_mut().enumerate() {
            if let Some(sitem) = inventory_player.get_item_idx(idx) {
                *item = sitem.0 as u32 + (((sitem.1 - 1) as u32) << 16);
            }
        }

        for (idx, slot) in teleporter_slots.iter_mut().enumerate() {
            if let Some(&(index, event_num)) = state.teleporter_slots.get(idx) {
                slot.index = index as u32;
                slot.event_num = event_num as u32;
            }
        }

        let mut map_flags = [0u8; 128];
        for (idx, map_flag) in state.map_flags.iter().enumerate() {
            if let Some(out) = map_flags.get_mut(idx) {
                *out = if map_flag { 1 } else { 0 };
            } else {
                break;
            }
        }

        let mut flags = [0u8; 1000];
        state.game_flags.copy_to_slice(&mut flags);

        let timestamp = get_timestamp();
        let difficulty = state.difficulty as u8;
        let eggfish_killed = false; // TODO

        GameProfile {
            current_map,
            current_song,
            pos_x,
            pos_y,
            direction,
            max_life,
            max_life_p2,
            stars,
            life,
            life_p2,
            current_weapon,
            current_item,
            equipment,
            control_mode,
            counter,
            weapon_data,
            weapon_data_p2,
            items,
            teleporter_slots,
            map_flags,
            flags,
            timestamp,
            difficulty,
            eggfish_killed
        }
    }

    pub fn write_save<W: io::Write>(&self, data: &mut W, format: &SaveFormat) -> GameResult {
        data.write_u64::<BE>(SIG_Do041220)?;

        data.write_u32::<LE>(self.current_map)?;
        data.write_u32::<LE>(self.current_song)?;
        data.write_i32::<LE>(self.pos_x)?;
        data.write_i32::<LE>(self.pos_y)?;
        data.write_u32::<LE>(self.direction as u32)?;
        data.write_u16::<LE>(self.max_life)?;

        // TODO: P2 values
        if *format == SaveFormat::Switch {
            data.write_u16::<LE>(self.max_life)?;
        }

        data.write_u16::<LE>(self.stars)?;
        data.write_u16::<LE>(self.life)?;

        // TODO: P2 values
        if *format == SaveFormat::Switch {
            data.write_u16::<LE>(self.life)?;
        }

        data.write_u16::<LE>(0)?;
        data.write_u32::<LE>(self.current_weapon)?;
        data.write_u32::<LE>(self.current_item)?;
        data.write_u32::<LE>(self.equipment)?;
        data.write_u32::<LE>(self.control_mode)?;
        data.write_u32::<LE>(self.counter)?;

        for weapon in &self.weapon_data {
            data.write_u32::<LE>(weapon.weapon_id)?;
            data.write_u32::<LE>(weapon.level)?;
            data.write_u32::<LE>(weapon.exp)?;
            data.write_u32::<LE>(weapon.max_ammo)?;
            data.write_u32::<LE>(weapon.ammo)?;
        }
        if *format == SaveFormat::Switch {
            for weapon in &self.weapon_data_p2.unwrap_or([WeaponData::default(); 8]) {
                data.write_u32::<LE>(weapon.weapon_id)?;
                data.write_u32::<LE>(weapon.level)?;
                data.write_u32::<LE>(weapon.exp)?;
                data.write_u32::<LE>(weapon.max_ammo)?;
                data.write_u32::<LE>(weapon.ammo)?;
            }
        }

        for item in self.items.iter().copied() {
            data.write_u32::<LE>(item)?;
        }

        for slot in &self.teleporter_slots {
            data.write_u32::<LE>(slot.index)?;
            data.write_u32::<LE>(slot.event_num)?;
        }

        // Probably map flags, but unused anyway
        let something = [0u8; 0x80];
        data.write(&something)?;

        data.write_u32::<BE>(SIG_FLAG)?;
        data.write(&self.flags)?;

        if *format == SaveFormat::Plus {
            data.write_u32::<LE>(0)?; // unused(?) CS+ space
        }

        data.write_u64::<LE>(self.timestamp)?;
        data.write_u8(self.difficulty)?;

        if format.is_csp() {
            // unused(?) CS+ space
            let zeros = [0u8; 15];
            data.write(&zeros);
        }

        Ok(())
    }

    pub fn load_from_save<R: io::Read>(data: &mut R, format: SaveFormat) -> GameResult<GameProfile> {
        let magic = data.read_u64::<BE>()?;
        if magic == 0 && format.is_csp() {
            // Some of the CS+ slots may be filled with 0, so all signatures and magic numbers will be invalid
            let profile_size = if format == SaveFormat::Plus { 0x620 } else { 0x680 } as usize;
            let mut dummy: Vec<u8> = Vec::with_capacity(profile_size);

            let _ = data.read(&mut dummy)?;
            return Ok(GameProfile::default());
        } else if magic != SIG_Do041220 && magic != SIG_Do041115 {
            return Err(ResourceLoadError("Invalid magic".to_owned()));
        }

        let current_map = data.read_u32::<LE>()?;
        let current_song = data.read_u32::<LE>()?;
        let pos_x = data.read_i32::<LE>()?;
        let pos_y = data.read_i32::<LE>()?;
        let direction = data.read_u32::<LE>()?;
        let max_life = data.read_u16::<LE>()?;
        // TODO: P2 values
        let max_life_p2 = if format == SaveFormat::Switch { data.read_u16::<LE>()? } else { max_life };
        let stars = data.read_u16::<LE>()?;
        let life = data.read_u16::<LE>()?;
        // TODO: P2 values
        let life_p2 = if format == SaveFormat::Switch { data.read_u16::<LE>()? } else { life };
        let _ = data.read_u16::<LE>()?; // ???
        let current_weapon = data.read_u32::<LE>()?;
        let current_item = data.read_u32::<LE>()?;
        let equipment = data.read_u32::<LE>()?;
        let control_mode = data.read_u32::<LE>()?;
        let counter = data.read_u32::<LE>()?;
        let mut weapon_data = [WeaponData::default(); 8];
        let mut weapon_data_p2 = None;
        let mut items = [0u32; 32];
        let mut teleporter_slots = [TeleporterSlotData::default(); 8];

        fn load_weapons<R: io::Read>(data: &mut R, weapons: &mut [WeaponData]) -> GameResult {
            for WeaponData { weapon_id, level, exp, max_ammo, ammo } in weapons {
                *weapon_id = data.read_u32::<LE>()?;
                *level = data.read_u32::<LE>()?;
                *exp = data.read_u32::<LE>()?;
                *max_ammo = data.read_u32::<LE>()?;
                *ammo = data.read_u32::<LE>()?;
            }

            Ok(())
        }

        load_weapons(data, &mut weapon_data)?;
        if format == SaveFormat::Switch {
            weapon_data_p2 = Some([WeaponData::default(); 8]);
            load_weapons(data, weapon_data_p2.as_mut().unwrap())?;
        }

        for item in &mut items {
            *item = data.read_u32::<LE>()?;
        }

        for TeleporterSlotData { index, event_num } in &mut teleporter_slots {
            *index = data.read_u32::<LE>()?;
            *event_num = data.read_u32::<LE>()?;
        }

        let mut map_flags = [0u8; 0x80];
        data.read_exact(&mut map_flags)?;

        if data.read_u32::<BE>()? != SIG_FLAG {
            return Err(ResourceLoadError("Invalid FLAG signature".to_owned()));
        }

        let mut flags = [0u8; 1000];
        data.read_exact(&mut flags)?;

        let _ = data.read_u32::<LE>().unwrap_or(0); // unused(?) CS+ space

        let timestamp = data.read_u64::<LE>().unwrap_or(0);
        let difficulty = data.read_u8().unwrap_or(0);
        let eggfish_killed = false; //

        if format.is_csp() {
            // unused(?) CS+ space
            let mut zeros = [0u8; 15];
            data.read(&mut zeros)?;
        }

        Ok(GameProfile {
            current_map,
            current_song,
            pos_x,
            pos_y,
            direction: Direction::from_int(direction as usize).unwrap_or(Direction::Left),
            max_life,
            max_life_p2,
            stars,
            life,
            life_p2,
            current_weapon,
            current_item,
            equipment,
            control_mode,
            counter,
            weapon_data,
            weapon_data_p2,
            items,
            teleporter_slots,
            map_flags,
            flags,
            timestamp,
            difficulty,
            eggfish_killed
        })
    }

    pub fn is_empty(&self) -> bool {
        self.timestamp == 0
    }

}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SaveSlot {
    MainGame(usize), // (save slot)
    CSPMod(u8, usize), // (mod save set, save_slot)
    Mod(String, usize), // (mod id, save slot)
}

impl SaveSlot {
    pub fn into_idx(self) -> Self {
        match self {
            Self::MainGame(save_slot) => if save_slot == 0 {
                Self::MainGame(save_slot)
            } else {
                Self::MainGame(save_slot - 1)
            },
            Self::CSPMod(save_set, save_slot) => { Self::CSPMod(save_set - 1, save_slot - 1) },
            Self::Mod(mod_id, save_slot) => if save_slot == 0 {
                Self::Mod(mod_id, save_slot)
            } else {
                Self::Mod(mod_id, save_slot - 1)
            },
        }
    }
}

impl FromStr for SaveSlot {
    type Err = GameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn extract_num(chars: &mut Chars, replace_emtpy: Option<usize>) -> Option<usize> {
            let mut buf = String::new();

            for char in chars {
                log::debug!("{}", char);
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum SaveFormat {
    Freeware,
    Plus,

    // TODO: add version (v1.2 or v1.3)
    Switch,
    Generic,
}

impl SaveFormat {
    pub fn recognise(data: &[u8]) -> GameResult<SaveFormat> {
        let mut cur = std::io::Cursor::new(data);
        let magic = cur.read_u64::<BE>()?;

        let original_format = match magic {
            // In CS+ a profile signature at the start of the save file is present only
            // if the first game slot profile exists. Otherwise it will be filled with zeros.
            // TODO: should we handle
            SIG_Do041220 =>
                match data.len() {
                    0x604..=0x620 => Some(SaveFormat::Freeware),
                    0x20020 => Some(SaveFormat::Plus),

                    // 0x20fb0 — v1.2
                    // 0x20fb4 — v1.3
                    0x20fb0 | 0x20fb4 => Some(SaveFormat::Switch),
                    _ => None
                },

            SIG_Do041115 => Some(SaveFormat::Freeware),
            _ => None
        };

        if let Some(format) = original_format {
            return Ok(format);
        }

        // Generic save is stored in JSON format, so it must start with '{' character
        if data[0] == '{' as u8 {
            cur.set_position(0);
            if serde_json::from_reader::<_, SaveContainer>(cur).is_ok() {
                return Ok(SaveFormat::Generic);
            }
        }

        Err(ResourceLoadError("Unsupported or invalid save file".to_owned()))
    }


    pub fn is_csp(&self) -> bool {
        match self {
            SaveFormat::Plus | SaveFormat::Switch => true,
            _ => false
        }
    }

    // TODO: compatibility warnings
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CSPModProfile {
    pub profiles: HashMap<usize, GameProfile>,
    // TODO: add TimingMode for best times
    pub time: usize, // Best time for challenges
}

impl CSPModProfile {
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty() && self.time == 0
    }
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
        // Import/export all slots by default
        Self {
            slots: vec![],
            settings: true
        }
    }
}

// Generic container to store all possible info from original game saves
#[derive(Debug, Deserialize, Serialize)]
pub struct SaveContainer {
    pub version: usize,
    pub game_profiles: HashMap<usize, GameProfile>,
    pub csp_mods: HashMap<u8, CSPModProfile>, // save_set number -> saves & time

    #[serde(skip)]
    patchset: HashMap<SaveSlot, SavePatch>,
    // TODO: engine and mods specific fields
}

impl Default for SaveContainer {
    fn default() -> SaveContainer {
        SaveContainer {
            version: 1,

            game_profiles: HashMap::new(),
            csp_mods: HashMap::new(),

            patchset: HashMap::new(),
        }
    }
}

impl SaveContainer {
    pub fn load(ctx: &mut Context, state: &mut SharedGameState) -> GameResult<SaveContainer> {
        log::debug!("DEBUG LOAD SAVE");

        if let Ok(mut file) = user_open(ctx, "/save.json") {
            log::debug!("DEBUG LOAD SAVE - FILE EXISTS");
            // Using of buf significantly speed up the deserialization.
            let mut buf: Vec<u8> = Vec::new();
            file.read_to_end(&mut buf)?;

            return Ok(Self::load_from_buf(ctx, state, buf.as_slice()).unwrap_or_default());
        }

        log::debug!("DEBUG LOAD SAVE - DEFAULT CREATED");

        Ok(Self::default())
    }

    fn load_from_buf(ctx: &mut Context, state: &mut SharedGameState, buf: &[u8]) -> GameResult<SaveContainer> {
        serde_json::from_slice::<SaveContainer>(buf)
            .map_err(|err| GameError::ResourceLoadError(format!("Failed to deserialize a generic save. {}", err.to_string())))
            .map(| container| container.upgrade())
    }

    pub fn save(&mut self, ctx: &mut Context, state: &mut SharedGameState, params: SaveParams) -> GameResult {
        self.write_save(ctx, state, SaveFormat::Generic, None, None, &params)?;
        self.write_save(ctx, state, state.settings.save_format, None, None, &params)?;
        self.patchset.clear();
        Ok(())
    }

    pub fn write_save(&mut self, ctx: &mut Context, state: &mut SharedGameState, format: SaveFormat, slot: Option<SaveSlot>, mut out_path: Option<PathBuf>, params: &SaveParams) -> GameResult {
        log::debug!("DEBUG WRITE SAVE");

        let save_path = Self::get_save_filename(&format, slot.clone());

        match format {
            SaveFormat::Generic => {
                let mut file = user_create(ctx, &save_path)?;

                // Using of buf significantly speed up the serializing.
                let buf = serde_json::to_vec(&self)?;
                file.write_all(&buf)?;
                file.flush()?
            },
            SaveFormat::Freeware => {
                for (save_slot, profile) in &self.game_profiles {
                    if params.slots.is_empty() || params.slots.contains(&SaveSlot::MainGame(*save_slot)) {
                        let mut buf = Vec::new();
                        let mut cur = std::io::Cursor::new(&mut buf);
                        profile.write_save(&mut cur, &format)?;

                        let mut filename = Self::get_save_filename(&format, Some(SaveSlot::MainGame(*save_slot)));
                        if let Some(path) = &mut out_path {
                            let os_filename = OsString::from_str(filename.split_off(1).as_str()).unwrap();
                            let _ = path.set_file_name(os_filename);

                            let mut file = File::create(path)?;
                            file.write_all(&buf)?;
                        } else {
                            let mut file = user_create(ctx, filename)?;
                            file.write_all(&buf)?;
                        }
                    }
                }

                for (save_set, csp_mod) in &self.csp_mods {
                    for (slot, profile) in &csp_mod.profiles {
                        if params.slots.is_empty() || params.slots.contains(&SaveSlot::CSPMod(*save_set, *slot)) {
                            let mut buf = Vec::new();
                            let mut cur = std::io::Cursor::new(&mut buf);
                            profile.write_save(&mut cur, &format)?;

                            let mut filename = Self::get_save_filename(&format, Some(SaveSlot::CSPMod(*save_set, *slot)));
                            if let Some(path) = &mut out_path {
                                let os_filename = OsString::from_str(filename.split_off(1).as_str()).unwrap();
                                let _ = path.set_file_name(os_filename);

                                let mut file = File::create(path)?;
                                file.write_all(&buf)?;
                            } else {
                                let mut file = user_create(ctx, filename)?;
                                file.write_all(&buf)?;
                            }
                        }
                    }
                }

                for (patch_slot, patch_state) in self.patchset.iter() {
                    if *patch_state == SavePatch::Deleted {
                        // TODO: should we unwrap the result?
                        user_delete(ctx, Self::get_save_filename(&format, Some(patch_slot.clone())))?;
                    }
                }
            },
            SaveFormat::Plus | SaveFormat::Switch => {
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
                let unused = [0u8; 14];
                let mut jukebox: [u8; 48] = [
                    0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80,
                    0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80,
                    0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80,
                    0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80,
                    0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80,
                    0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80
                ]; // TODO
                let mut eggfish_killed = [0u8; 3];
                let mut nikumaru = NikumaruCounter::new();
                nikumaru.load_counter(state, ctx);

                let mut buf = Vec::new();
                let mut cur = std::io::Cursor::new(&mut buf);

                // TODO
                // In CS+, only slots up to the last non-empty one are written to the save file.
                // E.g., if the user saved only the profile in slot 3, then profiles from slots 1, 2 and 3 will be written to the file.
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

                // TODO
                // TODO: write some fields only in the v1.3 version mode
                if format == SaveFormat::Switch {
                    cur.write(&jukebox)?;
                    cur.write_u8(0)?; // Unlock notifications.
                    cur.write_u8(0)?; // Shared health bar.

                    // Something
                    cur.write_u24::<LE>(0)?;
                } else {
                    let zeros = [0u8; 7];
                    cur.write(&zeros)?;
                }

                // Challange best times
                cur.write_u32::<LE>(nikumaru.tick.try_into().unwrap_or(u32::MAX))?;
                for save_set in 1..=26 {
                    let csp_mod = self.csp_mods.get(&save_set).unwrap_or(&default_csp_profile);
                    cur.write_u32::<LE>(csp_mod.time.try_into().unwrap_or(u32::MAX))?;
                }

                // TODO
                // TODO: write some fields only in the v1.3 version mode
                if format == SaveFormat::Switch {
                    cur.write_u8(0)?;

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
                    user_create(ctx, save_path)?.write(&buf)?;
                }
            },
            _ => todo!()
        }

        Ok(())
    }

    pub fn get_save_filename(format: &SaveFormat, slot: Option<SaveSlot>) -> String {
        match format {
            SaveFormat::Generic => "/save.json".to_owned(),
            SaveFormat::Plus | SaveFormat::Switch => "/Profile.dat".to_owned(),
            SaveFormat::Freeware => {
                match slot {
                    Some(SaveSlot::MainGame(save_slot)) => if save_slot == 1 {
                        "/Profile.dat".to_owned()
                    } else {
                        format!("/Profile{}.dat", save_slot)
                    },
                    Some(SaveSlot::CSPMod(save_set, save_slot)) => format!("/Mod{}_Profile{}.dat", save_set, save_slot),
                    Some(SaveSlot::Mod(mod_id, save_slot)) => unimplemented!(),
                    _ => "/Profile.dat".to_owned()
                }
            }
        }
    }

    pub fn upgrade(self) -> Self {
        log::debug!("DEBUG UPGRADE SAVE");
        let initial_version = self.version;

        if self.version != initial_version {
            log::info!("Upgraded generic save from version {} to {}.", initial_version, self.version);
        }

        self
    }


    pub fn set_profile(&mut self, slot: SaveSlot, profile: GameProfile) {
        log::debug!("Debug profile set: {:?}; {}", slot, profile.timestamp);

        let prev_save: Option<GameProfile>;
        match slot {
            SaveSlot::MainGame(save_slot) => {
                prev_save = self.game_profiles.insert(save_slot, profile);
            },
            SaveSlot::CSPMod(save_set, save_slot) => {
                prev_save = self.csp_mods.entry(save_set)
                    .or_insert(CSPModProfile::default())
                    .profiles
                    .insert(save_slot, profile);
            },
            SaveSlot::Mod(ref mod_id, save_slot) => {
                // TODO
                prev_save = None;
                unimplemented!();
            }
        }

        if prev_save.is_none() {
            self.patchset.insert(slot, SavePatch::Added);
        } else {
            self.patchset.insert(slot, SavePatch::Modified);
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
            SaveSlot::Mod(mod_id, save_slot) => unimplemented!()
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
            SaveSlot::Mod(mod_id, save_slot) => unimplemented!()
        }

        self.patchset.insert(slot, SavePatch::Deleted);
    }

    pub fn is_empty(&self) -> bool {
        if !self.game_profiles.is_empty() {
            return false;
        }

        for (_, csp_mod) in self.csp_mods.iter() {
            if csp_mod.is_empty() {
                return false;
            }
        }

        true
    }



    fn merge(&mut self, b: &Self) {
        self.game_profiles.extend(b.game_profiles.iter());
        for (b_set, b_csp_mod) in &b.csp_mods {
            if let Some(csp_mod) = self.csp_mods.get_mut(b_set) {
                csp_mod.profiles.extend(b_csp_mod.profiles.iter());
                csp_mod.time = csp_mod.time.min(b_csp_mod.time);
            }
        }
    }


    pub fn import(&mut self, state: &mut SharedGameState, ctx: &mut Context, format: Option<SaveFormat>, params: SaveParams, save_path: PathBuf) -> GameResult {
        let path = save_path.clone().into_boxed_path();
        let data = std::fs::read(path)?;

        let format = format.unwrap_or(SaveFormat::recognise(data.as_slice())?);

        log::trace!("Import format: {:?}.", format);
        log::trace!("Import params: {:?}.", params);
        log::trace!("Import path: {:?}.", save_path);

        match format {
            SaveFormat::Generic => {
                *self = Self::load_from_buf(ctx, state, data.as_slice())?;
            }
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
            }
            SaveFormat::Plus | SaveFormat::Switch => {
                let mut container = Self::default();

                let mut cur = Cursor::new(data);
                let mut active_slots = [0u8; 32];

                for save_slot in 1..=3 {
                    log::debug!("{}", save_slot);
                    let profile: Result<GameProfile, GameError> = GameProfile::load_from_save(&mut cur, format);
                    container.set_profile(SaveSlot::MainGame(save_slot), profile?);
                }

                for save_set in 1..=26 {
                    for save_slot in 1..=3 {
                        let profile = GameProfile::load_from_save(&mut cur, format)?;
                        container.set_profile(SaveSlot::CSPMod(save_set, save_slot), profile);
                    }
                }

                cur.read_exact(&mut active_slots)?;

                for save_set in 0..28 {
                    for save_slot in 1..=3 {
                        let is_inactive = !(active_slots[save_set] & (1u8 << (save_slot - 1)) == 0);

                        if is_inactive {
                            continue;
                        }

                        let slot = if save_set == 0 { SaveSlot::MainGame(save_slot) } else { SaveSlot::CSPMod(save_set as u8, save_slot) };
                        container.delete_profile(ctx, slot);
                    }
                }

                // TODO: change settings in the menu
                if params.settings {
                    state.settings.bgm_volume = cur.read_u32::<LE>()? as f32 / 10.0;
                    state.settings.sfx_volume = cur.read_u32::<LE>()? as f32 / 10.0;
                    state.settings.seasonal_textures = (cur.read_u8()? == 1);
                    state.settings.soundtrack = match cur.read_u8()? {
                        2 => "organya",
                        3 => "new",
                        4 => "remastered",
                        5 => "famitracks",
                        6 => "ridiculon",
                        _ => state.settings.soundtrack.as_str()
                    }.to_owned();
                    let graphics = (cur.read_u8()? == 1);
                    if graphics != state.settings.original_textures {
                        state.settings.original_textures = graphics;
                        state.reload_resources(ctx)?;
                    }

                    state.settings.save(ctx)?;

                    // TODO: should we import locale?
                    let locale = cur.read_u8()?;
                } else {
                    // Skip settings
                    let mut settings = [0u8; 12];
                    cur.read_exact(&mut settings)?;
                }

                state.mod_requirements.beat_hell = (cur.read_u8()? == 1);

                if format == SaveFormat::Switch {
                     // TODO
                    let mut jukebox = [0u8; 48];

                    let unlock_notifications: u8;
                    let shared_healthbar: u8;

                    cur.read_exact(&mut jukebox)?;
                    unlock_notifications = cur.read_u8()?;
                    shared_healthbar = cur.read_u8()?;

                    // unused
                    let _ = cur.read_u24::<LE>()?;
                } else {
                    let mut unused = [0u8; 7];
                    cur.read_exact(&mut unused);
                }

                let mut nikumaru_counter = NikumaruCounter::new();
                nikumaru_counter.load_counter(state, ctx)?;

                let counter = cur.read_u32::<LE>()? as usize;
                if counter < nikumaru_counter.tick {
                    nikumaru_counter.tick = counter;
                    nikumaru_counter.save_counter(state, ctx)?;
                }

                // TODO: import CSP mod times
                let mut best_times = [0u32; 26];
                for mod_id in 1..=26 {
                    best_times[mod_id - 1] = cur.read_u32::<LE>()?;
                }

                // TODO: load all other fields


                self.merge(&container);
            }
        }

        Ok(())
    }

    pub fn export(&mut self, state: &mut SharedGameState, ctx: &mut Context, format: SaveFormat, params: SaveParams, out_path: PathBuf) -> GameResult {
        self.write_save(ctx, state, format, None, Some(out_path.clone()), &params)?;

        log::trace!("Export format: {:?}.", format);
        log::trace!("Export params: {:?}.", params);
        log::trace!("Export path: {:?}.", out_path);
        Ok(())
    }
}
