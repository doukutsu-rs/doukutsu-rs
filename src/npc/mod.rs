use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::io;
use std::io::Cursor;
use std::ops::DerefMut;

use bitvec::vec::BitVec;
use byteorder::{LE, ReadBytesExt};
use ggez::{Context, GameResult};
use num_traits::abs;

use crate::bitfield;
use crate::caret::CaretType;
use crate::common::{Condition, interpolate_fix9_scale, Rect};
use crate::common::Direction;
use crate::common::Flag;
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::map::NPCData;
use crate::npc::boss::BossNPC;
use crate::physics::PhysicalEntity;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;
use crate::stage::Stage;
use crate::str;

pub mod balrog;
pub mod boss;
pub mod chaco;
pub mod characters;
pub mod egg_corridor;
pub mod first_cave;
pub mod grasstown;
pub mod igor;
pub mod intro;
pub mod maze;
pub mod mimiga_village;
pub mod misc;
pub mod misery;
pub mod npc_utils;
pub mod pickups;
pub mod quote;
pub mod sand_zone;
pub mod santa;
pub mod sue;
pub mod toroko;
pub mod weapon_trail;

bitfield! {
  #[derive(Clone, Copy)]
  pub struct NPCFlag(u16);
  impl Debug;

  pub solid_soft, set_solid_soft: 0; // 0x01
  pub ignore_tile_44, set_ignore_tile_44: 1; // 0x02
  pub invulnerable, set_invulnerable: 2; // 0x04
  pub ignore_solidity, set_ignore_solidity: 3; // 0x08
  pub bouncy, set_bouncy: 4; // 0x10
  pub shootable, set_shootable: 5; // 0x20
  pub solid_hard, set_solid_hard: 6; // 0x40
  pub rear_and_top_not_hurt, set_rear_and_top_not_hurt: 7; // 0x80
  pub event_when_touched, set_event_when_touched: 8; // 0x100
  pub event_when_killed, set_event_when_killed: 9; // 0x200
  pub flag_x400, set_flag_x400: 10; // 0x400
  pub appear_when_flag_set, set_appear_when_flag_set: 11; // 0x800
  pub spawn_facing_right, set_spawn_facing_right: 12; // 0x1000
  pub interactable, set_interactable: 13; // 0x2000
  pub hide_unless_flag_set, set_hide_unless_flag_set: 14; // 0x4000
  pub show_damage, set_show_damage: 15; // 0x8000
}

#[derive(Debug, Clone, Copy)]
pub struct NPC {
    pub id: u16,
    pub npc_type: u16,
    pub x: isize,
    pub y: isize,
    /// X velocity, affected by physics code
    pub vel_x: isize,
    /// Y velocity, affected by physics code
    pub vel_y: isize,
    /// X velocity, unaffected by physics code
    pub vel_x2: isize,
    /// Y velocity, unaffected by physics code
    pub vel_y2: isize,
    pub target_x: isize,
    pub target_y: isize,
    /// Previous X position, used by frame interpolator
    pub prev_x: isize,
    /// Previous Y position, used by frame interpolator
    pub prev_y: isize,
    pub exp: u16,
    pub size: u8,
    pub shock: u16,
    pub life: u16,
    pub damage: u16,
    pub cond: Condition,
    pub flags: Flag,
    pub npc_flags: NPCFlag,
    pub direction: Direction,
    /// Raw direction value set by TSC because some NPCs have it set outside 0-4 range,
    /// breaking the direction type.
    pub tsc_direction: u16,
    pub display_bounds: Rect<usize>,
    pub hit_bounds: Rect<usize>,
    pub parent_id: u16,
    pub action_num: u16,
    pub anim_num: u16,
    pub flag_num: u16,
    pub event_num: u16,
    pub action_counter: u16,
    pub action_counter2: u16,
    pub anim_counter: u16,
    pub anim_rect: Rect<u16>,
}

static PARTICLE_NPCS: [u16; 11] = [1, 4, 11, 73, 84, 86, 87, 108, 129, 199, 355];

impl NPC {
    pub fn get_start_index(&self) -> u16 {
        if PARTICLE_NPCS.contains(&self.npc_type) {
            0x100
        } else {
            0
        }
    }

