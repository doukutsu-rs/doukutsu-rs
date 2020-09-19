use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::io;
use std::io::Cursor;

use bitvec::vec::BitVec;
use byteorder::{LE, ReadBytesExt};
use itertools::Itertools;

use crate::{bitfield, SharedGameState};
use crate::caret::CaretType;
use crate::common::{Condition, Rect};
use crate::common::Direction;
use crate::common::Flag;
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::ggez::{Context, GameResult};
use crate::map::NPCData;
use crate::physics::PhysicalEntity;
use crate::player::Player;
use crate::str;

pub mod characters;
pub mod egg_corridor;
pub mod first_cave;
pub mod mimiga_village;
pub mod misc;
pub mod pickups;

bitfield! {
  #[derive(Clone, Copy)]
  pub struct NPCFlag(u16);
  impl Debug;

  pub solid_soft, set_solid_soft: 0;
  pub ignore_tile_44, set_ignore_tile_44: 1;
  pub invulnerable, set_invulnerable: 2;
  pub ignore_solidity, set_ignore_solidity: 3;
  pub bouncy, set_bouncy: 4;
  pub shootable, set_shootable: 5;
  pub solid_hard, set_solid_hard: 6;
  pub rear_and_top_not_hurt, set_rear_and_top_not_hurt: 7;
  pub event_when_touched, set_event_when_touched: 8;
  pub event_when_killed, set_event_when_killed: 9;
  pub flag_x400, set_flag_x400: 10;
  pub appear_when_flag_set, set_appear_when_flag_set: 11;
  pub spawn_facing_right, set_spawn_facing_right: 12;
  pub interactable, set_interactable: 13;
  pub hide_unless_flag_set, set_hide_unless_flag_set: 14;
  pub show_damage, set_show_damage: 15;
}

#[derive(Debug, Clone, Copy)]
pub struct NPC {
    pub id: u16,
    pub npc_type: u16,
    pub x: isize,
    pub y: isize,
    pub vel_x: isize,
    pub vel_y: isize,
    pub target_x: isize,
    pub target_y: isize,
    pub exp: u16,
    pub size: u8,
    pub shock: u16,
    pub life: u16,
    pub damage: u16,
    pub cond: Condition,
    pub flags: Flag,
    pub npc_flags: NPCFlag,
    pub direction: Direction,
    pub display_bounds: Rect<usize>,
    pub hit_bounds: Rect<usize>,
    pub hurt_sound: u8,
    pub action_num: u16,
    pub anim_num: u16,
    pub flag_num: u16,
    pub event_num: u16,
    pub action_counter: u16,
    pub action_counter2: u16,
    pub anim_counter: u16,
    pub anim_rect: Rect<usize>,
}

impl NPC {
    pub fn get_start_index(&self) -> u16 {
        if self.npc_type == 1 || self.npc_type == 4 {
            0x100
        } else {
            0
        }
    }
}

