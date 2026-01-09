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

use crate::bitfield;
use crate::common::{get_timestamp, Direction, FadeState, Version};
use crate::framework::context::Context;
use crate::framework::error::GameError::{self, ResourceLoadError};
use crate::framework::error::GameResult;
use crate::framework::filesystem::{self, user_create, user_delete, user_exists, user_open};
use crate::game::player::{ControlMode, TargetPlayer};
use crate::game::save_container::SaveFormat;
use crate::game::shared_game_state::{GameDifficulty, PlayerCount, SharedGameState, TimingMode};
use crate::game::weapon::{WeaponLevel, WeaponType};
use crate::game::inventory::{Inventory, Item};
use crate::mod_list::ModList;
use crate::mod_requirements::{self, ModRequirements};
use crate::scene::game_scene::GameScene;
use crate::util::rng::RNG;

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
    pub items: [Item; 32],
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
            items: [Item::default(); 32],
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
            let _ = state.mod_requirements.append_item(ctx, item.0);
            if item.0 == 0 {
                break;
            }

            // TODO: original save formats can't store inventory for player 2, but we can, so should we save it,
            // even if it's equal to the inventroy of player 1?
            game_scene.inventory_player1.add_item_amount(item.0, item.1);
            game_scene.inventory_player2.add_item_amount(item.0, item.1);
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
        let mut items = [Item::default(); 32];
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
                *item = *sitem;
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
        if self.is_empty() && format.is_csp() {
            let dummy = vec![0u8; format.profile_size()];
            data.write(&dummy)?;

            return Ok(());
        }

        data.write_u64::<BE>(SIG_Do041220)?;

        data.write_u32::<LE>(self.current_map)?;
        data.write_u32::<LE>(self.current_song)?;
        data.write_i32::<LE>(self.pos_x)?;
        data.write_i32::<LE>(self.pos_y)?;
        data.write_u32::<LE>(self.direction as u32)?;
        data.write_u16::<LE>(self.max_life)?;

        if format.is_switch() {
            let max_life_p2 = if self.max_life_p2 != 0 { self.max_life_p2 } else { self.max_life };
            data.write_u16::<LE>(max_life_p2)?;
        }

        data.write_u16::<LE>(self.stars)?;
        data.write_u16::<LE>(self.life)?;

        if format.is_switch() {
            let life_p2 = if self.life_p2 != 0 { self.life_p2 } else { self.life };
            data.write_u16::<LE>(life_p2)?;
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
            let amount = if !format.is_switch() { 0 } else { item.1 };

            data.write_u16::<LE>(item.0)?;
            data.write_u16::<LE>(amount)?;
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
            let profile_size = format.profile_size();

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
        let max_life_p2 = if format.is_switch() { data.read_u16::<LE>()? } else { max_life };
        let stars = data.read_u16::<LE>()?;
        let life = data.read_u16::<LE>()?;
        let life_p2 = if format.is_switch() { data.read_u16::<LE>()? } else { life };
        let _ = data.read_u16::<LE>()?; // ???
        let current_weapon = data.read_u32::<LE>()?;
        let current_item = data.read_u32::<LE>()?;
        let equipment = data.read_u32::<LE>()?;
        let control_mode = data.read_u32::<LE>()?;
        let counter = data.read_u32::<LE>()?;
        let mut weapon_data = [WeaponData::default(); 8];
        let mut weapon_data_p2 = None;
        let mut items = [Item::default(); 32];
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
            for weapon in &mut weapon_data {
                weapon.weapon_id = data.read_u32::<LE>()?;
                weapon.max_ammo = data.read_u32::<LE>()?;
            }
        }

        load_weapons(data, &mut weapon_data, &format)?;
        if format.is_switch() {
            weapon_data_p2 = Some([WeaponData::default(); 8]);
            load_weapons(data, weapon_data_p2.as_mut().unwrap(), &format)?;
        }

        for item in &mut items {
            item.0 = data.read_u16::<LE>()?;
            item.1 = data.read_u16::<LE>()?;

            if !format.is_switch() {
                item.1 += 1;
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
        self.timestamp == 0 && self.current_map == 0
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Serialize)]
pub struct ChallengeTime {
    pub timing_mode: TimingMode,
    pub ticks: usize,
}

impl ChallengeTime {
    pub fn new() -> Self {
        Self {
            timing_mode: TimingMode::FrameSynchronized,
            ticks: 0
        }
    }

    pub fn load(ctx: &Context, state: &SharedGameState, filename: String) -> GameResult<Self> {
        let mut file = filesystem::user_open(ctx, filename)?;

        let mut time = ChallengeTime::new();
        time.load_time(&mut file, SaveFormat::Freeware)?;

        Ok(time.with_timing(state.settings.timing_mode))
    }

    pub fn write(&self, ctx: &Context, state: &SharedGameState, filename: String) -> GameResult {
        let mut file = filesystem::user_create(ctx, filename)?;

        let mut time = ChallengeTime::new();
        time.load_time(&mut file, SaveFormat::Freeware)?;

        time
            .with_timing(state.settings.timing_mode)
            .write_time(file, state, SaveFormat::Freeware)
    }


    pub fn convert_timing(&self, new_timing: TimingMode) -> usize {
        if self.timing_mode == TimingMode::FrameSynchronized || new_timing == TimingMode::FrameSynchronized || self.timing_mode == new_timing {
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

    pub fn load_time<R: io::Read>(&mut self, data: &mut R, format: SaveFormat) -> GameResult {
        if format.is_csp() {
            self.timing_mode = TimingMode::_60Hz;
            self.ticks = data.read_u32::<LE>()? as usize;
            return Ok(());
        }

        let mut ticks: [u32; 4] = [0; 4];

        for iter in 0..=3 {
            ticks[iter] = data.read_u32::<LE>()?;
        }

        let random = data.read_u32::<LE>()?;
        let tps = data.read_u16::<LE>().unwrap_or(0);
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
        // TODO: should we assumme that the best time in freeware format has a timing mode of 50 TPS?
        self.timing_mode = TimingMode::from_tps(tps as usize);

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
        data.write_u16::<LE>(self.timing_mode.get_tps() as u16)?;

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