    pub fn empty() -> NPC {
        NPC {
            id: 0,
            npc_type: 0,
            x: 0,
            y: 0,
            vel_x: 0,
            vel_y: 0,
            vel_x2: 0,
            vel_y2: 0,
            target_x: 0,
            target_y: 0,
            prev_x: 0,
            prev_y: 0,
            exp: 0,
            size: 0,
            shock: 0,
            life: 0,
            damage: 0,
            cond: Condition(0),
            flags: Flag(0),
            npc_flags: NPCFlag(0),
            direction: Direction::Left,
            tsc_direction: 0,
            display_bounds: Rect { left: 0, top: 0, right: 0, bottom: 0 },
            hit_bounds: Rect { left: 0, top: 0, right: 0, bottom: 0 },
            parent_id: 0,
            action_num: 0,
            anim_num: 0,
            flag_num: 0,
            event_num: 0,
            action_counter: 0,
            action_counter2: 0,
            anim_counter: 0,
            anim_rect: Rect { left: 0, top: 0, right: 0, bottom: 0 },
        }
    }
}

impl GameEntity<([&mut Player; 2], &BTreeMap<u16, RefCell<NPC>>, &mut Stage)> for NPC {
    fn tick(&mut self, state: &mut SharedGameState, (players, map, stage): ([&mut Player; 2], &BTreeMap<u16, RefCell<NPC>>, &mut Stage)) -> GameResult {
        match self.npc_type {
            0 => self.tick_n000_null(),
            1 => self.tick_n001_experience(state),
            2 => self.tick_n002_behemoth(state),
            3 => self.tick_n003_dead_enemy(),
            4 => self.tick_n004_smoke(state),
            5 => self.tick_n005_green_critter(state, players),
            6 => self.tick_n006_green_beetle(state),
            7 => self.tick_n007_basil(state, players),
            8 => self.tick_n008_blue_beetle(state, players),
            9 => self.tick_n009_balrog_falling_in(state),
            10 => self.tick_n010_balrog_shooting(state, players),
            11 => self.tick_n011_balrogs_projectile(state),
            12 => self.tick_n012_balrog_cutscene(state, players, map, stage),
            13 => self.tick_n013_forcefield(state),
            14 => self.tick_n014_key(state),
            15 => self.tick_n015_chest_closed(state),
            16 => self.tick_n016_save_point(state),
            17 => self.tick_n017_health_refill(state),
            18 => self.tick_n018_door(state),
            19 => self.tick_n019_balrog_bust_in(state),
            20 => self.tick_n020_computer(state),
            21 => self.tick_n021_chest_open(state),
            22 => self.tick_n022_teleporter(state),
            23 => self.tick_n023_teleporter_lights(state),
            24 => self.tick_n024_power_critter(state, players),
            25 => self.tick_n025_lift(state),
            26 => self.tick_n026_bat_flying(state, players),
            27 => self.tick_n027_death_trap(state),
            28 => self.tick_n028_flying_critter(state, players),
            29 => self.tick_n029_cthulhu(state, players),
            30 => self.tick_n030_gunsmith(state),
            31 => self.tick_n031_bat_hanging(state, players),
            32 => self.tick_n032_life_capsule(state),
            33 => self.tick_n033_balrog_bouncing_projectile(state),
            34 => self.tick_n034_bed(state),
            35 => self.tick_n035_mannan(state),
            36 => self.tick_n036_balrog_hover(state, players),
            37 => self.tick_n037_sign(state),
            38 => self.tick_n038_fireplace(state),
            39 => self.tick_n039_save_sign(state),
            40 => self.tick_n040_santa(state, players),
            41 => self.tick_n041_busted_door(state),
            42 => self.tick_n042_sue(state, players[0], map),
            43 => self.tick_n043_chalkboard(state),
            46 => self.tick_n046_hv_trigger(players),
            52 => self.tick_n052_sitting_blue_robot(state),
            55 => self.tick_n055_kazuma(state),
            58 => self.tick_n058_basu(state, players),
            59 => self.tick_n059_eye_door(state, players[0]),
            60 => self.tick_n060_toroko(state, players[0]),
            61 => self.tick_n061_king(state),
            62 => self.tick_n062_kazuma_computer(state),
            63 => self.tick_n063_toroko_stick(state),
            64 => self.tick_n064_first_cave_critter(state, players[0]),
            65 => self.tick_n065_first_cave_bat(state, players[0]),
            66 => self.tick_n066_misery_bubble(state, map),
            67 => self.tick_n067_misery_floating(state),
            68 => self.tick_n068_balrog_running(state, players),
            69 => self.tick_n069_pignon(state),
            70 => self.tick_n070_sparkle(state),
            71 => self.tick_n071_chinfish(state),
            72 => self.tick_n072_sprinkler(state, players[0]),
            73 => self.tick_n073_water_droplet(state, stage),
            74 => self.tick_n074_jack(state),
            75 => self.tick_n075_kanpachi(state, players[0]),
            76 => self.tick_n076_flowers(),
            77 => self.tick_n077_yamashita(state),
            78 => self.tick_n078_pot(state),
            79 => self.tick_n079_mahin(state, players),
            80 => self.tick_n080_gravekeeper(state, players),
            81 => self.tick_n081_giant_pignon(state, players),
            82 => self.tick_n082_misery_standing(state),
            83 => self.tick_n083_igor_cutscene(state),
            84 => self.tick_n084_basu_projectile(state),
            85 => self.tick_n085_terminal(state, players),
            86 => self.tick_n086_missile_pickup(state),
            87 => self.tick_n087_heart_pickup(state),
            88 => self.tick_n088_igor_boss(state, players[0]),
            89 => self.tick_n089_igor_dead(state, players[0]),
            91 => self.tick_n091_mimiga_cage(state),
            92 => self.tick_n092_sue_at_pc(state),
            93 => self.tick_n093_chaco(state, players[0]),
            94 => self.tick_n094_kulala(state, players[0]),
            95 => self.tick_n095_jelly(state),
            96 => self.tick_n096_fan_left(state, players),
            97 => self.tick_n097_fan_up(state, players[0]),
            98 => self.tick_n098_fan_right(state, players[0]),
            99 => self.tick_n099_fan_down(state, players[0]),
            100 => self.tick_n100_grate(state),
            101 => self.tick_n101_malco_screen(state),
            102 => self.tick_n102_malco_computer_wave(state),
            103 => self.tick_n103_mannan_projectile(state),
            104 => self.tick_n104_frog(state, players),
            105 => self.tick_n105_hey_bubble_low(state),
            106 => self.tick_n106_hey_bubble_high(state),
            107 => self.tick_n107_malco_broken(state),
            108 => self.tick_n108_balfrog_projectile(state),
            109 => self.tick_n109_malco_powered_on(state, players),
            110 => self.tick_n110_puchi(state, players),
            111 => self.tick_n111_quote_teleport_out(state, players),
            112 => self.tick_n112_quote_teleport_in(state, players),
            114 => self.tick_n114_press(state, players[0]),
            129 => self.tick_n129_fireball_snake_trail(state),
            149 => self.tick_n149_horizontal_moving_block(state, players[0]),
            150 => self.tick_n150_quote(state, players[0]),
            154 => self.tick_n154_gaudi_dead(state),
            157 => self.tick_n157_vertical_moving_block(state, players[0]),
            192 => self.tick_n192_scooter(state),
            193 => self.tick_n193_broken_scooter(state),
            194 => self.tick_n194_broken_blue_robot(state),
            199 => self.tick_n199_wind_particles(state),
            211 => self.tick_n211_small_spikes(state),
            298 => self.tick_n298_intro_doctor(state),
            299 => self.tick_n299_intro_balrog_misery(state),
            300 => self.tick_n300_intro_demon_crown(state),
            361 => self.tick_n361_gaudi_dashing(state, players[0]),
            _ => Ok(()),
        }?;

        if self.shock > 0 {
            self.shock -= 1;
        }

        if abs(self.prev_x - self.x) > 0x1000 {
            self.prev_x = self.x;
        }

        if abs(self.prev_y - self.y) > 0x1000 {
            self.prev_y = self.y;
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
            interpolate_fix9_scale(self.prev_x - off_x - frame.prev_x,
                                   self.x - off_x - frame.x,
                                   state.frame_time) + shock,
            interpolate_fix9_scale(self.prev_y - self.display_bounds.top as isize - frame.prev_y,
                                   self.y - self.display_bounds.top as isize - frame.y,
                                   state.frame_time),
            &self.anim_rect,
        );
        batch.draw(ctx)?;

        Ok(())
    }
}

