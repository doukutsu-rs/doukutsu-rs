use std::collections::{HashMap, hash_map::Entry};
use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::marker::Copy;
use std::path::Path;

use byteorder::{BE, LE, ReadBytesExt, WriteBytesExt};
use num_traits::{clamp, FromPrimitive};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::common::{Direction, FadeState, get_timestamp};
use crate::components::nikumaru::NikumaruCounter;
use crate::framework::context::Context;
use crate::framework::error::GameError::ResourceLoadError;
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


#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct WeaponData {
    pub weapon_id: u32,
    pub level: u32,
    pub exp: u32,
    pub max_ammo: u32,
    pub ammo: u32,
}

impl Default for WeaponData {
    fn default() -> WeaponData {
        WeaponData {
            weapon_id: 0,
            level: 0,
            exp: 0,
            max_ammo: 0,
            ammo: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct TeleporterSlotData {
    pub index: u32,
    pub event_num: u32,
}

impl Default for TeleporterSlotData {
    fn default() -> TeleporterSlotData {
        TeleporterSlotData {
            index: 0,
            event_num: 0
        }
    }
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
    pub stars: u16,
    pub life: u16,
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
            stars: 0,
            life: 0,
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
        let max_life = player.max_life;
        let stars = player.stars as u16;
        let life = player.life;
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
            stars,
            life,
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
        if *format == SaveFormat::Freeware {
            data.write_u64::<BE>(SIG_Do041220)?;
        }

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
            let zeros = [0u8; 15];
            data.write(&zeros);
        }

        Ok(())
    }

    pub fn load_from_save<R: io::Read>(data: &mut R, format: SaveFormat) -> GameResult<GameProfile> {
/*
        let magic = data.read_u64::<BE>()?;
        if magic != SIG_Do041220 && magic != SIG_Do041115 {
            return Err(ResourceLoadError("Invalid magic".to_owned()));
        }
*/
        let current_map = data.read_u32::<LE>()?;
        let current_song = data.read_u32::<LE>()?;
        let pos_x = data.read_i32::<LE>()?;
        let pos_y = data.read_i32::<LE>()?;
        let direction = data.read_u32::<LE>()?;
        let max_life = data.read_u16::<LE>()?;
        let stars = data.read_u16::<LE>()?;
        let life = data.read_u16::<LE>()?;
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
            load_weapons(data, weapon_data_p2.as_mut().unwrap());
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

        data.read_u32::<LE>().unwrap_or(0); // unused(?) CS+ space

        let timestamp = data.read_u64::<LE>().unwrap_or(0);
        let difficulty = data.read_u8().unwrap_or(0);
        let eggfish_killed = false; // TODO

        Ok(GameProfile {
            current_map,
            current_song,
            pos_x,
            pos_y,
            direction: Direction::from_int(direction as usize).unwrap_or(Direction::Left),
            max_life,
            stars,
            life,
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

#[derive(Clone, Debug)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum SaveFormat {
    Freeware,
    Plus,
    Switch,
    //Native,
    Generic,
}

impl SaveFormat {
    pub fn recognise(data: &[u8]) -> GameResult<SaveFormat> {
        let mut cur = std::io::Cursor::new(data);
        let magic = cur.read_u64::<BE>()?;
        if magic == SIG_Do041220 {
            let len = data.len();
            if len >= 0x604 && len < 0x20020 {
                return Ok(SaveFormat::Freeware);
            } else if len >= 0x20020 && len < 0x5e0 {
                return Ok(SaveFormat::Plus);
            } else if len >= 0x5e0 {
                return Ok(SaveFormat::Switch);
            };
        } else if magic == SIG_Do041115 {
            // 0.9.x.x beta saves
            return Ok(SaveFormat::Freeware);
        }

        // TODO: detect generic saves

        //Ok(SaveFormat::Generic)
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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

impl Default for CSPModProfile {
    fn default() -> CSPModProfile {
        CSPModProfile {
            profiles: HashMap::new(),
            time: 0,
        }
    }
}

// Generic container to store all possible info from original game saves
#[derive(Debug, Deserialize, Serialize)]
pub struct SaveContainer {
    pub version: usize,

    // We're intended to use heap containers like `Vec` or `HashMap`, as using of size-fixed arrays cause stack overflow
    // while a save parsing.
    // Some useless calculations: `GameProfile` occupies ~1.5KB, original CS+ Switch save format
    // can save 3 main game profiles + 78 mod profiles = 81 `GameProfile` structs. So to load such save we need ~120KB.
    // That's not too much, but for unknown reason `serde` fails to parse such "huge" struct, even when the app compiled
    // with 8-32MB stack. I didn't explore this problem deeply, but in debugger some references that passed to the
    // internal serde functions are null. Use of `serde_stacker` to "extend" the stack capacity, available for parsing,
    // doesn't help either.

    pub game_profiles: HashMap<usize, GameProfile>,
    pub csp_mods: HashMap<u8, CSPModProfile>, // save_set number -> saves & time

    // TODO: engine and mods specific fields
}

impl Default for SaveContainer {
    fn default() -> SaveContainer {
        SaveContainer {
            version: 1,

            game_profiles: HashMap::new(),
            csp_mods: HashMap::new()
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

            log::debug!("DEBUG LOAD SAVE - PREDESERIALIZE. Len: {}", buf.len());
            let now = std::time::Instant::now();
            match serde_json::from_slice::<SaveContainer>(buf.as_mut_slice()) {
                Ok(mut save) => {
                    log::debug!("DEBUG LOAD SAVE - PREUPGRADE");
                    save.upgrade();
                    log::debug!("DEBUG LOAD SAVE - UPGRADED");
                    log::info!("DEBUG LOAD - {} SECS ELAPSED", now.elapsed().as_secs_f64());
                    //log::debug!("{:?}", save);
                    return Ok(save);
                },
                Err(err) => log::warn!("Failed to deserialize a generic save: {}", err),
            }
        }

        log::debug!("DEBUG LOAD SAVE - DEFAULT CREATED");

        let container = SaveContainer::default();
        container.write_save(ctx, state, SaveFormat::Generic, None, None)?;
        Ok(container)
    }

    pub fn save(&self, ctx: &mut Context, state: &mut SharedGameState) -> GameResult {
        self.write_save(ctx, state, SaveFormat::Generic, None, None)?;
        self.write_save(ctx, state, state.settings.save_format, None, None)?;
        Ok(())
    }

    pub fn write_save(&self, ctx: &mut Context, state: &mut SharedGameState, format: SaveFormat, slot: Option<SaveSlot>, out_path: Option<String>) -> GameResult {
        log::debug!("DEBUG WRITE SAVE");

        let save_path = Self::get_save_filename(&format, slot.clone());

        match format {
            SaveFormat::Generic => {
                let exists = user_exists(ctx, &save_path);
                let mut file = user_create(ctx, &save_path)?;

                // Using of buf significantly speed up the serializing.
                let buf = serde_json::to_vec(&self)?;
                file.write_all(&buf)?;
                file.flush()?
            },
            SaveFormat::Freeware => {
                if let Some(save_slot) = slot {

                } else {
                    if let Some(path) = out_path {
                        let base_dir = std::path::Path::new(&path);
                        if !base_dir.is_dir() {
                            // Provided path is a file. So export only the first profile.
                            // TODO
                        }

                        // TODO
                    } else {
                        // Export all profiles
                        for (save_slot, profile) in &self.game_profiles {
                            let mut buf = Vec::new();
                            let mut cur = std::io::Cursor::new(&mut buf);
                            profile.write_save(&mut cur, &format)?;

                            let mut file = user_create(ctx, Self::get_save_filename(&format, Some(SaveSlot::MainGame(*save_slot))))?;
                            file.write_all(&buf);
                            file.flush()?;
                        }

                        for (save_set, csp_mod) in &self.csp_mods {
                            for (slot, profile) in &csp_mod.profiles {
                                let mut buf = Vec::new();
                                let mut cur = std::io::Cursor::new(&mut buf);
                                profile.write_save(&mut cur, &format)?;

                                let mut file = user_create(ctx, Self::get_save_filename(&format, Some(SaveSlot::CSPMod(*save_set, *slot))))?;
                                file.write_all(&buf);
                                file.flush()?;
                            }
                        }
                    }
                }
            },
            SaveFormat::Plus | SaveFormat::Switch => {
                if let Some(save_slot) = slot {
                    // TODO
                } else {
                    if let Some(path) = out_path {
                        // TODO
                    } else {
                        // Export all profiles

                        let mut active_slots = [0u8; 32];

                        // Setting
                        let bgm_volume = ((state.settings.bgm_volume * 10.0) as u8).min(10);
                        let sfx_volume = ((state.settings.bgm_volume * 10.0) as u8).min(10);
                        let seasonal_textures = state.settings.seasonal_textures as u8;
                        let soundtrack = match state.settings.soundtrack.as_str() {
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



                        cur.write_u64::<BE>(SIG_Do041220)?;

                        let default_profile = GameProfile::default();
                        for save_slot in 1..=3 {
                            log::debug!("Writing Game profile: {}", save_slot);
                            let profile = if let Some(game_profile) = self.game_profiles.get(&save_slot) {
                                active_slots[0] |= (1u8 << (save_slot - 1));
                                game_profile
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
                                log::debug!("Writing CSP profile: {} - {}", save_set, save_slot);
                                let profile = if let Some(game_profile) = csp_mod.profiles.get(&save_slot) {
                                    active_slots[save_set as usize] |= (1u8 << (save_slot - 1));
                                    game_profile
                                } else {
                                    &default_profile
                                };
                                profile.write_save(&mut cur, &format)?;
                            }
                        }

                        cur.write(&active_slots)?;

                        // Settings
                        cur.write_u8(bgm_volume)?;
                        cur.write_u8(sfx_volume)?;
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
                            cur.write_u16::<LE>(0)?;
                            cur.write_u8(0)?;
                        } else {
                            let zeros = [0u8; 7];
                            cur.write(&zeros)?;
                        }

                        // Challange best times
                        cur.write_u32::<LE>(nikumaru.tick.try_into().unwrap_or(u32::MAX));
                        for save_set in 1..=26 {
                            let csp_mod = self.csp_mods.get(&save_set).unwrap_or(&default_csp_profile);
                            cur.write_u32::<LE>(csp_mod.time.try_into().unwrap_or(u32::MAX));
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
                            cur.write(&eggfish_killed);

                            let something = [0u8; 0x3d];
                            cur.write(&something)?;

                            let something2 = [0u8; 0xf20];
                            cur.write(&something2)?;
                        }

                        let mut file = user_create(ctx, save_path)?;
                        file.write(&buf)?;
                    }
                }
            },
            _ => todo!()
        }

        Ok(())
    }

    fn get_save_filename(format: &SaveFormat, slot: Option<SaveSlot>) -> String {
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
                    _ => "Profile.dat".to_owned()
                }
            }
        }
    }

    pub fn upgrade(&mut self) {
        log::debug!("DEBUG UPGRADE SAVE");
        let initial_version = self.version;

        if self.version != initial_version {
            log::info!("Upgraded generic save from version {} to {}.", initial_version, self.version);
        }
    }

    // I'm encoutered some troubles with manually manipulating profiles in the menu's code, so this methods makes
    // the logic more clear and separated from other components.

    pub fn set_profile(&mut self, slot: SaveSlot, profile: GameProfile) {
        log::debug!("Debug profile set: {:?}; {}", slot, profile.timestamp);
        match slot {
            SaveSlot::MainGame(save_slot) => {
                let _ = self.game_profiles.insert(save_slot, profile);
            },
            SaveSlot::CSPMod(save_set, save_slot) => {
                let _ = self.csp_mods.entry(save_set)
                    .or_insert(CSPModProfile::default())
                    .profiles
                    .insert(save_slot, profile);
            },
            SaveSlot::Mod(mod_id, save_slot) => {
                // TODO
            }
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

        // TODO: delete original save files
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



    pub fn import(
        &mut self,
        state: &mut SharedGameState,
        ctx: &mut Context,
        import_location: &Path,
        import_format: Option<SaveFormat>,
        slot: Option<SaveSlot>
    ) -> GameResult {
        let save_format = if import_format.is_none() {
            log::warn!("Importing saves from dir without specifying of the save format");

            // TODO: check if the Profile.dat exists and try to recognise format
            SaveFormat::Freeware
        } else { import_format.unwrap() };

        if save_format == SaveFormat::Freeware {
            let base_dir = import_location.to_path_buf().into_os_string();
            if slot.is_some() {
                // Import only specified slot

                let filename = Self::get_save_filename(&save_format, slot.clone());
                let mut file_path = base_dir.clone();
                file_path.push(&OsString::from(filename));

                let mut file = File::open(file_path)?;
                self.import_file(state, ctx, &mut file, save_format, slot)?;
            } else {
                // Import all available files
                for i in 1..=3 {
                    let save_slot = Some(SaveSlot::MainGame(i));
                    let filename = Self::get_save_filename(&save_format, save_slot.clone());
                    let mut file_path = base_dir.clone();
                    file_path.push(&OsString::from(filename));

                    // Ignore if failed to open the file
                    if let Ok(mut file) = File::open(file_path) {
                        self.import_file(state, ctx, &mut file, save_format, save_slot);
                    }
                }

                for save_set in 1..=26 {
                    for profile_slot in 1..=3 {
                        let save_slot = Some(SaveSlot::CSPMod(save_set, profile_slot));
                        let filename = Self::get_save_filename(&save_format, save_slot.clone());
                        let mut file_path = base_dir.clone();
                        file_path.push(&OsString::from(filename));

                        // Ignore if failed to open the file
                        if let Ok(mut file) = File::open(file_path) {
                            self.import_file(state, ctx, &mut file, save_format, save_slot);
                        }
                    }
                }
            }
        } else {
            let mut file = File::open(import_location)?;
            self.import_file(state, ctx, &mut file, save_format, slot)?;
        }

        Ok(())
    }

    fn import_file(
        &mut self,
        state: &mut SharedGameState,
        ctx: &mut Context,
        file: &mut File,
        save_format: SaveFormat,
        slot: Option<SaveSlot>
    ) -> GameResult {
        let mut buf: Vec<u8> = vec![];
        file.read_to_end(&mut buf);

        self.import_buf(state, ctx, &buf, save_format, slot)?;

        Ok(())
    }

    fn import_buf(
        &mut self,
        state: &mut SharedGameState,
        ctx: &mut Context,
        buf: &Vec<u8>,
        save_format: SaveFormat,
        slot: Option<SaveSlot>
    ) -> GameResult {
        let mut cur = std::io::Cursor::new(buf);

        match save_format {
            SaveFormat::Freeware => {
                if slot.is_none() {
                    log::warn!("Import freeware save without specifing of slot. Fallback to the first main game slot.");
                }

                let save_slot = slot.unwrap_or(SaveSlot::MainGame(1));
                let profile = GameProfile::load_from_save(&mut cur, save_format)?;

                self.set_profile(save_slot, profile);
            },
            SaveFormat::Plus | SaveFormat::Switch => {
                // Import specific slot
                let magic = cur.read_u64::<BE>()?;
                if magic != SIG_Do041220 {
                    return Err(ResourceLoadError("Invalid magic".to_owned()));
                }
/*
                if Some(save_slot) = slot {
                    // TODO: dehardcode profile size(1532)
                    let offset: u64 = match save_slot {
                        SaveSlot::MainGame(profile_slot) => {
                            offset = 1532 * (profile_slot - 1);
                        },
                        SaveSlot::CSPMod(save_set, profile_slot) => {
                            1532 * (3 * save_set + profile_slot - 1)
                        },
                        // We won't write and won't read such slots, while we don't have any Mod API
                        SaveSlot::Mod() => todo!(),
                    };

                    cur.seek(io::SeekForm::Current(offset.try_into()?));
                    let profile = GameProfile::load_from_save(&mut cur, format)?;

                    self.set_profile(save_slot, profile)?;
                } else {

                }
*/

                for profile_slot in 1..=3 {
                    let profile = GameProfile::load_from_save(&mut cur, save_format)?;
                    self.game_profiles.insert(profile_slot, profile);
                }

                for save_set in 1..=26 {
                    let mut csp_mod = CSPModProfile::default();
                    for i in 1..3 {
                        let profile = GameProfile::load_from_save(&mut cur, save_format)?;
                        csp_mod.profiles.insert(i, profile);
                    }

                    self.csp_mods.insert(save_set, csp_mod);
                }

                for save_set in 0..27 {
                    let active_game_profiles = cur.read_u8()?;
                    let profiles: &mut HashMap<usize, GameProfile> = if save_set == 0 {
                        &mut self.game_profiles
                    } else {
                        &mut self.csp_mods.get_mut(&save_set).unwrap().profiles
                    };

                    for i in 0..2 {
                        let is_active = active_game_profiles & (1u8 << i);
                        if is_active == 0 {
                            profiles.remove(&(i + 1));
                        }
                    }
                }

                // TODO: overwrite settings, if requested
                let bgm_volume = cur.read_u8()?;
                let sfx_volume = cur.read_u8()?;
                let seasonal_textures = cur.read_u8()? != 0;
                let soundtrack = match cur.read_u8()? {
                    1 => "organya",
                    2 => "organya",
                    3 => "new",
                    4 => "remastered",
                    5 => "famitracks",
                    6 => "ridiculon",
                    _ => "organya" // Fallback to Organya
                };
                let original_textures = cur.read_u8()? != 0;
                let beat_hell = cur.read_u8()? != 0;

                // Currently we're overwriting only negative value
                if beat_hell {
                    state.mod_requirements.beat_hell = beat_hell;
                }

                if save_format == SaveFormat::Switch {
                    // TODO
                    let mut jukebox = [0u8; 48];
                    cur.read(&mut jukebox)?;

                    let unlock_notifications = cur.read_u8()?;// TODO

                    // TODO: v1.3 only
                    let shared_healthbar = cur.read_u8()? != 0;
                    let mut unknown_field = [0u8; 3];
                    cur.read(&mut unknown_field);
                }

                let mut something = [0u8; 7];
                cur.read(&mut something);

                // TODO
                let nikumaru_counter = cur.read_u64::<LE>()?;

                for save_set in 1..=26 {
                    let time = cur.read_u32::<LE>()?;

                    let csp_mod = self.csp_mods.get_mut(&save_set).unwrap();
                    csp_mod.time = time as usize;

                    if csp_mod.is_empty() {
                        self.csp_mods.remove(&save_set);
                    }
                }

                if save_format == SaveFormat::Plus {
                    for i in 1..=3 {
                        if let Some(profile) = self.game_profiles.get_mut(&i) {
                            profile.eggfish_killed = cur.read_u8()? != 0;
                        }
                    }

                    let mut something2 = [0u8; 61];
                    let mut something3 = [0u8; 0xf20];

                    cur.read(&mut something2)?;
                    cur.read(&mut something3)?;
                } else {
                    let challange_unlocks = cur.read_u8()?; // TODO

                    let mut something2 = [0u8; 0x77];
                    let mut something3 = [0u8; 6];

                    cur.read(&mut something2)?;
                    let p2_char_unlocks = cur.read_u16::<LE>()?;
                    cur.read(&mut something3)?;
                }
            },
            SaveFormat::Generic => todo!(),
        }

        Ok(())
    }

    fn default_import(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult<SaveFormat> {
        // If Profile.dat exists, it can be CS+ or Freeware format. Otherwise it's freeware format.
        if let Ok(mut file) = user_open(ctx, "/Profile.dat".to_string()) {
            let mut buf: Vec<u8> = vec![];
            file.read_to_end(&mut buf)?;

            let format = SaveFormat::recognise(&buf).unwrap_or(state.settings.save_format);
            return Ok(format);
        }

        //let dir =

        Ok(state.settings.save_format)
    }
}
