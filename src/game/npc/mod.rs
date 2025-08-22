use std::cell::{Ref, RefCell};
use std::io;
use std::io::Cursor;
use std::ops::Deref;
use std::rc::Rc;

use byteorder::{ReadBytesExt, LE};

use crate::bitfield;
use crate::common::Direction;
use crate::common::Flag;
use crate::common::{interpolate_fix9_scale, Condition, Rect};
use crate::components::flash::Flash;
use crate::components::number_popup::NumberPopup;
use crate::entity::GameEntity;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::game::frame::Frame;
use crate::game::npc::boss::BossNPC;
use crate::game::npc::list::{BorrowedNPC, NPCList};
use crate::game::physics::PhysicalEntity;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::game::stage::{Stage, StageTexturePaths};
use crate::game::weapon::bullet::BulletManager;
use crate::util::rng::Xoroshiro32PlusPlus;

use super::physics::HitExtents;

pub mod ai;
pub mod boss;
pub mod list;
pub mod utils;

bitfield! {
    #[derive(Clone, Copy)]
    pub struct NPCFlag(u16);
    impl Debug;
    /// Represented by 0x01
    pub solid_soft, set_solid_soft: 0;
    /// Represented by 0x02
    pub ignore_tile_44, set_ignore_tile_44: 1;
    /// Represented by 0x04
    pub invulnerable, set_invulnerable: 2;
    /// Represented by 0x08
    pub ignore_solidity, set_ignore_solidity: 3;
    /// Represented by 0x10
    pub bouncy, set_bouncy: 4;
    /// Represented by 0x20
    pub shootable, set_shootable: 5;
    /// Represented by 0x40
    pub solid_hard, set_solid_hard: 6;
    /// Represented by 0x80
    pub rear_and_top_not_hurt, set_rear_and_top_not_hurt: 7;
    /// Represented by 0x100
    pub event_when_touched, set_event_when_touched: 8;
    /// Represented by 0x200
    pub event_when_killed, set_event_when_killed: 9;
    /// Represented by 0x400
    pub flag_x400, set_flag_x400: 10;
    /// Represented by 0x800
    pub appear_when_flag_set, set_appear_when_flag_set: 11;
    /// Represented by 0x1000
    pub spawn_facing_right, set_spawn_facing_right: 12;
    /// Represented by 0x2000
    pub interactable, set_interactable: 13;
    /// Represented by 0x4000
    pub hide_unless_flag_set, set_hide_unless_flag_set: 14;
    /// Represented by 0x8000
    pub show_damage, set_show_damage: 15;
}

#[derive(Debug, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum NPCLayer {
    Background = 0,
    Middleground = 1,
    Foreground = 2,
}

/// Represents an NPC object.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct NPC {
    pub id: u16,
    pub npc_type: u16,
    pub x: i32,
    pub y: i32,
    /// X velocity, affected by physics code
    pub vel_x: i32,
    /// Y velocity, affected by physics code
    pub vel_y: i32,
    /// X velocity, unaffected by physics code
    pub vel_x2: i32,
    /// Y velocity, unaffected by physics code
    pub vel_y2: i32,
    pub target_x: i32,
    pub target_y: i32,
    /// Previous X position, used by frame interpolator
    pub prev_x: i32,
    /// Previous Y position, used by frame interpolator
    pub prev_y: i32,
    pub exp: u16,
    pub layer: NPCLayer,
    pub size: u8,
    pub shock: u16,
    pub life: u16,
    pub damage: u16,
    pub spritesheet_id: u16,
    pub cond: Condition,
    pub flags: Flag,
    pub npc_flags: NPCFlag,
    pub direction: Direction,
    /// Raw direction value set by TSC because some NPCs have it set outside 0-4 range,
    /// breaking the direction type.
    pub tsc_direction: u16,
    pub parent_id: u16,
    pub action_num: u16,
    pub anim_num: u16,
    pub flag_num: u16,
    pub event_num: u16,
    pub action_counter: u16,
    pub action_counter2: u16,
    pub action_counter3: u16,
    pub anim_counter: u16,
    pub anim_rect: Rect<u16>,
    pub display_bounds: Rect<u32>,
    pub hit_bounds: HitExtents,
    pub rng: Xoroshiro32PlusPlus,
    pub popup: NumberPopup,
    pub splash: bool,
}

