use std::collections::HashMap;
use std::ffi::OsString;
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

use crate::common::{get_timestamp, Direction, FadeState, Version};
use crate::framework::context::Context;
use crate::framework::error::GameError::{self, ResourceLoadError};
use crate::framework::error::GameResult;
use crate::framework::filesystem::{user_create, user_delete, user_exists, user_open};
use crate::game::player::{ControlMode, TargetPlayer};
use crate::game::shared_game_state::{GameDifficulty, PlayerCount, SharedGameState, TimingMode};
use crate::game::weapon::{WeaponLevel, WeaponType};
use crate::game::inventory::Inventory;
use crate::mod_list::ModList;
use crate::mod_requirements::{self, ModRequirements};
use crate::scene::game_scene::GameScene;
use crate::util::rng::RNG;

const SIG_Do041115: u64 = 0x446f303431313135;
const SIG_Do041220: u64 = 0x446f303431323230;
const SIG_FLAG: u32 = 0x464c4147;

pub const SWITCH_VER_1_2: Version = Version { major: 1, minor: 2, patch: 0 };
pub const SWITCH_VER_1_3: Version = Version { major: 1, minor: 3, patch: 0 };

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
        if format.is_switch() {
            data.write_u16::<LE>(self.max_life)?;
        }

        data.write_u16::<LE>(self.stars)?;
        data.write_u16::<LE>(self.life)?;

        // TODO: P2 values
        if format.is_switch() {
            data.write_u16::<LE>(self.life)?;
        }

        data.write_u16::<LE>(0)?; // something
        data.write_u32::<LE>(self.current_weapon)?;
        data.write_u32::<LE>(self.current_item)?;
        data.write_u32::<LE>(self.equipment)?;
        data.write_u32::<LE>(self.control_mode)?;
        data.write_u32::<LE>(self.counter)?;

        fn write_weapon<W: io::Write>(data: &mut W, weapon: &WeaponData, format: &SaveFormat) -> io::Result<()> {
            if format.is_switch() {
                data.write_u32::<LE>(weapon.level)?;
                data.write_u32::<LE>(weapon.exp)?;
                data.write_u32::<LE>(weapon.ammo)?;
            } else {
                data.write_u32::<LE>(weapon.weapon_id)?;
                data.write_u32::<LE>(weapon.level)?;
                data.write_u32::<LE>(weapon.exp)?;
                data.write_u32::<LE>(weapon.max_ammo)?;
                data.write_u32::<LE>(weapon.ammo)?;
            }

            Ok(())
        }

        if format.is_switch() {
            for weapon in &self.weapon_data {
                data.write_u32::<LE>(weapon.weapon_id)?;
                data.write_u32::<LE>(weapon.max_ammo)?;
            }
        }

        for weapon in &self.weapon_data {
            write_weapon(data, weapon, format)?;
        }

        if format.is_switch() {
            for weapon in &self.weapon_data_p2.unwrap_or([WeaponData::default(); 8]) {
                write_weapon(data, weapon, format)?;
            }
        }

        for item in self.items.iter().copied() {
            if format.is_switch() {
                data.write_u16::<LE>(item.try_into().unwrap_or(u16::MAX))?;
                data.write_u16::<LE>(1)?; // TODO: store real quantity of items
            } else {
                data.write_u32::<LE>(item)?;
            }
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

        if !format.is_switch() {
            data.write_u32::<LE>(0)?; // unused(?) CS+ space
        }

        data.write_u64::<LE>(self.timestamp)?;
        data.write_u8(self.difficulty)?;

        if format.is_csp() {
            // unused(?) CS+ space
            let zeros = [0u8; 15];
            data.write(&zeros)?;
        }

        Ok(())
    }

    pub fn load_from_save<R: io::Read>(data: &mut R, format: SaveFormat) -> GameResult<GameProfile> {
        let magic = data.read_u64::<BE>()?;
        if magic == 0 && format.is_csp() {
            // Some of the CS+ slots may be filled with 0, so all signatures and magic numbers will be invalid
            let profile_size: usize = if format == SaveFormat::Plus { 0x620 } else { 0x680 };

            // We've already read the magic, so we substract it from the profile size
            let mut dummy = vec![0u8; profile_size - 8];
            data.read_exact(&mut dummy)?;

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
        let max_life_p2 = if format.is_switch() { data.read_u16::<LE>()? } else { max_life };
        let stars = data.read_u16::<LE>()?;
        let life = data.read_u16::<LE>()?;
        // TODO: P2 values
        let life_p2 = if format.is_switch() { data.read_u16::<LE>()? } else { life };
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

        fn load_weapons<R: io::Read>(data: &mut R, weapons: &mut [WeaponData], format: &SaveFormat) -> GameResult {
            for WeaponData { weapon_id, level, exp, max_ammo, ammo } in weapons {
                if format.is_switch() {
                    *level = data.read_u32::<LE>()?;
                    *exp = data.read_u32::<LE>()?;
                    *ammo = data.read_u32::<LE>()?;
                } else {
                    *weapon_id = data.read_u32::<LE>()?;
                    *level = data.read_u32::<LE>()?;
                    *exp = data.read_u32::<LE>()?;
                    *max_ammo = data.read_u32::<LE>()?;
                    *ammo = data.read_u32::<LE>()?;
                }
            }

            Ok(())
        }

        if format.is_switch() {
            for WeaponData { weapon_id, level, exp, max_ammo, ammo } in &mut weapon_data {
                *weapon_id = data.read_u32::<LE>()?;
                *max_ammo = data.read_u32::<LE>()?;
            }
        }

        load_weapons(data, &mut weapon_data, &format)?;
        if format.is_switch() {
            weapon_data_p2 = Some([WeaponData::default(); 8]);
            load_weapons(data, weapon_data_p2.as_mut().unwrap(), &format)?;
        }

        for item in &mut items {
            if format.is_switch() {
                *item = data.read_u16::<LE>()? as u32;
                let item_qty = data.read_u16::<LE>()?; // TODO: store items quantity
            } else {
                *item = data.read_u32::<LE>()?;
            }
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

        if !format.is_switch() {
            let _ = data.read_u32::<LE>().unwrap_or(0); // unused(?) CS+ (PC) space
        }

        let timestamp = data.read_u64::<LE>().unwrap_or(0);
        let difficulty = data.read_u8().unwrap_or(0);
        let eggfish_killed = false; // TODO

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
        let mut cur = std::io::Cursor::new(data);
        let magic = cur.read_u64::<BE>()?;
log::debug!("Len: {}", data.len());
        fn recognise_switch_version(cur: &mut std::io::Cursor<&[u8]>, data: &[u8]) -> GameResult<Version> {
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

    // TODO: compatibility warnings
}

#[derive(Clone, Copy, Deserialize, Eq, Serialize)]
pub struct ChallengeTime {
    pub timing_mode: TimingMode,
    pub ticks: usize,
}

impl ChallengeTime {
    pub fn new(timing_mode: TimingMode) -> Self {
        Self {
            timing_mode,
            ticks: 0
        }
    }

    pub fn convert_timing(&self, new_timing: TimingMode) -> usize {
        if self.timing_mode == TimingMode::FrameSynchronized || new_timing == TimingMode::FrameSynchronized {
            return self.ticks;
        }

        self.ticks / self.timing_mode.get_tps() * new_timing.get_tps()
    }

    pub fn with_timing(self, new_timing: TimingMode) -> Self {
        Self {
            timing_mode: new_timing,
            ticks: self.convert_timing(new_timing)
        }
    }

    pub fn load_time<R: io::Read>(&mut self, mut data: R, format: SaveFormat) -> GameResult {
        if format.is_csp() {
            self.timing_mode = TimingMode::_60Hz;
            self.ticks = data.read_u32::<LE>()? as usize;
            return Ok(());
        }

        // TODO: should we assumme that the best time in freeware format has a timing mode of 50 TPS?

        let mut ticks: [u32; 4] = [0; 4];

        for iter in 0..=3 {
            ticks[iter] = data.read_u32::<LE>()?;
        }

        let random = data.read_u32::<LE>()?;
        let random_list: [u8; 4] = random.to_le_bytes();

        for iter in 0..=3 {
            ticks[iter] = u32::from_le_bytes([
                ticks[iter].to_le_bytes()[0].wrapping_sub(random_list[iter]),
                ticks[iter].to_le_bytes()[1].wrapping_sub(random_list[iter]),
                ticks[iter].to_le_bytes()[2].wrapping_sub(random_list[iter]),
                ticks[iter].to_le_bytes()[3].wrapping_sub(random_list[iter] / 2),
            ]);
        }

        self.ticks = if ticks[0] == ticks[1] && ticks[0] == ticks[2] { ticks[0] as usize } else { 0 };

        Ok(())
    }

    pub fn write_time<W: io::Write>(&self, mut data: W, state: &SharedGameState, format: SaveFormat) -> GameResult {
        let time: u32 = self.ticks.try_into().unwrap_or(u32::MAX);
        if format.is_csp() {
            data.write_u32::<LE>(time)?;
            return Ok(());
        }

        let mut ticks: [u32; 4] = [time; 4];
        let mut random_list: [u8; 4] = [0; 4];

        for iter in 0..=3 {
            random_list[iter] = state.effect_rng.range(0..250) as u8 + iter as u8;

            ticks[iter] = u32::from_le_bytes([
                ticks[iter].to_le_bytes()[0].wrapping_add(random_list[iter]),
                ticks[iter].to_le_bytes()[1].wrapping_add(random_list[iter]),
                ticks[iter].to_le_bytes()[2].wrapping_add(random_list[iter]),
                ticks[iter].to_le_bytes()[3].wrapping_add(random_list[iter] / 2),
            ]);

            data.write_u32::<LE>(ticks[iter])?;
        }

        data.write_u32::<LE>(u32::from_le_bytes(random_list))?;

        Ok(())
    }
}

impl Ord for ChallengeTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.convert_timing(other.timing_mode).cmp(&other.ticks)
    }
}

impl PartialOrd for ChallengeTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ChallengeTime {
    fn eq(&self, other: &Self) -> bool {
        self.convert_timing(other.timing_mode) == self.ticks
    }
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

#[derive(Clone, Debug, PartialEq)]
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
        // Import/export all slots by default
        Self {
            slots: vec![],
            settings: true
        }
    }
}