impl GameEntity<&mut Player> for NPC {
    fn tick(&mut self, state: &mut SharedGameState, player: &mut Player) -> GameResult {
        match self.npc_type {
            0 => { self.tick_n000_null() }
            1 => { self.tick_n001_experience(state) }
            2 => { self.tick_n002_behemoth(state) }
            3 => { self.tick_n003_dead_enemy() }
            4 => { self.tick_n004_smoke(state) }
            5 => { self.tick_n005_green_critter(state, player) }
            6 => { self.tick_n006_green_beetle(state) }
            7 => { self.tick_n007_basil(state, player) }
            8 => { self.tick_n008_blue_beetle(state, player) }
            15 => { self.tick_n015_chest_closed(state) }
            16 => { self.tick_n016_save_point(state) }
            17 => { self.tick_n017_health_refill(state) }
            18 => { self.tick_n018_door(state) }
            20 => { self.tick_n020_computer(state) }
            21 => { self.tick_n021_chest_open(state) }
            22 => { self.tick_n022_teleporter(state) }
            27 => { self.tick_n027_death_trap(state) }
            30 => { self.tick_n030_gunsmith(state) }
            32 => { self.tick_n032_life_capsule(state) }
            34 => { self.tick_n034_bed(state) }
            37 => { self.tick_n037_sign(state) }
            38 => { self.tick_n038_fireplace(state) }
            39 => { self.tick_n039_save_sign(state) }
            41 => { self.tick_n041_busted_door(state) }
            43 => { self.tick_n043_chalkboard(state) }
            46 => { self.tick_n046_hv_trigger(state, player) }
            52 => { self.tick_n052_sitting_blue_robot(state) }
            55 => { self.tick_n055_kazuma(state) }
            59 => { self.tick_n059_eye_door(state, player) }
            60 => { self.tick_n060_toroko(state, player) }
            61 => { self.tick_n061_king(state) }
            62 => { self.tick_n062_kazuma_computer(state) }
            63 => { self.tick_n063_toroko_stick(state) }
            64 => { self.tick_n064_first_cave_critter(state, player) }
            65 => { self.tick_n065_first_cave_bat(state, player) }
            70 => { self.tick_n070_sparkle(state) }
            71 => { self.tick_n071_chinfish(state) }
            72 => { self.tick_n072_sprinkler(state) }
            74 => { self.tick_n074_jack(state) }
            75 => { self.tick_n075_kanpachi(state, player) }
            77 => { self.tick_n077_yamashita(state) }
            78 => { self.tick_n078_pot(state) }
            79 => { self.tick_n079_mahin(state, player) }
            211 => { self.tick_n211_small_spikes(state) }
            _ => { Ok(()) }
        }?;

        if self.shock > 0 {
            self.shock -= 1;
        }

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult {
        if !self.cond.alive() || self.cond.hidden() {
            return Ok(());
        }

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, state.npc_table.get_texture_name(self.npc_type))?;

        let off_x = if self.direction == Direction::Left { self.display_bounds.left } else { self.display_bounds.right } as isize;
        let shock = if self.shock > 0 {
            (2 * ((self.shock as isize / 2) % 2) - 1) as f32
        } else { 0.0 };

        batch.add_rect(
            (((self.x - off_x) / 0x200) - (frame.x / 0x200)) as f32 + shock,
            (((self.y - self.display_bounds.top as isize) / 0x200) - (frame.y / 0x200)) as f32,
            &self.anim_rect,
        );
        batch.draw(ctx)?;

        Ok(())
    }
}

impl PhysicalEntity for NPC {
    #[inline(always)]
    fn x(&self) -> isize {
        self.x
    }

    #[inline(always)]
    fn y(&self) -> isize {
        self.y
    }

    #[inline(always)]
    fn vel_x(&self) -> isize {
        self.vel_x
    }

    #[inline(always)]
    fn vel_y(&self) -> isize {
        self.vel_y
    }

    #[inline(always)]
    fn size(&self) -> u8 {
        self.size
    }

    #[inline(always)]
    fn hit_bounds(&self) -> &Rect<usize> {
        &self.hit_bounds
    }

    #[inline(always)]
    fn set_x(&mut self, x: isize) {
        self.x = x;
    }

    #[inline(always)]
    fn set_y(&mut self, y: isize) {
        self.y = y;
    }

    #[inline(always)]
    fn set_vel_x(&mut self, vel_x: isize) {
        self.vel_x = vel_x;
    }

    #[inline(always)]
    fn set_vel_y(&mut self, vel_y: isize) {
        self.vel_y = vel_y;
    }

    #[inline(always)]
    fn cond(&mut self) -> &mut Condition {
        &mut self.cond
    }

    #[inline(always)]
    fn flags(&mut self) -> &mut Flag {
        &mut self.flags
    }

    #[inline(always)]
    fn direction(&self) -> Direction {
        self.direction
    }

    #[inline(always)]
    fn is_player(&self) -> bool {
        false
    }

    #[inline(always)]
    fn ignore_tile_44(&self) -> bool {
        self.npc_flags.ignore_tile_44()
    }
}

pub struct NPCMap {
    /// A sorted pool of used IDs to used to iterate over NPCs in order, as original game does.
    pub npc_ids: BTreeSet<u16>,
    /// Do not iterate over this directly outside render pipeline.
    pub npcs: HashMap<u16, RefCell<NPC>>,
}

