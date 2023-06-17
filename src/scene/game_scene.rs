use std::cell::RefCell;
use std::ops::{Deref, Range};
use std::rc::Rc;

use log::info;

use crate::common::{interpolate_fix9_scale, Color, Direction, Rect};
use crate::components::background::Background;
use crate::components::boss_life_bar::BossLifeBar;
use crate::components::credits::Credits;
use crate::components::draw_common::Alignment;
use crate::components::fade::Fade;
use crate::components::falling_island::FallingIsland;
use crate::components::flash::Flash;
use crate::components::hud::HUD;
use crate::components::inventory::InventoryUI;
use crate::components::map_system::MapSystem;
use crate::components::nikumaru::NikumaruCounter;
use crate::components::replay::Replay;
use crate::components::stage_select::StageSelect;
use crate::components::text_boxes::TextBoxes;
use crate::components::tilemap::{TileLayer, Tilemap};
use crate::components::water_renderer::{WaterLayer, WaterRenderer};
use crate::components::whimsical_star::WhimsicalStar;
use crate::entity::GameEntity;
use crate::framework::backend::SpriteBatchCommand;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics::{draw_rect, BlendMode, FilterMode};
use crate::framework::keyboard::ScanCode;
use crate::framework::ui::Components;
use crate::framework::{filesystem, gamepad, graphics};
use crate::game::caret::CaretType;
use crate::game::frame::{Frame, UpdateTarget};
use crate::game::inventory::{Inventory, TakeExperienceResult};
use crate::game::map::WaterParams;
use crate::game::npc::boss::BossNPC;
use crate::game::npc::list::NPCList;
use crate::game::npc::{NPCLayer, NPC};
use crate::game::physics::{PhysicalEntity, OFFSETS};
use crate::game::player::{ControlMode, Player, TargetPlayer};
use crate::game::scripting::tsc::credit_script::CreditScriptVM;
use crate::game::scripting::tsc::text_script::{ScriptMode, TextScriptExecutionState, TextScriptVM};
use crate::game::settings::ControllerType;
use crate::game::shared_game_state::{CutsceneSkipMode, PlayerCount, ReplayState, SharedGameState, TileSize};
use crate::game::stage::{BackgroundType, Stage, StageTexturePaths};
use crate::game::weapon::bullet::BulletManager;
use crate::game::weapon::{Weapon, WeaponType};
use crate::graphics::font::{Font, Symbols};
use crate::graphics::texture_set::SpriteBatch;
use crate::input::touch_controls::TouchControlType;
use crate::menu::pause_menu::PauseMenu;
use crate::scene::title_scene::TitleScene;
use crate::scene::Scene;
use crate::util::rng::RNG;

pub struct GameScene {
    pub tick: u32,
    pub stage: Stage,
    pub water_params: WaterParams,
    pub water_renderer: WaterRenderer,
    pub boss_life_bar: BossLifeBar,
    pub stage_select: StageSelect,
    pub flash: Flash,
    pub credits: Credits,
    pub falling_island: FallingIsland,
    pub inventory_ui: InventoryUI,
    pub map_system: MapSystem,
    pub hud_player1: HUD,
    pub hud_player2: HUD,
    pub nikumaru: NikumaruCounter,
    pub whimsical_star: WhimsicalStar,
    pub background: Background,
    pub tilemap: Tilemap,
    pub text_boxes: TextBoxes,
    pub fade: Fade,
    pub frame: Frame,
    pub player1: Player,
    pub player2: Player,
    pub inventory_player1: Inventory,
    pub inventory_player2: Inventory,
    pub stage_id: usize,
    pub npc_list: NPCList,
    pub boss: BossNPC,
    pub bullet_manager: BulletManager,
    pub lighting_mode: LightingMode,
    pub intro_mode: bool,
    pub pause_menu: PauseMenu,
    pub stage_textures: Rc<RefCell<StageTexturePaths>>,
    pub replay: Replay,
    map_name_counter: u16,
    skip_counter: u16,
    inventory_dim: f32,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum LightingMode {
    None,
    BackgroundOnly,
    Ambient,
}

const P2_OFFSCREEN_TEXT: &'static str = "P2";
const CUTSCENE_SKIP_WAIT: u16 = 50;

impl GameScene {
    pub fn new(state: &mut SharedGameState, ctx: &mut Context, id: usize) -> GameResult<Self> {
        info!("Loading stage {} ({})", id, &state.stages[id].map);
        let stage = Stage::load(&state.constants.base_paths, &state.stages[id], ctx)?;
        info!("Loaded stage: {}", stage.data.name);

        GameScene::from_stage(state, ctx, stage, id)
    }

    pub fn from_stage(state: &mut SharedGameState, ctx: &mut Context, stage: Stage, id: usize) -> GameResult<Self> {
        let mut water_params = WaterParams::new();
        let mut water_renderer = WaterRenderer::new();
        let mut tilemap = Tilemap::new();

        if !state.settings.original_textures {
            if let Ok(water_param_file) = filesystem::open_find(
                ctx,
                &state.constants.base_paths,
                ["Stage/", &state.stages[id].tileset.name, ".pxw"].join(""),
            ) {
                water_params.load_from(water_param_file)?;
                info!("Loaded water parameters file.");

                let regions = stage.map.find_water_regions(&water_params);
                water_renderer.initialize(regions, &water_params, &stage);
                tilemap.no_water = true;
            }
        }

        let stage_textures = {
            let mut textures = StageTexturePaths::new();
            textures.update(&stage);
            Rc::new(RefCell::new(textures))
        };

        let mut player2 = Player::new(state, ctx);

        if state.player2_skin_location.texture_index != 0 {
            let skinsheet_name =
                state.constants.player_skin_paths[state.player2_skin_location.texture_index as usize].as_str();
            player2.load_skin(skinsheet_name.to_owned(), state, ctx);
        }

        Ok(Self {
            tick: 0,
            stage,
            water_params,
            water_renderer,
            player1: Player::new(state, ctx),
            player2: player2,
            inventory_player1: Inventory::new(),
            inventory_player2: Inventory::new(),
            boss_life_bar: BossLifeBar::new(),
            stage_select: StageSelect::new(),
            flash: Flash::new(),
            credits: Credits::new(),
            falling_island: FallingIsland::new(),
            inventory_ui: InventoryUI::new(),
            map_system: MapSystem::new(),
            hud_player1: HUD::new(Alignment::Left),
            hud_player2: HUD::new(Alignment::Right),
            nikumaru: NikumaruCounter::new(),
            whimsical_star: WhimsicalStar::new(),
            background: Background::new(),
            tilemap,
            text_boxes: TextBoxes::new(),
            fade: Fade::new(),
            frame: Frame::new(),
            stage_id: id,
            npc_list: NPCList::new(),
            boss: BossNPC::new(),
            bullet_manager: BulletManager::new(),
            lighting_mode: LightingMode::None,
            intro_mode: false,
            pause_menu: PauseMenu::new(),
            stage_textures,
            map_name_counter: 0,
            skip_counter: 0,
            inventory_dim: 0.0,
            replay: Replay::new(),
        })
    }

    pub fn display_map_name(&mut self, ticks: u16) {
        self.map_name_counter = ticks;
    }

    pub fn add_player2(&mut self, state: &mut SharedGameState, ctx: &mut Context) {
        self.player2.cond.set_alive(true);
        self.player2.cond.set_hidden(self.player1.cond.hidden());

        let skinsheet_name =
            state.constants.player_skin_paths[state.player2_skin_location.texture_index as usize].as_str();
        self.player2.load_skin(skinsheet_name.to_owned(), state, ctx);
        self.player2.skin.set_skinsheet_offset(state.player2_skin_location.offset);

        self.player2.x = self.player1.x;
        self.player2.y = self.player1.y;
        self.player2.vel_x = self.player1.vel_x;
        self.player2.vel_y = self.player1.vel_y;
    }

    pub fn drop_player2(&mut self) {
        self.player2.cond.set_alive(false);
    }

    fn draw_npc_layer(&self, state: &mut SharedGameState, ctx: &mut Context, layer: NPCLayer) -> GameResult {
        for npc in self.npc_list.iter_alive() {
            if npc.layer != layer
                || npc.x < (self.frame.x - 128 * 0x200 - npc.display_bounds.width() as i32 * 0x200)
                || npc.x
                    > (self.frame.x
                        + 128 * 0x200
                        + (state.canvas_size.0 as i32 + npc.display_bounds.width() as i32) * 0x200)
                    && npc.y < (self.frame.y - 128 * 0x200 - npc.display_bounds.height() as i32 * 0x200)
                || npc.y
                    > (self.frame.y
                        + 128 * 0x200
                        + (state.canvas_size.1 as i32 + npc.display_bounds.height() as i32) * 0x200)
            {
                continue;
            }

            npc.draw(state, ctx, &self.frame)?;
        }

        Ok(())
    }

    fn draw_npc_popup(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        for npc in self.npc_list.iter_alive() {
            npc.popup.draw(state, ctx, &self.frame)?;
        }
        Ok(())
    }

    fn draw_boss_popup(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        for part in self.boss.parts.iter() {
            part.popup.draw(state, ctx, &self.frame)?;
        }
        Ok(())
    }