impl NPC {
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
            layer: NPCLayer::Middleground,
            size: 0,
            shock: 0,
            life: 0,
            damage: 0,
            spritesheet_id: 0,
            cond: Condition(0),
            flags: Flag(0),
            npc_flags: NPCFlag(0),
            direction: Direction::Left,
            tsc_direction: 0,
            display_bounds: Rect { left: 0, top: 0, right: 0, bottom: 0 },
            hit_bounds: HitExtents { left: 0, top: 0, right: 0, bottom: 0 },
            parent_id: 0,
            action_num: 0,
            anim_num: 0,
            flag_num: 0,
            event_num: 0,
            action_counter: 0,
            action_counter2: 0,
            action_counter3: 0,
            anim_counter: 0,
            anim_rect: Rect { left: 0, top: 0, right: 0, bottom: 0 },
            rng: Xoroshiro32PlusPlus::new(0),
            popup: NumberPopup::new(),
            splash: false,
        }
    }

    pub fn draw_if_layer(
        &self,
        state: &mut SharedGameState,
        ctx: &mut Context,
        frame: &Frame,
        layer: NPCLayer,
    ) -> GameResult {
        if self.layer == layer {
            self.npc_draw(state, ctx, frame)?
        }

        Ok(())
    }

    pub fn draw_lightmap(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult {
        if !self.cond.alive() || self.cond.hidden() {
            return Ok(());
        }

        let texture = &*state.npc_table.get_texture_ref(self.spritesheet_id);

        if let Some(batch) = state.texture_set.get_or_load_batch(ctx, &state.constants, texture)?.glow() {
            let off_x =
                if self.direction == Direction::Left { self.display_bounds.left } else { self.display_bounds.right }
                    as i32;
            let shock = if self.shock > 0 { (2 * ((self.shock as i32 / 2) % 2) - 1) as f32 } else { 0.0 };

            let (frame_x, frame_y) = frame.xy_interpolated(state.frame_time);

            batch.add_rect(
                interpolate_fix9_scale(self.prev_x - off_x, self.x - off_x, state.frame_time) + shock - frame_x,
                interpolate_fix9_scale(
                    self.prev_y - self.display_bounds.top as i32,
                    self.y - self.display_bounds.top as i32,
                    state.frame_time,
                ) - frame_y,
                &self.anim_rect,
            );

            batch.draw(ctx)?;
        }

        Ok(())
    }

    fn is_sue(&self) -> bool {
        [42, 92, 280, 284].contains(&self.npc_type)
    }

    pub fn get_headband_spritesheet(state: &SharedGameState, texture_name: &str) -> String {
        let base_dir = if state.settings.original_textures || state.constants.is_base() { "ogph" } else { "plus" };
        format!("headband/{}/{}", base_dir, texture_name)
    }
}

pub struct NPCContext<'a> {
    pub players: [&'a mut Player; 2],
    pub npc_list: &'a NPCList,
    pub stage: &'a mut Stage,
    pub bullet_manager: &'a mut BulletManager,
    pub flash: &'a mut Flash,
    pub boss: &'a mut BossNPC,
}

