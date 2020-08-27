use log::info;

use crate::common::Rect;
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::ggez::{Context, GameResult, timer};
use crate::ggez::graphics::{Drawable, DrawParam, Text, TextFragment};
use crate::ggez::nalgebra::clamp;
use crate::player::Player;
use crate::scene::Scene;
use crate::SharedGameState;
use crate::stage::{BackgroundType, Stage};
use crate::str;
use crate::text_script::TextScriptVM;
use crate::ui::Components;

pub struct GameScene {
    pub tick: usize,
    pub stage: Stage,
    pub frame: Frame,
    pub player: Player,
    pub stage_id: usize,
    tex_background_name: String,
    tex_caret_name: String,
    tex_face_name: String,
    tex_hud_name: String,
    tex_npcsym_name: String,
    tex_tileset_name: String,
    life_bar: usize,
    life_bar_count: usize,
}

#[derive(Debug, EnumIter, PartialEq, Eq, Hash, Copy, Clone)]
pub enum TileLayer {
    All,
    Background,
    Foreground,
    Snack,
}

#[derive(Debug, EnumIter, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Alignment {
    Left,
    Right,
}

impl GameScene {
    pub fn new(state: &mut SharedGameState, ctx: &mut Context, id: usize) -> GameResult<Self> {
        let stage = Stage::load(&state.base_path, &state.stages[id], ctx)?;
        info!("Loaded stage: {}", stage.data.name);
        info!("Map size: {}x{}", stage.map.width, stage.map.height);

        let tex_background_name = stage.data.background.filename();
        let tex_caret_name = str!("Caret");
        let tex_face_name = str!("Face");
        let tex_hud_name = str!("TextBox");
        let tex_npcsym_name = str!("Npc/NpcSym");
        let tex_tileset_name = ["Stage/", &stage.data.tileset.filename()].join("");

        Ok(Self {
            tick: 0,
            stage,
            player: Player::new(state),
            frame: Frame {
                x: 0,
                y: 0,
                wait: 16,
            },
            stage_id: id,
            tex_background_name,
            tex_caret_name,
            tex_face_name,
            tex_hud_name,
            tex_npcsym_name,
            tex_tileset_name,
            life_bar: 3,
            life_bar_count: 0,
        })
    }

    fn draw_number(&self, x: f32, y: f32, val: usize, align: Alignment, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, &self.tex_hud_name)?;
        let n = val.to_string();
        let align_offset = if align == Alignment::Right { n.len() as f32 * 8.0 } else { 0.0 };

        for (offset, chr) in n.chars().enumerate() {
            let idx = chr as usize - '0' as usize;
            batch.add_rect(x - align_offset + offset as f32 * 8.0, y, &Rect::<usize>::new_size(idx * 8, 56, 8, 8));
        }