    fn draw_bullets(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Bullet")?;
        let mut x: i32;
        let mut y: i32;
        let mut prev_x: i32;
        let mut prev_y: i32;

        for bullet in self.bullet_manager.bullets.iter() {
            match bullet.direction {
                Direction::Left => {
                    x = bullet.x - bullet.display_bounds.left as i32;
                    y = bullet.y - bullet.display_bounds.top as i32;
                    prev_x = bullet.prev_x - bullet.display_bounds.left as i32;
                    prev_y = bullet.prev_y - bullet.display_bounds.top as i32;
                }
                Direction::Up => {
                    x = bullet.x - bullet.display_bounds.top as i32;
                    y = bullet.y - bullet.display_bounds.left as i32;
                    prev_x = bullet.prev_x - bullet.display_bounds.top as i32;
                    prev_y = bullet.prev_y - bullet.display_bounds.left as i32;
                }
                Direction::Right => {
                    x = bullet.x - bullet.display_bounds.right as i32;
                    y = bullet.y - bullet.display_bounds.top as i32;
                    prev_x = bullet.prev_x - bullet.display_bounds.right as i32;
                    prev_y = bullet.prev_y - bullet.display_bounds.top as i32;
                }
                Direction::Bottom => {
                    x = bullet.x - bullet.display_bounds.top as i32;
                    y = bullet.y - bullet.display_bounds.right as i32;
                    prev_x = bullet.prev_x - bullet.display_bounds.top as i32;
                    prev_y = bullet.prev_y - bullet.display_bounds.right as i32;
                }
                Direction::FacingPlayer => unreachable!(),
            }

            batch.add_rect(
                interpolate_fix9_scale(prev_x - self.frame.prev_x, x - self.frame.x, state.frame_time),
                interpolate_fix9_scale(prev_y - self.frame.prev_y, y - self.frame.y, state.frame_time),
                &bullet.anim_rect,
            );
        }

        batch.draw(ctx)?;
        Ok(())
    }

    fn draw_carets(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Caret")?;

        for caret in state.carets.iter() {
            batch.add_rect(
                interpolate_fix9_scale(
                    caret.prev_x - caret.offset_x - self.frame.prev_x,
                    caret.x - caret.offset_x - self.frame.x,
                    state.frame_time,
                ),
                interpolate_fix9_scale(
                    caret.prev_y - caret.offset_y - self.frame.prev_y,
                    caret.y - caret.offset_y - self.frame.y,
                    state.frame_time,
                ),
                &caret.anim_rect,
            );
        }

        batch.draw(ctx)?;
        Ok(())
    }

    fn draw_black_bars(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let (x, y) = self.frame.xy_interpolated(state.frame_time);
        let (x, y) = (x * state.scale, y * state.scale);
        let canvas_w_scaled = state.canvas_size.0 as f32 * state.scale;
        let canvas_h_scaled = state.canvas_size.1 as f32 * state.scale;
        let half_block = self.stage.map.tile_size.as_float() * 0.5 * state.scale;
        let level_width = (self.stage.map.width as f32) * self.stage.map.tile_size.as_float();
        let level_height = (self.stage.map.height as f32) * self.stage.map.tile_size.as_float();
        let left_side = -x - half_block;
        let right_side = left_side + level_width * state.scale;
        let upper_side = -y - half_block;
        let lower_side = upper_side + level_height * state.scale;

        if left_side > 0.0 {
            let rect = Rect::new(0, 0, left_side as isize, canvas_h_scaled as isize);
            graphics::draw_rect(ctx, rect, Color::from_rgb(0, 0, 0))?;
        }

        if right_side < canvas_w_scaled {
            let rect = Rect::new(
                right_side as isize,
                0,
                (state.canvas_size.0 * state.scale) as isize,
                (state.canvas_size.1 * state.scale) as isize,
            );
            graphics::draw_rect(ctx, rect, Color::from_rgb(0, 0, 0))?;
        }

        if upper_side > 0.0 {
            let rect = Rect::new(0, 0, canvas_w_scaled as isize, upper_side as isize);
            graphics::draw_rect(ctx, rect, Color::from_rgb(0, 0, 0))?;
        }

        if lower_side < canvas_h_scaled {
            let rect = Rect::new(0, lower_side as isize, canvas_w_scaled as isize, canvas_h_scaled as isize);
            graphics::draw_rect(ctx, rect, Color::from_rgb(0, 0, 0))?;
        }

        Ok(())
    }

    fn set_ironhead_clip(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let x_size = if !state.constants.is_switch { 320.0 } else { 426.0 };
        let clip_rect: Rect = Rect::new_size(
            (((state.canvas_size.0 - x_size) * 0.5) * state.scale) as _,
            (((state.canvas_size.1 - 240.0) * 0.5) * state.scale) as _,
            (x_size * state.scale) as _,
            (240.0 * state.scale) as _,
        );
        graphics::set_clip_rect(ctx, Some(clip_rect))?;
        Ok(())
    }

    fn draw_light(&self, x: f32, y: f32, size: f32, color: (u8, u8, u8), batch: &mut Box<dyn SpriteBatch>) {
        batch.add_rect_scaled_tinted(
            x - size * 32.0,
            y - size * 32.0,
            (color.0, color.1, color.2, 255),
            size,
            size,
            &Rect::new(0, 0, 64, 64),
        )
    }

    fn draw_light_raycast(
        &self,
        tile_size: TileSize,
        world_point_x: i32,
        world_point_y: i32,
        (br, bg, bb): (u8, u8, u8),
        att: f32,
        angle: Range<i32>,
        batch: &mut Box<dyn SpriteBatch>,
    ) {
        let px = world_point_x as f32 / 512.0;
        let py = world_point_y as f32 / 512.0;

        let fx2 = self.frame.x as f32 / 512.0;
        let fy2 = self.frame.y as f32 / 512.0;

        let ti = tile_size.as_int();
        let tf = tile_size.as_float();
        let tih = ti / 2;
        let tfq = tf / 4.0;
        let (br, bg, bb) = (br as f32, bg as f32, bb as f32);
        let ahalf = (angle.end - angle.start) as f32 / 2.0;

        'ray: for (i, deg) in angle.enumerate() {
            let d = deg as f32 * (std::f32::consts::PI / 180.0);
            let dx = d.cos() * -5.0;
            let dy = d.sin() * -5.0;
            let m = 1.0 - ((ahalf - i as f32).abs() / ahalf);
            let mut x = px;
            let mut y = py;
            let mut r = br;
            let mut g = bg;
            let mut b = bb;

            for i in 0..40 {
                x += dx;
                y += dy;

                const ARR: [(i32, i32); 4] = [(0, 0), (0, 1), (1, 0), (1, 1)];
                for (ox, oy) in ARR.iter() {
                    let bx = (x as i32).wrapping_div(ti).wrapping_add(*ox);
                    let by = (y as i32).wrapping_div(ti).wrapping_add(*oy);

                    let tile = self.stage.map.attrib[self.stage.tile_at(bx as usize, by as usize) as usize];
                    let bxmth = (bx * ti - tih) as f32;
                    let bxpth = (bx * ti + tih) as f32;
                    let bymth = (by * ti - tih) as f32;
                    let bypth = (by * ti + tih) as f32;

                    if ((tile == 0x62 || tile == 0x41 || tile == 0x43 || tile == 0x46)
                        && x >= bxmth
                        && x <= bxpth
                        && y >= bymth
                        && y <= bypth)
                        || ((tile == 0x50 || tile == 0x70)
                            && x >= bxmth
                            && x <= bxpth
                            && y <= ((by as f32 * tf) - (x - bx as f32 * tf) / 2.0 + tfq)
                            && y >= bymth)
                        || ((tile == 0x51 || tile == 0x71)
                            && x >= bxmth
                            && x <= bxpth
                            && y <= ((by as f32 * tf) - (x - bx as f32 * tf) / 2.0 - tfq)
                            && y >= bymth)
                        || ((tile == 0x52 || tile == 0x72)
                            && x >= bxmth
                            && x <= bxpth
                            && y <= ((by as f32 * tf) + (x - bx as f32 * tf) / 2.0 - tfq)
                            && y >= bymth)
                        || ((tile == 0x53 || tile == 0x73)
                            && x >= bxmth
                            && x <= bxpth
                            && y <= ((by as f32 * tf) + (x - bx as f32 * tf) / 2.0 + tfq)
                            && y >= bymth)
                        || ((tile == 0x54 || tile == 0x74)
                            && x >= bxmth
                            && x <= bxpth
                            && y >= ((by as f32 * tf) + (x - bx as f32 * tf) / 2.0 - tfq)
                            && y <= bypth)
                        || ((tile == 0x55 || tile == 0x75)
                            && x >= bxmth
                            && x <= bxpth
                            && y >= ((by as f32 * tf) + (x - bx as f32 * tf) / 2.0 + tfq)
                            && y <= bypth)
                        || ((tile == 0x56 || tile == 0x76)
                            && x >= bxmth
                            && x <= bxpth
                            && y >= ((by as f32 * tf) - (x - bx as f32 * tf) / 2.0 + tfq)
                            && y <= bypth)
                        || ((tile == 0x57 || tile == 0x77)
                            && x >= bxmth
                            && x <= bxpth
                            && y >= ((by as f32 * tf) - (x - bx as f32 * tf) / 2.0 - tfq)
                            && y <= bypth)
                    {
                        continue 'ray;
                    }
                }

                r *= att;
                g *= att;
                b *= att;

                if r <= 1.0 && g <= 1.0 && b <= 1.0 {
                    continue 'ray;
                }

                self.draw_light(
                    x - fx2,
                    y - fy2,
                    0.15 + i as f32 / 75.0,
                    ((r * m) as u8, (g * m) as u8, (b * m) as u8),
                    batch,
                );
            }
        }
    }

