use std::ops::Deref;

use ggez::{Context, GameResult, graphics, timer};
use ggez::graphics::{BlendMode, Color, Drawable, DrawParam, FilterMode, mint};
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::nalgebra::{clamp, Vector2};
use log::info;
use num_traits::abs;

use crate::bullet::BulletManager;
use crate::caret::CaretType;
use crate::common::{Direction, FadeDirection, FadeState, fix9_scale, interpolate_fix9_scale, Rect};
use crate::components::boss_life_bar::BossLifeBar;
use crate::components::draw_common::{Alignment, draw_number};
use crate::components::hud::HUD;
use crate::components::stage_select::StageSelect;
use crate::entity::GameEntity;
use crate::frame::{Frame, UpdateTarget};
use crate::inventory::{Inventory, TakeExperienceResult};
use crate::npc::{NPC, NPCMap};
use crate::physics::PhysicalEntity;
use crate::player::{Player, PlayerAppearance, TargetPlayer};
use crate::rng::RNG;
use crate::scene::Scene;
use crate::scene::title_scene::TitleScene;
use crate::shared_game_state::{Season, SharedGameState};
use crate::stage::{BackgroundType, Stage};
use crate::text_script::{ConfirmSelection, ScriptMode, TextScriptExecutionState, TextScriptVM};
use crate::texture_set::SizedBatch;
use crate::ui::Components;
use crate::weapon::WeaponType;

pub struct GameScene {
    pub tick: usize,
    pub stage: Stage,
    pub boss_life_bar: BossLifeBar,
    pub stage_select: StageSelect,
    pub hud_player1: HUD,
    pub hud_player2: HUD,
    pub frame: Frame,
    pub player1: Player,
    pub player2: Player,
    pub inventory_player1: Inventory,
    pub inventory_player2: Inventory,
    pub stage_id: usize,
    pub npc_map: NPCMap,
    pub bullet_manager: BulletManager,
    pub intro_mode: bool,
    water_visible: bool,
    tex_background_name: String,
    tex_tileset_name: String,
    map_name_counter: u16,
}

#[derive(Debug, EnumIter, PartialEq, Eq, Hash, Copy, Clone)]
pub enum TileLayer {
    All,
    Background,
    Foreground,
    Snack,
}

const FACE_TEX: &str = "Face";
const SWITCH_FACE_TEX: [&str; 4] = ["Face1", "Face2", "Face3", "Face4"];
const P2_LEFT_TEXT: &str = "< P2";
const P2_RIGHT_TEXT: &str = "P2 >";

impl GameScene {
    pub fn new(state: &mut SharedGameState, ctx: &mut Context, id: usize) -> GameResult<Self> {
        info!("Loading stage {} ({})", id, &state.stages[id].map);
        let stage = Stage::load(&state.base_path, &state.stages[id], ctx)?;
        info!("Loaded stage: {}", stage.data.name);

        let tex_background_name = stage.data.background.filename();
        let tex_tileset_name = ["Stage/", &stage.data.tileset.filename()].join("");

        Ok(Self {
            tick: 0,
            stage,
            player1: Player::new(state),
            player2: Player::new(state),
            inventory_player1: Inventory::new(),
            inventory_player2: Inventory::new(),
            boss_life_bar: BossLifeBar::new(),
            stage_select: StageSelect::new(),
            hud_player1: HUD::new(Alignment::Left),
            hud_player2: HUD::new(Alignment::Right),
            frame: Frame {
                x: 0,
                y: 0,
                prev_x: 0,
                prev_y: 0,
                update_target: UpdateTarget::Player,
                target_x: 0,
                target_y: 0,
                wait: 16,
            },
            stage_id: id,
            npc_map: NPCMap::new(),
            bullet_manager: BulletManager::new(),
            intro_mode: false,
            water_visible: true,
            tex_background_name,
            tex_tileset_name,
            map_name_counter: 0,
        })
    }

    pub fn display_map_name(&mut self, ticks: u16) {
        self.map_name_counter = ticks;
    }

    pub fn add_player2(&mut self) {
        self.player2.cond.set_alive(true);
        self.player2.cond.set_hidden(self.player1.cond.hidden());
        self.player2.appearance = PlayerAppearance::YellowQuote;
        self.player2.x = self.player1.x;
        self.player2.y = self.player1.y;
        self.player2.vel_x = self.player1.vel_x;
        self.player2.vel_y = self.player1.vel_y;
    }

    pub fn drop_player2(&mut self) {
        self.player2.cond.set_alive(false);
    }

    fn draw_background(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, &self.tex_background_name)?;
        let scale = state.scale;
        let (frame_x, frame_y) = self.frame.xy_interpolated(state.frame_time, state.scale);

        match self.stage.data.background_type {
            BackgroundType::Stationary => {
                graphics::clear(ctx, Color::from_rgb(0, 0, 0));

                let count_x = state.canvas_size.0 as usize / batch.width() + 1;
                let count_y = state.canvas_size.1 as usize / batch.height() + 1;

                for y in 0..count_y {
                    for x in 0..count_x {
                        batch.add((x * batch.width()) as f32, (y * batch.height()) as f32);
                    }
                }
            }
            BackgroundType::MoveDistant | BackgroundType::MoveNear => {
                graphics::clear(ctx, Color::from_rgb(0, 0, 0));

                let (off_x, off_y) = if self.stage.data.background_type == BackgroundType::MoveNear {
                    (
                        frame_x % (batch.width() as f32),
                        frame_y % (batch.height() as f32)
                    )
                } else {
                    (
                        ((frame_x / 2.0 * scale).floor() / scale) % (batch.width() as f32),
                        ((frame_y / 2.0 * scale).floor() / scale) % (batch.height() as f32)
                    )
                };

                let count_x = state.canvas_size.0 as usize / batch.width() + 2;
                let count_y = state.canvas_size.1 as usize / batch.height() + 2;

                for y in 0..count_y {
                    for x in 0..count_x {
                        batch.add((x * batch.width()) as f32 - off_x,
                                  (y * batch.height()) as f32 - off_y);
                    }
                }
            }
            BackgroundType::Water => {
                graphics::clear(ctx, Color::from_rgb(0, 0, 32));
            }
            BackgroundType::Black => {
                graphics::clear(ctx, Color::from_rgb(0, 0, 32));
            }
            BackgroundType::Autoscroll => {}
            BackgroundType::OutsideWind | BackgroundType::Outside => {
                graphics::clear(ctx, Color::from_rgb(0, 0, 0));

                let offset = (self.tick % 640) as isize;

                for x in (0..(state.canvas_size.0 as isize)).step_by(200) {
                    batch.add_rect(x as f32, 0.0,
                                   &Rect::new_size(0, 0, 200, 88));
                }

                batch.add_rect(state.canvas_size.0 - 320.0, 0.0,
                               &Rect::new_size(0, 0, 320, 88));

                for x in ((-offset / 2)..(state.canvas_size.0 as isize)).step_by(320) {
                    batch.add_rect(x as f32, 88.0,
                                   &Rect::new_size(0, 88, 320, 35));
                }

                for x in ((-offset % 320)..(state.canvas_size.0 as isize)).step_by(320) {
                    batch.add_rect(x as f32, 123.0,
                                   &Rect::new_size(0, 123, 320, 23));
                }

                for x in ((-offset * 2)..(state.canvas_size.0 as isize)).step_by(320) {
                    batch.add_rect(x as f32, 146.0,
                                   &Rect::new_size(0, 146, 320, 30));
                }

                for x in ((-offset * 4)..(state.canvas_size.0 as isize)).step_by(320) {
                    batch.add_rect(x as f32, 176.0,
                                   &Rect::new_size(0, 176, 320, 64));
                }
            }
        }