impl NPCMap {
    #[allow(clippy::new_without_default)]
    pub fn new() -> NPCMap {
        NPCMap {
            npc_ids: BTreeSet::new(),
            npcs: HashMap::with_capacity(256),
        }
    }

    pub fn clear(&mut self) {
        self.npc_ids.clear();
        self.npcs.clear();
    }

    pub fn create_npc_from_data(&mut self, table: &NPCTable, data: &NPCData) -> &mut NPC {
        let display_bounds = table.get_display_bounds(data.npc_type);
        let hit_bounds = table.get_hit_bounds(data.npc_type);
        let (size, life, damage, flags, exp, hurt_sound) = match table.get_entry(data.npc_type) {
            Some(entry) => { (entry.size, entry.life, entry.damage as u16, entry.npc_flags, entry.experience as u16, entry.hurt_sound) }
            None => { (1, 0, 0, NPCFlag(0), 0, 0) }
        };
        let npc_flags = NPCFlag(data.flags | flags.0);

        let npc = NPC {
            id: data.id,
            npc_type: data.npc_type,
            x: data.x as isize * 16 * 0x200,
            y: data.y as isize * 16 * 0x200,
            vel_x: 0,
            vel_y: 0,
            target_x: 0,
            target_y: 0,
            action_num: 0,
            anim_num: 0,
            flag_num: data.flag_num,
            event_num: data.event_num,
            shock: 0,
            exp,
            size,
            life,
            damage,
            cond: Condition(0x00),
            flags: Flag(0),
            direction: if npc_flags.spawn_facing_right() { Direction::Right } else { Direction::Left },
            npc_flags,
            display_bounds,
            hit_bounds,
            hurt_sound,
            action_counter: 0,
            action_counter2: 0,
            anim_counter: 0,
            anim_rect: Rect::new(0, 0, 0, 0),
        };

        self.npc_ids.insert(data.id);
        self.npcs.insert(data.id, RefCell::new(npc));

        self.npcs.get_mut(&data.id).unwrap().get_mut()
    }

    pub fn create_npc(npc_type: u16, table: &NPCTable) -> NPC {
        let display_bounds = table.get_display_bounds(npc_type);
        let hit_bounds = table.get_hit_bounds(npc_type);
        let (size, life, damage, flags, exp, hurt_sound) = match table.get_entry(npc_type) {
            Some(entry) => { (entry.size, entry.life, entry.damage as u16, entry.npc_flags, entry.experience as u16, entry.hurt_sound) }
            None => { (1, 0, 0, NPCFlag(0), 0, 0) }
        };
        let npc_flags = NPCFlag(flags.0);

        NPC {
            id: 0,
            npc_type,
            x: 0,
            y: 0,
            vel_x: 0,
            vel_y: 0,
            target_x: 0,
            target_y: 0,
            action_num: 0,
            anim_num: 0,
            flag_num: 0,
            event_num: 0,
            shock: 0,
            exp,
            size,
            life,
            damage,
            cond: Condition(0x00),
            flags: Flag(0),
            direction: if npc_flags.spawn_facing_right() { Direction::Right } else { Direction::Left },
            npc_flags,
            display_bounds,
            hit_bounds,
            hurt_sound,
            action_counter: 0,
            action_counter2: 0,
            anim_counter: 0,
            anim_rect: Rect::new(0, 0, 0, 0),
        }
    }

    pub fn garbage_collect(&mut self) {
        let dead_npcs = self.npcs.iter().filter_map(|(&id, npc_cell)| {
            if !npc_cell.borrow().cond.alive() {
                Some(id)
            } else {
                None
            }
        }).collect_vec();

        for npc_id in dead_npcs.iter() {
            self.npc_ids.remove(npc_id);
            self.npcs.remove(npc_id);
        }
    }

    pub fn remove_by_event(&mut self, event_num: u16, game_flags: &mut BitVec) {
        for npc_cell in self.npcs.values_mut() {
            let mut npc = npc_cell.borrow_mut();

            if npc.event_num == event_num {
                npc.cond.set_alive(false);
                game_flags.set(npc.flag_num as usize, true);
            }
        }
    }