    fn draw_light_map(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        {
            let maybe_canvas = state.lightmap_canvas.as_ref();

            if maybe_canvas.is_some() {
                graphics::set_render_target(ctx, maybe_canvas)?;
            } else {
                return Ok(());
            }
        }

        graphics::set_blend_mode(ctx, BlendMode::Add)?;

        graphics::clear(ctx, Color::from_rgb(100, 100, 110));

        for npc in self.npc_list.iter_alive() {
            if npc.x < (self.frame.x - 128 * 0x200 - npc.display_bounds.width() as i32 * 0x200)
                || npc.x
                    > (self.frame.x
                        + 128 * 0x200
                        + (state.canvas_size.0 as i32 + npc.display_bounds.width() as i32) * 0x200)
                    && npc.y < (self.frame.y - 128 * 0x200 - npc.display_bounds.height() as i32 * 0x200)
                || npc.y
                    > (self.frame.y
                        + 128 * 0x200
                        + (state.canvas_size.1 as i32 + npc.display_bounds.height() as i32) * 0x200)
            {
                continue;
            }

            npc.draw_lightmap(state, ctx, &self.frame)?;
        }

        {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "builtin/lightmap/spot")?;

            'cc: for (player, inv) in
                [(&self.player1, &self.inventory_player1), (&self.player2, &self.inventory_player2)].iter()
            {
                if player.cond.alive() && !player.cond.hidden() && inv.get_current_weapon().is_some() {
                    if state.settings.light_cone {
                        let range = match () {
                            _ if player.up => 60..120,
                            _ if player.down => 240..300,
                            _ if player.direction == Direction::Left => -30..30,
                            _ if player.direction == Direction::Right => 150..210,
                            _ => continue 'cc,
                        };

                        let (color, att) = match inv.get_current_weapon() {
                            Some(Weapon { wtype: WeaponType::Fireball, .. }) => ((170u8, 80u8, 0u8), 0.92),
                            Some(Weapon { wtype: WeaponType::PolarStar, .. }) => ((150u8, 150u8, 160u8), 0.92),
                            Some(Weapon { wtype: WeaponType::Spur, .. }) => ((170u8, 170u8, 200u8), 0.92),
                            Some(Weapon { wtype: WeaponType::Blade, .. }) => continue 'cc,
                            _ => ((150u8, 150u8, 150u8), 0.92),
                        };

                        let (_, gun_off_y) = player.skin.get_gun_offset();

                        self.draw_light_raycast(
                            state.tile_size,
                            player.x + player.direction.vector_x() * 0x800,
                            player.y + gun_off_y * 0x200 + 0x400,
                            color,
                            att,
                            range,
                            batch,
                        );
                    } else {
                        self.draw_light(
                            interpolate_fix9_scale(
                                player.prev_x - self.frame.prev_x,
                                player.x - self.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                player.prev_y - self.frame.prev_y,
                                player.y - self.frame.y,
                                state.frame_time,
                            ),
                            5.0,
                            (150, 150, 150),
                            batch,
                        );
                    }
                }
            }

            for bullet in self.bullet_manager.bullets.iter() {
                self.draw_light(
                    interpolate_fix9_scale(
                        bullet.prev_x - self.frame.prev_x,
                        bullet.x - self.frame.x,
                        state.frame_time,
                    ),
                    interpolate_fix9_scale(
                        bullet.prev_y - self.frame.prev_y,
                        bullet.y - self.frame.y,
                        state.frame_time,
                    ),
                    0.3,
                    (200, 200, 200),
                    batch,
                );
            }

            for caret in state.carets.iter() {
                match caret.ctype {
                    CaretType::ProjectileDissipation | CaretType::Shoot => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                caret.prev_x - self.frame.prev_x,
                                caret.x - self.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                caret.prev_y - self.frame.prev_y,
                                caret.y - self.frame.y,
                                state.frame_time,
                            ),
                            0.5,
                            (150, 150, 150),
                            batch,
                        );
                    }
                    _ => {}
                }
            }

            for npc in self.npc_list.iter_alive() {
                if npc.cond.hidden()
                    || (npc.x < (self.frame.x - 128 * 0x200 - npc.display_bounds.width() as i32 * 0x200)
                        || npc.x
                            > (self.frame.x
                                + 128 * 0x200
                                + (state.canvas_size.0 as i32 + npc.display_bounds.width() as i32) * 0x200)
                            && npc.y < (self.frame.y - 128 * 0x200 - npc.display_bounds.height() as i32 * 0x200)
                        || npc.y
                            > (self.frame.y
                                + 128 * 0x200
                                + (state.canvas_size.1 as i32 + npc.display_bounds.height() as i32) * 0x200))
                {
                    continue;
                }

                // NPC lighting
                match npc.npc_type {
                    1 => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - self.frame.prev_x,
                                npc.x - self.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - self.frame.prev_y,
                                npc.y - self.frame.y,
                                state.frame_time,
                            ),
                            0.33,
                            (255, 255, 50),
                            batch,
                        );
                    }
                    4 if npc.direction == Direction::Up => self.draw_light(
                        interpolate_fix9_scale(npc.prev_x - self.frame.prev_x, npc.x - self.frame.x, state.frame_time),
                        interpolate_fix9_scale(npc.prev_y - self.frame.prev_y, npc.y - self.frame.y, state.frame_time),
                        1.0,
                        (200, 100, 0),
                        batch,
                    ),
                    7 => self.draw_light(
                        interpolate_fix9_scale(npc.prev_x - self.frame.prev_x, npc.x - self.frame.x, state.frame_time),
                        interpolate_fix9_scale(npc.prev_y - self.frame.prev_y, npc.y - self.frame.y, state.frame_time),
                        1.0,
                        (100, 100, 100),
                        batch,
                    ),
                    17 if npc.anim_num == 0 => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - self.frame.prev_x,
                                npc.x - self.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - self.frame.prev_y,
                                npc.y - self.frame.y,
                                state.frame_time,
                            ),
                            1.25,
                            (100, 0, 0),
                            batch,
                        );
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - self.frame.prev_x,
                                npc.x - self.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - self.frame.prev_y,
                                npc.y - self.frame.y,
                                state.frame_time,
                            ),
                            0.5,
                            (255, 10, 10),
                            batch,
                        );
                    }
                    20 if npc.direction == Direction::Right => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - self.frame.prev_x,
                                npc.x - self.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - self.frame.prev_y,
                                npc.y - self.frame.y,
                                state.frame_time,
                            ),
                            1.5,
                            (30, 30, 130),
                            batch,
                        );

                        if npc.anim_num < 2 {
                            self.draw_light(
                                interpolate_fix9_scale(
                                    npc.prev_x - self.frame.prev_x,
                                    npc.x - self.frame.x,
                                    state.frame_time,
                                ),
                                interpolate_fix9_scale(
                                    npc.prev_y - self.frame.prev_y,
                                    npc.y - self.frame.y,
                                    state.frame_time,
                                ),
                                1.0,
                                (0, 0, 20),
                                batch,
                            );
                        }
                    }
                    22 if npc.action_num == 1 && npc.anim_num == 1 => self.draw_light(
                        interpolate_fix9_scale(npc.prev_x - self.frame.prev_x, npc.x - self.frame.x, state.frame_time),
                        interpolate_fix9_scale(npc.prev_y - self.frame.prev_y, npc.y - self.frame.y, state.frame_time),
                        3.0,
                        (0, 0, 255),
                        batch,
                    ),
                    32 | 87 => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - self.frame.prev_x,
                                npc.x - self.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - self.frame.prev_y,
                                npc.y - self.frame.y,
                                state.frame_time,
                            ),
                            0.75,
                            (255, 30, 30),
                            batch,
                        );
                    }
                    211 => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - self.frame.prev_x,
                                npc.x - self.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - self.frame.prev_y,
                                npc.y - self.frame.y,
                                state.frame_time,
                            ),
                            1.0,
                            (90, 0, 0),
                            batch,
                        );
                    }
                    27 => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - self.frame.prev_x,
                                npc.x - self.frame.x,
                                state.frame_time,
                            ) + 0.5,
                            interpolate_fix9_scale(
                                npc.prev_y - self.frame.prev_y,
                                npc.y - self.frame.y,
                                state.frame_time,
                            ),
                            3.0,
                            (96, 0, 0),
                            batch,
                        );
                    }
                    38 => {
                        let flicker = ((npc.anim_num.wrapping_add(npc.id) ^ 5) & 3) as u8 * 24;
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - self.frame.prev_x,
                                npc.x - self.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - self.frame.prev_y,
                                npc.y - self.frame.y,
                                state.frame_time,
                            ),
                            3.5,
                            (150 + flicker, 60 + flicker, 0),
                            batch,
                        );
                    }
                    69 | 81 => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - self.frame.prev_x,
                                npc.x - self.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - self.frame.prev_y,
                                npc.y - self.frame.y,
                                state.frame_time,
                            ),
                            if npc.npc_type == 69 { 0.5 } else { 1.0 },
                            (200, 200, 200),
                            batch,
                        );
                    }
                    70 => {
                        let flicker = 50 + npc.anim_num as u8 * 15;
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - self.frame.prev_x,
                                npc.x - self.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - self.frame.prev_y,
                                npc.y - self.frame.y,
                                state.frame_time,
                            ),
                            2.0,
                            (flicker, flicker, flicker),
                            batch,
                        );
                    }
                    85 if npc.action_num == 1 => {
                        let (color, color2) = if npc.direction == Direction::Left {
                            if state.constants.is_cs_plus {
                                ((20, 100, 20), (20, 50, 20))
                            } else {
                                ((20, 20, 100), (20, 20, 50))
                            }
                        } else {
                            ((150, 0, 0), (50, 0, 0))
                        };

                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - self.frame.prev_x,
                                npc.x - self.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - self.frame.prev_y,
                                npc.y - self.frame.y,
                                state.frame_time,
                            ),
                            0.75,
                            color,
                            batch,
                        );

                        if npc.anim_num < 2 && npc.direction == Direction::Right {
                            self.draw_light(
                                interpolate_fix9_scale(
                                    npc.prev_x - self.frame.prev_x,
                                    npc.x - self.frame.x,
                                    state.frame_time,
                                ),
                                interpolate_fix9_scale(
                                    npc.prev_y - self.frame.prev_y,
                                    npc.y - self.frame.y,
                                    state.frame_time,
                                ) - 8.0,
                                2.1,
                                color2,
                                batch,
                            );
                        }
                    }
                    101 | 102 => self.draw_light(
                        interpolate_fix9_scale(npc.prev_x - self.frame.prev_x, npc.x - self.frame.x, state.frame_time),
                        interpolate_fix9_scale(npc.prev_y - self.frame.prev_y, npc.y - self.frame.y, state.frame_time),
                        1.0,
                        (100, 100, 200),
                        batch,
                    ),
                    175 if npc.action_num < 10 => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - self.frame.prev_x,
                                npc.x - self.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - self.frame.prev_y,
                                npc.y - self.frame.y,
                                state.frame_time,
                            ),
                            1.0,
                            (128, 175, 200),
                            batch,
                        );
                    }
                    189 => self.draw_light(
                        interpolate_fix9_scale(npc.prev_x - self.frame.prev_x, npc.x - self.frame.x, state.frame_time),
                        interpolate_fix9_scale(npc.prev_y - self.frame.prev_y, npc.y - self.frame.y, state.frame_time),
                        1.0,
                        (10, 50, 255),
                        batch,
                    ),
                    270 => self.draw_light(
                        interpolate_fix9_scale(npc.prev_x - self.frame.prev_x, npc.x - self.frame.x, state.frame_time),
                        interpolate_fix9_scale(npc.prev_y - self.frame.prev_y, npc.y - self.frame.y, state.frame_time),
                        0.4,
                        (192, 0, 0),
                        batch,
                    ),
                    285 | 287 => self.draw_light(
                        interpolate_fix9_scale(npc.prev_x - self.frame.prev_x, npc.x - self.frame.x, state.frame_time),
                        interpolate_fix9_scale(npc.prev_y - self.frame.prev_y, npc.y - self.frame.y, state.frame_time),
                        1.0,
                        (150, 90, 0),
                        batch,
                    ),
                    293 => self.draw_light(
                        interpolate_fix9_scale(npc.prev_x - self.frame.prev_x, npc.x - self.frame.x, state.frame_time),
                        interpolate_fix9_scale(npc.prev_y - self.frame.prev_y, npc.y - self.frame.y, state.frame_time),
                        4.0,
                        (255, 255, 255),
                        batch,
                    ),
                    311 => {
                        let size = if npc.anim_num % 7 == 2 || npc.anim_num % 7 == 5 { 1.0 } else { 0.0 };

                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - self.frame.prev_x,
                                npc.x - self.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - self.frame.prev_y,
                                npc.y - self.frame.y,
                                state.frame_time,
                            ),
                            size,
                            (255, 255, 255),
                            batch,
                        )
                    }
                    312 => self.draw_light(
                        interpolate_fix9_scale(npc.prev_x - self.frame.prev_x, npc.x - self.frame.x, state.frame_time),
                        interpolate_fix9_scale(npc.prev_y - self.frame.prev_y, npc.y - self.frame.y, state.frame_time),
                        0.5,
                        (255, 255, 255),
                        batch,
                    ),
                    319 => {
                        let color = if npc.anim_num == 2 { (255, 29, 0) } else { (234, 157, 68) };

                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - self.frame.prev_x,
                                npc.x - self.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - self.frame.prev_y,
                                npc.y - self.frame.y,
                                state.frame_time,
                            ),
                            1.0,
                            color,
                            batch,
                        )
                    }
                    180 => {
                        if state.settings.light_cone {
                            // Curly's looking upward frames
                            let range = if [5, 6, 7, 8, 9].contains(&(npc.anim_num % 11)) {
                                60..120
                            } else if npc.action_num == 40 || npc.action_num == 41 {
                                0..0
                            } else if npc.direction() == Direction::Left {
                                -30..30
                            } else if npc.direction() == Direction::Right {
                                150..210
                            } else {
                                0..0
                            };

                            self.draw_light_raycast(
                                state.tile_size,
                                npc.x + npc.direction.opposite().vector_x() * 0x800,
                                npc.y + 2 * 0x200,
                                (19u8, 34u8, 117u8),
                                0.95,
                                range,
                                batch,
                            );
                        }
                    }
                    320 => {
                        if state.settings.light_cone {
                            let range = match npc.direction() {
                                Direction::Up => 60..120,
                                Direction::Bottom => 240..300,
                                Direction::Left => -30..30,
                                Direction::Right => 150..210,
                                _ => 0..0,
                            };

                            self.draw_light_raycast(
                                state.tile_size,
                                npc.x + npc.direction.opposite().vector_x() * 0x800,
                                npc.y + 2 * 0x200,
                                (19u8, 34u8, 117u8),
                                0.95,
                                range,
                                batch,
                            );
                        }
                    }
                    322 => {
                        let scale = 0.004 * (npc.action_counter as f32);

                        self.draw_light_raycast(state.tile_size, npc.x, npc.y, (255, 0, 0), scale, 0..360, batch)
                    }
                    325 => {
                        let size = 0.5 * (npc.anim_num as f32 + 1.0);
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - self.frame.prev_x,
                                npc.x - self.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - self.frame.prev_y,
                                npc.y - self.frame.y,
                                state.frame_time,
                            ),
                            size,
                            (255, 255, 255),
                            batch,
                        )
                    }
                    _ => {}
                }
            }

            batch.draw_filtered(FilterMode::Linear, ctx)?;
        }

        graphics::set_blend_mode(ctx, BlendMode::Multiply)?;
        graphics::set_render_target(ctx, None)?;

        {
            let canvas = state.lightmap_canvas.as_mut().unwrap();
            let rect = Rect { left: 0.0, top: 0.0, right: state.screen_size.0, bottom: state.screen_size.1 };

            canvas.clear();
            canvas.add(SpriteBatchCommand::DrawRect(rect, rect));
            canvas.draw()?;

            graphics::set_render_target(ctx, Some(canvas))?;
            graphics::draw_rect(
                ctx,
                Rect {
                    left: 0,
                    top: 0,
                    right: (state.screen_size.0 + 1.0) as isize,
                    bottom: (state.screen_size.1 + 1.0) as isize,
                },
                Color { r: 0.15, g: 0.12, b: 0.12, a: 1.0 },
            )?;
            graphics::set_render_target(ctx, None)?;
            graphics::set_blend_mode(ctx, BlendMode::Add)?;
            canvas.draw()?;

            graphics::set_blend_mode(ctx, BlendMode::Alpha)?;
        }

        Ok(())
    }

    fn tick_npc_splash(&mut self, state: &mut SharedGameState) {
        for npc in self.npc_list.iter_alive() {
            // Water Droplet
            if npc.npc_type == 73 {
                continue;
            }

            if !npc.splash && npc.flags.in_water() {
                let vertical_splash = !npc.flags.hit_bottom_wall() && npc.vel_y > 0x100;
                let horizontal_splash = npc.vel_x > 0x200 || npc.vel_x < -0x200;

                if vertical_splash || horizontal_splash {
                    let mut droplet = NPC::create(73, &state.npc_table);
                    droplet.cond.set_alive(true);
                    droplet.y = npc.y;
                    droplet.direction =
                        if npc.flags.water_splash_facing_right() { Direction::Right } else { Direction::Left };

                    for _ in 0..7 {
                        droplet.x = npc.x + (npc.rng.range(-8..8) * 0x200) as i32;

                        droplet.vel_x = npc.vel_x + npc.rng.range(-0x200..0x200);
                        droplet.vel_y = match () {
                            _ if vertical_splash => npc.rng.range(-0x200..0x80) - (npc.vel_y / 2),
                            _ if horizontal_splash => npc.rng.range(-0x200..0x80),
                            _ => 0,
                        };

                        let _ = self.npc_list.spawn(0x100, droplet.clone());
                    }

                    state.sound_manager.play_sfx(56);
                }

                npc.splash = true;
            }

            if !npc.flags.in_water() {
                npc.splash = false;
            }
        }
    }

    fn tick_npc_bullet_collissions(&mut self, state: &mut SharedGameState) {
        for npc in self.npc_list.iter_alive() {
            if npc.npc_flags.shootable() && npc.npc_flags.interactable() {
                continue;
            }

            for bullet in self.bullet_manager.bullets.iter_mut() {
                if !bullet.cond.alive() || bullet.damage < 0 {
                    continue;
                }

                if !npc.collides_with_bullet(bullet) {
                    continue;
                }

                if npc.npc_flags.shootable() {
                    npc.life = (npc.life as i32).saturating_sub(bullet.damage as i32).clamp(0, u16::MAX as i32) as u16;

                    if npc.life == 0 {
                        if npc.npc_flags.show_damage() {
                            npc.popup.add_value(-bullet.damage);
                        }

                        if self.player1.cond.alive() && npc.npc_flags.event_when_killed() {
                            state.control_flags.set_tick_world(true);
                            state.control_flags.set_interactions_disabled(true);
                            state.textscript_vm.start_script(npc.event_num);
                        } else {
                            npc.cond.set_explode_die(true);
                        }
                    } else {
                        if npc.shock < 14 {
                            if let Some(table_entry) = state.npc_table.get_entry(npc.npc_type) {
                                state.sound_manager.play_sfx(table_entry.hurt_sound);
                            }

                            npc.shock = 16;

                            for _ in 0..3 {
                                state.create_caret(
                                    (bullet.x + npc.x) / 2,
                                    (bullet.y + npc.y) / 2,
                                    CaretType::HurtParticles,
                                    Direction::Left,
                                );
                            }
                        }

                        if npc.npc_flags.show_damage() {
                            npc.popup.add_value(-bullet.damage);
                        }
                    }
                } else if !bullet.weapon_flags.no_proj_dissipation()
                    && bullet.btype != 13
                    && bullet.btype != 14
                    && bullet.btype != 15
                    && bullet.btype != 28
                    && bullet.btype != 29
                    && bullet.btype != 30
                {
                    state.create_caret(
                        (bullet.x + npc.x) / 2,
                        (bullet.y + npc.y) / 2,
                        CaretType::ProjectileDissipation,
                        Direction::Right,
                    );
                    state.sound_manager.play_sfx(31);
                    bullet.life = 0;
                    continue;
                }

                if bullet.life > 0 {
                    bullet.life -= 1;
                }
            }

            if npc.cond.explode_die() {
                let can_drop_missile = [&self.inventory_player1, &self.inventory_player2].iter().any(|inv| {
                    inv.has_weapon(WeaponType::MissileLauncher) || inv.has_weapon(WeaponType::SuperMissileLauncher)
                });

                self.npc_list.kill_npc(npc.id as usize, !npc.cond.drs_novanish(), can_drop_missile, state);
            }
        }

        for i in 0..self.boss.parts.len() {
            let mut idx = i;
            let mut npc = unsafe { self.boss.parts.get_unchecked_mut(i) };
            if !npc.cond.alive() {
                continue;
            }

            for bullet in self.bullet_manager.bullets.iter_mut() {
                if !bullet.cond.alive() || bullet.damage < 0 {
                    continue;
                }

                let hit = (npc.npc_flags.shootable()
                    && (npc.x - npc.hit_bounds.right as i32) < (bullet.x + bullet.enemy_hit_width as i32)
                    && (npc.x + npc.hit_bounds.right as i32) > (bullet.x - bullet.enemy_hit_width as i32)
                    && (npc.y - npc.hit_bounds.top as i32) < (bullet.y + bullet.enemy_hit_height as i32)
                    && (npc.y + npc.hit_bounds.bottom as i32) > (bullet.y - bullet.enemy_hit_height as i32))
                    || (npc.npc_flags.invulnerable()
                        && (npc.x - npc.hit_bounds.right as i32) < (bullet.x + bullet.hit_bounds.right as i32)
                        && (npc.x + npc.hit_bounds.right as i32) > (bullet.x - bullet.hit_bounds.left as i32)
                        && (npc.y - npc.hit_bounds.top as i32) < (bullet.y + bullet.hit_bounds.bottom as i32)
                        && (npc.y + npc.hit_bounds.bottom as i32) > (bullet.y - bullet.hit_bounds.top as i32));

                if !hit {
                    continue;
                }

                if npc.npc_flags.shootable() {
                    let shock = npc.shock;
                    if npc.cond.damage_boss() {
                        idx = 0;
                        npc = unsafe { self.boss.parts.get_unchecked_mut(0) };
                    }

                    npc.life = (npc.life as i32).saturating_sub(bullet.damage as i32).clamp(0, u16::MAX as i32) as u16;

                    if npc.life == 0 {
                        npc.life = npc.id;

                        if self.player1.cond.alive() && npc.npc_flags.event_when_killed() {
                            state.control_flags.set_tick_world(true);
                            state.control_flags.set_interactions_disabled(true);
                            state.textscript_vm.start_script(npc.event_num);
                        } else {
                            state.sound_manager.play_sfx(self.boss.death_sound[idx]);

                            let destroy_count = 4usize * (2usize).pow((npc.size as u32).saturating_sub(1));

                            self.npc_list.create_death_smoke(
                                npc.x,
                                npc.y,
                                npc.display_bounds.right as usize,
                                destroy_count,
                                state,
                                &npc.rng,
                            );
                            npc.cond.set_alive(false);
                        }
                    } else {
                        if shock < 14 {
                            for _ in 0..3 {
                                state.create_caret(bullet.x, bullet.y, CaretType::HurtParticles, Direction::Left);
                            }
                            state.sound_manager.play_sfx(self.boss.hurt_sound[idx]);
                        }

                        npc.shock = 8;
                        if npc.npc_flags.show_damage() {
                            npc.popup.add_value(-bullet.damage);
                        }

                        npc = unsafe { self.boss.parts.get_unchecked_mut(i) };
                        npc.shock = 8;
                    }

                    bullet.life = bullet.life.saturating_sub(1);
                    if bullet.life < 1 {
                        bullet.cond.set_alive(false);
                    }
                } else if [13, 14, 15, 28, 29, 30].contains(&bullet.btype) {
                    bullet.life = bullet.life.saturating_sub(1);
                } else if !bullet.weapon_flags.no_proj_dissipation() {
                    state.create_caret(bullet.x, bullet.y, CaretType::ProjectileDissipation, Direction::Right);
                    state.sound_manager.play_sfx(31);
                    bullet.life = 0;
                    continue;
                }
            }
        }
    }

    fn tick_world(&mut self, state: &mut SharedGameState) -> GameResult {
        self.nikumaru.tick(state, &self.player1)?;
        self.background.tick()?;
        self.hud_player1.visible = self.player1.cond.alive();
        self.hud_player2.visible = self.player2.cond.alive();
        self.hud_player1.has_player2 = self.player2.cond.alive() && !self.player2.cond.hidden();
        self.hud_player2.has_player2 = self.player1.cond.alive() && !self.player1.cond.hidden();

        self.player1.current_weapon = {
            if let Some(weapon) = self.inventory_player1.get_current_weapon_mut() {
                weapon.wtype as u8
            } else {
                0
            }
        };
        self.player2.current_weapon = {
            if let Some(weapon) = self.inventory_player2.get_current_weapon_mut() {
                weapon.wtype as u8
            } else {
                0
            }
        };
        self.player1.tick(state, &self.npc_list)?;
        self.player2.tick(state, &self.npc_list)?;
        state.textscript_vm.reset_invicibility = false;

        self.whimsical_star.tick(state, (&self.player1, &mut self.bullet_manager))?;

        if self.player1.damage > 0 {
            let xp_loss = self.player1.damage * if self.player1.equip.has_arms_barrier() { 1 } else { 2 };
            match self.inventory_player1.take_xp(xp_loss, state) {
                TakeExperienceResult::LevelDown if self.player1.life > 0 => {
                    state.create_caret(self.player1.x, self.player1.y, CaretType::LevelUp, Direction::Right);
                }
                _ => {}
            }

            self.player1.damage = 0;
        }

        if self.player2.damage > 0 {
            let xp_loss = self.player2.damage * if self.player2.equip.has_arms_barrier() { 1 } else { 2 };
            match self.inventory_player2.take_xp(xp_loss, state) {
                TakeExperienceResult::LevelDown if self.player2.life > 0 => {
                    state.create_caret(self.player2.x, self.player2.y, CaretType::LevelUp, Direction::Right);
                }
                _ => {}
            }

            self.player2.damage = 0;
        }

        for npc in self.npc_list.iter_alive() {
            npc.tick(
                state,
                (
                    [&mut self.player1, &mut self.player2],
                    &self.npc_list,
                    &mut self.stage,
                    &mut self.bullet_manager,
                    &mut self.flash,
                    &mut self.boss,
                ),
            )?;
        }
        self.boss.tick(
            state,
            (
                [&mut self.player1, &mut self.player2],
                &self.npc_list,
                &mut self.stage,
                &self.bullet_manager,
                &mut self.flash,
            ),
        )?;
        //decides if the player is tangible or not
        if !state.settings.noclip {
            self.player1.tick_map_collisions(state, &self.npc_list, &mut self.stage);
            self.player2.tick_map_collisions(state, &self.npc_list, &mut self.stage);

            self.player1.tick_npc_collisions(
                TargetPlayer::Player1,
                state,
                &self.npc_list,
                &mut self.boss,
                &mut self.inventory_player1,
            );
            self.player2.tick_npc_collisions(
                TargetPlayer::Player2,
                state,
                &self.npc_list,
                &mut self.boss,
                &mut self.inventory_player2,
            );
        }

        for npc in self.npc_list.iter_alive() {
            if !npc.npc_flags.ignore_solidity() {
                npc.tick_map_collisions(state, &self.npc_list, &mut self.stage);
            }
        }
        for npc in self.boss.parts.iter_mut() {
            if npc.cond.alive() && !npc.npc_flags.ignore_solidity() {
                npc.tick_map_collisions(state, &self.npc_list, &mut self.stage);
            }
        }

        if !self.water_params.entries.is_empty() {
            self.tick_npc_splash(state);
        }

        self.bullet_manager.tick_map_collisions(state, &self.npc_list, &mut self.stage);

        self.tick_npc_bullet_collissions(state);

        if state.control_flags.control_enabled() {
            self.inventory_player1.tick_weapons(
                state,
                &mut self.player1,
                TargetPlayer::Player1,
                &mut self.bullet_manager,
            );
            self.inventory_player2.tick_weapons(
                state,
                &mut self.player2,
                TargetPlayer::Player2,
                &mut self.bullet_manager,
            );
        }

        self.bullet_manager.tick_bullets(state, [&self.player1, &self.player2], &self.npc_list);
        state.tick_carets();

        match self.frame.update_target {
            UpdateTarget::Player => {
                if self.player2.cond.alive()
                    && !self.player2.cond.hidden()
                    && (self.player1.x - self.player2.x).abs() < 240 * 0x200
                    && (self.player1.y - self.player2.y).abs() < 200 * 0x200
                    && self.player1.control_mode != ControlMode::IronHead
                {
                    self.frame.target_x = (self.player1.target_x * 2 + self.player2.target_x) / 3;
                    self.frame.target_y = (self.player1.target_y * 2 + self.player2.target_y) / 3;

                    self.frame.target_x = self.frame.target_x.clamp(self.player1.x - 0x8000, self.player1.x + 0x8000);
                    self.frame.target_y = self.frame.target_y.clamp(self.player1.y, self.player1.y);
                } else {
                    self.frame.target_x = self.player1.target_x;
                    self.frame.target_y = self.player1.target_y;
                }

                if self.player2.cond.alive() && !self.player2.cond.hidden() {
                    if self.player2.x + 0x1000 < self.frame.x
                        || self.player2.x - 0x1000 > self.frame.x + state.canvas_size.0 as i32 * 0x200
                        || self.player2.y + 0x1000 < self.frame.y
                        || self.player2.y - 0x1000 > self.frame.y + state.canvas_size.1 as i32 * 0x200
                    {
                        self.player2.update_teleport_counter(state);

                        if self.player2.teleport_counter == 0 {
                            self.player2.x = self.player1.x;
                            self.player2.y = self.player1.y;

                            let mut npc = NPC::create(4, &state.npc_table);
                            npc.x = self.player2.x;
                            npc.y = self.player2.y;
                            npc.cond.set_alive(true);

                            let _ = self.npc_list.spawn(0x100, npc);
                        }
                    } else {
                        self.player2.teleport_counter = 0;
                    }
                }
            }
            UpdateTarget::NPC(npc_id) => {
                if let Some(npc) = self.npc_list.get_npc(npc_id as usize) {
                    if npc.cond.alive() {
                        self.frame.target_x = npc.x;
                        self.frame.target_y = npc.y;
                    }
                }
            }
            UpdateTarget::Boss(boss_id) => {
                if let Some(boss) = self.boss.parts.get(boss_id as usize) {
                    if boss.cond.alive() {
                        self.frame.target_x = boss.x;
                        self.frame.target_y = boss.y;
                    }
                }
            }
        }

        self.tilemap.tick()?;

        self.frame.update(state, &self.stage);

        if state.control_flags.control_enabled() {
            self.hud_player1.tick(state, (&self.player1, &mut self.inventory_player1))?;
            self.hud_player2.tick(state, (&self.player2, &mut self.inventory_player2))?;
            self.boss_life_bar.tick(state, (&self.npc_list, &self.boss))?;

            if state.textscript_vm.state == TextScriptExecutionState::Ended {
                if self.player1.controller.trigger_inventory() {
                    self.inventory_player1.current_item = 0;
                    state.textscript_vm.set_mode(ScriptMode::Inventory);
                    self.player1.cond.set_interacted(false);
                } else if self.player1.controller.trigger_map() && self.player1.equip.has_map() {
                    state.textscript_vm.state = TextScriptExecutionState::MapSystem;
                }
            }
        }

        if state.constants.is_switch {
            self.player1.has_dog = self.inventory_player1.has_item(14);
            self.player2.has_dog = self.inventory_player2.has_item(14);
        }

        self.water_renderer.tick(state, (&[&self.player1, &self.player2], &self.npc_list))?;

        if self.map_name_counter > 0 {
            self.map_name_counter -= 1;
        }

        Ok(())
    }

    fn draw_debug_object(
        &self,
        entity: &dyn PhysicalEntity,
        state: &mut SharedGameState,
        ctx: &mut Context,
    ) -> GameResult {
        if entity.x() < (self.frame.x - 128 - entity.display_bounds().width() as i32 * 0x200)
            || entity.x()
                > (self.frame.x + 128 + (state.canvas_size.0 as i32 + entity.display_bounds().width() as i32) * 0x200)
                && entity.y() < (self.frame.y - 128 - entity.display_bounds().height() as i32 * 0x200)
            || entity.y()
                > (self.frame.y + 128 + (state.canvas_size.1 as i32 + entity.display_bounds().height() as i32) * 0x200)
        {
            return Ok(());
        }

        {
            let hit_rect_size = entity.hit_rect_size().clamp(1, 4);
            let hit_rect_size = if state.tile_size == TileSize::Tile8x8 {
                4 * hit_rect_size * hit_rect_size
            } else {
                hit_rect_size * hit_rect_size
            };

            let tile_size = state.tile_size.as_int() * 0x200;
            let x = (entity.x() + entity.offset_x()) / tile_size;
            let y = (entity.y() + entity.offset_y()) / tile_size;

            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Caret")?;

            const CARET_RECT: Rect<u16> = Rect { left: 2, top: 74, right: 6, bottom: 78 };
            const CARET2_RECT: Rect<u16> = Rect { left: 65, top: 9, right: 71, bottom: 15 };

            for (idx, &(ox, oy)) in OFFSETS.iter().enumerate() {
                if idx == hit_rect_size {
                    break;
                }

                batch.add_rect(
                    ((x + ox) * tile_size - self.frame.x) as f32 / 512.0 - 2.0,
                    ((y + oy) * tile_size - self.frame.y) as f32 / 512.0 - 2.0,
                    &CARET_RECT,
                );
            }

            batch.add_rect(
                (entity.x() - self.frame.x) as f32 / 512.0 - 3.0,
                (entity.y() - self.frame.y) as f32 / 512.0 - 3.0,
                &CARET2_RECT,
            );

            batch.draw(ctx)?;
        }

        Ok(())
    }

    fn draw_debug_npc(&self, npc: &NPC, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.draw_debug_object(npc, state, ctx)?;

        let text = format!("{}:{}:{}", npc.id, npc.npc_type, npc.action_num);
        state
            .font
            .builder()
            .position(((npc.x - self.frame.x) / 0x200) as f32, ((npc.y - self.frame.y) / 0x200) as f32)
            .scale(0.5)
            .shadow(true)
            .color((255, 255, 0, 255))
            .draw(&text, ctx, &state.constants, &mut state.texture_set)?;

        Ok(())
    }

    fn draw_debug_outlines(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        for npc in self.npc_list.iter_alive() {
            self.draw_debug_npc(npc, state, ctx)?;
        }

        for boss in self.boss.parts.iter().filter(|n| n.cond.alive()) {
            self.draw_debug_npc(boss, state, ctx)?;
        }

        self.draw_debug_object(&self.player1, state, ctx)?;

        Ok(())
    }
}

