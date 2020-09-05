use std::collections::{BTreeSet, HashMap};
use std::io;
use std::io::Cursor;

use bitvec::vec::BitVec;
use byteorder::{LE, ReadBytesExt};

use crate::{bitfield, SharedGameState};
use crate::common::{Condition, Rect};
use crate::common::Direction;
use crate::common::Flag;
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::ggez::{Context, GameResult};
use crate::map::NPCData;
use crate::player::Player;
use crate::str;

pub mod misc;
pub mod critter;

bitfield! {
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
  pub spawn_in_other_direction, set_spawn_in_other_direction: 12;
  pub interactable, set_interactable: 13;
  pub hide_unless_flag_set, set_hide_unless_flag_set: 14;
  pub show_damage, set_show_damage: 15;
}

#[derive(Debug)]
pub struct NPC {
    pub id: u16,
    pub npc_type: u16,
    pub x: isize,
    pub y: isize,
    pub vel_x: isize,
    pub vel_y: isize,
    pub target_x: isize,
    pub target_y: isize,
    pub life: u16,
    pub cond: Condition,
    pub flags: Flag,
    pub npc_flags: NPCFlag,
    pub direction: Direction,
    pub display_bounds: Rect<usize>,
    pub hit_bounds: Rect<usize>,
    pub action_num: u16,
    pub anim_num: u16,
    pub flag_num: u16,
    pub event_num: u16,
    pub action_counter: u16,
    pub anim_counter: u16,
    pub anim_rect: Rect<usize>,
}

impl GameEntity<&mut Player> for NPC {
    fn tick(&mut self, state: &mut SharedGameState, scene: &mut Player) -> GameResult {
        // maybe use macros?
        match self.npc_type {
            0 => { NPC::tick_n000_null(self, state) }
            16 => { NPC::tick_n016_save_point(self, state) }
            17 => { NPC::tick_n017_health_refill(self, state) }
            18 => { NPC::tick_n018_door(self, state) }
            20 => { NPC::tick_n020_computer(self, state) }
            27 => { NPC::tick_n027_death_trap(self, state) }
            32 => { NPC::tick_n032_life_capsule(self, state) }
            34 => { NPC::tick_n034_bed(self, state) }
            37 => { NPC::tick_n037_sign(self, state) }
            38 => { NPC::tick_n038_fireplace(self, state) }
            39 => { NPC::tick_n039_save_sign(self, state) }
            _ => { Ok(()) }
        }
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult {
        if !self.cond.alive() || self.cond.hidden() {
            return Ok(());
        }

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, state.npc_table.get_texture_name(self.npc_type))?;

        let off_x = if self.direction == Direction::Left { self.display_bounds.left } else { self.display_bounds.right } as isize;
        batch.add_rect(
            (((self.x - off_x) / 0x200) - (frame.x / 0x200)) as f32,
            (((self.y - self.display_bounds.top as isize) / 0x200) - (frame.y / 0x200)) as f32,
            &self.anim_rect,
        );
        batch.draw(ctx)?;

        Ok(())
    }
}

pub struct NPCMap {
    /// A sorted pool of free IDs to make ID assignment for new entities a bit cheaper.
    free_npc_ids: BTreeSet<u16>,
    /// A sorted pool of used IDs to used to iterate over NPCs in order, as original game does.
    pub npc_ids: BTreeSet<u16>,
    /// Do not iterate over this directly outside render pipeline.
    pub npcs: HashMap<u16, NPC>,
}

impl NPCMap {
    #[allow(clippy::new_without_default)]
    pub fn new() -> NPCMap {
        NPCMap {
            npc_ids: BTreeSet::new(),
            free_npc_ids: BTreeSet::new(),
            npcs: HashMap::with_capacity(256),
        }
    }

    pub fn clear(&mut self) {
        self.free_npc_ids.clear();
        self.npc_ids.clear();
        self.npcs.clear();
    }

    pub fn create_npc_from_data(&mut self, table: &NPCTable, data: &NPCData) -> &mut NPC {
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
            life: table.get_life(data.npc_type),
            cond: Condition(0x00),
            flags: Flag(data.flag_num as u32),
            npc_flags: NPCFlag(data.flags),
            direction: Direction::Left,
            display_bounds: table.get_display_bounds(data.npc_type),
            hit_bounds: table.get_hit_bounds(data.npc_type),
            action_counter: 0,
            anim_counter: 0,
            anim_rect: Rect::new(0, 0, 0, 0),
        };

        self.free_npc_ids.remove(&data.id);
        self.npc_ids.insert(data.id);
        self.npcs.insert(data.id, npc);

        self.npcs.get_mut(&data.id).unwrap()
    }

    pub fn remove_by_event(&mut self, event_num: u16, game_flags: &mut BitVec) {
        for npc in self.npcs.values_mut() {
            if npc.event_num == event_num {
                npc.cond.set_alive(false);
                game_flags.set(npc.flag_num as usize, true);
            }
        }
    }
}

pub struct NPCTableEntry {
    npc_flags: NPCFlag,
    life: u16,
    spritesheet_id: u8,
    death_sound: u8,
    hurt_sound: u8,
    death_smoke: u8,
    experience: u32,
    damage: u32,
    display_bounds: Rect<u8>,
    hit_bounds: Rect<u8>,
}

pub struct NPCTable {
    entries: Vec<NPCTableEntry>,
    pub tex_npc1_name: String,
    pub tex_npc2_name: String,
}

impl NPCTable {
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
                death_smoke: 0,
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
            npc.death_sound = f.read_u8()?;
        }

        for npc in table.entries.iter_mut() {
            npc.hurt_sound = f.read_u8()?;
        }

        for npc in table.entries.iter_mut() {
            npc.death_smoke = f.read_u8()?;
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

    pub fn get_life(&self, npc_type: u16) -> u16 {
        if let Some(npc) = self.entries.get(npc_type as usize) {
            npc.life
        } else {
            0
        }
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