        batch.draw(ctx)?;

        Ok(())
    }

    fn draw_bullets(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Bullet")?;
        let mut x: isize;
        let mut y: isize;
        let mut prev_x: isize;
        let mut prev_y: isize;

        for bullet in self.bullet_manager.bullets.iter() {
            match bullet.direction {
                Direction::Left => {
                    x = bullet.x - bullet.display_bounds.left as isize;
                    y = bullet.y - bullet.display_bounds.top as isize;
                    prev_x = bullet.prev_x - bullet.display_bounds.left as isize;
                    prev_y = bullet.prev_y - bullet.display_bounds.top as isize;
                }
                Direction::Up => {
                    x = bullet.x - bullet.display_bounds.top as isize;
                    y = bullet.y - bullet.display_bounds.left as isize;
                    prev_x = bullet.prev_x - bullet.display_bounds.top as isize;
                    prev_y = bullet.prev_y - bullet.display_bounds.left as isize;
                }
                Direction::Right => {
                    x = bullet.x - bullet.display_bounds.right as isize;
                    y = bullet.y - bullet.display_bounds.top as isize;
                    prev_x = bullet.prev_x - bullet.display_bounds.right as isize;
                    prev_y = bullet.prev_y - bullet.display_bounds.top as isize;
                }
                Direction::Bottom => {
                    x = bullet.x - bullet.display_bounds.top as isize;
                    y = bullet.y - bullet.display_bounds.right as isize;
                    prev_x = bullet.prev_x - bullet.display_bounds.top as isize;
                    prev_y = bullet.prev_y - bullet.display_bounds.right as isize;
                }
                Direction::FacingPlayer => unreachable!(),
            }

            batch.add_rect(interpolate_fix9_scale(prev_x - self.frame.prev_x,
                                                  x - self.frame.x,
                                                  state.frame_time),
                           interpolate_fix9_scale(prev_y - self.frame.prev_y,
                                                  y - self.frame.y,
                                                  state.frame_time),
                           &bullet.anim_rect);
        }

        batch.draw(ctx)?;
        Ok(())
    }

    fn draw_carets(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Caret")?;

        for caret in state.carets.iter() {
            batch.add_rect(interpolate_fix9_scale(caret.prev_x - caret.offset_x - self.frame.prev_x,
                                                  caret.x - caret.offset_x - self.frame.x,
                                                  state.frame_time),
                           interpolate_fix9_scale(caret.prev_y - caret.offset_y - self.frame.prev_y,
                                                  caret.y - caret.offset_y - self.frame.y,
                                                  state.frame_time),
                           &caret.anim_rect);
        }

        batch.draw(ctx)?;
        Ok(())
    }

    fn draw_fade(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        match state.fade_state {
            FadeState::Visible => { return Ok(()); }
            FadeState::Hidden => {
                graphics::clear(ctx, Color::from_rgb(0, 0, 32));
            }
            FadeState::FadeIn(tick, direction) | FadeState::FadeOut(tick, direction) => {
                let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Fade")?;
                let mut rect = Rect::new(0, 0, 16, 16);

                match direction {
                    FadeDirection::Left | FadeDirection::Right => {
                        let mut frame = tick;

                        for x in (0..(state.canvas_size.0 as isize + 16)).step_by(16) {
                            if frame > 15 { frame = 15; } else { frame += 1; }

                            if frame >= 0 {
                                rect.left = frame as u16 * 16;
                                rect.right = rect.left + 16;

                                for y in (0..(state.canvas_size.1 as isize + 16)).step_by(16) {
                                    if direction == FadeDirection::Left {
                                        batch.add_rect(state.canvas_size.0 - x as f32, y as f32, &rect);
                                    } else {
                                        batch.add_rect(x as f32, y as f32, &rect);
                                    }
                                }
                            }
                        }
                    }
                    FadeDirection::Up | FadeDirection::Down => {
                        let mut frame = tick;

                        for y in (0..(state.canvas_size.1 as isize + 16)).step_by(16) {
                            if frame > 15 { frame = 15; } else { frame += 1; }

                            if frame >= 0 {
                                rect.left = frame as u16 * 16;
                                rect.right = rect.left + 16;

                                for x in (0..(state.canvas_size.0 as isize + 16)).step_by(16) {
                                    if direction == FadeDirection::Down {
                                        batch.add_rect(x as f32, y as f32, &rect);
                                    } else {
                                        batch.add_rect(x as f32, state.canvas_size.1 - y as f32, &rect);
                                    }
                                }
                            }
                        }
                    }
                    FadeDirection::Center => {
                        let center_x = (state.canvas_size.0 / 2.0 - 8.0) as isize;
                        let center_y = (state.canvas_size.1 / 2.0 - 8.0) as isize;
                        let mut start_frame = tick;

                        for x in (0..(center_x + 16)).step_by(16) {
                            let mut frame = start_frame;

                            for y in (0..(center_y + 16)).step_by(16) {
                                if frame > 15 { frame = 15; } else { frame += 1; }

                                if frame >= 0 {
                                    rect.left = frame as u16 * 16;
                                    rect.right = rect.left + 16;

                                    batch.add_rect((center_x - x) as f32, (center_y + y) as f32, &rect);
                                    batch.add_rect((center_x - x) as f32, (center_y - y) as f32, &rect);
                                    batch.add_rect((center_x + x) as f32, (center_y + y) as f32, &rect);
                                    batch.add_rect((center_x + x) as f32, (center_y - y) as f32, &rect);
                                }
                            }

                            start_frame += 1;
                        }
                    }
                }

                batch.draw(ctx)?;
            }
        }

        Ok(())
    }

    fn draw_black_bars(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw_text_boxes(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if !state.textscript_vm.flags.render() { return Ok(()); }

        let top_pos = if state.textscript_vm.flags.position_top() { 32.0 } else { state.canvas_size.1 as f32 - 66.0 };
        let left_pos = (state.canvas_size.0 / 2.0 - 122.0).floor();

        {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;
            if state.textscript_vm.flags.background_visible() {
                batch.add_rect(left_pos, top_pos, &state.constants.textscript.textbox_rect_top);
                for i in 1..7 {
                    batch.add_rect(left_pos, top_pos + i as f32 * 8.0, &state.constants.textscript.textbox_rect_middle);
                }
                batch.add_rect(left_pos, top_pos + 56.0, &state.constants.textscript.textbox_rect_bottom);
            }

            if state.textscript_vm.item != 0 {
                batch.add_rect((state.canvas_size.0 / 2.0 - 40.0).floor(), state.canvas_size.1 - 112.0,
                               &state.constants.textscript.get_item_top_left);
                batch.add_rect((state.canvas_size.0 / 2.0 - 40.0).floor(), state.canvas_size.1 - 96.0,
                               &state.constants.textscript.get_item_bottom_left);
                batch.add_rect((state.canvas_size.0 / 2.0 + 32.0).floor(), state.canvas_size.1 - 112.0,
                               &state.constants.textscript.get_item_top_right);
                batch.add_rect((state.canvas_size.0 / 2.0 + 32.0).floor(), state.canvas_size.1 - 104.0,
                               &state.constants.textscript.get_item_right);
                batch.add_rect((state.canvas_size.0 / 2.0 + 32.0).floor(), state.canvas_size.1 - 96.0,
                               &state.constants.textscript.get_item_right);
                batch.add_rect((state.canvas_size.0 / 2.0 + 32.0).floor(), state.canvas_size.1 - 88.0,
                               &state.constants.textscript.get_item_bottom_right);
            }

            if let TextScriptExecutionState::WaitConfirmation(_, _, _, wait, selection) = state.textscript_vm.state {
                let pos_y = if wait > 14 {
                    state.canvas_size.1 - 96.0 - (wait as f32 - 2.0) * 4.0
                } else {
                    state.canvas_size.1 - 96.0
                };

                batch.add_rect((state.canvas_size.0 / 2.0 + 56.0).floor(), pos_y,
                               &state.constants.textscript.textbox_rect_yes_no);

                if wait == 0 {
                    let pos_x = if selection == ConfirmSelection::No { 41.0 } else { 0.0 };

                    batch.add_rect((state.canvas_size.0 / 2.0 + 51.0).floor() + pos_x,
                                   state.canvas_size.1 - 86.0,
                                   &state.constants.textscript.textbox_rect_cursor);
                }
            }

            batch.draw(ctx)?;
        }

        if state.textscript_vm.face != 0 {
            let tex_name = if state.constants.textscript.animated_face_pics {
                SWITCH_FACE_TEX[0]
            } else {
                FACE_TEX
            };
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, tex_name)?;

            // switch version uses +1000 face offset to display a flipped version
            let flip = state.textscript_vm.face > 1000;
            let face_num = state.textscript_vm.face % 100;
            let (scale_x, scale_y) = batch.scale();

            batch.add_rect_scaled(left_pos + 14.0 + if flip { 48.0 } else { 0.0 }, top_pos + 8.0,
                                  scale_x * if flip { -1.0 } else { 1.0 }, scale_y,
                                  &Rect::new_size(
                                      (face_num as u16 % 6) * 48,
                                      (face_num as u16 / 6) * 48,
                                      48, 48,
                                  ));

            batch.draw(ctx)?;
        }

        if state.textscript_vm.item != 0 {
            let mut rect = Rect::new(0, 0, 0, 0);

            if state.textscript_vm.item < 1000 {
                let item_id = state.textscript_vm.item as u16;

                rect.left = (item_id % 16) * 16;
                rect.right = rect.left + 16;
                rect.top = (item_id / 16) * 16;
                rect.bottom = rect.top + 16;

                let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "ArmsImage")?;
                batch.add_rect((state.canvas_size.0 / 2.0 - 12.0).floor(), state.canvas_size.1 - 104.0, &rect);
                batch.draw(ctx)?;
            } else {
                let item_id = state.textscript_vm.item as u16 - 1000;

                rect.left = (item_id % 8) * 32;
                rect.right = rect.left + 32;
                rect.top = (item_id / 8) * 16;
                rect.bottom = rect.top + 16;

                let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "ItemImage")?;
                batch.add_rect((state.canvas_size.0 / 2.0 - 20.0).floor(), state.canvas_size.1 - 104.0, &rect);
                batch.draw(ctx)?;
            }
        }

        let text_offset = if state.textscript_vm.face == 0 { 0.0 } else { 56.0 };

        if !state.textscript_vm.line_1.is_empty() {
            state.font.draw_text(state.textscript_vm.line_1.iter().copied(), left_pos + text_offset + 14.0, top_pos + 10.0, &state.constants, &mut state.texture_set, ctx)?;
        }

        if !state.textscript_vm.line_2.is_empty() {
            state.font.draw_text(state.textscript_vm.line_2.iter().copied(), left_pos + text_offset + 14.0, top_pos + 10.0 + 16.0, &state.constants, &mut state.texture_set, ctx)?;
        }

        if !state.textscript_vm.line_3.is_empty() {
            state.font.draw_text(state.textscript_vm.line_3.iter().copied(), left_pos + text_offset + 14.0, top_pos + 10.0 + 32.0, &state.constants, &mut state.texture_set, ctx)?;
        }

        Ok(())
    }

    fn draw_light(&self, x: f32, y: f32, size: f32, color: (u8, u8, u8), batch: &mut SizedBatch) {
        batch.add_rect_scaled_tinted(x - size * 32.0, y - size * 32.0, color,
                                     size,
                                     size,
                                     &Rect::new(0, 0, 64, 64))
    }

    fn draw_light_map(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        graphics::set_canvas(ctx, Some(&state.lightmap_canvas));
        graphics::set_blend_mode(ctx, BlendMode::Add)?;

        graphics::clear(ctx, Color::from_rgb(100, 100, 110));
        {
            let scale = state.scale;
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "builtin/lightmap/spot")?;

            if !self.player1.cond.hidden() && self.inventory_player1.get_current_weapon().is_some() {
                self.draw_light(fix9_scale(self.player1.x - self.frame.x, scale),
                                fix9_scale(self.player1.y - self.frame.y, scale),
                                4.0, (140, 140, 140), batch);
            }

            for bullet in self.bullet_manager.bullets.iter() {
                self.draw_light(fix9_scale(bullet.x - self.frame.x, scale),
                                fix9_scale(bullet.y - self.frame.y, scale),
                                0.7, (200, 200, 200), batch);
            }

            for caret in state.carets.iter() {
                match caret.ctype {
                    CaretType::ProjectileDissipation | CaretType::Shoot => {
                        self.draw_light(fix9_scale(caret.x - self.frame.x, scale),
                                        fix9_scale(caret.y - self.frame.y, scale),
                                        1.0, (200, 200, 200), batch);
                    }
                    _ => {}
                }
            }

            for npc_cell in self.npc_map.npcs.values() {
                let npc = npc_cell.borrow();

                if !npc.cond.alive() || npc.cond.hidden() || (npc.x < (self.frame.x - 128 * 0x200 - npc.display_bounds.width() as isize * 0x200)
                    || npc.x > (self.frame.x + 128 * 0x200 + (state.canvas_size.0 as isize + npc.display_bounds.width() as isize) * 0x200)
                    && npc.y < (self.frame.y - 128 * 0x200 - npc.display_bounds.height() as isize * 0x200)
                    || npc.y > (self.frame.y + 128 * 0x200 + (state.canvas_size.1 as isize + npc.display_bounds.height() as isize) * 0x200)) {
                    continue;
                }

                match npc.npc_type {
                    1 => {
                        self.draw_light(fix9_scale(npc.x - self.frame.x, scale),
                                        fix9_scale(npc.y - self.frame.y, scale),
                                        0.4, (255, 255, 0), batch);
                    }
                    4 | 7 => self.draw_light(fix9_scale(npc.x - self.frame.x, scale),
                                             fix9_scale(npc.y - self.frame.y, scale),
                                             1.0, (100, 100, 100), batch),
                    17 if npc.anim_num == 0 => {
                        self.draw_light(fix9_scale(npc.x - self.frame.x, scale),
                                        fix9_scale(npc.y - self.frame.y, scale),
                                        2.0, (160, 0, 0), batch);
                        self.draw_light(fix9_scale(npc.x - self.frame.x, scale),
                                        fix9_scale(npc.y - self.frame.y, scale),
                                        0.5, (255, 0, 0), batch);
                    }
                    20 if npc.direction == Direction::Right => {
                        self.draw_light(fix9_scale(npc.x - self.frame.x, scale),
                                        fix9_scale(npc.y - self.frame.y, scale),
                                        2.0, (0, 0, 150), batch);

                        if npc.anim_num < 2 {
                            self.draw_light(fix9_scale(npc.x - self.frame.x, scale),
                                            fix9_scale(npc.y - self.frame.y, scale),
                                            2.1, (0, 0, 30), batch);
                        }
                    }
                    22 if npc.action_num == 1 && npc.anim_num == 1 =>
                        self.draw_light(fix9_scale(npc.x - self.frame.x, scale),
                                        fix9_scale(npc.y - self.frame.y, scale),
                                        3.0, (0, 0, 255), batch),
                    32 | 87 | 211 => {
                        self.draw_light(fix9_scale(npc.x - self.frame.x, scale),
                                        fix9_scale(npc.y - self.frame.y, scale),
                                        2.0, (255, 30, 30), batch);
                    }
                    38 => {
                        let flicker = (npc.anim_num ^ 5 & 3) as u8 * 15;
                        self.draw_light(fix9_scale(npc.x - self.frame.x, scale),
                                        fix9_scale(npc.y - self.frame.y, scale),
                                        3.5, (130 + flicker, 40 + flicker, 0), batch);
                    }
                    66 if npc.action_num == 1 && npc.anim_counter % 2 == 0 =>
                        self.draw_light(fix9_scale(npc.x - self.frame.x, scale),
                                        fix9_scale(npc.y - self.frame.y, scale),
                                        3.0, (0, 100, 255), batch),
                    67 => self.draw_light(fix9_scale(npc.x - self.frame.x, scale),
                                          fix9_scale(npc.y - self.frame.y, scale),
                                          2.0, (0, 100, 200), batch),
                    70 => {
                        let flicker = 50 + npc.anim_num as u8 * 15;
                        self.draw_light(fix9_scale(npc.x - self.frame.x, scale),
                                        fix9_scale(npc.y - self.frame.y, scale),
                                        2.0, (flicker, flicker, flicker), batch);
                    }
                    75 | 77 => self.draw_light(fix9_scale(npc.x - self.frame.x, scale),
                                               fix9_scale(npc.y - self.frame.y, scale),
                                               3.0, (255, 100, 0), batch),
                    85 if npc.action_num == 1 => {
                        let (color, color2) = if npc.direction == Direction::Left {
                            ((0, 150, 100), (0, 50, 30))
                        } else {
                            ((150, 0, 0), (50, 0, 0))
                        };

                        self.draw_light(fix9_scale(npc.x - self.frame.x, scale),
                                        fix9_scale(npc.y - self.frame.y, scale),
                                        1.5, color, batch);

                        if npc.anim_num < 2 {
                            self.draw_light(fix9_scale(npc.x - self.frame.x, scale),
                                            fix9_scale(npc.y - self.frame.y, scale) - 8.0,
                                            2.1, color2, batch);
                        }
                    }
                    299 => {
                        self.draw_light(fix9_scale(npc.x - self.frame.x, scale),
                                        fix9_scale(npc.y - self.frame.y, scale),
                                        4.0, (30, 30, 200), batch);
                    }
                    300 => {
                        self.draw_light(fix9_scale(npc.x - self.frame.x, scale),
                                        fix9_scale(npc.y - self.frame.y, scale),
                                        1.5, (200, 10, 10), batch);
                    }
                    _ => {}
                }
            }

            batch.draw_filtered(FilterMode::Linear, ctx)?;
        }

        graphics::set_blend_mode(ctx, BlendMode::Multiply)?;
        graphics::set_canvas(ctx, Some(&state.game_canvas));
        state.lightmap_canvas.set_filter(FilterMode::Linear);
        state.lightmap_canvas.draw(ctx, DrawParam::new()
            .scale(Vector2::new(1.0 / state.scale, 1.0 / state.scale)))?;

        graphics::set_blend_mode(ctx, BlendMode::Alpha)?;

        Ok(())
    }

    fn is_water(&self, tile: u8) -> bool {
        [0x02, 0x04, 0x60, 0x61, 0x62, 0x64, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0xa0, 0xa1, 0xa2, 0xa3].contains(&tile)
    }

    fn draw_water(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let (frame_x, frame_y) = self.frame.xy_interpolated(state.frame_time, state.scale);

        {
            state.shaders.water_shader_params.resolution = [state.canvas_size.0, state.canvas_size.1];
            state.shaders.water_shader_params.frame_pos = [frame_x, frame_y];
            state.shaders.water_shader_params.t = self.tick as f32;
            let _lock = graphics::use_shader(ctx, &state.shaders.water_shader);
            state.shaders.water_shader.send(ctx, state.shaders.water_shader_params)?;

            graphics::set_canvas(ctx, Some(&state.tmp_canvas));
            graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));
            state.game_canvas.draw(ctx, DrawParam::new()
                .scale(mint::Vector2 { x: 1.0 / state.scale, y: -1.0 / state.scale })
                .offset(mint::Point2 { x: 0.0, y: -1.0 }))?;
        }
        graphics::set_canvas(ctx, Some(&state.game_canvas));

        // cheap, clones a reference underneath
        let mut tmp_batch = SpriteBatch::new(state.tmp_canvas.image().clone());

        let tile_start_x = clamp(self.frame.x / 0x200 / 16, 0, self.stage.map.width as isize) as usize;
        let tile_start_y = clamp(self.frame.y / 0x200 / 16, 0, self.stage.map.height as isize) as usize;
        let tile_end_x = clamp((self.frame.x / 0x200 + 8 + state.canvas_size.0 as isize) / 16 + 1, 0, self.stage.map.width as isize) as usize;
        let tile_end_y = clamp((self.frame.y / 0x200 + 8 + state.canvas_size.1 as isize) / 16 + 1, 0, self.stage.map.height as isize) as usize;
        let mut rect = Rect {
            left: 0.0,
            top: 0.0,
            right: 16.0,
            bottom: 16.0,
        };

        for y in tile_start_y..tile_end_y {
            for x in tile_start_x..tile_end_x {
                let tile = self.stage.map.attrib[*self.stage.map.tiles
                    .get((y * self.stage.map.width) + x)
                    .unwrap() as usize];
                let tile_above = self.stage.map.attrib[*self.stage.map.tiles
                    .get((y.saturating_sub(1) * self.stage.map.width) + x)
                    .unwrap() as usize];

                if !self.is_water(tile) {
                    continue;
                }

                rect.left = (x as f32 * 16.0 - 8.0) - frame_x;
                rect.top = (y as f32 * 16.0 - 8.0) - frame_y;
                rect.right = rect.left + 16.0;
                rect.bottom = rect.top + 16.0;

                if tile_above == 0 {
                    rect.top += 3.0;
                }

                tmp_batch.add(DrawParam::new()
                    .src(ggez::graphics::Rect::new(rect.left / state.canvas_size.0,
                                                   rect.top / state.canvas_size.1,
                                                   (rect.right - rect.left) / state.canvas_size.0,
                                                   (rect.bottom - rect.top) / state.canvas_size.1))
                    .scale(mint::Vector2 {
                        x: 1.0 / state.scale,
                        y: 1.0 / state.scale,
                    })
                    .dest(mint::Point2 {
                        x: rect.left,
                        y: rect.top,
                    }));
            }
        }

        tmp_batch.draw(ctx, DrawParam::new())?;

        Ok(())
    }

    fn draw_tiles(&self, state: &mut SharedGameState, ctx: &mut Context, layer: TileLayer) -> GameResult {
        let tex = match layer {
            TileLayer::Snack => "Npc/NpcSym",
            _ => &self.tex_tileset_name,
        };
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, tex)?;
        let mut rect = Rect::new(0, 0, 16, 16);
        let (frame_x, frame_y) = self.frame.xy_interpolated(state.frame_time, state.scale);

        let tile_start_x = clamp(self.frame.x / 0x200 / 16, 0, self.stage.map.width as isize) as usize;
        let tile_start_y = clamp(self.frame.y / 0x200 / 16, 0, self.stage.map.height as isize) as usize;
        let tile_end_x = clamp((self.frame.x / 0x200 + 8 + state.canvas_size.0 as isize) / 16 + 1, 0, self.stage.map.width as isize) as usize;
        let tile_end_y = clamp((self.frame.y / 0x200 + 8 + state.canvas_size.1 as isize) / 16 + 1, 0, self.stage.map.height as isize) as usize;

        if layer == TileLayer::Snack {
            rect = state.constants.world.snack_rect;
        }

        for y in tile_start_y..tile_end_y {
            for x in tile_start_x..tile_end_x {
                let tile = *self.stage.map.tiles
                    .get((y * self.stage.map.width) + x)
                    .unwrap();

                match layer {
                    TileLayer::Background => {
                        if self.stage.map.attrib[tile as usize] >= 0x20 {
                            continue;
                        }

                        rect.left = (tile as u16 % 16) * 16;
                        rect.top = (tile as u16 / 16) * 16;
                        rect.right = rect.left + 16;
                        rect.bottom = rect.top + 16;
                    }
                    TileLayer::Foreground => {
                        let attr = self.stage.map.attrib[tile as usize];

                        if attr < 0x40 || attr >= 0x80 || attr == 0x43 {
                            continue;
                        }

                        rect.left = (tile as u16 % 16) * 16;
                        rect.top = (tile as u16 / 16) * 16;
                        rect.right = rect.left + 16;
                        rect.bottom = rect.top + 16;
                    }
                    TileLayer::Snack => {
                        if self.stage.map.attrib[tile as usize] != 0x43 {
                            continue;
                        }
                    }
                    _ => {}
                }

                batch.add_rect((x as f32 * 16.0 - 8.0) - frame_x,
                               (y as f32 * 16.0 - 8.0) - frame_y, &rect);
            }
        }

        batch.draw(ctx)?;

        Ok(())
    }

    fn tick_npc_bullet_collissions(&mut self, state: &mut SharedGameState) {
        let mut dead_npcs = Vec::new();

        for npc_cell in self.npc_map.npcs.values() {
            let mut npc = npc_cell.borrow_mut();

            if npc.cond.drs_destroyed() {
                dead_npcs.push(npc.id);
            }

            if !npc.cond.alive() {
                continue;
            }

            if npc.npc_flags.shootable() && npc.npc_flags.interactable() {
                continue;
            }

            for bullet in self.bullet_manager.bullets.iter_mut() {
                if !bullet.cond.alive() || bullet.damage < 1 {
                    continue;
                }

                let hit = (
                    npc.npc_flags.shootable()
                        && (npc.x - npc.hit_bounds.right as isize) < (bullet.x + bullet.enemy_hit_width as isize)
                        && (npc.x + npc.hit_bounds.right as isize) > (bullet.x - bullet.enemy_hit_width as isize)
                        && (npc.y - npc.hit_bounds.top as isize) < (bullet.y + bullet.enemy_hit_height as isize)
                        && (npc.y + npc.hit_bounds.bottom as isize) > (bullet.y - bullet.enemy_hit_height as isize)
                ) || (
                    npc.npc_flags.invulnerable()
                        && (npc.x - npc.hit_bounds.right as isize) < (bullet.x + bullet.hit_bounds.right as isize)
                        && (npc.x + npc.hit_bounds.right as isize) > (bullet.x - bullet.hit_bounds.left as isize)
                        && (npc.y - npc.hit_bounds.top as isize) < (bullet.y + bullet.hit_bounds.bottom as isize)
                        && (npc.y + npc.hit_bounds.bottom as isize) > (bullet.y - bullet.hit_bounds.top as isize)
                );

                if !hit {
                    continue;
                }

                if npc.npc_flags.shootable() {
                    npc.life = npc.life.saturating_sub(bullet.damage);

                    if npc.life == 0 {
                        if npc.npc_flags.show_damage() {
                            // todo show damage
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
                                state.create_caret((bullet.x + npc.x) / 2, (bullet.y + npc.y) / 2, CaretType::HurtParticles, Direction::Left);
                            }
                        }

                        if npc.npc_flags.show_damage() {
                            // todo show damage
                        }
                    }
                } else if !bullet.weapon_flags.flag_x10()
                    && bullet.btype != 13 && bullet.btype != 14 && bullet.btype != 15
                    && bullet.btype != 28 && bullet.btype != 29 && bullet.btype != 30 {
                    state.create_caret((bullet.x + npc.x) / 2, (bullet.y + npc.y) / 2, CaretType::ProjectileDissipation, Direction::Right);
                    state.sound_manager.play_sfx(31);
                    bullet.life = 0;
                    continue;
                }

                if bullet.life > 0 {
                    bullet.life -= 1;
                }
            }

            if npc.cond.explode_die() && !npc.cond.drs_destroyed() {
                dead_npcs.push(npc.id);
            }

            npc.cond.set_drs_destroyed(false);
        }

        for i in 0..self.npc_map.boss_map.parts.len() {
            let mut idx = i;
            let (mut destroy_x, mut destroy_y, mut destroy_radius, mut destroy_count) = (0, 0, 0, 0);
            let mut npc = unsafe { self.npc_map.boss_map.parts.get_unchecked_mut(i) };
            if !npc.cond.alive() {
                continue;
            }

            for bullet in self.bullet_manager.bullets.iter_mut() {
                if !bullet.cond.alive() || bullet.damage < 1 {
                    continue;
                }

                let hit = (
                    npc.npc_flags.shootable()
                        && (npc.x - npc.hit_bounds.right as isize) < (bullet.x + bullet.enemy_hit_width as isize)
                        && (npc.x + npc.hit_bounds.right as isize) > (bullet.x - bullet.enemy_hit_width as isize)
                        && (npc.y - npc.hit_bounds.top as isize) < (bullet.y + bullet.enemy_hit_height as isize)
                        && (npc.y + npc.hit_bounds.bottom as isize) > (bullet.y - bullet.enemy_hit_height as isize)
                ) || (
                    npc.npc_flags.invulnerable()
                        && (npc.x - npc.hit_bounds.right as isize) < (bullet.x + bullet.hit_bounds.right as isize)
                        && (npc.x + npc.hit_bounds.right as isize) > (bullet.x - bullet.hit_bounds.left as isize)
                        && (npc.y - npc.hit_bounds.top as isize) < (bullet.y + bullet.hit_bounds.bottom as isize)
                        && (npc.y + npc.hit_bounds.bottom as isize) > (bullet.y - bullet.hit_bounds.top as isize)
                );

                if !hit {
                    continue;
                }

                if npc.npc_flags.shootable() {
                    if npc.cond.damage_boss() {
                        idx = 0;
                        npc = unsafe { self.npc_map.boss_map.parts.get_unchecked_mut(0) };
                    }

                    npc.life = npc.life.saturating_sub(bullet.damage);

                    if npc.life == 0 {
                        npc.life = npc.id;

                        if self.player1.cond.alive() && npc.npc_flags.event_when_killed() {
                            state.control_flags.set_tick_world(true);
                            state.control_flags.set_interactions_disabled(true);
                            state.textscript_vm.start_script(npc.event_num);
                        } else {
                            state.sound_manager.play_sfx(self.npc_map.boss_map.death_sound[idx]);

                            destroy_x = npc.x;
                            destroy_y = npc.y;
                            destroy_radius = npc.display_bounds.right;
                            destroy_count = 4usize * (2usize).pow((npc.size as u32).saturating_sub(1));

                            npc.cond.set_alive(false);
                        }
                    } else {
                        if npc.shock < 14 {
                            for _ in 0..3 {
                                state.create_caret(bullet.x, bullet.y, CaretType::HurtParticles, Direction::Left);
                            }
                            state.sound_manager.play_sfx(self.npc_map.boss_map.hurt_sound[idx]);
                        }

                        npc.shock = 8;

                        npc = unsafe { self.npc_map.boss_map.parts.get_unchecked_mut(0) };
                        npc.shock = 8;
                    }

                    bullet.life = bullet.life.saturating_sub(1);
                    if bullet.life < 1 {
                        bullet.cond.set_alive(false);
                    }
                } else if [13, 14, 15, 28, 29, 30].contains(&bullet.btype) {
                    bullet.life = bullet.life.saturating_sub(1);
                } else if !bullet.weapon_flags.flag_x10() {
                    state.create_caret(bullet.x, bullet.y, CaretType::ProjectileDissipation, Direction::Right);
                    state.sound_manager.play_sfx(31);
                    bullet.life = 0;
                    continue;
                }
            }

            if destroy_count != 0 {
                self.npc_map.create_death_effect(destroy_x, destroy_y, destroy_radius, destroy_count, state);
            }
        }

        if !dead_npcs.is_empty() {
            let missile = self.inventory_player1.has_weapon(WeaponType::MissileLauncher)
                || self.inventory_player1.has_weapon(WeaponType::SuperMissileLauncher);
            self.npc_map.process_dead_npcs(&dead_npcs, missile, &self.player1, state);
            self.npc_map.garbage_collect();
        }
    }

    fn tick_world(&mut self, state: &mut SharedGameState) -> GameResult {
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
        self.player1.tick(state, ())?;
        self.player2.tick(state, ())?;

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

        {
            for npc_cell in self.npc_map.npcs.values() {
                let mut npc = npc_cell.borrow_mut();

                if npc.cond.alive() {
                    npc.tick(state, ([&mut self.player1, &mut self.player2], &self.npc_map.npcs, &mut self.stage, &self.bullet_manager))?;
                }
            }
        }
        self.npc_map.boss_map.tick(state, ([&mut self.player1, &mut self.player2], &self.npc_map.npcs, &mut self.stage, &self.bullet_manager))?;
        self.npc_map.process_npc_changes(&self.player1, state);
        self.npc_map.garbage_collect();

        self.player1.tick_map_collisions(state, &mut self.stage);
        self.player1.tick_npc_collisions(TargetPlayer::Player1, state, &mut self.npc_map, &mut self.inventory_player1);

        self.player2.tick_map_collisions(state, &mut self.stage);
        self.player2.tick_npc_collisions(TargetPlayer::Player2, state, &mut self.npc_map, &mut self.inventory_player2);

        for npc_cell in self.npc_map.npcs.values() {
            let mut npc = npc_cell.borrow_mut();

            if npc.cond.alive() && !npc.npc_flags.ignore_solidity() {
                npc.tick_map_collisions(state, &mut self.stage);
            }
        }
        for npc in self.npc_map.boss_map.parts.iter_mut() {
            if npc.cond.alive() && !npc.npc_flags.ignore_solidity() {
                npc.tick_map_collisions(state, &mut self.stage);
            }
        }
        self.npc_map.process_npc_changes(&self.player1, state);
        self.npc_map.garbage_collect();

        self.tick_npc_bullet_collissions(state);
        self.npc_map.process_npc_changes(&self.player1, state);

        self.bullet_manager.tick_bullets(state, [&self.player1, &self.player2], &mut self.stage);
        state.tick_carets();

        match self.frame.update_target {
            UpdateTarget::Player => {
                if self.player2.cond.alive() && !self.player2.cond.hidden()
                    && abs(self.player1.x - self.player2.x) < 240 * 0x200
                    && abs(self.player1.y - self.player2.y) < 200 * 0x200 {
                    self.frame.target_x = (self.player1.target_x * 2 + self.player2.target_x) / 3;
                    self.frame.target_y = (self.player1.target_y * 2 + self.player2.target_y) / 3;

                    self.frame.target_x = clamp(self.frame.target_x, self.player1.x - 0x8000, self.player1.x + 0x8000);
                    self.frame.target_y = clamp(self.frame.target_y, self.player1.y, self.player1.y);
                } else {
                    self.frame.target_x = self.player1.target_x;
                    self.frame.target_y = self.player1.target_y;
                }
            }
            UpdateTarget::NPC(npc_id) => {
                if let Some(npc_cell) = self.npc_map.npcs.get(&npc_id) {
                    let npc = npc_cell.borrow();

                    if npc.cond.alive() {
                        self.frame.target_x = npc.x;
                        self.frame.target_y = npc.y;
                    }
                }
            }
            UpdateTarget::Boss(boss_id) => {
                if let Some(boss) = self.npc_map.boss_map.parts.get(boss_id as usize) {
                    if boss.cond.alive() {
                        self.frame.target_x = boss.x;
                        self.frame.target_y = boss.y;
                    }
                }
            }
        }
        self.frame.update(state, &self.stage);

        if state.control_flags.control_enabled() {
            if let Some(weapon) = self.inventory_player1.get_current_weapon_mut() {
                weapon.shoot_bullet(&self.player1, TargetPlayer::Player1, &mut self.bullet_manager, state);
            }

            if let Some(weapon) = self.inventory_player2.get_current_weapon_mut() {
                weapon.shoot_bullet(&self.player2, TargetPlayer::Player2, &mut self.bullet_manager, state);
            }

            if self.player1.cond.alive() {
                if self.player1.controller.trigger_next_weapon() {
                    state.sound_manager.play_sfx(4);
                    self.inventory_player1.next_weapon();
                    self.hud_player1.weapon_x_pos = 32;
                }

                if self.player1.controller.trigger_prev_weapon() {
                    state.sound_manager.play_sfx(4);
                    self.inventory_player1.prev_weapon();
                    self.hud_player1.weapon_x_pos = 0;
                }
            }

            if self.player2.cond.alive() {
                if self.player2.controller.trigger_next_weapon() {
                    state.sound_manager.play_sfx(4);
                    self.inventory_player2.next_weapon();
                    self.hud_player2.weapon_x_pos = 32;
                }

                if self.player2.controller.trigger_prev_weapon() {
                    state.sound_manager.play_sfx(4);
                    self.inventory_player2.prev_weapon();
                    self.hud_player2.weapon_x_pos = 0;
                }
            }

            self.hud_player1.tick(state, (&self.player1, &self.inventory_player1))?;
            self.hud_player2.tick(state, (&self.player2, &self.inventory_player2))?;
            self.boss_life_bar.tick(state, &self.npc_map)?;
        }

        Ok(())
    }

    fn draw_debug_npc(&self, npc: &NPC, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if npc.x < (self.frame.x - 128 - npc.display_bounds.width() as isize * 0x200)
            || npc.x > (self.frame.x + 128 + (state.canvas_size.0 as isize + npc.display_bounds.width() as isize) * 0x200)
            && npc.y < (self.frame.y - 128 - npc.display_bounds.height() as isize * 0x200)
            || npc.y > (self.frame.y + 128 + (state.canvas_size.1 as isize + npc.display_bounds.height() as isize) * 0x200) {
            return Ok(());
        }

        {
            let hit_rect_size = clamp(npc.hit_rect_size(), 1, 4);
            let hit_rect_size = hit_rect_size * hit_rect_size;

            let x = (npc.x + npc.offset_x()) / (16 * 0x200);
            let y = (npc.y + npc.offset_y()) / (16 * 0x200);
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Caret")?;

            let caret_rect = Rect::new_size(2, 74, 4, 4);
            let caret2_rect = Rect::new_size(65, 9, 6, 6);

            for (idx, (&ox, &oy)) in crate::physics::OFF_X.iter()
                .zip(crate::physics::OFF_Y.iter()).enumerate() {
                if idx == hit_rect_size {
                    break;
                }

                batch.add_rect(((x + ox) * 16 - self.frame.x / 0x200) as f32 - 2.0,
                               ((y + oy) * 16 - self.frame.y / 0x200) as f32 - 2.0,
                               &caret_rect);
            }


            batch.add_rect(((npc.x - self.frame.x) / 0x200) as f32 - 3.0,
                           ((npc.y - self.frame.y) / 0x200) as f32 - 3.0,
                           &caret2_rect);

            batch.draw(ctx)?;
        }

        let text = format!("{}:{}:{}", npc.id, npc.npc_type, npc.action_num);
        state.font.draw_colored_text(text.chars(), ((npc.x - self.frame.x) / 0x200) as f32, ((npc.y - self.frame.y) / 0x200) as f32,
                                     ((npc.id & 0xf0) as u8, (npc.cond.0 >> 8) as u8, (npc.id & 0x0f << 4) as u8),
                                     &state.constants, &mut state.texture_set, ctx)?;

        Ok(())
    }

    fn draw_debug_outlines(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        for npc in self.npc_map.npcs.values().map(|n| n.borrow()).filter(|n| n.cond.alive()) {
            self.draw_debug_npc(npc.deref(), state, ctx)?;
        }

        for boss in self.npc_map.boss_map.parts.iter().filter(|n| n.cond.alive()) {
            self.draw_debug_npc(boss, state, ctx)?;
        }

        Ok(())
    }
}

