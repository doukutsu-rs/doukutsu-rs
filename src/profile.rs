use std::io;

use byteorder::{ReadBytesExt, WriteBytesExt, BE, LE};
use num_traits::{clamp, FromPrimitive};

use crate::common::{Direction, FadeState};
use crate::framework::context::Context;
use crate::framework::error::GameError::ResourceLoadError;
use crate::framework::error::GameResult;
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

        let _ = state.sound_manager.play_song(self.current_song as usize, &state.constants, &state.settings, ctx);

        game_scene.inventory_player1.current_weapon = self.current_weapon as u16;
        game_scene.inventory_player1.current_item = self.current_item as u16;
        for weapon in self.weapon_data.iter() {
            if weapon.weapon_id == 0 {
                continue;
            }
            let weapon_type: Option<WeaponType> = FromPrimitive::from_u8(weapon.weapon_id as u8);

            if let Some(wtype) = weapon_type {
                let w = game_scene.inventory_player1.add_weapon_data(
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

        for item in self.items.iter().copied() {
            if item == 0 {
                break;
            }

            game_scene.inventory_player1.add_item(item as u16);
        }

        for slot in self.teleporter_slots.iter() {
            if slot.event_num == 0 {
                break;
            }

            state.teleporter_slots.push((slot.index as u16, slot.event_num as u16));
        }

        for (idx, &flags) in self.flags.iter().enumerate() {
            if flags & 0b00000001 != 0 {
                state.game_flags.set(idx * 8, true);
            }
            if flags & 0b00000010 != 0 {
                state.game_flags.set(idx * 8 + 1, true);
            }
            if flags & 0b00000100 != 0 {
                state.game_flags.set(idx * 8 + 2, true);
            }
            if flags & 0b00001000 != 0 {
                state.game_flags.set(idx * 8 + 3, true);
            }
            if flags & 0b00010000 != 0 {
                state.game_flags.set(idx * 8 + 4, true);
            }
            if flags & 0b00100000 != 0 {
                state.game_flags.set(idx * 8 + 5, true);
            }
            if flags & 0b01000000 != 0 {
                state.game_flags.set(idx * 8 + 6, true);
            }
            if flags & 0b10000000 != 0 {
                state.game_flags.set(idx * 8 + 7, true);
            }
        }

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
        game_scene.inventory_player2 = game_scene.inventory_player1.clone();

        game_scene.player1.cond.0 = 0x80;
    }

    pub fn dump(state: &mut SharedGameState, game_scene: &mut GameScene) -> GameProfile {
        let current_map = game_scene.stage_id as u32;
        let current_song = state.sound_manager.current_song() as u32;
        let pos_x = game_scene.player1.x as i32;
        let pos_y = game_scene.player1.y as i32;
        let direction = game_scene.player1.direction;
        let max_life = game_scene.player1.max_life;
        let stars = game_scene.player1.stars as u16;
        let life = game_scene.player1.life;
        let current_weapon = game_scene.inventory_player1.current_weapon as u32;
        let current_item = game_scene.inventory_player1.current_item as u32;
        let equipment = game_scene.player1.equip.0 as u32;
        let control_mode = game_scene.player1.control_mode as u32;
        let counter = 0; // TODO
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

        for (idx, weap) in weapon_data.iter_mut().enumerate() {
            if let Some(weapon) = game_scene.inventory_player1.get_weapon(idx) {
                weap.weapon_id = weapon.wtype as u32;
                weap.level = weapon.level as u32;
                weap.exp = weapon.experience as u32;
                weap.max_ammo = weapon.max_ammo as u32;
                weap.ammo = weapon.ammo as u32;
            }
        }

        for (idx, item) in items.iter_mut().enumerate() {
            if let Some(sitem) = game_scene.inventory_player1.get_item_idx(idx) {
                *item = sitem.0 as u32;
            }
        }

        for (idx, slot) in teleporter_slots.iter_mut().enumerate() {
            if let Some(&(index, event_num)) = state.teleporter_slots.get(idx) {
                slot.index = index as u32;
                slot.event_num = event_num as u32;
            }
        }

        let mut bidx = 0;
        let mut flags = [0u8; 1000];
        for bits in state.game_flags.as_raw_slice() {
            let bytes = bits.to_le_bytes();
            for b in bytes.iter() {
                if let Some(out) = flags.get_mut(bidx) {
                    *out = *b;
                }
                bidx += 1;
            }
        }

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
            items,
            teleporter_slots,
            flags,
        }
    }

    pub fn write_save<W: io::Write>(&self, mut data: W) -> GameResult {
        data.write_u64::<BE>(0x446f303431323230)?;

        data.write_u32::<LE>(self.current_map)?;
        data.write_u32::<LE>(self.current_song)?;
        data.write_i32::<LE>(self.pos_x)?;
        data.write_i32::<LE>(self.pos_y)?;
        data.write_u32::<LE>(self.direction as u32)?;
        data.write_u16::<LE>(self.max_life)?;
        data.write_u16::<LE>(self.stars)?;
        data.write_u16::<LE>(self.life)?;
        data.write_u16::<LE>(0)?;
        data.write_u32::<LE>(self.current_weapon)?;
        data.write_u32::<LE>(self.current_item)?;
        data.write_u32::<LE>(self.equipment)?;
        data.write_u32::<LE>(self.control_mode)?;
        data.write_u32::<LE>(self.counter)?;

        for weapon in self.weapon_data.iter() {
            data.write_u32::<LE>(weapon.weapon_id)?;
            data.write_u32::<LE>(weapon.level)?;
            data.write_u32::<LE>(weapon.exp)?;
            data.write_u32::<LE>(weapon.max_ammo)?;
            data.write_u32::<LE>(weapon.ammo)?;
        }

        for item in self.items.iter().copied() {
            data.write_u32::<LE>(item)?;
        }

        for slot in self.teleporter_slots.iter() {
            data.write_u32::<LE>(slot.index)?;
            data.write_u32::<LE>(slot.event_num)?;
        }

        let something = [0u8; 0x80];
        data.write(&something)?;

        data.write_u32::<BE>(0x464c4147)?;
        data.write(&self.flags)?;

        Ok(())
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
        let control_mode = data.read_u32::<LE>()?;
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
            control_mode,
            counter,
            weapon_data,
            items,
            teleporter_slots,
            flags,
        })
    }
}