// Generic container to store all possible info from original game saves
#[derive(Deserialize, Serialize)]
pub struct SaveContainer {
    pub version: usize,
    pub game_profiles: HashMap<usize, GameProfile>,
    pub csp_mods: HashMap<u8, CSPModProfile>, // save_set number -> saves & time
    // TODO: use this field instead of 290 records
    pub best_times: HashMap<u8, ChallengeTime>, // mod_id -> time info

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
            best_times: HashMap::new(),

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

        // TODO: import existing saves and challenge times

        Ok(Self::default())
    }

    fn load_from_buf(_ctx: &mut Context, _state: &mut SharedGameState, buf: &[u8]) -> GameResult<SaveContainer> {
        serde_json::from_slice::<SaveContainer>(buf)
            .map_err(|err| GameError::ResourceLoadError(format!("Failed to deserialize a generic save. {}", err.to_string())))
            .map(|container| container.upgrade())
    }

    pub fn save(&mut self, ctx: &mut Context, state: &mut SharedGameState, params: SaveParams) -> GameResult {
        self.write_save(ctx, state, SaveFormat::Generic, None, None, &params)?;
        self.write_save(ctx, state, state.settings.save_format, None, None, &params)?;
        self.patchset.clear();
        Ok(())
    }

    pub fn write_save(&mut self, ctx: &mut Context, state: &mut SharedGameState, format: SaveFormat, slot: Option<SaveSlot>, mut out_path: Option<PathBuf>, params: &SaveParams) -> GameResult {
        log::debug!("DEBUG WRITE SAVE");

        let save_path = Self::get_save_filename(format, slot.clone());

        match format {
            SaveFormat::Generic => {
                let mut file = user_create(ctx, &save_path)?;

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
                        let mut cur = std::io::Cursor::new(&mut buf);
                        profile.write_save(&mut cur, &format)?;

                        let mut filename = Self::get_save_filename(format, Some(SaveSlot::MainGame(*save_slot)));
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

                            let mut filename = Self::get_save_filename(format, Some(SaveSlot::CSPMod(*save_set, *slot)));
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

                for (mod_id, best_time) in &self.best_times {
                    let mut filename = ["/".to_string(), Self::get_rec_filename(state, *mod_id)].join("");
                    if let Some(path) = &mut out_path {
                        let os_filename = OsString::from_str(filename.split_off(1).as_str()).unwrap();
                        let _ = path.set_file_name(os_filename);

                        let file = File::create(path)?;
                        best_time.write_time(file, state, format)?;
                    } else {
                        let file = user_create(ctx, filename)?;
                        best_time.write_time(file, state, format)?;
                    }
                }

                for (patch_slot, patch_state) in self.patchset.iter() {
                    if *patch_state == SavePatch::Deleted {
                        let slot = Some(patch_slot.clone());
                        user_delete(ctx, Self::get_save_filename(format, slot))?;
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
                let unused = [0u8; 14];
                let jukebox = [0xff as u8; 6]; // TODO: implement storing ids of played songs for jukebox
                let mut eggfish_killed = [0u8; 3];

                let mut buf = Vec::new();
                let mut cur = std::io::Cursor::new(&mut buf);

                // TODO
                // In CS+, only slots up to the last non-empty one are written to the save file.
                // E.g., if the user saved only the profile in slot 3, then profiles from slots 1, 2 and 3 will be written to the file.
                // All other profiles will be filled with zeros.
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

                if let SaveFormat::Switch(version) = format {
                    cur.write(&jukebox)?;
                    cur.write_u8(0)?; // Unlock notifications

                    cur.write_u8(0)?; // Shared health bar
                    cur.write_u24::<LE>(0)?; // Something
                } else {
                    let zeros = [0u8; 7];
                    cur.write(&zeros)?;
                }

                // Challenge best times
                // let best_times: u8 = if format.is_switch() { 29 } else { 26 };
                let best_times = match format {
                    SaveFormat::Switch(SWITCH_VER_1_2) => 29,
                    SaveFormat::Switch(SWITCH_VER_1_3) => {
                        // In v1.3, challenge times are shifted by 4 bytes
                        cur.write_u32::<LE>(0)?;
                        28
                    },
                    _ => 26
                };

                let default_best_time = ChallengeTime::new(state.settings.timing_mode);
                for mod_id in 0..=best_times {
                    let mod_time = self.best_times.get(&mod_id).unwrap_or(&default_best_time).clone();

                    let csp_time = mod_time.with_timing(TimingMode::_60Hz);
                    csp_time.write_time(&mut cur, state, format)?;
                }

                if format.is_switch() {
                    let challenge_unlocks = ChallengeUnlocks::dump(&state.mod_requirements);
                    cur.write_u8(challenge_unlocks)?;

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
                    Some(SaveSlot::Mod(mod_id, save_slot)) => unimplemented!(),
                    _ => "/Profile.dat".to_owned()
                }
            }
        }
    }

    pub fn get_rec_filename(state: &SharedGameState, mod_id: u8) -> String {
        if mod_id == 0 {
            return "290.rec".to_owned();
        }

        // TODO: maybe we should use mod ids instead of names? Because unloaded (removed from the mods.txt list) mods overwrite the 290.rec currently
        let name = state
            .mod_list
            .get_info_from_id(format!("cspmod_{mod_id}"))
            .and_then(|mod_info| Some(mod_info.get_rec_filename()))
            .unwrap_or("290".to_owned());

        [name, ".rec".to_string()].join("")
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
            let _ = self.csp_mods.entry(*b_set)
                .and_modify(|csp_mod| csp_mod.profiles.extend(b_csp_mod.profiles.iter()))
                .or_insert(b_csp_mod.clone());
        }

        for (b_mod_id, b_best_time) in &b.best_times {
            let _ = self.best_times.entry(*b_mod_id)
                .and_modify(|time| { *time = time.clone().min(*b_best_time) })
                .or_insert(*b_best_time);
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

        // Again, working with data in memory is much faster than reading every field from drive
        let data = std::fs::read(path)?;

        let format = format.unwrap_or(SaveFormat::recognise(data.as_slice())?);

        log::trace!("Import format: {:?}.", format);
        log::trace!("Import params: {:?}.", params);
        log::trace!("Import path: {:?}.", save_path);

        match format {
            SaveFormat::Generic => {
                *self = Self::load_from_buf(ctx, state, data.as_slice())?;
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

                let beat_hell = (cur.read_u8()? == 1);
                state.mod_requirements.beat_hell = state.mod_requirements.beat_hell || beat_hell;

                if format.is_switch() {
                    // TODO: jukebox
                    let mut jukebox = [0u8; 6];
                    cur.read_exact(&mut jukebox)?;

                    // TODO
                    let unlock_notifications = cur.read_u8()?;
                    let shared_healthbar = cur.read_u8()?;

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
                    let mut best_time = ChallengeTime::new(TimingMode::_60Hz);
                    best_time.load_time(&mut cur, format)?;

                    if best_time.ticks != 0 {
                        container.best_times.insert(mod_id, best_time);
                    }
                }

                if format == SaveFormat::Plus {
                    for save_slot in 1..=3 as usize {
                        let eggfish_killed = cur.read_u8()?;
                        if let Some(profile) = self.game_profiles.get_mut(&save_slot) {
                            profile.eggfish_killed = (eggfish_killed != 0);
                        }
                    }

                    let mut something = [0u8; 61];
                    cur.read_exact(&mut something)?;

                    let mut unused = [0u8; 32];
                    cur.read_exact(&mut unused)?;
                } else {
                    let challenge_unlocks = cur.read_u8()?;
                    ChallengeUnlocks::load(ctx, &mut state.mod_requirements, challenge_unlocks)?;

                    let mut something: [u8; 119] = [0u8; 119];
                    cur.read_exact(&mut something)?;

                    // TODO
                    let skins_p2 = cur.read_u16::<LE>()?;

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

struct ChallengeUnlocks;

// Hardcoded de-/encoding of CS+ challenge unlocks
impl ChallengeUnlocks {
    pub fn dump(mod_requirements: &ModRequirements) -> u8 {
        let mut challenge_unlocks: u8 = 1;

        // Mods with RG requirement
        // 0x2 - Sanctuary Time Attack.
        // 0x4 - Boss Attack.
        // 0x10 - Wind Fortress.
        if mod_requirements.beat_hell {
            challenge_unlocks |= 0x2 | 0x4 | 0x10;
        }

        // Mods with RA requirement

        if mod_requirements.has_weapon(12) {
            // Nemesis Challenge. RA12
            challenge_unlocks |= 0x20;
        }

        if mod_requirements.has_weapon(3) {
            // Sand Pit. RA3
            challenge_unlocks |= 0x80;
        }

        // Mods with RI requirement

        // Mods with RI35 requirement.
        // 0x8 - Curly Story.
        // 0x40 - Machine Gun Challenge.
        if mod_requirements.has_item(35) {
           challenge_unlocks |= 0x8 | 0x40;
        }

        challenge_unlocks
    }

    pub fn load(ctx: &Context, mod_requirements: &mut ModRequirements, raw: u8) -> GameResult {
        if raw & (0x2 | 0x4 | 0x10) != 0 {
            mod_requirements.beat_hell = true;
        }

        if raw & 0x20 != 0 {
            mod_requirements.append_weapon(ctx, 12)?;
        }

        if raw & 0x80 != 0 {
            mod_requirements.append_weapon(ctx, 3)?;
        }

        if raw & (0x8 | 0x40) != 0 {
            mod_requirements.append_item(ctx, 35)?;
        }

        Ok(())
    }
}