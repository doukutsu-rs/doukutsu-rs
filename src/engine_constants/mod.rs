use std::collections::HashMap;
use std::io::{BufRead, BufReader, Cursor, Read};

use byteorder::{ReadBytesExt, LE};
use case_insensitive_hashmap::CaseInsensitiveHashMap;
use xmltree::Element;

use crate::case_insensitive_hashmap;
use crate::common::{BulletFlag, Color, Colorf, Rect};
use crate::engine_constants::npcs::NPCConsts;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::framework::gamepad::{Axis, Button};
use crate::game::player::ControlMode;
use crate::game::scripting::tsc::text_script::TextScriptEncoding;
use crate::game::settings::Settings;
use crate::game::shared_game_state::{FontData, Season};
use crate::i18n::Locale;
use crate::sound::pixtone::{Channel, Envelope, PixToneParameters, Waveform};
use crate::sound::SoundManager;

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

#[derive(Debug, Copy, Clone)]
pub struct GameConsts {
    pub intro_stage: u16,
    pub intro_event: u16,
    pub intro_player_pos: (i16, i16),
    pub new_game_stage: u16,
    pub new_game_event: u16,
    pub new_game_player_pos: (i16, i16),
    pub tile_offset_x: i32,
}

#[derive(Debug, Clone)]
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
    pub small_projectile_dissipation: Vec<Rect<u16>>,
    pub empty_text: Vec<Rect<u16>>,
    pub push_jump_key: Vec<Rect<u16>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TextureSizeTable {
    sizes: HashMap<String, (u16, u16)>,
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

#[derive(Debug, Clone)]
pub struct WeaponConsts {
    pub bullet_table: Vec<BulletData>,
    pub bullet_rects: BulletRects,
    pub level_table: [[u16; 3]; 14],
}

#[derive(Debug, Copy, Clone)]
pub struct WorldConsts {
    pub snack_rect: Rect<u16>,
    pub water_push_rect: Rect<u16>,
}

#[derive(Debug, Clone)]
pub struct AnimatedFace {
    pub face_id: u16,
    pub anim_id: u16,
    pub anim_frames: Vec<(u16, u16)>,
}

#[derive(Debug, Clone)]
pub struct ExtraSoundtrack {
    pub id: String,
    pub path: String,
    pub available: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct TextScriptConsts {
    pub encoding: TextScriptEncoding,
    pub encrypted: bool,
    pub reset_invicibility_on_any_script: bool,
    pub animated_face_pics: bool,
    pub textbox_rect_top: Rect<u16>,
    pub textbox_rect_middle: Rect<u16>,
    pub textbox_rect_bottom: Rect<u16>,
    pub textbox_rect_yes_no: Rect<u16>,
    pub textbox_rect_cursor: Rect<u16>,
    pub textbox_item_marker_rect: Rect<u16>,
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
    pub fade_ticks: i8,
}

#[derive(Debug, Clone)]
pub struct TitleConsts {
    pub intro_text: String,
    pub logo_rect: Rect<u16>,
    pub logo_splash_rect: Rect<u16>,
    pub menu_left_top: Rect<u16>,
    pub menu_right_top: Rect<u16>,
    pub menu_left_bottom: Rect<u16>,
    pub menu_right_bottom: Rect<u16>,
    pub menu_top: Rect<u16>,
    pub menu_bottom: Rect<u16>,
    pub menu_middle: Rect<u16>,
    pub menu_left: Rect<u16>,
    pub menu_right: Rect<u16>,
    pub cursor_quote: [Rect<u16>; 4],
    pub cursor_curly: [Rect<u16>; 4],
    pub cursor_toroko: [Rect<u16>; 4],
    pub cursor_king: [Rect<u16>; 4],
    pub cursor_sue: [Rect<u16>; 4],
}

#[derive(Debug, Clone)]
pub struct GamepadConsts {
    pub button_rects: HashMap<Button, [Rect<u16>; 4]>,
    pub axis_rects: HashMap<Axis, [Rect<u16>; 4]>,
}

impl GamepadConsts {
    fn rects(base: Rect<u16>) -> [Rect<u16>; 4] {
        [
            base,
            Rect::new(base.left + 64, base.top, base.right + 64, base.bottom),
            Rect::new(base.left + 128, base.top, base.right + 128, base.bottom),
            Rect::new(base.left + 64, base.top + 128, base.right + 64, base.bottom + 128),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct EngineConstants {
    pub base_paths: Vec<String>,
    pub is_cs_plus: bool,
    pub is_switch: bool,
    pub is_demo: bool,
    pub supports_og_textures: bool,
    pub has_difficulty_menu: bool,
    pub supports_two_player: bool,
    pub game: GameConsts,
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
    pub font_space_offset: f32,
    pub soundtracks: Vec<ExtraSoundtrack>,
    pub music_table: Vec<String>,
    pub organya_paths: Vec<String>,
    pub credit_illustration_paths: Vec<String>,
    pub player_skin_paths: Vec<String>,
    pub animated_face_table: Vec<AnimatedFace>,
    pub string_table: HashMap<String, String>,
    pub missile_flags: Vec<u16>,
    pub locales: Vec<Locale>,
    pub gamepad: GamepadConsts,
    pub stage_encoding: Option<TextScriptEncoding>,
}

impl EngineConstants {
    pub fn defaults() -> Self {
        EngineConstants {
            base_paths: Vec::new(),
            is_cs_plus: false,
            is_switch: false,
            is_demo: false,
            supports_og_textures: false,
            has_difficulty_menu: true,
            supports_two_player: cfg!(not(target_os = "android")),
            game: GameConsts {
                intro_stage: 72,
                intro_event: 100,
                intro_player_pos: (3, 3),
                new_game_stage: 13,
                new_game_event: 200,
                new_game_player_pos: (10, 8),
                tile_offset_x: 0,
            },
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
                small_projectile_dissipation: vec![
                    Rect { left: 0, top: 72, right: 8, bottom: 80 },
                    Rect { left: 8, top: 72, right: 16, bottom: 80 },
                    Rect { left: 16, top: 72, right: 24, bottom: 80 },
                    Rect { left: 24, top: 72, right: 32, bottom: 80 },
                ],
                empty_text: vec![
                    Rect { left: 104, top: 96, right: 144, bottom: 104 },
                    Rect { left: 104, top: 104, right: 144, bottom: 112 },
                ],
                push_jump_key: vec![
                    Rect { left: 0, top: 144, right: 144, bottom: 152 },
                    Rect { left: 0, top: 0, right: 0, bottom: 0 },
                ],
            },
            world: WorldConsts {
                snack_rect: Rect { left: 256, top: 48, right: 272, bottom: 64 },
                water_push_rect: Rect { left: 224, top: 48, right: 240, bottom: 64 },
            },
            npc: serde_json::from_str("{}").unwrap(),
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
                        flags: BulletFlag(0x24),
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
                        flags: BulletFlag(0x24),
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
                        flags: BulletFlag(0x24),
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
                        flags: BulletFlag(0x20),
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
                        flags: BulletFlag(0x20),
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
                        flags: BulletFlag(0x20),
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
                        flags: BulletFlag(0x08),
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
                        flags: BulletFlag(0x08),
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
                        flags: BulletFlag(0x08),
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
                        flags: BulletFlag(0x20),
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
                        flags: BulletFlag(0x20),
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
                        flags: BulletFlag(0x20),
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
                        flags: BulletFlag(0x28),
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
                        flags: BulletFlag(0x28),
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
                        flags: BulletFlag(0x28),
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
                        flags: BulletFlag(0x14),
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
                        flags: BulletFlag(0x14),
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
                        flags: BulletFlag(0x14),
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
                        flags: BulletFlag(0x08),
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
                        flags: BulletFlag(0x08),
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
                        flags: BulletFlag(0x08),
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
                        flags: BulletFlag(0x20),
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
                        flags: BulletFlag(0x24),
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
                        flags: BulletFlag(0x04),
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
                        flags: BulletFlag(0x24),
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
                        flags: BulletFlag(0x24),
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
                        flags: BulletFlag(0x24),
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
                        flags: BulletFlag(0x28),
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
                        flags: BulletFlag(0x28),
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
                        flags: BulletFlag(0x28),
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
                        flags: BulletFlag(0x14),
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
                        flags: BulletFlag(0x14),
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
                        flags: BulletFlag(0x14),
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
                        flags: BulletFlag(0x20),
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
                        flags: BulletFlag(0x20),
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
                        flags: BulletFlag(0x20),
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
                        flags: BulletFlag(0x40),
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
                        flags: BulletFlag(0x40),
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
                        flags: BulletFlag(0x40),
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
                        flags: BulletFlag(0x20),
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
                        flags: BulletFlag(0x20),
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
                        flags: BulletFlag(0x20),
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
                        flags: BulletFlag(0x20),
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
                        flags: BulletFlag(0x04),
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
                        flags: BulletFlag(0x24),
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
                "buttons" => (256, 256),
                "Bullet" => (320, 176),
                "Caret" => (320, 240),
                "casts" => (320, 240),
                "Credit01" => (160, 240),
                "Credit01a" => (160, 240),
                "Credit02" => (160, 240),
                "Credit02a" => (160, 240),
                "Credit03" => (160, 240),
                "Credit03a" => (160, 240),
                "Credit04" => (160, 240),
                "Credit05" => (160, 240),
                "Credit06" => (160, 240),
                "Credit07" => (160, 240),
                "Credit08" => (160, 240),
                "Credit09" => (160, 240),
                "Credit10" => (160, 240),
                "Credit11" => (160, 240),
                "Credit12" => (160, 240),
                "Credit13" => (160, 240),
                "Credit14" => (160, 240),
                "Credit15" => (160, 240),
                "Credit16" => (160, 240),
                "Credit17" => (160, 240),
                "Credit18" => (160, 240),
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
                "headband/ogph/Casts" => (320, 240),
                "headband/ogph/Npc/NpcGuest" => (320, 184),
                "headband/ogph/Npc/NpcMiza" => (320, 240),
                "headband/ogph/Npc/NpcRegu" => (320, 240),
                "headband/plus/Casts" => (320, 240),
                "headband/plus/Npc/NpcGuest" => (320, 184),
                "headband/plus/Npc/NpcMiza" => (320, 240),
                "headband/plus/Npc/NpcRegu" => (320, 240),
                "ItemImage" => (256, 128),
                "Loading" => (64, 8),
                "MyChar" => (200, 64),
                "mychar_p2" => (200, 384), // switch
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
                "triangles" => (20, 5),
            },
            textscript: TextScriptConsts {
                encoding: TextScriptEncoding::ShiftJIS,
                encrypted: true,
                reset_invicibility_on_any_script: true,
                animated_face_pics: false,
                textbox_rect_top: Rect { left: 0, top: 0, right: 244, bottom: 8 },
                textbox_rect_middle: Rect { left: 0, top: 8, right: 244, bottom: 16 },
                textbox_rect_bottom: Rect { left: 0, top: 16, right: 244, bottom: 24 },
                textbox_rect_yes_no: Rect { left: 152, top: 48, right: 244, bottom: 80 },
                textbox_rect_cursor: Rect { left: 112, top: 88, right: 128, bottom: 104 },
                textbox_item_marker_rect: Rect { left: 64, top: 48, right: 70, bottom: 54 },
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
                fade_ticks: 15,
            },
            title: TitleConsts {
                intro_text: "Studio Pixel presents".to_owned(),
                logo_rect: Rect { left: 0, top: 0, right: 144, bottom: 40 },
                logo_splash_rect: Rect { left: 0, top: 0, right: 0, bottom: 0 }, //Hidden so patches can display splash art / subtitle
                menu_left_top: Rect { left: 0, top: 0, right: 8, bottom: 8 },
                menu_right_top: Rect { left: 236, top: 0, right: 244, bottom: 8 },
                menu_left_bottom: Rect { left: 0, top: 16, right: 8, bottom: 24 },
                menu_right_bottom: Rect { left: 236, top: 16, right: 244, bottom: 24 },
                menu_top: Rect { left: 8, top: 0, right: 232, bottom: 8 },
                menu_middle: Rect { left: 8, top: 8, right: 236, bottom: 16 },
                menu_bottom: Rect { left: 8, top: 16, right: 236, bottom: 24 },
                menu_left: Rect { left: 0, top: 8, right: 8, bottom: 16 },
                menu_right: Rect { left: 236, top: 8, right: 244, bottom: 16 },
                cursor_quote: [
                    Rect { left: 0, top: 16, right: 16, bottom: 32 },
                    Rect { left: 16, top: 16, right: 32, bottom: 32 },
                    Rect { left: 0, top: 16, right: 16, bottom: 32 },
                    Rect { left: 32, top: 16, right: 48, bottom: 32 },
                ],
                cursor_curly: [
                    Rect { left: 0, top: 112, right: 16, bottom: 128 },
                    Rect { left: 16, top: 112, right: 32, bottom: 128 },
                    Rect { left: 0, top: 112, right: 16, bottom: 128 },
                    Rect { left: 32, top: 112, right: 48, bottom: 128 },
                ],
                cursor_toroko: [
                    Rect { left: 64, top: 80, right: 80, bottom: 96 },
                    Rect { left: 80, top: 80, right: 96, bottom: 96 },
                    Rect { left: 64, top: 80, right: 80, bottom: 96 },
                    Rect { left: 96, top: 80, right: 112, bottom: 96 },
                ],
                cursor_king: [
                    Rect { left: 224, top: 48, right: 240, bottom: 64 },
                    Rect { left: 288, top: 48, right: 304, bottom: 64 },
                    Rect { left: 224, top: 48, right: 240, bottom: 64 },
                    Rect { left: 304, top: 48, right: 320, bottom: 64 },
                ],
                cursor_sue: [
                    Rect { left: 0, top: 16, right: 16, bottom: 32 },
                    Rect { left: 32, top: 16, right: 48, bottom: 32 },
                    Rect { left: 0, top: 16, right: 16, bottom: 32 },
                    Rect { left: 48, top: 16, right: 64, bottom: 32 },
                ],
            },
            inventory_dim_color: Color::from_rgba(0, 0, 0, 0),
            font_path: "csfont.fnt".to_owned(),
            font_space_offset: 0.0,
            soundtracks: vec![
                ExtraSoundtrack { id: "remastered".to_owned(), path: "/base/Ogg11/".to_owned(), available: false },
                ExtraSoundtrack { id: "new".to_owned(), path: "/base/Ogg/".to_owned(), available: false },
                ExtraSoundtrack { id: "famitracks".to_owned(), path: "/base/ogg17/".to_owned(), available: false },
                ExtraSoundtrack { id: "ridiculon".to_owned(), path: "/base/ogg_ridic/".to_owned(), available: false },
            ],
            music_table: vec![
                "xxxx".to_owned(),
                "wanpaku".to_owned(),
                "anzen".to_owned(),
                "gameover".to_owned(),
                "gravity".to_owned(),
                "weed".to_owned(),
                "mdown2".to_owned(),
                "fireeye".to_owned(),
                "vivi".to_owned(),
                "mura".to_owned(),
                "fanfale1".to_owned(),
                "ginsuke".to_owned(),
                "cemetery".to_owned(),
                "plant".to_owned(),
                "kodou".to_owned(),
                "fanfale3".to_owned(),
                "fanfale2".to_owned(),
                "dr".to_owned(),
                "escape".to_owned(),
                "jenka".to_owned(),
                "maze".to_owned(),
                "access".to_owned(),
                "ironh".to_owned(),
                "grand".to_owned(),
                "curly".to_owned(),
                "oside".to_owned(),
                "requiem".to_owned(),
                "wanpak2".to_owned(),
                "quiet".to_owned(),
                "lastcave".to_owned(),
                "balcony".to_owned(),
                "lastbtl".to_owned(),
                "lastbt3".to_owned(),
                "ending".to_owned(),
                "zonbie".to_owned(),
                "bdown".to_owned(),
                "hell".to_owned(),
                "jenka2".to_owned(),
                "marine".to_owned(),
                "ballos".to_owned(),
                "toroko".to_owned(),
                "white".to_owned(),
                "kaze".to_owned(),
                "ika".to_owned(),
            ],
            organya_paths: vec![
                "/org/".to_owned(),          // NXEngine
                "/base/Org/".to_owned(),     // CS+
                "/Resource/ORG/".to_owned(), // CSE2E
            ],
            credit_illustration_paths: vec![
                String::new(),
                "Resource/BITMAP/".to_owned(), // CSE2E
                "endpic/".to_owned(),          // NXEngine
            ],
            player_skin_paths: vec!["MyChar".to_owned()],
            animated_face_table: vec![AnimatedFace { face_id: 0, anim_id: 0, anim_frames: vec![(0, 0)] }],
            string_table: HashMap::new(),
            missile_flags: vec![200, 201, 202, 218, 550, 766, 880, 920, 1551],
            locales: Vec::new(),
            gamepad: {
                let mut holder = GamepadConsts {
                    button_rects: HashMap::from([
                        (Button::North, GamepadConsts::rects(Rect::new(0, 0, 32, 16))),
                        (Button::South, GamepadConsts::rects(Rect::new(0, 16, 32, 32))),
                        (Button::East, GamepadConsts::rects(Rect::new(0, 32, 32, 48))),
                        (Button::West, GamepadConsts::rects(Rect::new(0, 48, 32, 64))),
                        (Button::DPadDown, GamepadConsts::rects(Rect::new(0, 64, 32, 80))),
                        (Button::DPadUp, GamepadConsts::rects(Rect::new(0, 80, 32, 96))),
                        (Button::DPadRight, GamepadConsts::rects(Rect::new(0, 96, 32, 112))),
                        (Button::DPadLeft, GamepadConsts::rects(Rect::new(0, 112, 32, 128))),
                        (Button::LeftShoulder, GamepadConsts::rects(Rect::new(32, 32, 64, 48))),
                        (Button::RightShoulder, GamepadConsts::rects(Rect::new(32, 48, 64, 64))),
                        (Button::Start, GamepadConsts::rects(Rect::new(32, 96, 64, 112))),
                        (Button::Back, GamepadConsts::rects(Rect::new(32, 112, 64, 128))),
                        (Button::LeftStick, GamepadConsts::rects(Rect::new(32, 0, 64, 16))),
                        (Button::RightStick, GamepadConsts::rects(Rect::new(32, 16, 64, 32))),
                    ]),
                    axis_rects: HashMap::from([
                        (Axis::LeftX, GamepadConsts::rects(Rect::new(32, 0, 64, 16))),
                        (Axis::LeftY, GamepadConsts::rects(Rect::new(32, 0, 64, 16))),
                        (Axis::RightX, GamepadConsts::rects(Rect::new(32, 16, 64, 32))),
                        (Axis::RightY, GamepadConsts::rects(Rect::new(32, 16, 64, 32))),
                        (Axis::TriggerLeft, GamepadConsts::rects(Rect::new(32, 64, 64, 80))),
                        (Axis::TriggerRight, GamepadConsts::rects(Rect::new(32, 80, 64, 96))),
                    ]),
                };
                // Swap x-y and a-b buttons on nintendo map (must be done here to comply with the nicalis button table)
                let button = holder.button_rects.get(&Button::North).unwrap()[3].clone();
                holder.button_rects.get_mut(&Button::North).unwrap()[3] = holder.button_rects.get(&Button::West).unwrap()[3];
                holder.button_rects.get_mut(&Button::West).unwrap()[3] = button;
                let button = holder.button_rects.get(&Button::South).unwrap()[3].clone();
                holder.button_rects.get_mut(&Button::South).unwrap()[3] = holder.button_rects.get(&Button::East).unwrap()[3];
                holder.button_rects.get_mut(&Button::East).unwrap()[3] = button;

                holder
            },
            stage_encoding: None,
        }
    }

    pub fn apply_csplus_patches(&mut self, sound_manager: &mut SoundManager) {
        log::info!("Applying Cave Story+ constants patches...");

        self.is_cs_plus = true;
        self.supports_og_textures = true;
        self.tex_sizes.insert("Caret".to_owned(), (320, 320));
        self.tex_sizes.insert("MyChar".to_owned(), (200, 384));
        self.tex_sizes.insert("Npc/NpcRegu".to_owned(), (320, 410));
        self.tex_sizes.insert("ui".to_owned(), (128, 32));
        self.textscript.reset_invicibility_on_any_script = false;
        self.title.logo_rect = Rect { left: 0, top: 0, right: 216, bottom: 48 };

        self.title.menu_left_top = Rect { left: 0, top: 0, right: 4, bottom: 4 };
        self.title.menu_right_top = Rect { left: 12, top: 0, right: 16, bottom: 4 };
        self.title.menu_left_bottom = Rect { left: 0, top: 12, right: 4, bottom: 16 };
        self.title.menu_right_bottom = Rect { left: 12, top: 12, right: 16, bottom: 16 };

        self.title.menu_top = Rect { left: 4, top: 0, right: 8, bottom: 4 };
        self.title.menu_middle = Rect { left: 8, top: 8, right: 12, bottom: 12 };
        self.title.menu_bottom = Rect { left: 4, top: 12, right: 8, bottom: 16 };
        self.title.menu_left = Rect { left: 0, top: 4, right: 4, bottom: 12 };
        self.title.menu_right = Rect { left: 12, top: 4, right: 16, bottom: 12 };

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

        let _ = sound_manager.set_sample_params(2, typewriter_sample);
    }

    pub fn is_base(&self) -> bool {
        !self.is_switch && !self.is_cs_plus && !self.is_demo
    }

    pub fn apply_csplus_nx_patches(&mut self) {
        log::info!("Applying Switch-specific Cave Story+ constants patches...");

        self.is_switch = true;
        self.supports_og_textures = true;
        self.tex_sizes.insert("bkMoon".to_owned(), (427, 240));
        self.tex_sizes.insert("bkFog".to_owned(), (427, 240));
        self.tex_sizes.insert("ui".to_owned(), (128, 32));
        self.tex_sizes.insert("uimusic".to_owned(), (192, 144));
        self.title.logo_rect = Rect { left: 0, top: 0, right: 214, bottom: 62 };
        self.inventory_dim_color = Color::from_rgba(0, 0, 32, 150);
        self.textscript.encoding = TextScriptEncoding::UTF8;
        self.textscript.encrypted = false;
        self.textscript.animated_face_pics = true;
        self.textscript.text_shadow = true;
        self.textscript.text_speed_normal = 1;
        self.textscript.text_speed_fast = 0;
        self.textscript.fade_ticks = 21;
        self.game.tile_offset_x = 3;
        self.game.new_game_player_pos = (13, 8);
        self.player_skin_paths.push("mychar_p2".to_owned());
    }

    pub fn apply_csdemo_patches(&mut self) {
        log::info!("Applying Wiiware DEMO-specific Cave Story+ constants patches...");

        self.is_demo = true;
        self.supports_og_textures = true;
        self.game.new_game_stage = 11;
        self.game.new_game_event = 302;
        self.game.new_game_player_pos = (8, 6);
        self.title.logo_splash_rect = Rect { left: 224, top: 0, right: 320, bottom: 48 };
    }

    pub fn rebuild_path_list(&mut self, mod_path: Option<String>, season: Season, settings: &Settings) {
        self.base_paths.clear();
        self.base_paths.push("/builtin/builtin_data/".to_owned());
        self.base_paths.push("/".to_owned());

        if self.is_cs_plus {
            self.base_paths.insert(0, "/base/".to_owned());

            if settings.original_textures {
                self.base_paths.insert(0, "/base/ogph/".to_string())
            } else if settings.seasonal_textures {
                match season {
                    Season::Halloween => self.base_paths.insert(0, "/Halloween/season/".to_string()),
                    Season::Christmas => self.base_paths.insert(0, "/Christmas/season/".to_string()),
                    _ => {}
                }
            }

            if settings.locale != "en".to_string() {
                self.base_paths.insert(0, format!("/base/{}/", settings.locale));
            }
        } else {
            if settings.locale != "en".to_string() {
                self.base_paths.insert(0, format!("/{}/", settings.locale));
            }
        }

        if let Some(mut mod_path) = mod_path {
            self.base_paths.insert(0, mod_path.clone());
            if settings.original_textures {
                mod_path.push_str("ogph/");
                self.base_paths.insert(0, mod_path);
            }

            // Nicalis left a landmine of a file in the original graphics for the nemesis challenge
            // It has 17 colors defined for a 4-bit color depth bitmap
            if self.is_cs_plus && !self.is_switch {
                self.base_paths.retain(|path| !path.contains("ogph/"));
            }
        }
    }

    pub fn special_treatment_for_csplus_mods(&mut self, mod_path: Option<&String>) {
        if !self.is_cs_plus {
            return;
        }

        let mut pos = if self.is_switch { (13, 8) } else { (10, 8) };

        if let Some(mod_path) = mod_path {
            if mod_path == "/TimeTrial/mod/" {
                pos = (8, 9);
            } else if mod_path == "/boss/mod/" {
                pos = (57, 21);
            }
        }

        self.game.new_game_player_pos = pos;
    }

    pub fn load_nx_stringtable(&mut self, ctx: &mut Context) -> GameResult {
        if let Ok(file) = filesystem::open(ctx, "/base/stringtable.sta") {
            let mut reader = BufReader::new(file);

            // Only some versions start with the BOM marker, thankfully the file isn't that large to read twice
            let mut bom = [0xef, 0xbb, 0xbf];
            let buf = reader.fill_buf()?;
            if buf.len() > 3 && buf[0..3] == bom {
                reader.read_exact(&mut bom)?;
            }

            if let Ok(xml) = Element::parse(reader) {
                for node in &xml.get_child("category").unwrap().children {
                    let element = node.as_element().unwrap();
                    let key = element.attributes.get_key_value("name").unwrap().1.to_string();
                    let english = element
                        .get_child("string")
                        .unwrap()
                        .get_text()
                        .unwrap_or(std::borrow::Cow::Borrowed(""))
                        .to_string();
                    self.string_table.insert(key, english);
                }
            }
        }
        Ok(())
    }

    pub fn load_locales(&mut self, ctx: &mut Context) -> GameResult {
        self.locales.clear();

        let locale_files = filesystem::read_dir_find(ctx, &self.base_paths, "locale/");

        for locale_file in locale_files.unwrap() {
            if locale_file.extension().unwrap() != "json" {
                continue;
            }

            let locale_code = {
                let filename = locale_file.file_name().unwrap().to_string_lossy();
                let mut parts = filename.split('.');
                parts.next().unwrap().to_string()
            };

            let mut locale = Locale::new(ctx, &self.base_paths, &locale_code);

            if locale_code == "jp" && filesystem::exists(ctx, "/base/credit_jp.tsc") {
                locale.set_font(FontData::new("csfontjp.fnt".to_owned(), 0.5, 0.0));
            }

            self.locales.push(locale.clone());
            log::info!("Loaded locale {} ({})", locale_code, locale.name.clone());
        }

        Ok(())
    }

    pub fn apply_constant_json_files(&mut self) {}

    pub fn load_texture_size_hints(&mut self, ctx: &mut Context) -> GameResult {
        if let Ok(file) = filesystem::open_find(ctx, &self.base_paths, "texture_sizes.json") {
            match serde_json::from_reader::<_, TextureSizeTable>(file) {
                Ok(tex_overrides) => {
                    for (key, (x, y)) in tex_overrides.sizes {
                        self.tex_sizes.insert(key, (x, y));
                    }
                }
                Err(err) => log::warn!("Failed to deserialize texture sizes: {}", err),
            }
        }
        Ok(())
    }

    /// Loads bullet.tbl and arms_level.tbl from CS+ files,
    /// even though they match vanilla 1:1, we should load them for completeness
    /// or if any crazy person uses it for a CS+ mod...
    pub fn load_csplus_tables(&mut self, ctx: &mut Context) -> GameResult {
        if let Ok(mut file) = filesystem::open_find(ctx, &self.base_paths, "bullet.tbl") {
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            let bullets = data.len() / 0x2A;
            let mut f = Cursor::new(data);

            let mut new_bullet_table = Vec::new();
            for _ in 0..bullets {
                let bullet = BulletData {
                    damage: f.read_u8()?,
                    life: f.read_u8()?,
                    lifetime: f.read_u32::<LE>()? as u16,
                    flags: BulletFlag(f.read_u32::<LE>()? as u8),
                    enemy_hit_width: f.read_u32::<LE>()? as u16,
                    enemy_hit_height: f.read_u32::<LE>()? as u16,
                    block_hit_width: f.read_u32::<LE>()? as u16,
                    block_hit_height: f.read_u32::<LE>()? as u16,
                    display_bounds: Rect {
                        left: f.read_u32::<LE>()? as u8,
                        top: f.read_u32::<LE>()? as u8,
                        right: f.read_u32::<LE>()? as u8,
                        bottom: f.read_u32::<LE>()? as u8,
                    },
                };
                new_bullet_table.push(bullet);
            }

            self.weapon.bullet_table = new_bullet_table;
            log::info!("Loaded bullet.tbl.");
        }

        if let Ok(mut file) = filesystem::open_find(ctx, &self.base_paths, "arms_level.tbl") {
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            let mut f = Cursor::new(data);

            let mut new_level_table = EngineConstants::defaults().weapon.level_table;
            for iter in 0..14 {
                let level1 = f.read_u32::<LE>()? as u16;
                let level2 = f.read_u32::<LE>()? as u16;
                let level3 = f.read_u32::<LE>()? as u16;
                new_level_table[iter] = [level1, level2, level3];
            }

            self.weapon.level_table = new_level_table;
            log::info!("Loaded arms_level.tbl.");
        }

        Ok(())
    }

    /// Load in the `faceanm.dat` file that details the Switch extensions to the <FAC command
    /// It's actually a text file, go figure
    pub fn load_animated_faces(&mut self, ctx: &mut Context) -> GameResult {
        self.animated_face_table.clear();

        // Bugfix for Malco cutscene - this face should be used but the original tsc has the wrong ID
        self.animated_face_table.push(AnimatedFace { face_id: 5, anim_id: 4, anim_frames: vec![(4, 0)] });

        if let Ok(file) = filesystem::open_find(ctx, &self.base_paths, "faceanm.dat") {
            let buf = BufReader::new(file);
            let mut face_id = 1;
            let mut anim_id = 0;

            for line in buf.lines() {
                let line_str = line?.to_owned().replace(",", " ");
                let mut anim_frames = Vec::new();

                if line_str.find("\\") == None {
                    continue;
                } else if line_str == "\\end" {
                    face_id += 1;
                    anim_id = 0;
                    continue;
                }

                for split in line_str.split_whitespace() {
                    // The animation labels aren't actually used
                    // There are also comments on some lines that we need to ignore
                    if split.find("\\") != None {
                        continue;
                    } else if split.find("//") != None {
                        break;
                    }
                    let mut parse = split.split(":");
                    let frame = (
                        parse.next().unwrap().parse::<u16>().unwrap_or(0),
                        parse.next().unwrap().parse::<u16>().unwrap_or(0),
                    );
                    anim_frames.push(frame);
                }

                self.animated_face_table.push(AnimatedFace { face_id, anim_id, anim_frames });
                anim_id += 1;
            }
        }
        Ok(())
    }
}