impl PhysicalEntity for NPC {
    #[inline(always)]
    fn x(&self) -> isize { self.x }

    #[inline(always)]
    fn y(&self) -> isize { self.y }

    #[inline(always)]
    fn vel_x(&self) -> isize { self.vel_x }

    #[inline(always)]
    fn vel_y(&self) -> isize { self.vel_y }

    #[inline(always)]
    fn hit_rect_size(&self) -> usize {
        if self.size >= 3 {
            if self.cond.drs_boss() { 4 } else { 3 }
        } else {
            2
        }
    }

    #[inline(always)]
    fn offset_x(&self) -> isize { if self.size >= 3 && !self.cond.drs_boss() { -0x1000 } else { 0 } }

    #[inline(always)]
    fn offset_y(&self) -> isize { if self.size >= 3 && !self.cond.drs_boss() { -0x1000 } else { 0 } }

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
    ids: HashSet<u16>,
    pub npcs: BTreeMap<u16, RefCell<NPC>>,
    /// NPCMap but for bosses and of static size.
    pub boss_map: BossNPC,
}

impl NPCMap {
    #[allow(clippy::new_without_default)]
    pub fn new() -> NPCMap {
        NPCMap {
            ids: HashSet::new(),
            npcs: BTreeMap::new(),
            boss_map: BossNPC::new(),
        }
    }