    pub fn allocate_id(&mut self, start: u16) -> u16 {
        for i in start..(u16::MAX) {
            if !self.npc_ids.contains(&i) {
                return i;
            }
        }

        unreachable!()
    }

    pub fn create_death_effect(&self, x: isize, y: isize, radius: usize, count: usize, state: &mut SharedGameState) {
        let radius = radius as i32 / 0x200;

        for _ in 0..count {
            let off_x = state.game_rng.range(-radius..radius) as isize * 0x200;
            let off_y = state.game_rng.range(-radius..radius) as isize * 0x200;

            let mut npc = NPCMap::create_npc(4, &state.npc_table);

            npc.cond.set_alive(true);
            npc.direction = Direction::Left;
            npc.x = x + off_x;
            npc.y = y + off_y;

            state.new_npcs.push(npc);
        }

        state.create_caret(x, y, CaretType::Explosion, Direction::Left);
    }

    pub fn process_dead_npcs(&mut self, list: &[u16], state: &mut SharedGameState) {
        for id in list {
            let npc_cell = self.npcs.get(id);
            if npc_cell.is_some() {
                let mut npc = npc_cell.unwrap().borrow_mut();

                match npc.size {
                    1 => { self.create_death_effect(npc.x, npc.y, npc.display_bounds.right, 3, state); }
                    2 => { self.create_death_effect(npc.x, npc.y, npc.display_bounds.right, 7, state); }
                    3 => { self.create_death_effect(npc.x, npc.y, npc.display_bounds.right, 12, state); }
                    _ => {}
                };

                if npc.exp != 0 {
                    //if state.game_rng.range(0..4) == 0 {
                    // health

                    //} else {
                    let mut exp = npc.exp;

                    while exp > 0 {
                        let exp_piece = if exp >= 20 {
                            exp -= 20;
                            20
                        } else if exp >= 5 {
                            exp -= 5;
                            5
                        } else {
                            exp -= 1;
                            1
                        };

                        let mut xp_npc = NPCMap::create_npc(1, &state.npc_table);
                        xp_npc.cond.set_alive(true);
                        xp_npc.direction = Direction::Left;
                        xp_npc.x = npc.x;
                        xp_npc.y = npc.y;
                        xp_npc.exp = exp_piece;

                        state.new_npcs.push(xp_npc);
                    }
                    //}
                }

                state.game_flags.set(npc.flag_num as usize, true);

                // todo vanish / show damage

                npc.cond.set_alive(false);
            }
        }

        self.process_npc_changes(state);
    }

    pub fn process_npc_changes(&mut self, state: &mut SharedGameState) {
        if !state.new_npcs.is_empty() {
            for mut npc in state.new_npcs.iter_mut() {
                let id = if npc.id == 0 {
                    self.allocate_id(npc.get_start_index())
                } else {
                    npc.id
                };

                npc.id = id;
                self.npc_ids.insert(id);
                self.npcs.insert(id, RefCell::new(*npc));
            }

            state.new_npcs.clear();
        }
    }

    pub fn is_alive(&self, npc_id: u16) -> bool {
        if let Some(npc_cell) = self.npcs.get(&npc_id) {
            return npc_cell.borrow().cond.alive();
        }

        false
    }

    pub fn is_alive_by_event(&self, event_num: u16) -> bool {
        for npc_cell in self.npcs.values() {
            let npc = npc_cell.borrow();
            if npc.cond.alive() && npc.event_num == event_num {
                return true;
            }
        }

        false
    }
}

pub struct NPCTableEntry {
    pub npc_flags: NPCFlag,
    pub life: u16,
    pub spritesheet_id: u8,
    pub death_sound: u8,
    pub hurt_sound: u8,
    pub size: u8,
    pub experience: u32,
    pub damage: u32,
    pub display_bounds: Rect<u8>,
    pub hit_bounds: Rect<u8>,
}

pub struct NPCTable {
    entries: Vec<NPCTableEntry>,
    pub tex_npc1_name: String,
    pub tex_npc2_name: String,
}