        batch.draw(ctx)?;
        Ok(())
    }

    fn draw_hud(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, &self.tex_hud_name)?;
        // todo: max ammo display

        // none
        batch.add_rect(16.0 + 48.0, 16.0,
                       &Rect::<usize>::new_size(80, 48, 16, 8));
        batch.add_rect(16.0 + 48.0, 24.0,
                       &Rect::<usize>::new_size(80, 48, 16, 8));

        // per
        batch.add_rect(16.0 + 32.0, 24.0,
                       &Rect::<usize>::new_size(72, 48, 8, 8));
        // lv
        batch.add_rect(16.0, 32.0,
                       &Rect::<usize>::new_size(80, 80, 16, 8));
        // xp box
        batch.add_rect(40.0, 32.0,
                       &Rect::<usize>::new_size(0, 72, 40, 8));
        // experience
        //batch.add_rect(40.0, 32.0,
        //               &Rect::<usize>::new_size(0, 80, (40), 8)); // todo
        // life box
        batch.add_rect(16.0, 40.0,
                       &Rect::<usize>::new_size(0, 40, 64, 8));
        // yellow bar
        batch.add_rect(40.0, 40.0,
                       &Rect::<usize>::new_size(0, 32, ((self.life_bar as usize * 40) / self.player.max_life as usize) - 1, 8));
        // life
        batch.add_rect(40.0, 40.0,
                       &Rect::<usize>::new_size(0, 24, ((self.player.life as usize * 40) / self.player.max_life as usize) - 1, 8));

        batch.draw(ctx)?;

        self.draw_number(40.0, 32.0, 0, Alignment::Right, state, ctx)?;
        self.draw_number(40.0, 40.0, self.life_bar as usize, Alignment::Right, state, ctx)?;

        Ok(())
    }

    fn draw_background(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, &self.tex_background_name)?;

        match self.stage.data.background_type {
            BackgroundType::Stationary => {
                let count_x = state.canvas_size.0 as usize / batch.width() + 1;
                let count_y = state.canvas_size.1 as usize / batch.height() + 1;

                for y in 0..count_y {
                    for x in 0..count_x {
                        batch.add((x * batch.width()) as f32, (y * batch.height()) as f32);
                    }
                }
            }
            BackgroundType::MoveDistant => {
                let off_x = self.frame.x as usize / 2 % (batch.width() * 0x200);
                let off_y = self.frame.y as usize / 2 % (batch.height() * 0x200);

                let count_x = state.canvas_size.0 as usize / batch.width() + 2;
                let count_y = state.canvas_size.1 as usize / batch.height() + 2;

                for y in 0..count_y {
                    for x in 0..count_x {
                        batch.add((x * batch.width()) as f32 - (off_x / 0x200) as f32,
                                  (y * batch.height()) as f32 - (off_y / 0x200) as f32);
                    }
                }
            }
            BackgroundType::MoveNear => {}
            BackgroundType::Water => {}
            BackgroundType::Black => {}
            BackgroundType::Autoscroll => {}
            BackgroundType::OutsideWind | BackgroundType::Outside => {
                let offset = (self.tick % 640) as isize;

                batch.add_rect(((state.canvas_size.0 - 320.0) / 2.0).floor(), 0.0,
                               &Rect::<usize>::new_size(0, 0, 320, 88));

                for x in ((-offset / 2)..(state.canvas_size.0 as isize)).step_by(320) {
                    batch.add_rect(x as f32, 88.0,
                                   &Rect::<usize>::new_size(0, 88, 320, 35));
                }

                for x in ((-offset % 320)..(state.canvas_size.0 as isize)).step_by(320) {
                    batch.add_rect(x as f32, 123.0,
                                   &Rect::<usize>::new_size(0, 123, 320, 23));
                }

                for x in ((-offset * 2)..(state.canvas_size.0 as isize)).step_by(320) {
                    batch.add_rect(x as f32, 146.0,
                                   &Rect::<usize>::new_size(0, 146, 320, 30));
                }

                for x in ((-offset * 4)..(state.canvas_size.0 as isize)).step_by(320) {
                    batch.add_rect(x as f32, 176.0,
                                   &Rect::<usize>::new_size(0, 176, 320, 64));
                }
            }
        }

        batch.draw(ctx)?;

        Ok(())
    }

    fn draw_carets(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, &self.tex_caret_name)?;

        for caret in state.carets.iter() {
            batch.add_rect((((caret.x - caret.offset_x) / 0x200) - (self.frame.x / 0x200)) as f32,
                           (((caret.y - caret.offset_y) / 0x200) - (self.frame.y / 0x200)) as f32,
                           &caret.anim_rect);
        }

        batch.draw(ctx)?;
        Ok(())
    }

    fn draw_black_bars(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw_text_boxes(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if !state.textscript_vm.flags.render() { return Ok(()); }

        let top_pos = if state.textscript_vm.flags.position_top() { 32.0 } else { state.canvas_size.1 as f32 - 64.0 };
        let left_pos = (state.canvas_size.0 / 2.0 - 122.0).floor();

        if state.textscript_vm.flags.background_visible() {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, &self.tex_hud_name)?;

            batch.add_rect(left_pos, top_pos, &state.constants.textscript.textbox_rect_top);
            for i in 1..7 {
                batch.add_rect(left_pos, top_pos + i as f32 * 8.0, &state.constants.textscript.textbox_rect_middle);
            }
            batch.add_rect(left_pos, top_pos + 56.0, &state.constants.textscript.textbox_rect_bottom);

            batch.draw(ctx)?;
        }

        if state.textscript_vm.face != 0 {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, &self.tex_face_name)?;
            batch.add_rect(left_pos + 14.0, top_pos + 8.0, &Rect::<usize>::new_size(
                (state.textscript_vm.face as usize % 6) * 48,
                (state.textscript_vm.face as usize / 6) * 48,
                48, 48,
            ));

            batch.draw(ctx)?;
        }

        let text_offset = if state.textscript_vm.face == 0 { 0.0 } else { 56.0 };

        // todo: proper text rendering
        if !state.textscript_vm.line_1.is_empty() {
            let line1: String = state.textscript_vm.line_1.iter().collect();
            Text::new(TextFragment::from(line1.as_str())).draw(ctx, DrawParam::new()
                .dest(nalgebra::Point2::new((left_pos + text_offset) * 2.0 + 32.0, top_pos * 2.0 + 32.0))
                .scale(nalgebra::Vector2::new(0.5, 0.5)))?;
        }

        if !state.textscript_vm.line_2.is_empty() {
            let line2: String = state.textscript_vm.line_2.iter().collect();
            Text::new(TextFragment::from(line2.as_str())).draw(ctx, DrawParam::new()
                .dest(nalgebra::Point2::new((left_pos + text_offset) * 2.0 + 32.0, (top_pos + 12.0) * 2.0 + 32.0))
                .scale(nalgebra::Vector2::new(0.5, 0.5)))?;
        }

        if !state.textscript_vm.line_3.is_empty() {
            let line3: String = state.textscript_vm.line_3.iter().collect();
            Text::new(TextFragment::from(line3.as_str())).draw(ctx, DrawParam::new()
                .dest(nalgebra::Point2::new((left_pos + text_offset) * 2.0 + 32.0, (top_pos + 24.0) * 2.0 + 32.0))
                .scale(nalgebra::Vector2::new(0.5, 0.5)))?;
        }

        Ok(())
    }

    fn draw_tiles(&self, state: &mut SharedGameState, ctx: &mut Context, layer: TileLayer) -> GameResult {
        let tex = match layer {
            TileLayer::Snack => &self.tex_npcsym_name,
            _ => &self.tex_tileset_name,
        };
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, tex)?;
        let mut rect = Rect::<usize>::new(0, 0, 16, 16);

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

                        rect.left = (tile as usize % 16) * 16;
                        rect.top = (tile as usize / 16) * 16;
                        rect.right = rect.left + 16;
                        rect.bottom = rect.top + 16;
                    }
                    TileLayer::Foreground => {
                        let attr = self.stage.map.attrib[tile as usize];

                        if attr < 0x40 || attr >= 0x80 || attr == 0x43 {
                            continue;
                        }

                        rect.left = (tile as usize % 16) * 16;
                        rect.top = (tile as usize / 16) * 16;
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

                batch.add_rect((x as f32 * 16.0 - 8.0) - (self.frame.x / 0x200) as f32,
                               (y as f32 * 16.0 - 8.0) - (self.frame.y / 0x200) as f32, &rect);
            }
        }

        batch.draw(ctx)?;

        Ok(())
    }
}