impl Scene for GameScene {
    fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if state.mod_path.is_some() && state.replay_state == ReplayState::Recording {
            self.replay.initialize_recording(state);
        }
        if state.player_count == PlayerCount::Two {
            self.add_player2(state, ctx);
        } else {
            self.drop_player2();
        }

        if state.mod_path.is_some() {
            if let ReplayState::Playback(replay_kind) = state.replay_state {
                self.replay.initialize_playback(state, ctx, replay_kind)?;
            }
        }

        self.npc_list.set_rng_seed(state.game_rng.next());
        self.boss.init_rng(state.game_rng.next());
        state.textscript_vm.set_scene_script(self.stage.load_text_script(
            &state.constants.base_paths,
            &state.constants,
            ctx,
        )?);
        state.textscript_vm.suspend = false;
        state.tile_size = self.stage.map.tile_size;
        #[cfg(feature = "scripting-lua")]
        state.lua.set_game_scene(self as *mut _);

        self.player1.controller = state.settings.create_player1_controller();
        self.player2.controller = state.settings.create_player2_controller();

        let npcs = self.stage.load_npcs(&state.constants.base_paths, ctx)?;
        for npc_data in npcs.iter() {
            log::info!("creating npc: {:?}", npc_data);

            let mut npc = NPC::create_from_data(npc_data, &state.npc_table, state.tile_size);
            if npc.npc_flags.appear_when_flag_set() {
                if state.get_flag(npc_data.flag_num as _) {
                    npc.cond.set_alive(true);
                }
            } else if npc.npc_flags.hide_unless_flag_set() {
                if !state.get_flag(npc_data.flag_num as _) {
                    npc.cond.set_alive(true);
                }
            } else {
                npc.cond.set_alive(true);
            }

            self.npc_list.spawn_at_slot(npc_data.id, npc)?;
        }