impl NPCTable {
    #[allow(clippy::new_without_default)]
    pub fn new() -> NPCTable {
        NPCTable {
            entries: Vec::new(),
            tex_npc1_name: str!("Npc/Npc0"),
            tex_npc2_name: str!("Npc/Npc0"),
        }
    }

    pub fn load_from<R: io::Read>(mut data: R) -> GameResult<NPCTable> {
        let mut table = NPCTable::new();

        let mut buf = Vec::new();
        data.read_to_end(&mut buf)?;

        let count = buf.len() / 0x18;
        let mut f = Cursor::new(buf);

        for _ in 0..count {
            table.entries.push(NPCTableEntry {
                npc_flags: NPCFlag(0),
                life: 0,
                spritesheet_id: 0,
                death_sound: 0,
                hurt_sound: 0,
                size: 0,
                experience: 0,
                damage: 0,
                display_bounds: Rect::new(0, 0, 0, 0),
                hit_bounds: Rect::new(0, 0, 0, 0),
            });
        }

        for npc in table.entries.iter_mut() {
            npc.npc_flags.0 = f.read_u16::<LE>()?;
        }

        for npc in table.entries.iter_mut() {
            npc.life = f.read_u16::<LE>()?;
        }

        for npc in table.entries.iter_mut() {
            npc.spritesheet_id = f.read_u8()?;
        }

        for npc in table.entries.iter_mut() {
            npc.hurt_sound = f.read_u8()?;
        }

        for npc in table.entries.iter_mut() {
            npc.death_sound = f.read_u8()?;
        }

        for npc in table.entries.iter_mut() {
            npc.size = f.read_u8()?;
        }

        for npc in table.entries.iter_mut() {
            npc.experience = f.read_u32::<LE>()?;
        }

        for npc in table.entries.iter_mut() {
            npc.damage = f.read_u32::<LE>()?;
        }

        for npc in table.entries.iter_mut() {
            npc.hit_bounds.left = f.read_u8()?;
            npc.hit_bounds.top = f.read_u8()?;
            npc.hit_bounds.right = f.read_u8()?;
            npc.hit_bounds.bottom = f.read_u8()?;
        }

        for npc in table.entries.iter_mut() {
            npc.display_bounds.left = f.read_u8()?;
            npc.display_bounds.top = f.read_u8()?;
            npc.display_bounds.right = f.read_u8()?;
            npc.display_bounds.bottom = f.read_u8()?;
        }

        Ok(table)
    }

    pub fn get_entry(&self, npc_type: u16) -> Option<&NPCTableEntry> {
        self.entries.get(npc_type as usize)
    }

    pub fn get_display_bounds(&self, npc_type: u16) -> Rect<usize> {
        if let Some(npc) = self.entries.get(npc_type as usize) {
            Rect {
                left: npc.display_bounds.left as usize * 0x200,
                top: npc.display_bounds.top as usize * 0x200,
                right: npc.display_bounds.right as usize * 0x200,
                bottom: npc.display_bounds.bottom as usize * 0x200,
            }
        } else {
            Rect { left: 0, top: 0, right: 0, bottom: 0 }
        }
    }

    pub fn get_hit_bounds(&self, npc_type: u16) -> Rect<usize> {
        if let Some(npc) = self.entries.get(npc_type as usize) {
            Rect {
                left: npc.hit_bounds.left as usize * 0x200,
                top: npc.hit_bounds.top as usize * 0x200,
                right: npc.hit_bounds.right as usize * 0x200,
                bottom: npc.hit_bounds.bottom as usize * 0x200,
            }
        } else {
            Rect { left: 0, top: 0, right: 0, bottom: 0 }
        }
    }

    pub fn get_texture_name(&self, npc_type: u16) -> &str {
        if let Some(npc) = self.entries.get(npc_type as usize) {
            match npc.spritesheet_id {
                20 => "Npc/NpcSym",
                21 => self.tex_npc1_name.as_str(),
                22 => self.tex_npc2_name.as_str(),
                23 => "Npc/NpcRegu",
                _ => "Npc/Npc0"
            }
        } else {
            "Npc/Npc0"
        }
    }
}