impl Scene for GameScene {
    fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        state.textscript_vm.set_scene_script(self.stage.load_text_script(&state.base_path, ctx)?);
        state.textscript_vm.suspend = false;

        //self.player.equip.set_booster_2_0(true);
        //state.control_flags.set_flag_x01(true);
        //state.control_flags.set_control_enabled(true);
        Ok(())
    }

    fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        state.update_key_trigger();

        if state.control_flags.flag_x01() {
            self.player.tick(state, ctx)?;

            self.player.flags.0 = 0;
            state.tick_carets();
            self.player.tick_map_collisions(state, &self.stage);
            self.player.tick_npc_collisions(state, &self.stage);

            self.frame.update(state, &self.player, &self.stage);
        }

        if state.control_flags.control_enabled() {
            // update health bar
            if self.life_bar < self.player.life as usize {
                self.life_bar = self.player.life as usize;
            }

            if self.life_bar > self.player.life as usize {
                self.life_bar_count += 1;
                if self.life_bar_count > 30 {
                    self.life_bar -= 1;
                }
            } else {
                self.life_bar_count = 0;
            }
        }

        TextScriptVM::run(state, self, ctx)?;
        self.tick = self.tick.wrapping_add(1);
        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.draw_background(state, ctx)?;
        self.draw_tiles(state, ctx, TileLayer::Background)?;
        self.player.draw(state, ctx, &self.frame)?;
        self.draw_tiles(state, ctx, TileLayer::Foreground)?;
        self.draw_tiles(state, ctx, TileLayer::Snack)?;
        self.draw_carets(state, ctx)?;
        self.draw_black_bars(state, ctx)?;

        self.draw_hud(state, ctx)?;
        self.draw_number(state.canvas_size.0 - 8.0, 8.0, timer::fps(ctx) as usize, Alignment::Right, state, ctx)?;

        self.draw_text_boxes(state, ctx)?;

        Ok(())
    }

    fn debug_overlay_draw(&mut self, components: &mut Components, state: &mut SharedGameState, ctx: &mut Context, ui: &mut imgui::Ui) -> GameResult {
        components.live_debugger.run_ingame(self, state, ctx, ui)?;
        Ok(())
    }
}