impl GameEntity<NPCContext<'_>> for BorrowedNPC<'_> {
    fn tick(&mut self, state: &mut SharedGameState, ctx: NPCContext) -> GameResult {
        match self.npc_type {
            0 => self.tick_n000_null(state, ctx),
            1 => self.tick_n001_experience(state, ctx),
            2 => self.tick_n002_behemoth(state, ctx),
            3 => self.tick_n003_dead_enemy(state, ctx),
            4 => self.tick_n004_smoke(state, ctx),
            5 => self.tick_n005_green_critter(state, ctx),
            6 => self.tick_n006_green_beetle(state, ctx),
            7 => self.tick_n007_basil(state, ctx),
            8 => self.tick_n008_blue_beetle(state, ctx),
            9 => self.tick_n009_balrog_falling_in(state, ctx),
            10 => self.tick_n010_balrog_shooting(state, ctx),
            11 => self.tick_n011_balrogs_projectile(state, ctx),
            12 => self.tick_n012_balrog_cutscene(state, ctx),
            13 => self.tick_n013_forcefield(state, ctx),
            14 => self.tick_n014_key(state, ctx),
            15 => self.tick_n015_chest_closed(state, ctx),
            16 => self.tick_n016_save_point(state, ctx),
            17 => self.tick_n017_health_refill(state, ctx),
            18 => self.tick_n018_door(state, ctx),
            19 => self.tick_n019_balrog_bust_in(state, ctx),
            20 => self.tick_n020_computer(state, ctx),
            21 => self.tick_n021_chest_open(state, ctx),
            22 => self.tick_n022_teleporter(state, ctx),
            23 => self.tick_n023_teleporter_lights(state, ctx),
            24 => self.tick_n024_power_critter(state, ctx),
            25 => self.tick_n025_lift(state, ctx),
            26 => self.tick_n026_bat_flying(state, ctx),
            27 => self.tick_n027_death_trap(state, ctx),
            28 => self.tick_n028_flying_critter(state, ctx),
            29 => self.tick_n029_cthulhu(state, ctx),
            30 => self.tick_n030_gunsmith(state, ctx),
            31 => self.tick_n031_bat_hanging(state, ctx),
            32 => self.tick_n032_life_capsule(state, ctx),
            33 => self.tick_n033_balrog_bouncing_projectile(state, ctx),
            34 => self.tick_n034_bed(state, ctx),
            35 => self.tick_n035_mannan(state, ctx),
            36 => self.tick_n036_balrog_hover(state, ctx),
            37 => self.tick_n037_sign(state, ctx),
            38 => self.tick_n038_fireplace(state, ctx),
            39 => self.tick_n039_save_sign(state, ctx),
            40 => self.tick_n040_santa(state, ctx),
            41 => self.tick_n041_busted_door(state, ctx),
            42 => self.tick_n042_sue(state, ctx),
            43 => self.tick_n043_chalkboard(state, ctx),
            44 => self.tick_n044_polish(state, ctx),
            45 => self.tick_n045_baby(state, ctx),
            46 => self.tick_n046_hv_trigger(state, ctx),
            47 => self.tick_n047_sandcroc(state, ctx),
            48 => self.tick_n048_omega_projectiles(state, ctx),
            49 => self.tick_n049_skullhead(state, ctx),
            50 => self.tick_n050_skeleton_projectile(state, ctx),
            51 => self.tick_n051_crow_and_skullhead(state, ctx),
            52 => self.tick_n052_sitting_blue_robot(state, ctx),
            53 => self.tick_n053_skullstep_leg(state, ctx),
            54 => self.tick_n054_skullstep(state, ctx),
            55 => self.tick_n055_kazuma(state, ctx),
            56 => self.tick_n056_tan_beetle(state, ctx),
            57 => self.tick_n057_crow(state, ctx),
            58 => self.tick_n058_basu(state, ctx),
            59 => self.tick_n059_eye_door(state, ctx),
            60 => self.tick_n060_toroko(state, ctx),
            61 => self.tick_n061_king(state, ctx),
            62 => self.tick_n062_kazuma_computer(state, ctx),
            63 => self.tick_n063_toroko_stick(state, ctx),
            64 => self.tick_n064_first_cave_critter(state, ctx),
            65 => self.tick_n065_first_cave_bat(state, ctx),
            66 => self.tick_n066_misery_bubble(state, ctx),
            67 => self.tick_n067_misery_floating(state, ctx),
            68 => self.tick_n068_balrog_running(state, ctx),
            69 => self.tick_n069_pignon(state, ctx),
            70 => self.tick_n070_sparkle(state, ctx),
            71 => self.tick_n071_chinfish(state, ctx),
            72 => self.tick_n072_sprinkler(state, ctx),
            73 => self.tick_n073_water_droplet(state, ctx),
            74 => self.tick_n074_jack(state, ctx),
            75 => self.tick_n075_kanpachi(state, ctx),
            76 => self.tick_n076_flowers(state, ctx),
            77 => self.tick_n077_yamashita(state, ctx),
            78 => self.tick_n078_pot(state, ctx),
            79 => self.tick_n079_mahin(state, ctx),
            80 => self.tick_n080_gravekeeper(state, ctx),
            81 => self.tick_n081_giant_pignon(state, ctx),
            82 => self.tick_n082_misery_standing(state, ctx),
            83 => self.tick_n083_igor_cutscene(state, ctx),
            84 => self.tick_n084_basu_projectile(state, ctx),
            85 => self.tick_n085_terminal(state, ctx),
            86 => self.tick_n086_missile_pickup(state, ctx),
            87 => self.tick_n087_heart_pickup(state, ctx),
            88 => self.tick_n088_igor_boss(state, ctx),
            89 => self.tick_n089_igor_dead(state, ctx),
            90 => self.tick_n090_background(state, ctx),
            91 => self.tick_n091_mimiga_cage(state, ctx),
            92 => self.tick_n092_sue_at_pc(state, ctx),
            93 => self.tick_n093_chaco(state, ctx),
            94 => self.tick_n094_kulala(state, ctx),
            95 => self.tick_n095_jelly(state, ctx),
            96 => self.tick_n096_fan_left(state, ctx),
            97 => self.tick_n097_fan_up(state, ctx),
            98 => self.tick_n098_fan_right(state, ctx),
            99 => self.tick_n099_fan_down(state, ctx),
            100 => self.tick_n100_grate(state, ctx),
            101 => self.tick_n101_malco_screen(state, ctx),
            102 => self.tick_n102_malco_computer_wave(state, ctx),
            103 => self.tick_n103_mannan_projectile(state, ctx),
            104 => self.tick_n104_frog(state, ctx),
            105 => self.tick_n105_hey_bubble_low(state, ctx),
            106 => self.tick_n106_hey_bubble_high(state, ctx),
            107 => self.tick_n107_malco_broken(state, ctx),
            108 => self.tick_n108_balfrog_projectile(state, ctx),
            109 => self.tick_n109_malco_powered_on(state, ctx),
            110 => self.tick_n110_puchi(state, ctx),
            111 => self.tick_n111_quote_teleport_out(state, ctx),
            112 => self.tick_n112_quote_teleport_in(state, ctx),
            113 => self.tick_n113_professor_booster(state, ctx),
            114 => self.tick_n114_press(state, ctx),
            115 => self.tick_n115_ravil(state, ctx),
            116 => self.tick_n116_red_petals(state, ctx),
            117 => self.tick_n117_curly(state, ctx),
            118 => self.tick_n118_curly_boss(state, ctx),
            119 => self.tick_n119_table_chair(state, ctx),
            120 => self.tick_n120_colon_a(state, ctx),
            121 => self.tick_n121_colon_b(state, ctx),
            122 => self.tick_n122_colon_enraged(state, ctx),
            123 => self.tick_n123_curly_boss_bullet(state, ctx),
            124 => self.tick_n124_sunstone(state, ctx),
            125 => self.tick_n125_hidden_item(state, ctx),
            126 => self.tick_n126_puppy_running(state, ctx),
            127 => self.tick_n127_machine_gun_trail_l2(state, ctx),
            128 => self.tick_n128_machine_gun_trail_l3(state, ctx),
            129 => self.tick_n129_fireball_snake_trail(state, ctx),
            130 => self.tick_n130_puppy_sitting(state, ctx),
            131 => self.tick_n131_puppy_sleeping(state, ctx),
            132 => self.tick_n132_puppy_barking(state, ctx),
            133 => self.tick_n133_jenka(state, ctx),
            134 => self.tick_n134_armadillo(state, ctx),
            135 => self.tick_n135_skeleton(state, ctx),
            136 => self.tick_n136_puppy_carried(state, ctx),
            137 => self.tick_n137_large_door_frame(state, ctx),
            138 => self.tick_n138_large_door(state, ctx),
            139 => self.tick_n139_doctor(state, ctx),
            140 => self.tick_n140_toroko_frenzied(state, ctx),
            141 => self.tick_n141_toroko_block_projectile(state, ctx),
            142 => self.tick_n142_flower_cub(state, ctx),
            143 => self.tick_n143_jenka_collapsed(state, ctx),
            144 => self.tick_n144_toroko_teleporting_in(state, ctx),
            145 => self.tick_n145_king_sword(state, ctx),
            146 => self.tick_n146_lightning(state, ctx),
            147 => self.tick_n147_critter_purple(state, ctx),
            148 => self.tick_n148_critter_purple_projectile(state, ctx),
            149 => self.tick_n149_horizontal_moving_block(state, ctx),
            150 => self.tick_n150_quote(state, ctx),
            151 => self.tick_n151_blue_robot_standing(state, ctx),
            152 => self.tick_n152_shutter_stuck(state, ctx),
            153 => self.tick_n153_gaudi(state, ctx),
            154 => self.tick_n154_gaudi_dead(state, ctx),
            155 => self.tick_n155_gaudi_flying(state, ctx),
            156 => self.tick_n156_gaudi_projectile(state, ctx),
            157 => self.tick_n157_vertical_moving_block(state, ctx),
            158 => self.tick_n158_fish_missile(state, ctx),
            159 => self.tick_n159_monster_x_defeated(state, ctx),
            160 => self.tick_n160_puu_black(state, ctx),
            161 => self.tick_n161_puu_black_projectile(state, ctx),
            162 => self.tick_n162_puu_black_dead(state, ctx),
            163 => self.tick_n163_dr_gero(state, ctx),
            164 => self.tick_n164_nurse_hasumi(state, ctx),
            165 => self.tick_n165_curly_collapsed(state, ctx),
            166 => self.tick_n166_chaba(state, ctx),
            167 => self.tick_n167_booster_falling(state, ctx),
            168 => self.tick_n168_boulder(state, ctx),
            169 => self.tick_n169_balrog_shooting_missiles(state, ctx),
            170 => self.tick_n170_balrog_missile(state, ctx),
            171 => self.tick_n171_fire_whirrr(state, ctx),
            172 => self.tick_n172_fire_whirrr_projectile(state, ctx),
            173 => self.tick_n173_gaudi_armored(state, ctx),
            174 => self.tick_n174_gaudi_armored_projectile(state, ctx),
            175 => self.tick_n175_gaudi_egg(state, ctx),
            176 => self.tick_n176_buyo_buyo_base(state, ctx),
            177 => self.tick_n177_buyo_buyo(state, ctx),
            178 => self.tick_n178_core_blade_projectile(state, ctx),
            179 => self.tick_n179_core_wisp_projectile(state, ctx),
            180 => self.tick_n180_curly_ai(state, ctx),
            181 => self.tick_n181_curly_ai_machine_gun(state, ctx),
            182 => self.tick_n182_curly_ai_polar_star(state, ctx),
            183 => self.tick_n183_curly_air_tank_bubble(state, ctx),
            184 => self.tick_n184_shutter(state, ctx),
            185 => self.tick_n185_small_shutter(state, ctx),
            186 => self.tick_n186_lift_block(state, ctx),
            187 => self.tick_n187_fuzz_core(state, ctx),
            188 => self.tick_n188_fuzz(state, ctx),
            189 => self.tick_n189_homing_flame(state, ctx),
            190 => self.tick_n190_broken_robot(state, ctx),
            191 => self.tick_n191_water_level(state, ctx),
            192 => self.tick_n192_scooter(state, ctx),
            193 => self.tick_n193_broken_scooter(state, ctx),
            194 => self.tick_n194_broken_blue_robot(state, ctx),
            195 => self.tick_n195_background_grate(state, ctx),
            196 => self.tick_n196_ironhead_wall(state, ctx),
            197 => self.tick_n197_porcupine_fish(state, ctx),
            198 => self.tick_n198_ironhead_projectile(state, ctx),
            199 => self.tick_n199_wind_particles(state, ctx),
            200 => self.tick_n200_zombie_dragon(state, ctx),
            201 => self.tick_n201_zombie_dragon_dead(state, ctx),
            202 => self.tick_n202_zombie_dragon_projectile(state, ctx),
            203 => self.tick_n203_critter_destroyed_egg_corridor(state, ctx),
            204 => self.tick_n204_small_falling_spike(state, ctx),
            205 => self.tick_n205_large_falling_spike(state, ctx),
            206 => self.tick_n206_counter_bomb(state, ctx),
            207 => self.tick_n207_counter_bomb_countdown(state, ctx),
            208 => self.tick_n208_basu_destroyed_egg_corridor(state, ctx),
            209 => self.tick_n209_basu_projectile_destroyed_egg_corridor(state, ctx),
            210 => self.tick_n210_beetle_destroyed_egg_corridor(state, ctx),
            211 => self.tick_n211_small_spikes(state, ctx),
            212 => self.tick_n212_sky_dragon(state, ctx),
            213 => self.tick_n213_night_spirit(state, ctx),
            214 => self.tick_n214_night_spirit_projectile(state, ctx),
            215 => self.tick_n215_sandcroc_outer_wall(state, ctx),
            216 => self.tick_n216_debug_cat(state, ctx),
            217 => self.tick_n217_itoh(state, ctx),
            218 => self.tick_n218_core_giant_ball(state, ctx),
            219 => self.tick_n219_smoke_generator(state, ctx),
            220 => self.tick_n220_shovel_brigade(state, ctx),
            221 => self.tick_n221_shovel_brigade_walking(state, ctx),
            222 => self.tick_n222_prison_bars(state, ctx),
            223 => self.tick_n223_momorin(state, ctx),
            224 => self.tick_n224_chie(state, ctx),
            225 => self.tick_n225_megane(state, ctx),
            226 => self.tick_n226_kanpachi_plantation(state, ctx),
            227 => self.tick_n227_bucket(state, ctx),
            228 => self.tick_n228_droll(state, ctx),
            229 => self.tick_n229_red_flowers_sprouts(state, ctx),
            230 => self.tick_n230_red_flowers_blooming(state, ctx),
            231 => self.tick_n231_rocket(state, ctx),
            232 => self.tick_n232_orangebell(state, ctx),
            233 => self.tick_n233_orangebell_bat(state, ctx),
            234 => self.tick_n234_red_flowers_picked(state, ctx),
            235 => self.tick_n235_midorin(state, ctx),
            236 => self.tick_n236_gunfish(state, ctx),
            237 => self.tick_n237_gunfish_projectile(state, ctx),
            238 => self.tick_n238_press_sideways(state, ctx),
            239 => self.tick_n239_cage_bars(state, ctx),
            240 => self.tick_n240_mimiga_jailed(state, ctx),
            241 => self.tick_n241_critter_red(state, ctx),
            242 => self.tick_n242_bat_last_cave(state, ctx),
            243 => self.tick_n243_bat_generator(state, ctx),
            244 => self.tick_n244_lava_drop(state, ctx),
            245 => self.tick_n245_lava_drop_generator(state, ctx),
            246 => self.tick_n246_press_proximity(state, ctx),
            247 => self.tick_n247_misery_boss(state, ctx),
            248 => self.tick_n248_misery_boss_vanishing(state, ctx),
            249 => self.tick_n249_misery_boss_appearing(state, ctx),
            250 => self.tick_n250_misery_boss_lightning_ball(state, ctx),
            251 => self.tick_n251_misery_boss_lightning(state, ctx),
            252 => self.tick_n252_misery_boss_bats(state, ctx),
            253 => self.tick_n253_experience_capsule(state, ctx),
            254 => self.tick_n254_helicopter(state, ctx),
            255 => self.tick_n255_helicopter_blades(state, ctx),
            256 => self.tick_n256_doctor_facing_away(state, ctx),
            257 => self.tick_n257_red_crystal(state, ctx),
            258 => self.tick_n258_mimiga_sleeping(state, ctx),
            259 => self.tick_n259_curly_unconscious(state, ctx),
            260 => self.tick_n260_shovel_brigade_caged(state, ctx),
            261 => self.tick_n261_chie_caged(state, ctx),
            262 => self.tick_n262_chaco_caged(state, ctx),
            263 => self.tick_n263_doctor_boss(state, ctx),
            264 => self.tick_n264_doctor_boss_red_projectile(state, ctx),
            265 => self.tick_n265_doctor_boss_red_projectile_trail(state, ctx),
            266 => self.tick_n266_doctor_boss_red_projectile_bouncing(state, ctx),
            267 => self.tick_n267_muscle_doctor(state, ctx),
            268 => self.tick_n268_igor_enemy(state, ctx),
            269 => self.tick_n269_red_bat_bouncing(state, ctx),
            270 => self.tick_n270_doctor_red_energy(state, ctx),
            271 => self.tick_n271_ironhead_block(state, ctx),
            272 => self.tick_n272_ironhead_block_generator(state, ctx),
            273 => self.tick_n273_droll_projectile(state, ctx),
            274 => self.tick_n274_droll(state, ctx),
            275 => self.tick_n275_puppy_plantation(state, ctx),
            276 => self.tick_n276_red_demon(state, ctx),
            277 => self.tick_n277_red_demon_projectile(state, ctx),
            278 => self.tick_n278_little_family(state, ctx),
            279 => self.tick_n279_large_falling_block(state, ctx),
            280 => self.tick_n280_sue_teleported(state, ctx),
            281 => self.tick_n281_doctor_energy_form(state, ctx),
            282 => self.tick_n282_mini_undead_core_active(state, ctx),
            283 => self.tick_n283_misery_possessed(state, ctx),
            284 => self.tick_n284_sue_possessed(state, ctx),
            285 => self.tick_n285_undead_core_spiral_projectile(state, ctx),
            286 => self.tick_n286_undead_core_spiral_projectile_trail(state, ctx),
            287 => self.tick_n287_orange_smoke(state, ctx),
            288 => self.tick_n288_undead_core_exploding_rock(state, ctx),
            289 => self.tick_n289_critter_orange(state, ctx),
            290 => self.tick_n290_bat_misery(state, ctx),
            291 => self.tick_n291_mini_undead_core_inactive(state, ctx),
            292 => self.tick_n292_quake(state, ctx),
            293 => self.tick_n293_undead_core_energy_shot(state, ctx),
            294 => self.tick_n294_quake_falling_block_generator(state, ctx),
            295 => self.tick_n295_cloud(state, ctx),
            296 => self.tick_n296_cloud_generator(state, ctx),
            297 => self.tick_n297_sue_dragon_mouth(state, ctx),
            298 => self.tick_n298_intro_doctor(state, ctx),
            299 => self.tick_n299_intro_balrog_misery(state, ctx),
            300 => self.tick_n300_intro_demon_crown(state, ctx),
            301 => self.tick_n301_misery_fish_missile(state, ctx),
            302 => self.tick_n302_camera_focus_marker(state, ctx),
            303 => self.tick_n303_curly_machine_gun(state, ctx),
            304 => self.tick_n304_gaudi_hospital(state, ctx),
            305 => self.tick_n305_small_puppy(state, ctx),
            306 => self.tick_n306_balrog_nurse(state, ctx),
            307 => self.tick_n307_santa_caged(state, ctx),
            308 => self.tick_n308_stumpy(state, ctx),
            309 => self.tick_n309_bute(state, ctx),
            310 => self.tick_n310_bute_sword(state, ctx),
            311 => self.tick_n311_bute_archer(state, ctx),
            312 => self.tick_n312_bute_arrow_projectile(state, ctx),
            313 => self.tick_n313_ma_pignon(state, ctx),
            314 => self.tick_n314_ma_pignon_rock(state, ctx),
            315 => self.tick_n315_ma_pignon_clone(state, ctx),
            316 => self.tick_n316_bute_dead(state, ctx),
            317 => self.tick_n317_mesa(state, ctx),
            318 => self.tick_n318_mesa_dead(state, ctx),
            319 => self.tick_n319_mesa_block(state, ctx),
            320 => self.tick_n320_curly_carried(state, ctx),
            321 => self.tick_n321_curly_nemesis(state, ctx),
            322 => self.tick_n322_deleet(state, ctx),
            323 => self.tick_n323_bute_spinning(state, ctx),
            324 => self.tick_n324_bute_generator(state, ctx),
            325 => self.tick_n325_heavy_press_lightning(state, ctx),
            326 => self.tick_n326_sue_itoh_human_transition(state, ctx),
            327 => self.tick_n327_sneeze(state, ctx),
            328 => self.tick_n328_human_transform_machine(state, ctx),
            329 => self.tick_n329_laboratory_fan(state, ctx),
            330 => self.tick_n330_rolling(state, ctx),
            331 => self.tick_n331_ballos_bone_projectile(state, ctx),
            332 => self.tick_n332_ballos_shockwave(state, ctx),
            333 => self.tick_n333_ballos_lightning(state, ctx),
            334 => self.tick_n334_sweat(state, ctx),
            335 => self.tick_n335_ikachan(state, ctx),
            336 => self.tick_n336_ikachan_generator(state, ctx),
            337 => self.tick_n337_numahachi(state, ctx),
            338 => self.tick_n338_green_devil(state, ctx),
            339 => self.tick_n339_green_devil_generator(state, ctx),
            340 => self.tick_n340_ballos(state, ctx),
            341 => self.tick_n341_ballos_1_head(state, ctx),
            342 => self.tick_n342_ballos_orbiting_eye(state, ctx),
            343 => self.tick_n343_ballos_3_cutscene(state, ctx),
            344 => self.tick_n344_ballos_3_eyes(state, ctx),
            345 => self.tick_n345_ballos_skull_projectile(state, ctx),
            346 => self.tick_n346_ballos_orbiting_platform(state, ctx),
            347 => self.tick_n347_hoppy(state, ctx),
            348 => self.tick_n348_ballos_4_spikes(state, ctx),
            349 => self.tick_n349_statue(state, ctx),
            350 => self.tick_n350_flying_bute_archer(state, ctx),
            351 => self.tick_n351_statue_shootable(state, ctx),
            352 => self.tick_n352_ending_characters(state, ctx),
            353 => self.tick_n353_bute_sword_flying(state, ctx),
            354 => self.tick_n354_invisible_deathtrap_wall(state, ctx),
            355 => self.tick_n355_quote_and_curly_on_balrog(state, ctx),
            356 => self.tick_n356_balrog_rescuing(state, ctx),
            357 => self.tick_n357_puppy_ghost(state, ctx),
            358 => self.tick_n358_misery_credits(state, ctx),
            359 => self.tick_n359_water_droplet_generator(state, ctx),
            360 => self.tick_n360_credits_thank_you(state, ctx),
            361 => self.tick_n361_flying_gaudi(state, ctx),
            362 => self.tick_n362_curly_clone(state, ctx),
            363 => self.tick_n363_dead_curly_clone(state, ctx),
            364 => self.tick_n364_fast_bullet(state, ctx),
            365 => self.tick_n365_still_curly_clone(state, ctx),
            366 => self.tick_n366_zombie_curly_clone(state, ctx),
            367 => self.tick_n367_curly_clone_incubator(state, ctx),
            368 => self.tick_n368_gclone(state, ctx),
            369 => self.tick_n369_gclone_curly_clone(state, ctx),
            370 => self.tick_n370_second_quote(state, ctx),
            _ => Ok(()),
        }?;

        // I don't know where the best place to put this is, but let's try putting it here
        if self.shock == 0 && self.npc_flags.show_damage() && self.popup.value != 0 {
            self.popup.update_displayed_value();
        }
        self.popup.x = self.x;
        self.popup.y = self.y;
        self.popup.tick(state, ())?;

        if self.shock > 0 {
            self.shock -= 1;
        }

        if (self.prev_x - self.x).abs() > 0x1000 {
            self.prev_x = self.x;
        }

        if (self.prev_y - self.y).abs() > 0x1000 {
            self.prev_y = self.y;
        }

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult {
        self.npc_draw(state, ctx, frame)
    }
}

impl NPC {
    pub fn npc_draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult {
        if !self.cond.alive() || self.cond.hidden() {
            return Ok(());
        }

        let texture_ref = state.npc_table.get_texture_ref(self.spritesheet_id);

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, &*texture_ref)?;