        state.npc_table.stage_textures = self.stage_textures.clone();

        self.boss.boss_type = self.stage.data.boss_no as u16;
        self.player1.target_x = self.player1.x;
        self.player1.target_y = self.player1.y;
        self.player1.camera_target_x = 0;
        self.player1.camera_target_y = 0;
        self.player2.target_x = self.player2.x;
        self.player2.target_y = self.player2.y;
        self.player2.camera_target_x = 0;
        self.player2.camera_target_y = 0;
        self.frame.target_x = self.player1.x;
        self.frame.target_y = self.player1.y;
        self.frame.immediate_update(state, &self.stage);

        // I'd personally set it to something higher but left it as is for accuracy.
        state.water_level = 0x1e0000;

        state.carets.clear();

        self.lighting_mode = match () {
            _ if self.intro_mode => LightingMode::None,
            _ if !state.constants.is_switch
                && (self.stage.data.background_type == BackgroundType::Black
                    || self.stage.data.background.name() == "bkBlack") =>
            {
                LightingMode::Ambient
            }
            _ if state.constants.is_switch
                && (self.stage.data.background_type == BackgroundType::Black
                    || self.stage.data.background.name() == "bkBlack") =>
            {
                LightingMode::None
            }
            _ if self.stage.data.background.name() == "bkFall" => LightingMode::None,
            _ if self.stage.data.background_type != BackgroundType::Black
                && self.stage.data.background_type != BackgroundType::Outside
                && self.stage.data.background_type != BackgroundType::OutsideWind
                && self.stage.data.background.name() != "bkBlack" =>
            {
                LightingMode::BackgroundOnly
            }
            _ => LightingMode::None,
        };