    pub fn clear(&mut self) {
        self.ids.clear();
        self.npcs.clear();
    }

    pub fn create_npc_from_data(&mut self, table: &NPCTable, data: &NPCData) -> &mut NPC {
        let display_bounds = table.get_display_bounds(data.npc_type);
        let hit_bounds = table.get_hit_bounds(data.npc_type);
        let (size, life, damage, flags, exp) = match table.get_entry(data.npc_type) {
            Some(entry) => { (entry.size, entry.life, entry.damage as u16, entry.npc_flags, entry.experience as u16) }
            None => { (1, 0, 0, NPCFlag(0), 0) }
        };
        let npc_flags = NPCFlag(data.flags | flags.0);

        let npc = NPC {
            id: data.id,
            npc_type: data.npc_type,
            x: data.x as isize * 16 * 0x200,
            y: data.y as isize * 16 * 0x200,
            vel_x: 0,
            vel_y: 0,
            vel_x2: 0,
            vel_y2: 0,
            target_x: 0,
            target_y: 0,
            prev_x: 0,
            prev_y: 0,
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
            tsc_direction: 0,
            npc_flags,
            display_bounds,
            hit_bounds,
            parent_id: 0,
            action_counter: 0,
            action_counter2: 0,
            anim_counter: 0,
            anim_rect: Rect::new(0, 0, 0, 0),
        };

        let cell = RefCell::new(npc);
        self.npcs.insert(data.id, cell);
        self.ids.insert(data.id);

        self.npcs.get_mut(&data.id).unwrap().get_mut()
    }

    pub fn create_npc(npc_type: u16, table: &NPCTable) -> NPC {
        let display_bounds = table.get_display_bounds(npc_type);
        let hit_bounds = table.get_hit_bounds(npc_type);
        let (size, life, damage, flags, exp) = match table.get_entry(npc_type) {
            Some(entry) => { (entry.size, entry.life, entry.damage as u16, entry.npc_flags, entry.experience as u16) }
            None => { (2, 0, 0, NPCFlag(0), 0) }
        };
        let npc_flags = NPCFlag(flags.0);

        NPC {
            id: 0,
            npc_type,
            x: 0,
            y: 0,
            vel_x: 0,
            vel_y: 0,
            vel_x2: 0,
            vel_y2: 0,
            target_x: 0,
            target_y: 0,
            prev_x: 0,
            prev_y: 0,
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
            tsc_direction: 0,
            npc_flags,
            display_bounds,
            hit_bounds,
            parent_id: 0,
            action_counter: 0,
            action_counter2: 0,
            anim_counter: 0,
            anim_rect: Rect::new(0, 0, 0, 0),
        }
    }