        let off_x =
            if self.direction == Direction::Left { self.display_bounds.left } else { self.display_bounds.right } as i32;
        let shock = if self.shock > 0 { (2 * ((self.shock as i32 / 2) % 2) - 1) as f32 } else { 0.0 };

        let (frame_x, frame_y) = frame.xy_interpolated(state.frame_time);

        let final_x = interpolate_fix9_scale(self.prev_x - off_x, self.x - off_x, state.frame_time) + shock - frame_x;
        let final_y = interpolate_fix9_scale(
            self.prev_y - self.display_bounds.top as i32,
            self.y - self.display_bounds.top as i32,
            state.frame_time,
        ) - frame_y;

        if self.is_sue() && state.more_rust {
            // tint sue blue
            batch.add_rect_tinted(final_x, final_y, (200, 200, 255, 255), &self.anim_rect);
            batch.draw(ctx)?;
        } else {
            batch.add_rect(final_x, final_y, &self.anim_rect);
            batch.draw(ctx)?;
        }

        if self.is_sue() && state.more_rust {
            // draw crab headband
            let headband_spritesheet = Self::get_headband_spritesheet(state, &*texture_ref);
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, headband_spritesheet.as_str())?;
            batch.add_rect(final_x, final_y, &self.anim_rect);
            batch.draw(ctx)?;
        }