impl Scene for GameScene {
    fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let seed = (self.player1.max_life as i32)
            .wrapping_add(self.player1.x as i32)
            .wrapping_add(self.player1.y as i32)
            .wrapping_add(self.stage_id as i32)
            .wrapping_mul(7);
        state.game_rng = RNG::new(seed);
        state.textscript_vm.set_scene_script(self.stage.load_text_script(&state.base_path, &state.constants, ctx)?);
        state.textscript_vm.suspend = false;

        self.player1.controller = state.settings.create_player1_controller();
        self.player2.controller = state.settings.create_player2_controller();

        let npcs = self.stage.load_npcs(&state.base_path, ctx)?;
        for npc_data in npcs.iter() {
            log::info!("creating npc: {:?}", npc_data);

            let npc = self.npc_map.create_npc_from_data(&state.npc_table, npc_data);
            if npc.npc_flags.appear_when_flag_set() {
                if let Some(true) = state.game_flags.get(npc_data.flag_num as usize) {
                    npc.cond.set_alive(true);
                }
            } else if npc.npc_flags.hide_unless_flag_set() {
                if let Some(false) = state.game_flags.get(npc_data.flag_num as usize) {
                    npc.cond.set_alive(true);
                }
            } else {
                npc.cond.set_alive(true);
            }
        }