        self.pause_menu.init(state, ctx)?;
        self.whimsical_star.init(&self.player1);

        #[cfg(feature = "discord-rpc")]
        {
            if self.stage.data.map == state.stages[state.constants.game.intro_stage as usize].map {
                state.discord_rpc.set_initializing()?;
            } else {
                state.discord_rpc.update_hp(&self.player1)?;
                state.discord_rpc.update_stage(&self.stage.data)?;
                state.discord_rpc.set_in_game()?;
            }
        }

        Ok(())
    }

    fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if !self.pause_menu.is_paused() {
            if let ReplayState::Playback(_) = state.replay_state {
                self.replay.tick(state, (ctx, &mut self.player1))?;
            }
        }

        if state.player_count_modified_in_game {
            if state.player_count == PlayerCount::Two {
                self.add_player2(state, ctx);
            } else {
                self.drop_player2();
            }

            state.player_count_modified_in_game = false;
        }

        self.player1.controller.update(state, ctx)?;
        self.player1.controller.update_trigger();
        self.player2.controller.update(state, ctx)?;
        self.player2.controller.update_trigger();

        state.touch_controls.control_type = if state.control_flags.control_enabled() && !self.pause_menu.is_paused() {
            TouchControlType::Controls
        } else {
            TouchControlType::None
        };

        if state.settings.touch_controls {
            state.touch_controls.interact_icon = false;
        }

        if self.intro_mode {
            state.touch_controls.control_type = TouchControlType::Dialog;

            if let TextScriptExecutionState::WaitTicks(_, _, 9999) = state.textscript_vm.state {
                state.next_scene = Some(Box::new(TitleScene::new()));
            }

            if self.player1.controller.trigger_menu_ok() {
                state.next_scene = Some(Box::new(TitleScene::new()));
            }
        }

        if self.player1.controller.trigger_menu_pause() {
            self.pause_menu.pause(state);
        }

        if self.pause_menu.is_paused() {
            self.pause_menu.tick(state, ctx)?;
            return Ok(());
        }

        if state.replay_state == ReplayState::Recording {
            self.replay.tick(state, (ctx, &mut self.player1))?;
        }

        match state.textscript_vm.state {
            TextScriptExecutionState::Running(_, _)
            | TextScriptExecutionState::WaitTicks(_, _, _)
            | TextScriptExecutionState::WaitInput(_, _, _)
            | TextScriptExecutionState::WaitStanding(_, _)
            | TextScriptExecutionState::WaitFade(_, _)
            | TextScriptExecutionState::Msg(_, _, _, _)
            | TextScriptExecutionState::MsgNewLine(_, _, _, _, _)
            | TextScriptExecutionState::FallingIsland(_, _, _, _, _, _)
                if !state.control_flags.control_enabled() =>
            {
                state.touch_controls.control_type = TouchControlType::Dialog;
                match state.settings.cutscene_skip_mode {
                    CutsceneSkipMode::Hold if !state.textscript_vm.flags.cutscene_skip() => {
                        if self.player1.controller.skip() {
                            self.skip_counter += 1;
                            if self.skip_counter >= CUTSCENE_SKIP_WAIT {
                                state.textscript_vm.flags.set_cutscene_skip(true);
                                state.tutorial_counter = 0;
                            }
                        } else if self.skip_counter > 0 {
                            self.skip_counter -= 1;
                        }
                    }
                    CutsceneSkipMode::FastForward => {
                        if self.player1.controller.skip() {
                            state.textscript_vm.flags.set_cutscene_skip(true);
                        } else {
                            state.textscript_vm.flags.set_cutscene_skip(false);
                        }
                    }
                    _ => (),
                }
            }
            _ => {
                self.skip_counter = 0;
            }
        }

        self.map_system.tick(state, ctx, &self.stage, [&self.player1, &self.player2])?;

        match state.textscript_vm.mode {
            ScriptMode::Map | ScriptMode::Debug => {
                TextScriptVM::run(state, self, ctx)?;

                match state.textscript_vm.state {
                    TextScriptExecutionState::FallingIsland(_, _, _, _, _, _) => (),
                    TextScriptExecutionState::MapSystem => (),
                    _ => {
                        if state.control_flags.tick_world() {
                            self.tick_world(state)?;
                        }
                    }
                }
            }
            ScriptMode::StageSelect => {
                self.stage_select.tick(state, (ctx, &self.player1, &self.player2))?;

                TextScriptVM::run(state, self, ctx)?;
            }
            ScriptMode::Inventory => {
                self.inventory_ui
                    .tick(state, (ctx, &mut self.player1, &mut self.inventory_player1, &mut self.hud_player1))?;

                TextScriptVM::run(state, self, ctx)?;
            }
        }

        if state.control_flags.credits_running() {
            self.skip_counter = 0;
            CreditScriptVM::run(state, ctx)?;
        }

        self.fade.tick(state, ())?;
        self.flash.tick(state, ())?;
        self.text_boxes.tick(state, ())?;

        #[cfg(feature = "scripting-lua")]
        state.lua.scene_tick();

        if state.control_flags.tick_world() {
            self.tick = self.tick.wrapping_add(1);
        }

        if state.tutorial_counter > 0 {
            state.tutorial_counter = state.tutorial_counter.saturating_sub(1);
            if state.control_flags.control_enabled() {
                state.tutorial_counter = 0;
            }
        }

        if state.quake_rumble_counter > 0 {
            gamepad::set_quake_rumble_all(ctx, state, state.quake_rumble_counter)?;
            state.quake_rumble_counter = 0;
        }

        if state.super_quake_rumble_counter > 0 {
            gamepad::set_super_quake_rumble_all(ctx, state, state.super_quake_rumble_counter)?;
            state.super_quake_rumble_counter = 0;
        }

        Ok(())
    }

    fn draw_tick(&mut self, state: &mut SharedGameState) -> GameResult {
        self.frame.prev_x = self.frame.x;
        self.frame.prev_y = self.frame.y;
        self.player1.prev_x = self.player1.x;
        self.player1.prev_y = self.player1.y;
        self.player1.damage_popup.prev_x = self.player1.damage_popup.x;
        self.player1.damage_popup.prev_y = self.player1.damage_popup.y;
        self.player1.exp_popup.prev_x = self.player1.exp_popup.x;
        self.player1.exp_popup.prev_y = self.player1.exp_popup.y;
        self.player2.prev_x = self.player2.x;
        self.player2.prev_y = self.player2.y;
        self.player2.damage_popup.prev_x = self.player2.damage_popup.x;
        self.player2.damage_popup.prev_y = self.player2.damage_popup.y;
        self.player2.exp_popup.prev_x = self.player2.exp_popup.x;
        self.player2.exp_popup.prev_y = self.player2.exp_popup.y;

        for npc in self.npc_list.iter_alive() {
            npc.prev_x = npc.x;
            npc.prev_y = npc.y;
            npc.popup.prev_x = npc.prev_x;
            npc.popup.prev_y = npc.prev_y;
        }

        for npc in self.boss.parts.iter_mut() {
            if npc.cond.alive() {
                npc.prev_x = npc.x;
                npc.prev_y = npc.y;
                npc.popup.prev_x = npc.prev_x;
                npc.popup.prev_y = npc.prev_y;
            }
        }

        for bullet in self.bullet_manager.bullets.iter_mut() {
            if bullet.cond.alive() {
                bullet.prev_x = bullet.x;
                bullet.prev_y = bullet.y;
            }
        }

        for caret in state.carets.iter_mut() {
            if caret.cond.alive() {
                caret.prev_x = caret.x;
                caret.prev_y = caret.y;
            }
        }

        self.whimsical_star.set_prev();

        self.tilemap.set_prev()?;

        self.inventory_dim += 0.1
            * if state.textscript_vm.mode == ScriptMode::Inventory
                || state.textscript_vm.state == TextScriptExecutionState::MapSystem
                || self.pause_menu.is_paused()
            {
                state.frame_time as f32
            } else {
                -(state.frame_time as f32)
            };

        self.inventory_dim = self.inventory_dim.clamp(0.0, 1.0);
        self.background.draw_tick()?;
        self.credits.draw_tick(state);

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        //graphics::set_canvas(ctx, Some(&state.game_canvas));

        if self.player1.control_mode == ControlMode::IronHead {
            self.set_ironhead_clip(state, ctx)?;
        }

        let stage_textures_ref = &*self.stage_textures.deref().borrow();
        self.background.draw(state, ctx, &self.frame, stage_textures_ref, &self.stage)?;
        self.tilemap.draw(state, ctx, &self.frame, TileLayer::Background, stage_textures_ref, &self.stage)?;
        self.draw_npc_layer(state, ctx, NPCLayer::Background)?;
        self.tilemap.draw(state, ctx, &self.frame, TileLayer::Middleground, stage_textures_ref, &self.stage)?;

        if state.settings.shader_effects && self.lighting_mode == LightingMode::BackgroundOnly {
            self.draw_light_map(state, ctx)?;
        }

        self.boss.draw(state, ctx, &self.frame)?;
        self.draw_npc_layer(state, ctx, NPCLayer::Middleground)?;
        self.draw_bullets(state, ctx)?;
        self.player2.draw(state, ctx, &self.frame)?;
        self.player1.draw(state, ctx, &self.frame)?;

        if !self.player1.cond.hidden() {
            self.whimsical_star.draw(state, ctx, &self.frame)?;
        }

        self.water_renderer.draw(state, ctx, &self.frame, WaterLayer::Back)?;
        self.tilemap.draw(state, ctx, &self.frame, TileLayer::Foreground, stage_textures_ref, &self.stage)?;
        self.tilemap.draw(state, ctx, &self.frame, TileLayer::Snack, stage_textures_ref, &self.stage)?;
        self.water_renderer.draw(state, ctx, &self.frame, WaterLayer::Front)?;

        self.draw_carets(state, ctx)?;
        self.player1.exp_popup.draw(state, ctx, &self.frame)?;
        self.player1.damage_popup.draw(state, ctx, &self.frame)?;
        self.player2.exp_popup.draw(state, ctx, &self.frame)?;
        self.player2.damage_popup.draw(state, ctx, &self.frame)?;
        self.draw_npc_popup(state, ctx)?;
        self.draw_boss_popup(state, ctx)?;

        if !state.control_flags.credits_running()
            && state.settings.shader_effects
            && self.lighting_mode == LightingMode::Ambient
        {
            self.draw_light_map(state, ctx)?;
        }
        self.flash.draw(state, ctx, &self.frame)?;

        self.draw_black_bars(state, ctx)?;

        if self.player1.control_mode == ControlMode::IronHead {
            graphics::set_clip_rect(ctx, None)?;
        }

        if self.inventory_dim > 0.0 {
            let rect = Rect::new(0, 0, state.screen_size.0 as isize + 1, state.screen_size.1 as isize + 1);
            let mut dim_color = state.constants.inventory_dim_color;
            dim_color.a *= self.inventory_dim;
            graphics::draw_rect(ctx, rect, dim_color)?;
        }

        match state.textscript_vm.mode {
            ScriptMode::Map | ScriptMode::Debug if state.control_flags.control_enabled() => {
                self.hud_player1.draw(state, ctx, &self.frame)?;
                self.hud_player2.draw(state, ctx, &self.frame)?;
                self.boss_life_bar.draw(state, ctx, &self.frame)?;

                if self.player2.cond.alive() && !self.player2.cond.hidden() {
                    if self.player2.teleport_counter < state.settings.timing_mode.get_tps() as u16 * 3
                        || self.player2.teleport_counter % 5 != 0
                    {
                        if self.player2.y + 0x1000 < self.frame.y {
                            let scale = 1.0 + (self.frame.y as f32 / self.player2.y as f32 / 2.0 - 0.5).clamp(0.0, 2.0);

                            let x = interpolate_fix9_scale(
                                self.player2.prev_x - self.frame.prev_x,
                                self.player2.x - self.frame.x,
                                state.frame_time,
                            );

                            let x = x.clamp(8.0, state.canvas_size.0 - 8.0 * scale - state.font.line_height());

                            state
                                .font
                                .builder()
                                .position(x, 8.0)
                                .scale(scale)
                                .shadow_color((0, 0, 130, 255))
                                .color((96, 96, 255, 255))
                                .shadow(true)
                                .draw(P2_OFFSCREEN_TEXT, ctx, &state.constants, &mut state.texture_set)?;
                        } else if self.player2.y - 0x1000 > self.frame.y + state.canvas_size.1 as i32 * 0x200 {
                            let scale = 1.0
                                + (self.player2.y as f32 / (self.frame.y as f32 + state.canvas_size.1 * 0x200 as f32)
                                    - 0.5)
                                    .clamp(0.0, 2.0);

                            let x = interpolate_fix9_scale(
                                self.player2.prev_x - self.frame.prev_x,
                                self.player2.x - self.frame.x,
                                state.frame_time,
                            );

                            let x = x.clamp(8.0, state.canvas_size.0 - 8.0 * scale - state.font.line_height());

                            state
                                .font
                                .builder()
                                .position(x, state.canvas_size.1 - 8.0 * scale - state.font.line_height())
                                .scale(scale)
                                .shadow_color((0, 0, 130, 255))
                                .color((96, 96, 255, 255))
                                .shadow(true)
                                .draw(P2_OFFSCREEN_TEXT, ctx, &state.constants, &mut state.texture_set)?;
                        } else if self.player2.x + 0x1000 < self.frame.x {
                            let scale = 1.0 + (self.frame.x as f32 / self.player2.x as f32 / 2.0 - 0.5).clamp(0.0, 2.0);

                            let y = interpolate_fix9_scale(
                                self.player2.prev_y - self.frame.prev_y,
                                self.player2.y - self.frame.y,
                                state.frame_time,
                            );
                            let y = y.clamp(8.0, state.canvas_size.1 - 8.0 * scale - state.font.line_height());

                            state
                                .font
                                .builder()
                                .position(8.0, y)
                                .scale(scale)
                                .shadow_color((0, 0, 130, 255))
                                .color((96, 96, 255, 255))
                                .shadow(true)
                                .draw(P2_OFFSCREEN_TEXT, ctx, &state.constants, &mut state.texture_set)?;
                        } else if self.player2.x - 0x1000 > self.frame.x + state.canvas_size.0 as i32 * 0x200 {
                            let scale = 1.0
                                + (self.player2.x as f32 / (self.frame.x as f32 + state.canvas_size.0 * 0x200 as f32)
                                    - 0.5)
                                    .clamp(0.0, 2.0);

                            let y = interpolate_fix9_scale(
                                self.player2.prev_y - self.frame.prev_y,
                                self.player2.y - self.frame.y,
                                state.frame_time,
                            );
                            let y = y.clamp(8.0, state.canvas_size.1 - 8.0 * scale - state.font.line_height());

                            let width = state.font.builder().compute_width(P2_OFFSCREEN_TEXT);

                            state
                                .font
                                .builder()
                                .shadow_color((0, 0, 130, 255))
                                .color((96, 96, 255, 255))
                                .shadow(true)
                                .position(state.canvas_size.0 - width - 8.0 * scale, y)
                                .scale(scale)
                                .draw(P2_OFFSCREEN_TEXT, ctx, &state.constants, &mut state.texture_set)?;
                        }
                    }
                }
            }
            ScriptMode::StageSelect => self.stage_select.draw(state, ctx, &self.frame)?,
            ScriptMode::Inventory => self.inventory_ui.draw(state, ctx, &self.frame)?,
            _ => {}
        }

        self.map_system.draw(state, ctx, &self.stage, [&self.player1, &self.player2])?;
        self.fade.draw(state, ctx, &self.frame)?;

        if state.textscript_vm.mode == ScriptMode::Map || state.textscript_vm.mode == ScriptMode::Debug {
            self.nikumaru.draw(state, ctx, &self.frame)?;
        }

        if (state.textscript_vm.mode == ScriptMode::Map || state.textscript_vm.mode == ScriptMode::Debug)
            && state.textscript_vm.state != TextScriptExecutionState::MapSystem
            && self.map_name_counter > 0
        {
            let map_name = if self.stage.data.name == "u" {
                state.constants.title.intro_text.as_str()
            } else {
                if state.constants.is_cs_plus && state.settings.locale == "jp" {
                    self.stage.data.name_jp.as_str()
                } else {
                    self.stage.data.name.as_str()
                }
            };

            state.font.builder().shadow(true).y(80.0).center(state.canvas_size.0).draw(
                map_name,
                ctx,
                &state.constants,
                &mut state.texture_set,
            )?;
        }

        if state.control_flags.credits_running() {
            self.credits.draw(state, ctx, &self.frame)?;
        }

        self.falling_island.draw(state, ctx, &self.frame)?;
        self.text_boxes.draw(state, ctx, &self.frame)?;

        if self.skip_counter > 1 || state.tutorial_counter > 0 {
            let key = {
                if state.settings.touch_controls {
                    ">>".to_owned()
                } else {
                    match state.settings.player1_controller_type {
                        ControllerType::Keyboard => format!("{:?}", state.settings.player1_key_map.skip),
                        ControllerType::Gamepad(_) => "=".to_owned(),
                    }
                }
            };

            let text = state.tt("game.cutscene_skip", &[("key", key.as_str())]);

            let gamepad_sprite_offset = match state.settings.player1_controller_type {
                ControllerType::Keyboard => 1,
                ControllerType::Gamepad(index) => ctx.gamepad_context.get_gamepad_sprite_offset(index as usize),
            };

            let symbols = Symbols {
                symbols: &[(
                    '=',
                    state.settings.player1_controller_button_map.skip.get_rect(gamepad_sprite_offset, &state.constants),
                )],
                texture: "buttons",
            };

            // let width = state.font.text_width_with_rects(text.chars(), &rect_map, &state.constants);
            let width = state.font.builder().with_symbols(Some(symbols)).compute_width(&text);
            let pos_x = state.canvas_size.0 - width - 20.0;
            let pos_y = 0.0;
            let line_height = state.font.line_height();
            let w = (self.skip_counter as f32 / CUTSCENE_SKIP_WAIT as f32) * (width + 20.0) / 2.0;
            let mut rect = Rect::new_size(
                (pos_x * state.scale) as isize,
                (pos_y * state.scale) as isize,
                ((20.0 + width) * state.scale) as isize,
                ((10.0 + line_height) * state.scale) as isize,
            );

            draw_rect(ctx, rect, Color::from_rgb(0, 0, 32))?;

            rect.right = rect.left + (w * state.scale) as isize;
            draw_rect(ctx, rect, Color::from_rgb(128, 128, 160))?;

            rect.left = ((state.canvas_size.0 - w) * state.scale) as isize;
            rect.right = rect.left + (w * state.scale).ceil() as isize;
            draw_rect(ctx, rect, Color::from_rgb(128, 128, 160))?;

            state.font.builder().position(pos_x + 10.0, pos_y + 5.0).shadow(true).with_symbols(Some(symbols)).draw(
                &text,
                ctx,
                &state.constants,
                &mut state.texture_set,
            )?;
        }

        if state.settings.debug_outlines {
            self.draw_debug_outlines(state, ctx)?;
        }

        if state.settings.god_mode {
            let debug_name = "GOD";
            state
                .font
                .builder()
                .x(state.canvas_size.0 - state.font.builder().compute_width(debug_name) - 10.0)
                .y(20.0)
                .shadow(true)
                .draw(debug_name, ctx, &state.constants, &mut state.texture_set)?;
        }

        if state.settings.infinite_booster {
            let debug_name = "INF.B";
            state
                .font
                .builder()
                .x(state.canvas_size.0 - state.font.builder().compute_width(debug_name) - 10.0)
                .y(32.0)
                .shadow(true)
                .draw(debug_name, ctx, &state.constants, &mut state.texture_set)?;
        }

        if state.settings.speed != 1.0 {
            let debug_name = &format!("{:.1}x SPD", state.settings.speed);
            state
                .font
                .builder()
                .x(state.canvas_size.0 - state.font.builder().compute_width(debug_name) - 10.0)
                .y(44.0)
                .shadow(true)
                .draw(debug_name, ctx, &state.constants, &mut state.texture_set)?;
        }

        if state.settings.noclip {
            let debug_name = "NOCLIP";
            state
                .font
                .builder()
                .x(state.canvas_size.0 - state.font.builder().compute_width(debug_name) - 10.0)
                .y(56.0)
                .shadow(true)
                .draw(debug_name, ctx, &state.constants, &mut state.texture_set)?;
        }

        self.replay.draw(state, ctx, &self.frame)?;

        self.pause_menu.draw(state, ctx)?;

        //draw_number(state.canvas_size.0 - 8.0, 8.0, timer::fps(ctx) as usize, Alignment::Right, state, ctx)?;
        Ok(())
    }

    fn imgui_draw(
        &mut self,
        components: &mut Components,
        state: &mut SharedGameState,
        ctx: &mut Context,
        ui: &mut imgui::Ui,
    ) -> GameResult {
        components.live_debugger.run_ingame(self, state, ctx, ui)?;
        Ok(())
    }

    fn process_debug_keys(&mut self, state: &mut SharedGameState, ctx: &mut Context, key_code: ScanCode) -> GameResult {
        if key_code == ScanCode::F3 && ctx.keyboard_context.active_mods().ctrl() {
            let _ = state.sound_manager.reload();
            return Ok(());
        }

        if key_code == ScanCode::S && ctx.keyboard_context.active_mods().ctrl() {
            let _ = state.save_game(self, ctx);
            state.sound_manager.play_sfx(18);
            return Ok(());
        }

        #[cfg(not(debug_assertions))]
        if !state.settings.debug_mode {
            return Ok(());
        }

        match key_code {
            ScanCode::F3 => state.settings.god_mode = !state.settings.god_mode,
            ScanCode::F4 => state.settings.infinite_booster = !state.settings.infinite_booster,
            ScanCode::F5 => state.settings.subpixel_coords = !state.settings.subpixel_coords,
            ScanCode::F6 => state.settings.motion_interpolation = !state.settings.motion_interpolation,
            ScanCode::F7 => state.set_speed(1.0),
            ScanCode::F8 => {
                if state.settings.speed > 0.2 {
                    state.set_speed(state.settings.speed - 0.1);
                }
            }
            ScanCode::F9 => {
                if state.settings.speed < 3.0 {
                    state.set_speed(state.settings.speed + 0.1);
                }
            }
            ScanCode::F10 => state.settings.debug_outlines = !state.settings.debug_outlines,
            ScanCode::F11 => state.settings.fps_counter = !state.settings.fps_counter,
            ScanCode::F12 => state.debugger = !state.debugger,
            ScanCode::Grave => {
                state.command_line = !state.command_line;

                if !state.command_line {
                    state.control_flags.set_tick_world(true);
                }
            }
            _ => {}
        };

        Ok(())
    }
}