        Ok(())
    }
}

impl PhysicalEntity for NPC {
    #[inline(always)]
    fn x(&self) -> i32 {
        self.x
    }

    #[inline(always)]
    fn y(&self) -> i32 {
        self.y
    }

    #[inline(always)]
    fn vel_x(&self) -> i32 {
        self.vel_x
    }

    #[inline(always)]
    fn vel_y(&self) -> i32 {
        self.vel_y
    }

    #[inline(always)]
    fn hit_rect_size(&self) -> usize {
        if self.size >= 3 {
            if self.cond.drs_boss() {
                4
            } else {
                3
            }
        } else {
            2
        }
    }

    #[inline(always)]
    fn offset_x(&self) -> i32 {
        if self.size >= 3 && !self.cond.drs_boss() {
            -0x1000
        } else {
            0
        }
    }

    #[inline(always)]
    fn offset_y(&self) -> i32 {
        if self.size >= 3 && !self.cond.drs_boss() {
            -0x1000
        } else {
            0
        }
    }

    #[inline(always)]
    fn hit_bounds(&self) -> &HitExtents {
        &self.hit_bounds
    }

    #[inline(always)]
    fn display_bounds(&self) -> &Rect<u32> {
        &self.display_bounds
    }

    #[inline(always)]
    fn set_x(&mut self, x: i32) {
        self.x = x;
    }