        state.npc_table.tileset_name = self.tex_tileset_name.to_owned();
        state.npc_table.tex_npc1_name = ["Npc/", &self.stage.data.npc1.filename()].join("");
        state.npc_table.tex_npc2_name = ["Npc/", &self.stage.data.npc2.filename()].join("");

        if state.constants.is_cs_plus {
            match state.season {
                Season::Halloween => self.player1.appearance = PlayerAppearance::HalloweenQuote,
                Season::Christmas => self.player1.appearance = PlayerAppearance::ReindeerQuote,
                _ => {}
            }
        }

        self.npc_map.boss_map.boss_type = self.stage.data.boss_no as u16;
        self.player1.target_x = self.player1.x;
        self.player1.target_y = self.player1.y;
        self.frame.target_x = self.player1.target_x;
        self.frame.target_y = self.player1.target_y;
        self.frame.immediate_update(state, &self.stage);

        Ok(())
    }

    fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.player1.controller.update(state, ctx)?;
        self.player1.controller.update_trigger();
        self.player2.controller.update(state, ctx)?;
        self.player2.controller.update_trigger();

        if self.intro_mode && (self.player1.controller.trigger_menu_ok() || self.tick >= 500) {
            state.next_scene = Some(Box::new(TitleScene::new()));
        }

        match state.textscript_vm.mode {
            ScriptMode::Map if state.control_flags.tick_world() => self.tick_world(state)?,
            ScriptMode::StageSelect => self.stage_select.tick(state, (&self.player1, &self.player2))?,
            _ => {}
        }

        if self.map_name_counter > 0 {
            self.map_name_counter -= 1;
        }

        match state.fade_state {
            FadeState::FadeOut(tick, direction) if tick < 15 => {
                state.fade_state = FadeState::FadeOut(tick + 1, direction);
            }
            FadeState::FadeOut(tick, _) if tick == 15 => {
                state.fade_state = FadeState::Hidden;
            }
            FadeState::FadeIn(tick, direction) if tick > -15 => {
                state.fade_state = FadeState::FadeIn(tick - 1, direction);
            }
            FadeState::FadeIn(tick, _) if tick == -15 => {
                state.fade_state = FadeState::Visible;
            }
            _ => {}
        }

        TextScriptVM::run(state, self, ctx)?;
        self.tick = self.tick.wrapping_add(1);
        Ok(())
    }

    fn draw_tick(&mut self, state: &mut SharedGameState) -> GameResult {
        self.frame.prev_x = self.frame.x;
        self.frame.prev_y = self.frame.y;
        self.player1.prev_x = self.player1.x;
        self.player1.prev_y = self.player1.y;
        self.player2.prev_x = self.player2.x;
        self.player2.prev_y = self.player2.y;

        for npc_cell in self.npc_map.npcs.values() {
            let mut npc = npc_cell.borrow_mut();

            if npc.cond.alive() {
                npc.prev_x = npc.x;
                npc.prev_y = npc.y;
            }
        }

        for npc in self.npc_map.boss_map.parts.iter_mut() {
            if npc.cond.alive() {
                npc.prev_x = npc.x;
                npc.prev_y = npc.y;
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

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        graphics::set_canvas(ctx, Some(&state.game_canvas));
        self.draw_background(state, ctx)?;
        self.draw_tiles(state, ctx, TileLayer::Background)?;
        if state.settings.shader_effects
            && self.stage.data.background_type != BackgroundType::Black
            && self.stage.data.background_type != BackgroundType::Outside
            && self.stage.data.background_type != BackgroundType::OutsideWind
            && self.stage.data.background.name() != "bkBlack" {
            self.draw_light_map(state, ctx)?;
        }

        self.npc_map.boss_map.draw(state, ctx, &self.frame)?;
        for npc_cell in self.npc_map.npcs.values() {
            let npc = npc_cell.borrow();

            if npc.x < (self.frame.x - 128 * 0x200 - npc.display_bounds.width() as isize * 0x200)
                || npc.x > (self.frame.x + 128 * 0x200 + (state.canvas_size.0 as isize + npc.display_bounds.width() as isize) * 0x200)
                && npc.y < (self.frame.y - 128 * 0x200 - npc.display_bounds.height() as isize * 0x200)
                || npc.y > (self.frame.y + 128 * 0x200 + (state.canvas_size.1 as isize + npc.display_bounds.height() as isize) * 0x200) {
                continue;
            }

            npc.draw(state, ctx, &self.frame)?;
        }
        self.draw_bullets(state, ctx)?;
        self.player2.draw(state, ctx, &self.frame)?;
        self.player1.draw(state, ctx, &self.frame)?;
        if state.settings.shader_effects && self.water_visible {
            self.draw_water(state, ctx)?;
        }

        self.draw_tiles(state, ctx, TileLayer::Foreground)?;
        self.draw_tiles(state, ctx, TileLayer::Snack)?;
        self.draw_carets(state, ctx)?;
        if state.settings.shader_effects
            && (self.stage.data.background_type == BackgroundType::Black
            || self.stage.data.background.name() == "bkBlack") {
            self.draw_light_map(state, ctx)?;
        }

        graphics::set_canvas(ctx, None);
        state.game_canvas.draw(ctx, DrawParam::new()
            .scale(Vector2::new(1.0 / state.scale, 1.0 / state.scale)))?;
        self.draw_black_bars(state, ctx)?;

        if state.control_flags.control_enabled() {
            self.hud_player1.draw(state, ctx, &self.frame)?;
            self.hud_player2.draw(state, ctx, &self.frame)?;
            self.boss_life_bar.draw(state, ctx, &self.frame)?;

            if self.player2.cond.alive() && !self.player2.cond.hidden() {
                let y = interpolate_fix9_scale(self.player2.prev_y - self.frame.prev_y, self.player2.y - self.frame.y, state.frame_time);
                let y = clamp(y, 8.0, state.canvas_size.1 - 8.0 - state.font.line_height(&state.constants));

                if self.player2.x + 8 * 0x200 < self.frame.x {
                    state.font.draw_colored_text(P2_LEFT_TEXT.chars(),
                                                 9.0, y + 1.0,
                                                 (0, 0, 130), &state.constants, &mut state.texture_set, ctx)?;

                    state.font.draw_colored_text(P2_LEFT_TEXT.chars(),
                                                 8.0, y,
                                                 (96, 96, 255), &state.constants, &mut state.texture_set, ctx)?;
                } else if self.player2.x - 8 * 0x200 > self.frame.x + state.canvas_size.0 as isize * 0x200 {
                    let width = state.font.text_width(P2_RIGHT_TEXT.chars(), &state.constants);

                    state.font.draw_colored_text(P2_RIGHT_TEXT.chars(),
                                                 state.canvas_size.0 - width - 8.0 + 1.0, y + 1.0,
                                                 (0, 0, 130), &state.constants, &mut state.texture_set, ctx)?;

                    state.font.draw_colored_text(P2_RIGHT_TEXT.chars(),
                                                 state.canvas_size.0 - width - 8.0, y,
                                                 (96, 96, 255), &state.constants, &mut state.texture_set, ctx)?;
                }
            }
        }

        if state.textscript_vm.mode == ScriptMode::StageSelect {
            self.stage_select.draw(state, ctx, &self.frame)?;
        }

        self.draw_fade(state, ctx)?;
        if self.map_name_counter > 0 {
            let map_name = if self.intro_mode {
                state.constants.title.intro_text.chars()
            } else {
                self.stage.data.name.chars()
            };
            let width = state.font.text_width(map_name.clone(), &state.constants);

            state.font.draw_text(map_name,
                                 ((state.canvas_size.0 - width) / 2.0).floor(), 80.0,
                                 &state.constants, &mut state.texture_set, ctx)?;
        }

        self.draw_text_boxes(state, ctx)?;

        if state.settings.debug_outlines {
            self.draw_debug_outlines(state, ctx)?;
        }

        draw_number(state.canvas_size.0 - 8.0, 8.0, timer::fps(ctx) as usize, Alignment::Right, state, ctx)?;
        Ok(())
    }

    fn debug_overlay_draw(&mut self, components: &mut Components, state: &mut SharedGameState, ctx: &mut Context, ui: &mut imgui::Ui) -> GameResult {
        components.live_debugger.run_ingame(self, state, ctx, ui)?;
        Ok(())
    }
}
