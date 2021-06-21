use std::collections::HashMap;

use case_insensitive_hashmap::CaseInsensitiveHashMap;
use log::info;

use crate::case_insensitive_hashmap;
use crate::common::{BulletFlag, Color, Rect};
use crate::engine_constants::npcs::NPCConsts;
use crate::player::ControlMode;
use crate::sound::pixtone::{Channel, PixToneParameters, Waveform, Envelope};
use crate::sound::SoundManager;
use crate::str;
use crate::text_script::TextScriptEncoding;

mod npcs;

#[derive(Debug, Copy, Clone)]
pub struct PhysicsConsts {
    pub max_dash: i32,
    pub max_move: i32,
    pub gravity_ground: i32,
    pub gravity_air: i32,
    pub dash_ground: i32,
    pub dash_air: i32,
    pub resist: i32,
    pub jump: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct BoosterConsts {
    pub fuel: u32,
    pub b2_0_up: i32,
    pub b2_0_up_nokey: i32,
    pub b2_0_down: i32,
    pub b2_0_left: i32,
    pub b2_0_right: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct PlayerConsts {
    pub life: u16,
    pub max_life: u16,
    pub control_mode: ControlMode,
    pub air_physics: PhysicsConsts,
    pub water_physics: PhysicsConsts,
    pub frames_left: [Rect<u16>; 12],
    pub frames_right: [Rect<u16>; 12],
    pub frames_bubble: [Rect<u16>; 2],
}

#[derive(Debug)]
pub struct CaretConsts {
    pub offsets: [(i32, i32); 18],
    pub bubble_left_rects: Vec<Rect<u16>>,
    pub bubble_right_rects: Vec<Rect<u16>>,
    pub projectile_dissipation_left_rects: Vec<Rect<u16>>,
    pub projectile_dissipation_right_rects: Vec<Rect<u16>>,
    pub projectile_dissipation_up_rects: Vec<Rect<u16>>,
    pub shoot_rects: Vec<Rect<u16>>,
    pub zzz_rects: Vec<Rect<u16>>,
    pub drowned_quote_left_rect: Rect<u16>,
    pub drowned_quote_right_rect: Rect<u16>,
    pub level_up_rects: Vec<Rect<u16>>,
    pub level_down_rects: Vec<Rect<u16>>,
    pub hurt_particles_rects: Vec<Rect<u16>>,
    pub explosion_rects: Vec<Rect<u16>>,
    pub little_particles_rects: Vec<Rect<u16>>,
    pub exhaust_rects: Vec<Rect<u16>>,
    pub question_left_rect: Rect<u16>,
    pub question_right_rect: Rect<u16>,
}

impl Clone for CaretConsts {
    fn clone(&self) -> Self {
        Self {
            offsets: self.offsets,
            bubble_left_rects: self.bubble_left_rects.clone(),
            bubble_right_rects: self.bubble_right_rects.clone(),
            projectile_dissipation_left_rects: self.projectile_dissipation_left_rects.clone(),
            projectile_dissipation_right_rects: self.projectile_dissipation_right_rects.clone(),
            projectile_dissipation_up_rects: self.projectile_dissipation_up_rects.clone(),
            shoot_rects: self.shoot_rects.clone(),
            zzz_rects: self.zzz_rects.clone(),
            drowned_quote_left_rect: self.drowned_quote_left_rect,
            drowned_quote_right_rect: self.drowned_quote_right_rect,
            level_up_rects: self.level_up_rects.clone(),
            level_down_rects: self.level_down_rects.clone(),
            hurt_particles_rects: self.hurt_particles_rects.clone(),
            explosion_rects: self.explosion_rects.clone(),
            little_particles_rects: self.little_particles_rects.clone(),
            exhaust_rects: self.exhaust_rects.clone(),
            question_left_rect: self.question_left_rect,
            question_right_rect: self.question_right_rect,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BulletData {
    pub damage: u8,
    pub life: u8,
    pub lifetime: u16,
    pub flags: BulletFlag,
    pub enemy_hit_width: u16,
    pub enemy_hit_height: u16,
    pub block_hit_width: u16,
    pub block_hit_height: u16,
    pub display_bounds: Rect<u8>,
}

#[derive(Debug, Copy, Clone)]
pub struct BulletRects {
    pub b001_snake_l1: [Rect<u16>; 8],
    pub b002_003_snake_l2_3: [Rect<u16>; 3],
    pub b004_polar_star_l1: [Rect<u16>; 2],
    pub b005_polar_star_l2: [Rect<u16>; 2],
    pub b006_polar_star_l3: [Rect<u16>; 2],
    pub b007_fireball_l1: [Rect<u16>; 8],
    pub b008_009_fireball_l2_3: [Rect<u16>; 6],
    pub b010_machine_gun_l1: [Rect<u16>; 4],
    pub b011_machine_gun_l2: [Rect<u16>; 4],
    pub b012_machine_gun_l3: [Rect<u16>; 4],
    pub b013_missile_l1: [Rect<u16>; 4],
    pub b014_missile_l2: [Rect<u16>; 4],
    pub b015_missile_l3: [Rect<u16>; 4],
    pub b019_bubble_l1: [Rect<u16>; 4],
    pub b020_bubble_l2: [Rect<u16>; 4],
    pub b021_bubble_l3: [Rect<u16>; 4],
    pub b022_bubble_spines: [Rect<u16>; 6],
    pub b023_blade_slash: [Rect<u16>; 10],
    pub b025_blade_l1: [Rect<u16>; 8],
    pub b026_blade_l2: [Rect<u16>; 8],
    pub b027_blade_l3: [Rect<u16>; 8],
    pub b028_super_missile_l1: [Rect<u16>; 4],
    pub b029_super_missile_l2: [Rect<u16>; 4],
    pub b030_super_missile_l3: [Rect<u16>; 4],
    pub b034_nemesis_l1: [Rect<u16>; 8],
    pub b035_nemesis_l2: [Rect<u16>; 8],
    pub b036_nemesis_l3: [Rect<u16>; 8],
    pub b037_spur_l1: [Rect<u16>; 2],
    pub b038_spur_l2: [Rect<u16>; 2],
    pub b039_spur_l3: [Rect<u16>; 2],
    pub b040_spur_trail_l1: [Rect<u16>; 6],
    pub b041_spur_trail_l2: [Rect<u16>; 6],
    pub b042_spur_trail_l3: [Rect<u16>; 6],
}

#[derive(Debug)]
pub struct WeaponConsts {
    pub bullet_table: Vec<BulletData>,
    pub bullet_rects: BulletRects,
    pub level_table: [[u16; 3]; 14],
}

impl Clone for WeaponConsts {
    fn clone(&self) -> WeaponConsts {
        WeaponConsts {
            bullet_table: self.bullet_table.clone(),
            bullet_rects: self.bullet_rects,
            level_table: self.level_table,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WorldConsts {
    pub snack_rect: Rect<u16>,
}

#[derive(Debug, Copy, Clone)]
pub struct TextScriptConsts {
    pub encoding: TextScriptEncoding,
    pub encrypted: bool,
    pub animated_face_pics: bool,
    pub textbox_rect_top: Rect<u16>,
    pub textbox_rect_middle: Rect<u16>,
    pub textbox_rect_bottom: Rect<u16>,
    pub textbox_rect_yes_no: Rect<u16>,
    pub textbox_rect_cursor: Rect<u16>,
    pub inventory_rect_top: Rect<u16>,
    pub inventory_rect_middle: Rect<u16>,
    pub inventory_rect_bottom: Rect<u16>,
    pub inventory_text_arms: Rect<u16>,
    pub inventory_text_item: Rect<u16>,
    pub get_item_top_left: Rect<u16>,
    pub get_item_bottom_left: Rect<u16>,
    pub get_item_top_right: Rect<u16>,
    pub get_item_right: Rect<u16>,
    pub get_item_bottom_right: Rect<u16>,
    pub stage_select_text: Rect<u16>,
    pub cursor: [Rect<u16>; 2],
    pub cursor_inventory_weapon: [Rect<u16>; 2],
    pub cursor_inventory_item: [Rect<u16>; 2],
    pub inventory_item_count_x: u8,
    pub text_shadow: bool,
    pub text_speed_normal: u8,
    pub text_speed_fast: u8,
}

#[derive(Debug)]
pub struct TitleConsts {
    pub intro_text: String,
    pub logo_rect: Rect<u16>,
    pub menu_left_top: Rect<u16>,
    pub menu_right_top: Rect<u16>,
    pub menu_left_bottom: Rect<u16>,
    pub menu_right_bottom: Rect<u16>,
    pub menu_top: Rect<u16>,
    pub menu_bottom: Rect<u16>,
    pub menu_middle: Rect<u16>,
    pub menu_left: Rect<u16>,
    pub menu_right: Rect<u16>,
}

impl Clone for TitleConsts {
    fn clone(&self) -> TitleConsts {
        TitleConsts {
            intro_text: self.intro_text.clone(),
            logo_rect: self.logo_rect,
            menu_left_top: self.menu_left_top,
            menu_right_top: self.menu_right_top,
            menu_left_bottom: self.menu_left_bottom,
            menu_right_bottom: self.menu_right_bottom,
            menu_top: self.menu_top,
            menu_bottom: self.menu_bottom,
            menu_middle: self.menu_middle,
            menu_left: self.menu_left,
            menu_right: self.menu_right,
        }
    }
}

#[derive(Debug)]
pub struct EngineConstants {
    pub is_cs_plus: bool,
    pub is_switch: bool,
    pub supports_og_textures: bool,
    pub player: PlayerConsts,
    pub booster: BoosterConsts,
    pub caret: CaretConsts,
    pub world: WorldConsts,
    pub npc: NPCConsts,
    pub weapon: WeaponConsts,
    pub tex_sizes: CaseInsensitiveHashMap<(u16, u16)>,
    pub textscript: TextScriptConsts,
    pub title: TitleConsts,
    pub inventory_dim_color: Color,
    pub font_path: String,
    pub font_scale: f32,
    pub font_space_offset: f32,
    pub soundtracks: HashMap<String, String>,
    pub music_table: Vec<String>,
    pub organya_paths: Vec<String>,
}

impl Clone for EngineConstants {
    fn clone(&self) -> EngineConstants {
        EngineConstants {
            is_cs_plus: self.is_cs_plus,
            is_switch: self.is_switch,
            supports_og_textures: self.supports_og_textures,
            player: self.player,
            booster: self.booster,
            caret: self.caret.clone(),
            world: self.world,
            npc: self.npc,
            weapon: self.weapon.clone(),
            tex_sizes: self.tex_sizes.clone(),
            textscript: self.textscript,
            title: self.title.clone(),
            inventory_dim_color: self.inventory_dim_color,
            font_path: self.font_path.clone(),
            font_scale: self.font_scale,
            font_space_offset: self.font_space_offset,
            soundtracks: self.soundtracks.clone(),
            music_table: self.music_table.clone(),
            organya_paths: self.organya_paths.clone(),
        }
    }
}

impl EngineConstants {
    pub fn defaults() -> Self {
        EngineConstants {
            is_cs_plus: false,
            is_switch: false,
            supports_og_textures: false,
            player: PlayerConsts {
                life: 3,
                max_life: 3,
                control_mode: ControlMode::Normal,
                air_physics: PhysicsConsts {
                    max_dash: 0x32c,
                    max_move: 0x5ff,
                    gravity_air: 0x20,
                    gravity_ground: 0x50,
                    dash_air: 0x20,
                    dash_ground: 0x55,
                    resist: 0x33,
                    jump: 0x500,
                },
                water_physics: PhysicsConsts {
                    max_dash: 0x196,
                    max_move: 0x2ff,
                    gravity_air: 0x10,
                    gravity_ground: 0x28,
                    dash_air: 0x10,
                    dash_ground: 0x2a,
                    resist: 0x19,
                    jump: 0x280,
                },
                frames_left: [
                    Rect { left: 0, top: 0, right: 16, bottom: 16 },
                    Rect { left: 16, top: 0, right: 32, bottom: 16 },
                    Rect { left: 0, top: 0, right: 16, bottom: 16 },
                    Rect { left: 32, top: 0, right: 48, bottom: 16 },
                    Rect { left: 0, top: 0, right: 16, bottom: 16 },
                    Rect { left: 48, top: 0, right: 64, bottom: 16 },
                    Rect { left: 64, top: 0, right: 80, bottom: 16 },
                    Rect { left: 48, top: 0, right: 64, bottom: 16 },
                    Rect { left: 80, top: 0, right: 96, bottom: 16 },
                    Rect { left: 48, top: 0, right: 64, bottom: 16 },
                    Rect { left: 96, top: 0, right: 112, bottom: 16 },
                    Rect { left: 112, top: 0, right: 128, bottom: 16 },
                ],
                frames_right: [
                    Rect { left: 0, top: 16, right: 16, bottom: 32 },
                    Rect { left: 16, top: 16, right: 32, bottom: 32 },
                    Rect { left: 0, top: 16, right: 16, bottom: 32 },
                    Rect { left: 32, top: 16, right: 48, bottom: 32 },
                    Rect { left: 0, top: 16, right: 16, bottom: 32 },
                    Rect { left: 48, top: 16, right: 64, bottom: 32 },
                    Rect { left: 64, top: 16, right: 80, bottom: 32 },
                    Rect { left: 48, top: 16, right: 64, bottom: 32 },
                    Rect { left: 80, top: 16, right: 96, bottom: 32 },
                    Rect { left: 48, top: 16, right: 64, bottom: 32 },
                    Rect { left: 96, top: 16, right: 112, bottom: 32 },
                    Rect { left: 112, top: 16, right: 128, bottom: 32 },
                ],
                frames_bubble: [
                    Rect { left: 56, top: 96, right: 80, bottom: 120 },
                    Rect { left: 80, top: 96, right: 104, bottom: 120 },
                ],
            },
            booster: BoosterConsts {
                fuel: 50,
                b2_0_up: -0x5ff,
                b2_0_up_nokey: -0x5ff,
                b2_0_down: 0x5ff,
                b2_0_left: -0x5ff,
                b2_0_right: 0x5ff,
            },
            caret: CaretConsts {
                offsets: [
                    (0, 0),
                    (0x800, 0x800),
                    (0x1000, 0x1000),
                    (0x1000, 0x1000),
                    (0x1000, 0x1000),
                    (0x800, 0x800),
                    (0x1000, 0x1000),
                    (0x800, 0x800),
                    (0x1000, 0x1000),
                    (0x1000, 0x1000),
                    (28 * 0x200, 0x1000),
                    (0x800, 0x800),
                    (0x2000, 0x2000),
                    (0x800, 0x800),
                    (20 * 0x200, 20 * 0x200),
                    (0x800, 0x800),
                    (20 * 0x200, 0x800),
                    (52 * 0x200, 0x800),
                ],
                bubble_left_rects: vec![
                    Rect { left: 0, top: 64, right: 8, bottom: 72 },
                    Rect { left: 8, top: 64, right: 16, bottom: 72 },
                    Rect { left: 16, top: 64, right: 24, bottom: 72 },
                    Rect { left: 24, top: 64, right: 32, bottom: 72 },
                ],
                bubble_right_rects: vec![
                    Rect { left: 64, top: 24, right: 72, bottom: 32 },
                    Rect { left: 72, top: 24, right: 80, bottom: 32 },
                    Rect { left: 80, top: 24, right: 88, bottom: 32 },
                    Rect { left: 88, top: 24, right: 96, bottom: 32 },
                ],
                projectile_dissipation_left_rects: vec![
                    Rect { left: 0, top: 32, right: 16, bottom: 48 },
                    Rect { left: 16, top: 32, right: 32, bottom: 48 },
                    Rect { left: 32, top: 32, right: 48, bottom: 48 },
                    Rect { left: 48, top: 32, right: 64, bottom: 48 },
                ],
                projectile_dissipation_right_rects: vec![
                    Rect { left: 176, top: 0, right: 192, bottom: 16 },
                    Rect { left: 192, top: 0, right: 208, bottom: 16 },
                    Rect { left: 208, top: 0, right: 224, bottom: 16 },
                    Rect { left: 224, top: 0, right: 240, bottom: 16 },
                ],
                projectile_dissipation_up_rects: vec![
                    Rect { left: 0, top: 32, right: 16, bottom: 48 },
                    Rect { left: 32, top: 32, right: 48, bottom: 48 },
                    Rect { left: 16, top: 32, right: 32, bottom: 48 },
                ],
                shoot_rects: vec![
                    Rect { left: 0, top: 48, right: 16, bottom: 64 },
                    Rect { left: 16, top: 48, right: 32, bottom: 64 },
                    Rect { left: 32, top: 48, right: 48, bottom: 64 },
                    Rect { left: 48, top: 48, right: 64, bottom: 64 },
                ],
                zzz_rects: vec![
                    Rect { left: 32, top: 64, right: 40, bottom: 72 },
                    Rect { left: 32, top: 72, right: 40, bottom: 80 },
                    Rect { left: 40, top: 64, right: 48, bottom: 72 },
                    Rect { left: 40, top: 72, right: 48, bottom: 80 },
                    Rect { left: 40, top: 64, right: 48, bottom: 72 },
                    Rect { left: 40, top: 72, right: 48, bottom: 80 },
                    Rect { left: 40, top: 64, right: 48, bottom: 72 },
                ],
                drowned_quote_left_rect: Rect { left: 16, top: 80, right: 32, bottom: 96 },
                drowned_quote_right_rect: Rect { left: 32, top: 80, right: 48, bottom: 96 },
                level_up_rects: vec![
                    Rect { left: 0, top: 0, right: 56, bottom: 16 },
                    Rect { left: 0, top: 16, right: 56, bottom: 32 },
                ],
                level_down_rects: vec![
                    Rect { left: 0, top: 96, right: 56, bottom: 112 },
                    Rect { left: 0, top: 112, right: 56, bottom: 128 },
                ],
                hurt_particles_rects: vec![
                    Rect { left: 56, top: 8, right: 64, bottom: 16 },
                    Rect { left: 64, top: 8, right: 72, bottom: 16 },
                    Rect { left: 72, top: 8, right: 80, bottom: 16 },
                    Rect { left: 80, top: 8, right: 88, bottom: 16 },
                    Rect { left: 88, top: 8, right: 96, bottom: 16 },
                    Rect { left: 96, top: 8, right: 104, bottom: 16 },
                    Rect { left: 104, top: 8, right: 112, bottom: 16 },
                ],
                explosion_rects: vec![
                    Rect { left: 112, top: 0, right: 144, bottom: 32 },
                    Rect { left: 144, top: 0, right: 176, bottom: 32 },
                ],
                little_particles_rects: vec![
                    Rect { left: 56, top: 24, right: 64, bottom: 32 },
                    Rect { left: 0, top: 0, right: 0, bottom: 0 },
                ],
                exhaust_rects: vec![
                    Rect { left: 56, top: 0, right: 64, bottom: 8 },
                    Rect { left: 64, top: 0, right: 72, bottom: 8 },
                    Rect { left: 72, top: 0, right: 80, bottom: 8 },
                    Rect { left: 80, top: 0, right: 88, bottom: 8 },
                    Rect { left: 88, top: 0, right: 96, bottom: 8 },
                    Rect { left: 96, top: 0, right: 104, bottom: 8 },
                    Rect { left: 104, top: 0, right: 112, bottom: 8 },
                ],
                question_left_rect: Rect { left: 0, top: 80, right: 16, bottom: 96 },
                question_right_rect: Rect { left: 48, top: 64, right: 64, bottom: 80 },
            },
            world: WorldConsts { snack_rect: Rect { left: 256, top: 48, right: 272, bottom: 64 } },
            npc: serde_yaml::from_str("dummy: \"lol\"").unwrap(),
            weapon: WeaponConsts {
                bullet_table: vec![
                    // Null
                    BulletData {
                        damage: 0,
                        life: 0,
                        lifetime: 0,
                        flags: BulletFlag(0),
                        enemy_hit_width: 0,
                        enemy_hit_height: 0,
                        block_hit_width: 0,
                        block_hit_height: 0,
                        display_bounds: Rect { left: 0, top: 0, right: 0, bottom: 0 },
                    },
                    // Snake
                    BulletData {
                        damage: 4,
                        life: 1,
                        lifetime: 20,
                        flags: BulletFlag(36),
                        enemy_hit_width: 4,
                        enemy_hit_height: 4,
                        block_hit_width: 2,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    BulletData {
                        damage: 6,
                        life: 1,
                        lifetime: 23,
                        flags: BulletFlag(36),
                        enemy_hit_width: 4,
                        enemy_hit_height: 4,
                        block_hit_width: 2,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    BulletData {
                        damage: 8,
                        life: 1,
                        lifetime: 30,
                        flags: BulletFlag(36),
                        enemy_hit_width: 4,
                        enemy_hit_height: 4,
                        block_hit_width: 2,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    // Polar Star
                    BulletData {
                        damage: 1,
                        life: 1,
                        lifetime: 8,
                        flags: BulletFlag(32),
                        enemy_hit_width: 6,
                        enemy_hit_height: 6,
                        block_hit_width: 2,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    BulletData {
                        damage: 2,
                        life: 1,
                        lifetime: 12,
                        flags: BulletFlag(32),
                        enemy_hit_width: 6,
                        enemy_hit_height: 6,
                        block_hit_width: 2,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    BulletData {
                        damage: 4,
                        life: 1,
                        lifetime: 16,
                        flags: BulletFlag(32),
                        enemy_hit_width: 6,
                        enemy_hit_height: 6,
                        block_hit_width: 2,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    // Fireball
                    BulletData {
                        damage: 2,
                        life: 2,
                        lifetime: 100,
                        flags: BulletFlag(8),
                        enemy_hit_width: 8,
                        enemy_hit_height: 16,
                        block_hit_width: 4,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    BulletData {
                        damage: 3,
                        life: 2,
                        lifetime: 100,
                        flags: BulletFlag(8),
                        enemy_hit_width: 4,
                        enemy_hit_height: 4,
                        block_hit_width: 4,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    BulletData {
                        damage: 3,
                        life: 2,
                        lifetime: 100,
                        flags: BulletFlag(8),
                        enemy_hit_width: 4,
                        enemy_hit_height: 4,
                        block_hit_width: 4,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    // Machine Gun
                    BulletData {
                        damage: 2,
                        life: 1,
                        lifetime: 20,
                        flags: BulletFlag(32),
                        enemy_hit_width: 2,
                        enemy_hit_height: 2,
                        block_hit_width: 2,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    BulletData {
                        damage: 4,
                        life: 1,
                        lifetime: 20,
                        flags: BulletFlag(32),
                        enemy_hit_width: 2,
                        enemy_hit_height: 2,
                        block_hit_width: 2,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    BulletData {
                        damage: 6,
                        life: 1,
                        lifetime: 20,
                        flags: BulletFlag(32),
                        enemy_hit_width: 2,
                        enemy_hit_height: 2,
                        block_hit_width: 2,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    // Missile Launcher
                    BulletData {
                        damage: 0,
                        life: 10,
                        lifetime: 50,
                        flags: BulletFlag(40),
                        enemy_hit_width: 2,
                        enemy_hit_height: 2,
                        block_hit_width: 2,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    BulletData {
                        damage: 0,
                        life: 10,
                        lifetime: 70,
                        flags: BulletFlag(40),
                        enemy_hit_width: 4,
                        enemy_hit_height: 4,
                        block_hit_width: 4,
                        block_hit_height: 4,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    BulletData {
                        damage: 0,
                        life: 10,
                        lifetime: 90,
                        flags: BulletFlag(40),
                        enemy_hit_width: 4,
                        enemy_hit_height: 4,
                        block_hit_width: 0,
                        block_hit_height: 0,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    // Missile Launcher explosion
                    BulletData {
                        damage: 1,
                        life: 100,
                        lifetime: 100,
                        flags: BulletFlag(20),
                        enemy_hit_width: 16,
                        enemy_hit_height: 16,
                        block_hit_width: 0,
                        block_hit_height: 0,
                        display_bounds: Rect { left: 0, top: 0, right: 0, bottom: 0 },
                    },
                    BulletData {
                        damage: 1,
                        life: 100,
                        lifetime: 100,
                        flags: BulletFlag(20),
                        enemy_hit_width: 16,
                        enemy_hit_height: 16,
                        block_hit_width: 0,
                        block_hit_height: 0,
                        display_bounds: Rect { left: 0, top: 0, right: 0, bottom: 0 },
                    },
                    BulletData {
                        damage: 1,
                        life: 100,
                        lifetime: 100,
                        flags: BulletFlag(20),
                        enemy_hit_width: 16,
                        enemy_hit_height: 16,
                        block_hit_width: 0,
                        block_hit_height: 0,
                        display_bounds: Rect { left: 0, top: 0, right: 0, bottom: 0 },
                    },
                    // Bubbler
                    BulletData {
                        damage: 1,
                        life: 1,
                        lifetime: 20,
                        flags: BulletFlag(8),
                        enemy_hit_width: 2,
                        enemy_hit_height: 2,
                        block_hit_width: 2,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 4, top: 4, right: 4, bottom: 4 },
                    },
                    BulletData {
                        damage: 2,
                        life: 1,
                        lifetime: 20,
                        flags: BulletFlag(8),
                        enemy_hit_width: 2,
                        enemy_hit_height: 2,
                        block_hit_width: 2,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 4, top: 4, right: 4, bottom: 4 },
                    },
                    BulletData {
                        damage: 2,
                        life: 1,
                        lifetime: 20,
                        flags: BulletFlag(8),
                        enemy_hit_width: 4,
                        enemy_hit_height: 4,
                        block_hit_width: 4,
                        block_hit_height: 4,
                        display_bounds: Rect { left: 4, top: 4, right: 4, bottom: 4 },
                    },
                    // Bubbler level 3 thorns
                    BulletData {
                        damage: 3,
                        life: 1,
                        lifetime: 32,
                        flags: BulletFlag(32),
                        enemy_hit_width: 2,
                        enemy_hit_height: 2,
                        block_hit_width: 2,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 4, top: 4, right: 4, bottom: 4 },
                    },
                    // Blade slashes
                    BulletData {
                        damage: 0,
                        life: 100,
                        lifetime: 0,
                        flags: BulletFlag(36),
                        enemy_hit_width: 8,
                        enemy_hit_height: 8,
                        block_hit_width: 8,
                        block_hit_height: 8,
                        display_bounds: Rect { left: 12, top: 12, right: 12, bottom: 12 },
                    },
                    // Falling spike
                    BulletData {
                        damage: 127,
                        life: 1,
                        lifetime: 2,
                        flags: BulletFlag(4),
                        enemy_hit_width: 8,
                        enemy_hit_height: 4,
                        block_hit_width: 8,
                        block_hit_height: 4,
                        display_bounds: Rect { left: 0, top: 0, right: 0, bottom: 0 },
                    },
                    // Blade
                    BulletData {
                        damage: 15,
                        life: 1,
                        lifetime: 30,
                        flags: BulletFlag(36),
                        enemy_hit_width: 8,
                        enemy_hit_height: 8,
                        block_hit_width: 4,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    BulletData {
                        damage: 6,
                        life: 3,
                        lifetime: 18,
                        flags: BulletFlag(36),
                        enemy_hit_width: 10,
                        enemy_hit_height: 10,
                        block_hit_width: 4,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 12, top: 12, right: 12, bottom: 12 },
                    },
                    BulletData {
                        damage: 1,
                        life: 100,
                        lifetime: 30,
                        flags: BulletFlag(36),
                        enemy_hit_width: 6,
                        enemy_hit_height: 6,
                        block_hit_width: 4,
                        block_hit_height: 4,
                        display_bounds: Rect { left: 12, top: 12, right: 12, bottom: 12 },
                    },
                    // Super Missile Launcher
                    BulletData {
                        damage: 0,
                        life: 10,
                        lifetime: 30,
                        flags: BulletFlag(40),
                        enemy_hit_width: 2,
                        enemy_hit_height: 2,
                        block_hit_width: 2,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    BulletData {
                        damage: 0,
                        life: 10,
                        lifetime: 40,
                        flags: BulletFlag(40),
                        enemy_hit_width: 4,
                        enemy_hit_height: 4,
                        block_hit_width: 4,
                        block_hit_height: 4,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    BulletData {
                        damage: 0,
                        life: 10,
                        lifetime: 40,
                        flags: BulletFlag(40),
                        enemy_hit_width: 4,
                        enemy_hit_height: 4,
                        block_hit_width: 0,
                        block_hit_height: 0,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    // Super Missile Launcher explosion
                    BulletData {
                        damage: 2,
                        life: 100,
                        lifetime: 100,
                        flags: BulletFlag(20),
                        enemy_hit_width: 12,
                        enemy_hit_height: 12,
                        block_hit_width: 0,
                        block_hit_height: 0,
                        display_bounds: Rect { left: 0, top: 0, right: 0, bottom: 0 },
                    },
                    BulletData {
                        damage: 2,
                        life: 100,
                        lifetime: 100,
                        flags: BulletFlag(20),
                        enemy_hit_width: 12,
                        enemy_hit_height: 12,
                        block_hit_width: 0,
                        block_hit_height: 0,
                        display_bounds: Rect { left: 0, top: 0, right: 0, bottom: 0 },
                    },
                    BulletData {
                        damage: 2,
                        life: 100,
                        lifetime: 100,
                        flags: BulletFlag(20),
                        enemy_hit_width: 12,
                        enemy_hit_height: 12,
                        block_hit_width: 0,
                        block_hit_height: 0,
                        display_bounds: Rect { left: 0, top: 0, right: 0, bottom: 0 },
                    },
                    // Nemesis
                    BulletData {
                        damage: 4,
                        life: 4,
                        lifetime: 20,
                        flags: BulletFlag(32),
                        enemy_hit_width: 4,
                        enemy_hit_height: 4,
                        block_hit_width: 3,
                        block_hit_height: 3,
                        display_bounds: Rect { left: 8, top: 8, right: 24, bottom: 8 },
                    },
                    BulletData {
                        damage: 4,
                        life: 2,
                        lifetime: 20,
                        flags: BulletFlag(32),
                        enemy_hit_width: 2,
                        enemy_hit_height: 2,
                        block_hit_width: 2,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 8, top: 8, right: 24, bottom: 8 },
                    },
                    BulletData {
                        damage: 1,
                        life: 1,
                        lifetime: 20,
                        flags: BulletFlag(32),
                        enemy_hit_width: 2,
                        enemy_hit_height: 2,
                        block_hit_width: 2,
                        block_hit_height: 2,
                        display_bounds: Rect { left: 8, top: 8, right: 24, bottom: 8 },
                    },
                    // Spur
                    BulletData {
                        damage: 4,
                        life: 4,
                        lifetime: 30,
                        flags: BulletFlag(64),
                        enemy_hit_width: 6,
                        enemy_hit_height: 6,
                        block_hit_width: 3,
                        block_hit_height: 3,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    BulletData {
                        damage: 8,
                        life: 8,
                        lifetime: 30,
                        flags: BulletFlag(64),
                        enemy_hit_width: 6,
                        enemy_hit_height: 6,
                        block_hit_width: 3,
                        block_hit_height: 3,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    BulletData {
                        damage: 12,
                        life: 12,
                        lifetime: 30,
                        flags: BulletFlag(64),
                        enemy_hit_width: 6,
                        enemy_hit_height: 6,
                        block_hit_width: 3,
                        block_hit_height: 3,
                        display_bounds: Rect { left: 8, top: 8, right: 8, bottom: 8 },
                    },
                    // Spur trail
                    BulletData {
                        damage: 3,
                        life: 100,
                        lifetime: 30,
                        flags: BulletFlag(32),
                        enemy_hit_width: 6,
                        enemy_hit_height: 6,
                        block_hit_width: 3,
                        block_hit_height: 3,
                        display_bounds: Rect { left: 4, top: 4, right: 4, bottom: 4 },
                    },
                    BulletData {
                        damage: 6,
                        life: 100,
                        lifetime: 30,
                        flags: BulletFlag(32),
                        enemy_hit_width: 6,
                        enemy_hit_height: 6,
                        block_hit_width: 3,
                        block_hit_height: 3,
                        display_bounds: Rect { left: 4, top: 4, right: 4, bottom: 4 },
                    },
                    BulletData {
                        damage: 11,
                        life: 100,
                        lifetime: 30,
                        flags: BulletFlag(32),
                        enemy_hit_width: 6,
                        enemy_hit_height: 6,
                        block_hit_width: 3,
                        block_hit_height: 3,
                        display_bounds: Rect { left: 4, top: 4, right: 4, bottom: 4 },
                    },
                    // Curly's Nemesis
                    BulletData {
                        damage: 4,
                        life: 4,
                        lifetime: 20,
                        flags: BulletFlag(32),
                        enemy_hit_width: 4,
                        enemy_hit_height: 4,
                        block_hit_width: 3,
                        block_hit_height: 3,
                        display_bounds: Rect { left: 8, top: 8, right: 24, bottom: 8 },
                    },
                    // EnemyClear?
                    BulletData {
                        damage: 0,
                        life: 4,
                        lifetime: 4,
                        flags: BulletFlag(4),
                        enemy_hit_width: 0,
                        enemy_hit_height: 0,
                        block_hit_width: 0,
                        block_hit_height: 0,
                        display_bounds: Rect { left: 0, top: 0, right: 0, bottom: 0 },
                    },
                    // Whimsical Star
                    BulletData {
                        damage: 1,
                        life: 1,
                        lifetime: 1,
                        flags: BulletFlag(36),
                        enemy_hit_width: 1,
                        enemy_hit_height: 1,
                        block_hit_width: 1,
                        block_hit_height: 1,
                        display_bounds: Rect { left: 1, top: 1, right: 1, bottom: 1 },
                    },
                ],
                bullet_rects: BulletRects {
                    b001_snake_l1: [
                        Rect { left: 136, top: 80, right: 152, bottom: 80 }, // left
                        Rect { left: 120, top: 80, right: 136, bottom: 96 },
                        Rect { left: 136, top: 64, right: 152, bottom: 80 },
                        Rect { left: 120, top: 64, right: 136, bottom: 80 },
                        Rect { left: 120, top: 64, right: 136, bottom: 80 }, // right
                        Rect { left: 136, top: 64, right: 152, bottom: 80 },
                        Rect { left: 120, top: 80, right: 136, bottom: 96 },
                        Rect { left: 136, top: 80, right: 152, bottom: 80 },
                    ],
                    b002_003_snake_l2_3: [
                        Rect { left: 192, top: 16, right: 208, bottom: 32 },
                        Rect { left: 208, top: 16, right: 224, bottom: 32 },
                        Rect { left: 224, top: 16, right: 240, bottom: 32 },
                    ],
                    b004_polar_star_l1: [
                        Rect { left: 128, top: 32, right: 144, bottom: 48 }, // horizontal
                        Rect { left: 144, top: 32, right: 160, bottom: 48 }, // vertical
                    ],
                    b005_polar_star_l2: [
                        Rect { left: 160, top: 32, right: 176, bottom: 48 }, // horizontal
                        Rect { left: 176, top: 32, right: 192, bottom: 48 }, // vertical
                    ],
                    b006_polar_star_l3: [
                        Rect { left: 128, top: 48, right: 144, bottom: 64 }, // horizontal
                        Rect { left: 144, top: 48, right: 160, bottom: 64 }, // vertical
                    ],
                    b007_fireball_l1: [
                        Rect { left: 128, top: 0, right: 144, bottom: 16 }, // left
                        Rect { left: 144, top: 0, right: 160, bottom: 16 },
                        Rect { left: 160, top: 0, right: 176, bottom: 16 },
                        Rect { left: 176, top: 0, right: 192, bottom: 16 },
                        Rect { left: 128, top: 16, right: 144, bottom: 32 }, // right
                        Rect { left: 144, top: 16, right: 160, bottom: 32 },
                        Rect { left: 160, top: 16, right: 176, bottom: 32 },
                        Rect { left: 176, top: 16, right: 192, bottom: 32 },
                    ],
                    b008_009_fireball_l2_3: [
                        Rect { left: 192, top: 16, right: 208, bottom: 32 }, // left
                        Rect { left: 208, top: 16, right: 224, bottom: 32 },
                        Rect { left: 224, top: 16, right: 240, bottom: 32 },
                        Rect { left: 224, top: 16, right: 240, bottom: 32 }, // right
                        Rect { left: 208, top: 16, right: 224, bottom: 32 },
                        Rect { left: 192, top: 16, right: 208, bottom: 32 },
                    ],
                    b010_machine_gun_l1: [
                        Rect { left: 64, top: 0, right: 80, bottom: 16 },
                        Rect { left: 80, top: 0, right: 96, bottom: 16 },
                        Rect { left: 96, top: 0, right: 112, bottom: 16 },
                        Rect { left: 112, top: 0, right: 128, bottom: 16 },
                    ],
                    b011_machine_gun_l2: [
                        Rect { left: 64, top: 16, right: 80, bottom: 32 },
                        Rect { left: 80, top: 16, right: 96, bottom: 32 },
                        Rect { left: 96, top: 16, right: 112, bottom: 32 },
                        Rect { left: 112, top: 16, right: 128, bottom: 32 },
                    ],
                    b012_machine_gun_l3: [
                        Rect { left: 64, top: 32, right: 80, bottom: 48 },
                        Rect { left: 80, top: 32, right: 96, bottom: 48 },
                        Rect { left: 96, top: 32, right: 112, bottom: 48 },
                        Rect { left: 112, top: 32, right: 128, bottom: 48 },
                    ],
                    b013_missile_l1: [
                        Rect { left: 0, top: 0, right: 16, bottom: 16 },
                        Rect { left: 16, top: 0, right: 32, bottom: 16 },
                        Rect { left: 32, top: 0, right: 48, bottom: 16 },
                        Rect { left: 48, top: 0, right: 64, bottom: 16 },
                    ],
                    b014_missile_l2: [
                        Rect { left: 0, top: 16, right: 16, bottom: 32 },
                        Rect { left: 16, top: 16, right: 32, bottom: 32 },
                        Rect { left: 32, top: 16, right: 48, bottom: 32 },
                        Rect { left: 48, top: 16, right: 64, bottom: 32 },
                    ],
                    b015_missile_l3: [
                        Rect { left: 0, top: 32, right: 16, bottom: 48 },
                        Rect { left: 16, top: 32, right: 32, bottom: 48 },
                        Rect { left: 32, top: 32, right: 48, bottom: 48 },
                        Rect { left: 48, top: 32, right: 64, bottom: 48 },
                    ],
                    b019_bubble_l1: [
                        Rect { left: 192, top: 0, right: 200, bottom: 8 },
                        Rect { left: 200, top: 0, right: 208, bottom: 8 },
                        Rect { left: 208, top: 0, right: 216, bottom: 8 },
                        Rect { left: 216, top: 0, right: 224, bottom: 8 },
                    ],
                    b020_bubble_l2: [
                        Rect { left: 192, top: 8, right: 200, bottom: 16 },
                        Rect { left: 200, top: 8, right: 208, bottom: 16 },
                        Rect { left: 208, top: 8, right: 216, bottom: 16 },
                        Rect { left: 216, top: 8, right: 224, bottom: 16 },
                    ],
                    b021_bubble_l3: [
                        Rect { left: 240, top: 16, right: 248, bottom: 24 },
                        Rect { left: 248, top: 16, right: 256, bottom: 24 },
                        Rect { left: 240, top: 24, right: 248, bottom: 32 },
                        Rect { left: 248, top: 24, right: 256, bottom: 32 },
                    ],
                    b022_bubble_spines: [
                        Rect { left: 224, top: 0, right: 232, bottom: 8 },
                        Rect { left: 232, top: 0, right: 240, bottom: 8 },
                        Rect { left: 224, top: 0, right: 232, bottom: 8 },
                        Rect { left: 232, top: 0, right: 240, bottom: 8 },
                        Rect { left: 224, top: 8, right: 232, bottom: 16 },
                        Rect { left: 232, top: 8, right: 240, bottom: 16 },
                    ],
                    b023_blade_slash: [
                        Rect { left: 0, top: 64, right: 24, bottom: 88 }, // left
                        Rect { left: 24, top: 64, right: 48, bottom: 88 },
                        Rect { left: 48, top: 64, right: 72, bottom: 88 },
                        Rect { left: 72, top: 64, right: 96, bottom: 88 },
                        Rect { left: 96, top: 64, right: 120, bottom: 88 },
                        Rect { left: 0, top: 88, right: 24, bottom: 112 }, // right
                        Rect { left: 24, top: 88, right: 48, bottom: 112 },
                        Rect { left: 48, top: 88, right: 72, bottom: 112 },
                        Rect { left: 72, top: 88, right: 96, bottom: 112 },
                        Rect { left: 96, top: 88, right: 120, bottom: 112 },
                    ],
                    b025_blade_l1: [
                        Rect { left: 0, top: 48, right: 16, bottom: 64 }, // left
                        Rect { left: 16, top: 48, right: 32, bottom: 64 },
                        Rect { left: 32, top: 48, right: 48, bottom: 64 },
                        Rect { left: 48, top: 48, right: 64, bottom: 64 },
                        Rect { left: 64, top: 48, right: 80, bottom: 64 }, // other directions
                        Rect { left: 80, top: 48, right: 96, bottom: 64 },
                        Rect { left: 96, top: 48, right: 112, bottom: 64 },
                        Rect { left: 112, top: 48, right: 128, bottom: 64 },
                    ],
                    b026_blade_l2: [
                        Rect { left: 160, top: 48, right: 184, bottom: 72 }, // left
                        Rect { left: 184, top: 48, right: 208, bottom: 72 },
                        Rect { left: 208, top: 48, right: 232, bottom: 72 },
                        Rect { left: 232, top: 48, right: 256, bottom: 72 },
                        Rect { left: 160, top: 72, right: 184, bottom: 96 }, // other directions
                        Rect { left: 184, top: 72, right: 208, bottom: 96 },
                        Rect { left: 208, top: 72, right: 232, bottom: 96 },
                        Rect { left: 232, top: 72, right: 256, bottom: 96 },
                    ],
                    b027_blade_l3: [
                        Rect { left: 272, top: 0, right: 296, bottom: 24 }, // left
                        Rect { left: 296, top: 0, right: 320, bottom: 24 },
                        Rect { left: 272, top: 48, right: 296, bottom: 72 }, // up
                        Rect { left: 296, top: 0, right: 320, bottom: 24 },
                        Rect { left: 272, top: 24, right: 296, bottom: 48 }, // right
                        Rect { left: 296, top: 24, right: 320, bottom: 48 },
                        Rect { left: 296, top: 48, right: 320, bottom: 72 }, // down
                        Rect { left: 296, top: 24, right: 320, bottom: 48 },
                    ],
                    b028_super_missile_l1: [
                        Rect { left: 120, top: 96, right: 136, bottom: 112 },
                        Rect { left: 136, top: 96, right: 152, bottom: 112 },
                        Rect { left: 152, top: 96, right: 168, bottom: 112 },
                        Rect { left: 168, top: 96, right: 184, bottom: 112 },
                    ],
                    b029_super_missile_l2: [
                        Rect { left: 184, top: 96, right: 200, bottom: 112 },
                        Rect { left: 200, top: 96, right: 216, bottom: 112 },
                        Rect { left: 216, top: 96, right: 232, bottom: 112 },
                        Rect { left: 232, top: 96, right: 248, bottom: 112 },
                    ],
                    b030_super_missile_l3: [
                        Rect { left: 120, top: 96, right: 136, bottom: 112 },
                        Rect { left: 136, top: 96, right: 152, bottom: 112 },
                        Rect { left: 152, top: 96, right: 168, bottom: 112 },
                        Rect { left: 168, top: 96, right: 184, bottom: 112 },
                    ],
                    b034_nemesis_l1: [
                        Rect { left: 0, top: 112, right: 32, bottom: 128 }, // left
                        Rect { left: 0, top: 128, right: 32, bottom: 144 },
                        Rect { left: 32, top: 112, right: 48, bottom: 144 }, // up
                        Rect { left: 48, top: 112, right: 64, bottom: 144 },
                        Rect { left: 64, top: 112, right: 96, bottom: 128 }, // right
                        Rect { left: 64, top: 128, right: 96, bottom: 144 },
                        Rect { left: 96, top: 112, right: 112, bottom: 144 }, // down
                        Rect { left: 112, top: 112, right: 128, bottom: 144 },
                    ],
                    b035_nemesis_l2: [
                        Rect { left: 128, top: 112, right: 160, bottom: 128 }, // left
                        Rect { left: 128, top: 128, right: 160, bottom: 144 },
                        Rect { left: 160, top: 112, right: 176, bottom: 144 }, // up
                        Rect { left: 176, top: 112, right: 192, bottom: 144 },
                        Rect { left: 192, top: 112, right: 224, bottom: 128 }, // right
                        Rect { left: 192, top: 128, right: 224, bottom: 144 },
                        Rect { left: 224, top: 112, right: 240, bottom: 144 }, // down
                        Rect { left: 240, top: 112, right: 256, bottom: 144 },
                    ],
                    b036_nemesis_l3: [
                        Rect { left: 0, top: 144, right: 32, bottom: 160 }, // left
                        Rect { left: 0, top: 160, right: 32, bottom: 176 },
                        Rect { left: 32, top: 144, right: 48, bottom: 176 }, // up
                        Rect { left: 48, top: 144, right: 64, bottom: 176 },
                        Rect { left: 64, top: 144, right: 96, bottom: 160 }, // right
                        Rect { left: 64, top: 160, right: 96, bottom: 176 },
                        Rect { left: 96, top: 144, right: 112, bottom: 176 }, // down
                        Rect { left: 112, top: 144, right: 128, bottom: 176 },
                    ],
                    b037_spur_l1: [
                        Rect { left: 128, top: 32, right: 144, bottom: 48 }, // horizontal
                        Rect { left: 144, top: 32, right: 160, bottom: 48 }, // vertical
                    ],
                    b038_spur_l2: [
                        Rect { left: 160, top: 32, right: 176, bottom: 48 }, // horizontal
                        Rect { left: 176, top: 32, right: 192, bottom: 48 }, // vertical
                    ],
                    b039_spur_l3: [
                        Rect { left: 128, top: 48, right: 144, bottom: 64 }, // horizontal
                        Rect { left: 144, top: 48, right: 160, bottom: 64 }, // vertical
                    ],
                    b040_spur_trail_l1: [
                        Rect { left: 192, top: 32, right: 200, bottom: 40 }, // horizontal
                        Rect { left: 200, top: 32, right: 208, bottom: 40 },
                        Rect { left: 208, top: 32, right: 216, bottom: 40 },
                        Rect { left: 192, top: 40, right: 200, bottom: 48 }, // vertical
                        Rect { left: 200, top: 40, right: 208, bottom: 48 },
                        Rect { left: 208, top: 40, right: 216, bottom: 48 },
                    ],
                    b041_spur_trail_l2: [
                        Rect { left: 216, top: 32, right: 224, bottom: 40 }, // horizontal
                        Rect { left: 224, top: 32, right: 232, bottom: 40 },
                        Rect { left: 232, top: 32, right: 240, bottom: 40 },
                        Rect { left: 216, top: 40, right: 224, bottom: 48 }, // vertical
                        Rect { left: 224, top: 40, right: 232, bottom: 48 },
                        Rect { left: 232, top: 40, right: 240, bottom: 48 },
                    ],
                    b042_spur_trail_l3: [
                        Rect { left: 240, top: 32, right: 248, bottom: 40 }, // horizontal
                        Rect { left: 248, top: 32, right: 256, bottom: 40 },
                        Rect { left: 256, top: 32, right: 264, bottom: 40 },
                        Rect { left: 240, top: 32, right: 248, bottom: 40 }, // vertical
                        Rect { left: 248, top: 32, right: 256, bottom: 40 },
                        Rect { left: 256, top: 32, right: 264, bottom: 40 },
                    ],
                },
                level_table: [
                    [0, 0, 100],
                    [30, 40, 16],
                    [10, 20, 10],
                    [10, 20, 20],
                    [30, 40, 10],
                    [10, 20, 10],
                    [10, 20, 30],
                    [10, 20, 5],
                    [10, 20, 100],
                    [30, 60, 0],
                    [30, 60, 10],
                    [10, 20, 100],
                    [1, 1, 1],
                    [40, 60, 200],
                ],
            },
            tex_sizes: case_insensitive_hashmap! {
                "ArmsImage" => (256, 16),
                "Arms" => (320, 200),
                "bk0" => (64, 64),
                "bkBlack" => (64, 64),
                "bkBlue" => (64, 64),
                "bkFall" => (64, 64),
                "bkFog" => (320, 240),
                "bkFog480fix" => (480, 272), // nxengine
                "bkGard" => (48, 64),
                "bkGray" => (64, 64),
                "bkGreen" => (64, 64),
                "bkHellish" => (320, 240), // nxengine
                "bkHellish480fix" => (480, 272), // nxengine
                "bkLight" => (320, 240), // nxengine
                "bkLight480fix" => (480, 272), // nxengine
                "bkMaze" => (64, 64),
                "bkMoon" => (320, 240),
                "bkMoon480fix" => (480, 272), // nxengine
                "bkRed" => (32, 32),
                "bkSunset" => (320, 240), // nxengine
                "bkSunset480fix" => (480, 272), // nxengine
                "bkWater" => (32, 48),
                "Bullet" => (320, 176),
                "Caret" => (320, 240),
                "casts" => (320, 240),
                "Face" => (288, 240),
                "Face_0" => (288, 240), // nxengine
                "Face_1" => (288, 240), // nxengine
                "Face_2" => (288, 240), // nxengine
                "Face1" => (288, 240), // switch
                "Face2" => (288, 240), // switch
                "Face3" => (288, 240), // switch
                "Face4" => (288, 240), // switch
                "Face5" => (288, 240), // switch
                "Fade" => (256, 32),
                "ItemImage" => (256, 128),
                "Loading" => (64, 8),
                "MyChar" => (200, 64),
                "Npc/Npc0" => (32, 32),
                "Npc/NpcAlmo1" => (320, 240),
                "Npc/NpcAlmo2" => (320, 240),
                "Npc/NpcBallos" => (320, 240),
                "Npc/NpcBllg" => (320, 96),
                "Npc/NpcCemet" => (320, 112),
                "Npc/NpcCent" => (320, 192),
                "Npc/NpcCurly" => (256, 80),
                "Npc/NpcDark" => (160, 64),
                "Npc/NpcDr" => (320, 240),
                "Npc/NpcEggs1" => (320, 112),
                "Npc/NpcEggs2" => (320, 128),
                "Npc/NpcFrog" => (320, 240),
                "Npc/NpcGuest" => (320, 184),
                "Npc/NpcHell" => (320, 160),
                "Npc/NpcHeri" => (320, 128),
                "Npc/NpcIronH" => (320, 72),
                "Npc/NpcIsland" => (320, 80),
                "Npc/NpcKaze" => (320, 240),
                "Npc/NpcKings" => (96, 48),
                "Npc/NpcMaze" => (320, 192),
                "Npc/NpcMiza" => (320, 240),
                "Npc/NpcMoon" => (320, 128),
                "Npc/NpcOmg" => (320, 120),
                "Npc/NpcPlant" => (320, 48),
                "Npc/NpcPress" => (320, 240),
                "Npc/NpcPriest" => (320, 240),
                "Npc/NpcRavil" => (320, 168),
                "Npc/NpcRed" => (320, 144),
                "Npc/NpcRegu" => (320, 240),
                "Npc/NpcSand" => (320, 176),
                "Npc/NpcStream" => (64, 32),
                "Npc/NpcSym" => (320, 240),
                "Npc/NpcToro" => (320, 144),
                "Npc/NpcTwinD" => (320, 144),
                "Npc/NpcWeed" => (320, 240),
                "Npc/NpcX" => (320, 240),
                "Resource/BITMAP/Credit01" => (160, 240), // cse2
                "Resource/BITMAP/Credit02" => (160, 240), // cse2
                "Resource/BITMAP/Credit03" => (160, 240), // cse2
                "Resource/BITMAP/Credit04" => (160, 240), // cse2
                "Resource/BITMAP/Credit05" => (160, 240), // cse2
                "Resource/BITMAP/Credit06" => (160, 240), // cse2
                "Resource/BITMAP/Credit07" => (160, 240), // cse2
                "Resource/BITMAP/Credit08" => (160, 240), // cse2
                "Resource/BITMAP/Credit09" => (160, 240), // cse2
                "Resource/BITMAP/Credit10" => (160, 240), // cse2
                "Resource/BITMAP/Credit11" => (160, 240), // cse2
                "Resource/BITMAP/Credit12" => (160, 240), // cse2
                "Resource/BITMAP/Credit14" => (160, 240), // cse2
                "Resource/BITMAP/Credit15" => (160, 240), // cse2
                "Resource/BITMAP/Credit16" => (160, 240), // cse2
                "Resource/BITMAP/Credit17" => (160, 240), // cse2
                "Resource/BITMAP/Credit18" => (160, 240), // cse2
                "Resource/BITMAP/pixel" => (160, 16), // cse2
                "StageImage" => (256, 16),
                "Stage/Prt0" => (32, 32),
                "Stage/PrtAlmond" => (256, 96),
                "Stage/PrtBarr" => (256, 88),
                "Stage/PrtCave" => (256, 80),
                "Stage/PrtCent" => (256, 128),
                "Stage/PrtEggIn" => (256, 80),
                "Stage/PrtEggs" => (256, 240),
                "Stage/PrtEggX" => (256, 240),
                "Stage/PrtFall" => (256, 128),
                "Stage/PrtGard" => (256, 97),
                "Stage/PrtHell" => (256, 240),
                "Stage/PrtHellStatue" => (256, 256),
                "Stage/PrtJail" => (256, 128),
                "Stage/PrtLabo" => (128, 64),
                "Stage/PrtMaze" => (256, 160),
                "Stage/PrtMimi" => (256, 160),
                "Stage/PrtOside" => (256, 64),
                "Stage/PrtPens" => (256, 64),
                "Stage/PrtRiver" => (256, 96),
                "Stage/PrtSand" => (256, 112),
                "Stage/PrtStore" => (256, 112),
                "Stage/PrtWeed" => (256, 128),
                "Stage/PrtWhite" => (256, 240),
                "TextBox" => (244, 144),
                "Title" => (320, 48),
            },
            textscript: TextScriptConsts {
                encoding: TextScriptEncoding::ShiftJIS,
                encrypted: true,
                animated_face_pics: false,
                textbox_rect_top: Rect { left: 0, top: 0, right: 244, bottom: 8 },
                textbox_rect_middle: Rect { left: 0, top: 8, right: 244, bottom: 16 },
                textbox_rect_bottom: Rect { left: 0, top: 16, right: 244, bottom: 24 },
                textbox_rect_yes_no: Rect { left: 152, top: 48, right: 244, bottom: 80 },
                textbox_rect_cursor: Rect { left: 112, top: 88, right: 128, bottom: 104 },
                inventory_rect_top: Rect { left: 0, top: 0, right: 244, bottom: 8 },
                inventory_rect_middle: Rect { left: 0, top: 8, right: 244, bottom: 16 },
                inventory_rect_bottom: Rect { left: 0, top: 16, right: 244, bottom: 24 },
                inventory_text_arms: Rect { left: 80, top: 48, right: 144, bottom: 56 },
                inventory_text_item: Rect { left: 80, top: 56, right: 144, bottom: 64 },
                get_item_top_left: Rect { left: 0, top: 0, right: 72, bottom: 16 },
                get_item_bottom_left: Rect { left: 0, top: 8, right: 72, bottom: 24 },
                get_item_top_right: Rect { left: 240, top: 0, right: 244, bottom: 8 },
                get_item_right: Rect { left: 240, top: 8, right: 244, bottom: 16 },
                get_item_bottom_right: Rect { left: 240, top: 16, right: 244, bottom: 24 },
                stage_select_text: Rect { left: 80, top: 64, right: 144, bottom: 72 },
                cursor: [
                    Rect { left: 80, top: 88, right: 112, bottom: 104 },
                    Rect { left: 80, top: 104, right: 112, bottom: 120 },
                ],
                cursor_inventory_weapon: [
                    Rect { left: 0, top: 88, right: 40, bottom: 128 },
                    Rect { left: 40, top: 88, right: 80, bottom: 128 },
                ],
                cursor_inventory_item: [
                    Rect { left: 80, top: 88, right: 112, bottom: 104 },
                    Rect { left: 80, top: 104, right: 112, bottom: 120 },
                ],
                inventory_item_count_x: 6,
                text_shadow: false,
                text_speed_normal: 4,
                text_speed_fast: 1,
            },
            title: TitleConsts {
                intro_text: "Studio Pixel presents".to_string(),
                logo_rect: Rect { left: 0, top: 0, right: 144, bottom: 40 },
                menu_left_top: Rect { left: 0, top: 0, right: 8, bottom: 8 },
                menu_right_top: Rect { left: 236, top: 0, right: 244, bottom: 8 },
                menu_left_bottom: Rect { left: 0, top: 16, right: 8, bottom: 24 },
                menu_right_bottom: Rect { left: 236, top: 16, right: 244, bottom: 24 },
                menu_top: Rect { left: 8, top: 0, right: 236, bottom: 8 },
                menu_middle: Rect { left: 8, top: 8, right: 236, bottom: 16 },
                menu_bottom: Rect { left: 8, top: 16, right: 236, bottom: 24 },
                menu_left: Rect { left: 0, top: 8, right: 8, bottom: 16 },
                menu_right: Rect { left: 236, top: 8, right: 244, bottom: 16 },
            },
            inventory_dim_color: Color::from_rgba(0, 0, 0, 0),
            font_path: "csfont.fnt".to_string(),
            font_scale: 1.0,
            font_space_offset: 0.0,
            soundtracks: HashMap::new(),
            music_table: vec![
                "xxxx".to_string(),
                "wanpaku".to_string(),
                "anzen".to_string(),
                "gameover".to_string(),
                "gravity".to_string(),
                "weed".to_string(),
                "mdown2".to_string(),
                "fireeye".to_string(),
                "vivi".to_string(),
                "mura".to_string(),
                "fanfale1".to_string(),
                "ginsuke".to_string(),
                "cemetery".to_string(),
                "plant".to_string(),
                "kodou".to_string(),
                "fanfale3".to_string(),
                "fanfale2".to_string(),
                "dr".to_string(),
                "escape".to_string(),
                "jenka".to_string(),
                "maze".to_string(),
                "access".to_string(),
                "ironh".to_string(),
                "grand".to_string(),
                "curly".to_string(),
                "oside".to_string(),
                "requiem".to_string(),
                "wanpak2".to_string(),
                "quiet".to_string(),
                "lastcave".to_string(),
                "balcony".to_string(),
                "lastbtl".to_string(),
                "lastbt3".to_string(),
                "ending".to_string(),
                "zonbie".to_string(),
                "bdown".to_string(),
                "hell".to_string(),
                "jenka2".to_string(),
                "marine".to_string(),
                "ballos".to_string(),
                "toroko".to_string(),
                "white".to_string(),
                "kaze".to_string(),
            ],
            organya_paths: vec![
                "/org/".to_string(),          // NXEngine
                "/base/Org/".to_string(),     // CS+
                "/Resource/ORG/".to_string(), // CSE2E
            ],
        }
    }

    pub fn apply_csplus_patches(&mut self, sound_manager: &SoundManager) {
        info!("Applying Cave Story+ constants patches...");

        self.is_cs_plus = true;
        self.supports_og_textures = true;
        self.tex_sizes.insert(str!("Caret"), (320, 320));
        self.tex_sizes.insert(str!("MyChar"), (200, 384));
        self.tex_sizes.insert(str!("Npc/NpcRegu"), (320, 410));
        self.title.logo_rect = Rect { left: 0, top: 0, right: 214, bottom: 50 };
        self.font_path = str!("csfont.fnt");
        self.font_scale = 0.5;
        self.font_space_offset = 2.0;
        self.soundtracks.insert("Remastered".to_string(), "/base/Ogg11/".to_string());
        self.soundtracks.insert("New".to_string(), "/base/Ogg/".to_string());

        let typewriter_sample = PixToneParameters {
            // fx2 (CS+)
            channels: [
                Channel {
                    enabled: true,
                    length: 2000,
                    carrier: Waveform { waveform_type: 0, pitch: 92.000000, level: 32, offset: 0 },
                    frequency: Waveform { waveform_type: 0, pitch: 3.000000, level: 44, offset: 0 },
                    amplitude: Waveform { waveform_type: 0, pitch: 0.000000, level: 32, offset: 0 },
                    envelope: Envelope {
                        initial: 7,
                        time_a: 2,
                        value_a: 18,
                        time_b: 128,
                        value_b: 0,
                        time_c: 255,
                        value_c: 0,
                    },
                },
                Channel::disabled(),
                Channel::disabled(),
                Channel::disabled(),
            ],
        };

        sound_manager.set_sample_params(2, typewriter_sample);
    }

    pub fn apply_csplus_nx_patches(&mut self) {
        info!("Applying Switch-specific Cave Story+ constants patches...");

        self.is_switch = true;
        self.supports_og_textures = true;
        self.tex_sizes.insert(str!("bkMoon"), (427, 240));
        self.tex_sizes.insert(str!("bkFog"), (427, 240));
        self.title.logo_rect = Rect { left: 0, top: 0, right: 214, bottom: 62 };
        self.inventory_dim_color = Color::from_rgba(0, 0, 32, 150);
        self.textscript.encoding = TextScriptEncoding::UTF8;
        self.textscript.encrypted = false;
        self.textscript.animated_face_pics = true;
        self.textscript.text_shadow = true;
        self.textscript.text_speed_normal = 1;
        self.textscript.text_speed_fast = 0;
        self.soundtracks.insert("Famitracks".to_string(), "/base/ogg17/".to_string());
        self.soundtracks.insert("Ridiculon".to_string(), "/base/ogg_ridic/".to_string());
    }
}