    #[inline(always)]
    fn set_y(&mut self, y: i32) {
        self.y = y;
    }

    #[inline(always)]
    fn set_vel_x(&mut self, vel_x: i32) {
        self.vel_x = vel_x;
    }

    #[inline(always)]
    fn set_vel_y(&mut self, vel_y: i32) {
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
    pub stage_textures: Rc<RefCell<StageTexturePaths>>,
}

impl NPCTable {
    #[allow(clippy::new_without_default)]
    pub fn new() -> NPCTable {
        NPCTable { entries: Vec::new(), stage_textures: Rc::new(RefCell::new(StageTexturePaths::new())) }
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

        for npc in &mut table.entries {
            npc.npc_flags.0 = f.read_u16::<LE>()?;
        }

        for npc in &mut table.entries {
            npc.life = f.read_u16::<LE>()?;
        }

        for npc in &mut table.entries {
            npc.spritesheet_id = f.read_u8()?;
        }

        for npc in &mut table.entries {
            npc.death_sound = f.read_u8()?;
        }

        for npc in &mut table.entries {
            npc.hurt_sound = f.read_u8()?;
        }

        for npc in &mut table.entries {
            npc.size = f.read_u8()?;
        }

        for npc in &mut table.entries {
            npc.experience = f.read_u32::<LE>()?;
        }

        for npc in &mut table.entries {
            npc.damage = f.read_u32::<LE>()?;
        }

        for npc in &mut table.entries {
            npc.hit_bounds.left = f.read_u8()?;
            npc.hit_bounds.top = f.read_u8()?;
            npc.hit_bounds.right = f.read_u8()?;
            npc.hit_bounds.bottom = f.read_u8()?;
        }

        for npc in &mut table.entries {
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

    pub fn get_display_bounds(&self, npc_type: u16) -> Rect<u32> {
        if let Some(npc) = self.entries.get(npc_type as usize) {
            Rect {
                left: npc.display_bounds.left as u32 * 0x200,
                top: npc.display_bounds.top as u32 * 0x200,
                right: npc.display_bounds.right as u32 * 0x200,
                bottom: npc.display_bounds.bottom as u32 * 0x200,
            }
        } else {
            Rect { left: 0, top: 0, right: 0, bottom: 0 }
        }
    }

    pub fn get_hit_bounds(&self, npc_type: u16) -> HitExtents {
        if let Some(npc) = self.entries.get(npc_type as usize) {
            HitExtents {
                left: npc.hit_bounds.left as u32 * 0x200,
                top: npc.hit_bounds.top as u32 * 0x200,
                right: npc.hit_bounds.right as u32 * 0x200,
                bottom: npc.hit_bounds.bottom as u32 * 0x200,
            }
        } else {
            HitExtents { left: 0, top: 0, right: 0, bottom: 0 }
        }
    }

    pub fn get_texture_ref(&self, spritesheet_id: u16) -> TexRef<'_> {
        match spritesheet_id {
            0 => TexRef::from_str("Title"),
            2 => TexRef { variant: TexRefVariant::StageTileset(self.stage_textures.deref().borrow()) },
            6 => TexRef::from_str("Fade"),
            8 => TexRef::from_str("ItemImage"),
            11 => TexRef::from_str("Arms"),
            12 => TexRef::from_str("ArmsImage"),
            14 => TexRef::from_str("StageImage"),
            15 => TexRef::from_str("Loading"),
            16 => TexRef::from_str("MyChar"),
            17 => TexRef::from_str("Bullet"),
            19 => TexRef::from_str("Caret"),
            20 => TexRef::from_str("Npc/NpcSym"),
            21 => TexRef { variant: TexRefVariant::StageNPC1(self.stage_textures.deref().borrow()) },
            22 => TexRef { variant: TexRefVariant::StageNPC2(self.stage_textures.deref().borrow()) },
            23 => TexRef::from_str("Npc/NpcRegu"),
            26 => TexRef::from_str("TextBox"),
            27 => TexRef::from_str("Face"),
            _ => TexRef::from_str("Npc/Npc0"),
        }
    }
}

pub struct TexRef<'a> {
    variant: TexRefVariant<'a>,
}

enum TexRefVariant<'a> {
    Str(&'static str),
    StageTileset(Ref<'a, StageTexturePaths>),
    StageNPC1(Ref<'a, StageTexturePaths>),
    StageNPC2(Ref<'a, StageTexturePaths>),
}

impl TexRef<'_> {
    #[inline]
    fn from_str(str: &'static str) -> TexRef {
        TexRef { variant: TexRefVariant::Str(str) }
    }
}

impl Deref for TexRef<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match &self.variant {
            TexRefVariant::Str(str) => str,
            TexRefVariant::StageTileset(paths) => &paths.tileset_fg,
            TexRefVariant::StageNPC1(paths) => &paths.npc1,
            TexRefVariant::StageNPC2(paths) => &paths.npc2,
        }
    }
}
