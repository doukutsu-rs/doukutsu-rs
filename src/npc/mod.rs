use std::io;
use std::io::Cursor;

use byteorder::{LE, ReadBytesExt};
use ggez::{Context, GameResult};
use num_traits::abs;

use crate::bitfield;
use crate::bullet::BulletManager;
use crate::common::{Condition, interpolate_fix9_scale, Rect};
use crate::common::Direction;
use crate::common::Flag;
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::npc::list::NPCList;
use crate::physics::PhysicalEntity;
use crate::player::Player;
use crate::rng::Xoroshiro32PlusPlus;
use crate::shared_game_state::SharedGameState;
use crate::stage::Stage;
use crate::str;

pub mod ai;
pub mod boss;
pub mod list;
pub mod utils;

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

/// Represents an NPC object.
#[derive(Debug, Clone)]
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
    pub rng: Xoroshiro32PlusPlus,
}

static PARTICLE_NPCS: [u16; 13] = [1, 4, 11, 45, 48, 73, 84, 86, 87, 108, 129, 199, 355];

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
            rng: Xoroshiro32PlusPlus::new(0),
        }
    }
}

impl GameEntity<([&mut Player; 2], &NPCList, &mut Stage, &BulletManager)> for NPC {
    fn tick(&mut self, state: &mut SharedGameState, (players, npc_list, stage, _bullet_manager): ([&mut Player; 2], &NPCList, &mut Stage, &BulletManager)) -> GameResult {
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
            9 => self.tick_n009_balrog_falling_in(state, npc_list),
            10 => self.tick_n010_balrog_shooting(state, players, npc_list),
            11 => self.tick_n011_balrogs_projectile(state),
            12 => self.tick_n012_balrog_cutscene(state, players, npc_list, stage),
            13 => self.tick_n013_forcefield(state),
            14 => self.tick_n014_key(state),
            15 => self.tick_n015_chest_closed(state, npc_list),
            16 => self.tick_n016_save_point(state),
            17 => self.tick_n017_health_refill(state),
            18 => self.tick_n018_door(state, npc_list),
            19 => self.tick_n019_balrog_bust_in(state, npc_list),
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
            35 => self.tick_n035_mannan(state, npc_list),
            36 => self.tick_n036_balrog_hover(state, players, npc_list),
            37 => self.tick_n037_sign(state),
            38 => self.tick_n038_fireplace(state),
            39 => self.tick_n039_save_sign(state),
            40 => self.tick_n040_santa(state, players),
            41 => self.tick_n041_busted_door(state),
            42 => self.tick_n042_sue(state, players, npc_list),
            43 => self.tick_n043_chalkboard(state),
            44 => self.tick_n044_polish(state, npc_list),
            45 => self.tick_n045_baby(state),
            46 => self.tick_n046_hv_trigger(players),
            47 => self.tick_n047_sandcroc(state, players),
            48 => self.tick_n048_omega_projectiles(state),
            49 => self.tick_n049_skullhead(state, npc_list),
            52 => self.tick_n052_sitting_blue_robot(state),
            55 => self.tick_n055_kazuma(state),
            58 => self.tick_n058_basu(state, players, npc_list),
            59 => self.tick_n059_eye_door(state, players),
            60 => self.tick_n060_toroko(state, players),
            61 => self.tick_n061_king(state),
            62 => self.tick_n062_kazuma_computer(state),
            63 => self.tick_n063_toroko_stick(state),
            64 => self.tick_n064_first_cave_critter(state, players),
            65 => self.tick_n065_first_cave_bat(state, players),
            66 => self.tick_n066_misery_bubble(state, npc_list),
            67 => self.tick_n067_misery_floating(state, npc_list),
            68 => self.tick_n068_balrog_running(state, players, npc_list),
            69 => self.tick_n069_pignon(state),
            70 => self.tick_n070_sparkle(state),
            71 => self.tick_n071_chinfish(state),
            72 => self.tick_n072_sprinkler(state, players, npc_list),
            73 => self.tick_n073_water_droplet(state, stage),
            74 => self.tick_n074_jack(state),
            75 => self.tick_n075_kanpachi(state, players),
            76 => self.tick_n076_flowers(),
            77 => self.tick_n077_yamashita(state),
            78 => self.tick_n078_pot(state),
            79 => self.tick_n079_mahin(state, players),
            80 => self.tick_n080_gravekeeper(state, players),
            81 => self.tick_n081_giant_pignon(state, players),
            82 => self.tick_n082_misery_standing(state, npc_list),
            83 => self.tick_n083_igor_cutscene(state),
            84 => self.tick_n084_basu_projectile(state),
            85 => self.tick_n085_terminal(state, players),
            86 => self.tick_n086_missile_pickup(state),
            87 => self.tick_n087_heart_pickup(state),
            88 => self.tick_n088_igor_boss(state, players, npc_list),
            89 => self.tick_n089_igor_dead(state, players, npc_list),
            91 => self.tick_n091_mimiga_cage(state),
            92 => self.tick_n092_sue_at_pc(state),
            93 => self.tick_n093_chaco(state, players),
            94 => self.tick_n094_kulala(state, players),
            95 => self.tick_n095_jelly(state),
            96 => self.tick_n096_fan_left(state, players, npc_list),
            97 => self.tick_n097_fan_up(state, players, npc_list),
            98 => self.tick_n098_fan_right(state, players, npc_list),
            99 => self.tick_n099_fan_down(state, players, npc_list),
            100 => self.tick_n100_grate(state),
            101 => self.tick_n101_malco_screen(state),
            102 => self.tick_n102_malco_computer_wave(state),
            103 => self.tick_n103_mannan_projectile(state),
            104 => self.tick_n104_frog(state, players),
            105 => self.tick_n105_hey_bubble_low(state),
            106 => self.tick_n106_hey_bubble_high(state, npc_list),
            107 => self.tick_n107_malco_broken(state, npc_list),
            108 => self.tick_n108_balfrog_projectile(state),
            109 => self.tick_n109_malco_powered_on(state, players, npc_list),
            110 => self.tick_n110_puchi(state, players),
            111 => self.tick_n111_quote_teleport_out(state, players),
            112 => self.tick_n112_quote_teleport_in(state, players),
            113 => self.tick_n113_professor_booster(state),
            114 => self.tick_n114_press(state, players, npc_list),
            124 => self.tick_n124_sunstone(state),
            125 => self.tick_n125_hidden_item(state, npc_list),
            129 => self.tick_n129_fireball_snake_trail(state),
            149 => self.tick_n149_horizontal_moving_block(state, players, npc_list),
            150 => self.tick_n150_quote(state, players, npc_list),
            151 => self.tick_n151_blue_robot_standing(state),
            154 => self.tick_n154_gaudi_dead(state),
            156 => self.tick_n156_gaudi_projectile(state),
            157 => self.tick_n157_vertical_moving_block(state, players, npc_list),
            158 => self.tick_n158_fish_missile(state, players),
            192 => self.tick_n192_scooter(state),
            193 => self.tick_n193_broken_scooter(state),
            194 => self.tick_n194_broken_blue_robot(state),
            199 => self.tick_n199_wind_particles(state),
            211 => self.tick_n211_small_spikes(state),
            234 => self.tick_n234_red_flowers_picked(state),
            298 => self.tick_n298_intro_doctor(state),
            299 => self.tick_n299_intro_balrog_misery(state),
            300 => self.tick_n300_intro_demon_crown(state),
            361 => self.tick_n361_gaudi_dashing(state, players, npc_list),
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