    pub fn garbage_collect(&mut self) {
        for npc_cell in self.npcs.values_mut() {
            let mut npc = npc_cell.borrow();

            if !npc.cond.alive() {
                self.ids.remove(&npc.id);
            }
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

    pub fn remove_by_type(&mut self, npc_type: u16, state: &mut SharedGameState) {
        for npc_cell in self.npcs.values() {
            let mut npc = npc_cell.borrow_mut();

            if npc.npc_type == npc_type {
                npc.cond.set_alive(false);
                state.game_flags.set(npc.flag_num as usize, true);

                match npc.size {
                    1 => self.create_death_effect(npc.x, npc.y, npc.display_bounds.right, 3, state),
                    2 => self.create_death_effect(npc.x, npc.y, npc.display_bounds.right, 7, state),
                    3 => self.create_death_effect(npc.x, npc.y, npc.display_bounds.right, 12, state),
                    _ => {}
                };
            }
        }
    }

    pub fn allocate_id(&mut self, start: u16) -> u16 {
        for i in start..(u16::MAX) {
            if !self.ids.contains(&i) {
                return i;
            }
        }

        0xffff
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

    pub fn process_dead_npcs(&mut self, list: &[u16], has_missile: bool, player: &Player, state: &mut SharedGameState) {
        for id in list {
            if let Some(npc_cell) = self.npcs.get(id) {
                let mut npc = npc_cell.borrow_mut();

                if let Some(table_entry) = state.npc_table.get_entry(npc.npc_type) {
                    state.sound_manager.play_sfx(table_entry.death_sound);
                }

                match npc.size {
                    1 => { self.create_death_effect(npc.x, npc.y, npc.display_bounds.right, 3, state); }
                    2 => { self.create_death_effect(npc.x, npc.y, npc.display_bounds.right, 7, state); }
                    3 => { self.create_death_effect(npc.x, npc.y, npc.display_bounds.right, 12, state); }
                    _ => {}
                };

                if npc.exp != 0 {
                    let rng = state.game_rng.range(0..4);
                    match rng {
                        0 => {
                            let mut heart_pick = NPCMap::create_npc(87, &state.npc_table);
                            heart_pick.cond.set_alive(true);
                            heart_pick.direction = Direction::Left;
                            heart_pick.x = npc.x;
                            heart_pick.y = npc.y;
                            heart_pick.exp = if npc.exp > 6 { 6 } else { 2 };

                            state.new_npcs.push(heart_pick);
                        }
                        1 if has_missile => {
                            let mut missile_pick = NPCMap::create_npc(86, &state.npc_table);
                            missile_pick.cond.set_alive(true);
                            missile_pick.direction = Direction::Left;
                            missile_pick.x = npc.x;
                            missile_pick.y = npc.y;
                            missile_pick.exp = if npc.exp > 6 { 3 } else { 1 };

                            state.new_npcs.push(missile_pick);
                        }
                        _ => {
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
                        }
                    }
                }

                state.game_flags.set(npc.flag_num as usize, true);

                // todo vanish / show damage

                if npc.cond.drs_dont_remove() {
                    npc.cond.set_drs_dont_remove(false);
                } else {
                    npc.cond.set_alive(false);
                }
            }
        }

        self.process_npc_changes(player, state);
    }

    pub fn process_npc_changes(&mut self, player: &Player, state: &mut SharedGameState) {
        if !state.new_npcs.is_empty() {
            for mut npc in state.new_npcs.iter_mut() {
                let id = if npc.id == 0 {
                    self.allocate_id(npc.get_start_index())
                } else {
                    npc.id
                };

                npc.id = id;
                if npc.tsc_direction == 0 {
                    npc.tsc_direction = npc.direction as u16;
                }

                if npc.direction == Direction::FacingPlayer {
                    npc.direction = if npc.x < player.x {
                        Direction::Right
                    } else {
                        Direction::Left
                    };
                }

                self.ids.insert(id);
                self.npcs.insert(id, RefCell::new(*npc));
            }

            state.new_npcs.clear();
        }
    }

    /// Returns true if at least one NPC with specified type is alive.
    pub fn is_alive_by_type(&self, npc_type: u16) -> bool {
        for npc_cell in self.npcs.values() {
            let npc = npc_cell.borrow();
            if npc.cond.alive() && npc.npc_type == npc_type {
                return true;
            }
        }

        false
    }

    /// Returns true if at least one NPC with specified event is alive.
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
    pub tileset_name: String,
    pub tex_npc1_name: String,
    pub tex_npc2_name: String,
}

impl NPCTable {
    #[allow(clippy::new_without_default)]
    pub fn new() -> NPCTable {
        NPCTable {
            entries: Vec::new(),
            tileset_name: str!("Stage/Prt0"),
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
            npc.death_sound = f.read_u8()?;
        }

        for npc in table.entries.iter_mut() {
            npc.hurt_sound = f.read_u8()?;
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
                2 => &self.tileset_name,
                6 => "Fade",
                8 => "ItemImage",
                11 => "Arms",
                12 => "ArmsImage",
                14 => "StageImage",
                15 => "Loading",
                16 => "MyChar",
                17 => "Bullet",
                19 => "Caret",
                20 => "Npc/NpcSym",
                21 => &self.tex_npc1_name,
                22 => &self.tex_npc2_name,
                23 => "Npc/NpcRegu",
                26 => "TextBox",
                _ => "Npc/Npc0"
            }
        } else {
            "Npc/Npc0"
        }
    }
}
