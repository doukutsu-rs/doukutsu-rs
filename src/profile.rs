use std::io;

use byteorder::{BE, LE, ReadBytesExt};
use num_traits::{clamp, FromPrimitive};

use crate::common::{Direction, FadeState};
use crate::ggez::{Context, GameResult};
use crate::ggez::GameError::ResourceLoadError;
use crate::player::ControlMode;
use crate::scene::game_scene::GameScene;
use crate::shared_game_state::SharedGameState;
use crate::str;
use crate::weapon::{WeaponLevel, WeaponType};

pub struct WeaponData {
    pub weapon_id: u32,
    pub level: u32,
    pub exp: u32,
    pub max_ammo: u32,
    pub ammo: u32,
}

pub struct TeleporterSlotData {
    pub index: u32,
    pub event_num: u32,
}

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
    pub items: [u32; 32],
    pub teleporter_slots: [TeleporterSlotData; 8],
    pub flags: [u8; 1000],
}

impl GameProfile {
    pub fn apply(&self, state: &mut SharedGameState, game_scene: &mut GameScene, ctx: &mut Context) {
        state.fade_state = FadeState::Visible;
        state.control_flags.set_tick_world(true);
        state.control_flags.set_control_enabled(true);

        state.sound_manager.play_song(self.current_song as usize, &state.constants, ctx);

        game_scene.inventory.current_weapon = self.current_weapon as u16;
        game_scene.inventory.current_item = self.current_item as u16;
        for weapon in self.weapon_data.iter() {
            if weapon.weapon_id == 0 { continue; }
            let weapon_type: Option<WeaponType> = FromPrimitive::from_u8(weapon.weapon_id as u8);

            if let Some(wtype) = weapon_type {
                let w = game_scene.inventory.add_weapon(wtype, weapon.max_ammo as u16);
                w.ammo = weapon.ammo as u16;
                w.level = match weapon.level {
                    2 => { WeaponLevel::Level2 }
                    3 => { WeaponLevel::Level3 }
                    _ => { WeaponLevel::Level1 }
                };
                w.experience = weapon.exp as u16;
            }
        }

        for item in self.items.iter().copied() {
            if item == 0 { break; }

            game_scene.inventory.add_item(item as u16);
        }

        for slot in self.teleporter_slots.iter() {
            if slot.event_num == 0 { break; }

            state.teleporter_slots.push((slot.index as u16, slot.event_num as u16));
        }

        for (idx, &flags) in self.flags.iter().enumerate() {
            if flags & 0b00000001 != 0 { state.game_flags.set(idx * 8, true); }
            if flags & 0b00000010 != 0 { state.game_flags.set(idx * 8 + 1, true); }
            if flags & 0b00000100 != 0 { state.game_flags.set(idx * 8 + 2, true); }
            if flags & 0b00001000 != 0 { state.game_flags.set(idx * 8 + 3, true); }
            if flags & 0b00010000 != 0 { state.game_flags.set(idx * 8 + 4, true); }
            if flags & 0b00100000 != 0 { state.game_flags.set(idx * 8 + 5, true); }
            if flags & 0b01000000 != 0 { state.game_flags.set(idx * 8 + 6, true); }
            if flags & 0b10000000 != 0 { state.game_flags.set(idx * 8 + 7, true); }
        }

        game_scene.player.equip.0 = self.equipment as u16;
        game_scene.player.cond.0 = 0x80;

        game_scene.player.x = self.pos_x as isize;
        game_scene.player.y = self.pos_y as isize;

        game_scene.player.control_mode = if self.control_mode == 1 { ControlMode::IronHead } else { ControlMode::Normal };
        game_scene.player.direction = self.direction;
        game_scene.player.life = self.life;
        game_scene.player.max_life = self.max_life;
        game_scene.player.stars = clamp(self.stars, 0, 3) as u8;
    }

    pub fn load_from_save<R: io::Read>(mut data: R) -> GameResult<GameProfile> {
        // Do041220
        if data.read_u64::<BE>()? != 0x446f303431323230 {
            return Err(ResourceLoadError(str!("Invalid magic")));
        }

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
        let move_mode = data.read_u32::<LE>()?;
        let counter = data.read_u32::<LE>()?;
        let mut weapon_data = [
            WeaponData { weapon_id: 0, level: 0, exp: 0, max_ammo: 0, ammo: 0 },
            WeaponData { weapon_id: 0, level: 0, exp: 0, max_ammo: 0, ammo: 0 },
            WeaponData { weapon_id: 0, level: 0, exp: 0, max_ammo: 0, ammo: 0 },
            WeaponData { weapon_id: 0, level: 0, exp: 0, max_ammo: 0, ammo: 0 },
            WeaponData { weapon_id: 0, level: 0, exp: 0, max_ammo: 0, ammo: 0 },
            WeaponData { weapon_id: 0, level: 0, exp: 0, max_ammo: 0, ammo: 0 },
            WeaponData { weapon_id: 0, level: 0, exp: 0, max_ammo: 0, ammo: 0 },
            WeaponData { weapon_id: 0, level: 0, exp: 0, max_ammo: 0, ammo: 0 },
        ];
        let mut items = [0u32; 32];
        let mut teleporter_slots = [
            TeleporterSlotData { index: 0, event_num: 0 },
            TeleporterSlotData { index: 0, event_num: 0 },
            TeleporterSlotData { index: 0, event_num: 0 },
            TeleporterSlotData { index: 0, event_num: 0 },
            TeleporterSlotData { index: 0, event_num: 0 },
            TeleporterSlotData { index: 0, event_num: 0 },
            TeleporterSlotData { index: 0, event_num: 0 },
            TeleporterSlotData { index: 0, event_num: 0 },
        ];

        for weap in weapon_data.iter_mut() {
            weap.weapon_id = data.read_u32::<LE>()?;
            weap.level = data.read_u32::<LE>()?;
            weap.exp = data.read_u32::<LE>()?;
            weap.max_ammo = data.read_u32::<LE>()?;
            weap.ammo = data.read_u32::<LE>()?;
        }

        for item in items.iter_mut() {
            *item = data.read_u32::<LE>()?;
        }

        for slot in teleporter_slots.iter_mut() {
            slot.index = data.read_u32::<LE>()?;
            slot.event_num = data.read_u32::<LE>()?;
        }

        let mut something = [0u8; 0x80];
        data.read_exact(&mut something)?;

        if data.read_u32::<BE>()? != 0x464c4147 {
            return Err(ResourceLoadError(str!("Invalid FLAG signature")));
        }

        let mut flags = [0u8; 1000];
        data.read_exact(&mut flags)?;

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
            control_mode: move_mode,
            counter,
            weapon_data,
            items,
            teleporter_slots,
            flags,
        })
    }
}
