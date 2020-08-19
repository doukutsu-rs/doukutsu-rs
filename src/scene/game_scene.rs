use ggez::{Context, GameResult};
use ggez::GameError::EventLoopError;
use ggez::nalgebra::clamp;

use crate::common::Rect;
use crate::entity::GameEntity;
use crate::game_state::GameState;
use crate::GameContext;
use crate::scene::Scene;
use crate::stage::BackgroundType;
use crate::str;
use crate::live_debugger::LiveDebugger;

pub struct GameScene {
    debugger: LiveDebugger,
    tex_tileset_name: String,
    tex_background_name: String,
    tex_hud_name: String,
    life_bar: usize,
    life_bar_count: usize,
}

#[derive(Debug, EnumIter, PartialEq, Eq, Hash, Copy, Clone)]
pub enum TileLayer {
    All,
    Background,
    Foreground,
}

#[derive(Debug, EnumIter, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Alignment {
    Left,
    Right,
}

impl GameScene {
    pub fn new(state: &GameState, game_ctx: &mut GameContext, ctx: &mut Context) -> GameResult<Self> {
        let tex_tileset_name = str!(["Stage/", &state.stage.data.tileset.filename()].join(""));
        let tex_background_name = state.stage.data.background.filename();
        let tex_hud_name = str!("TextBox");

        game_ctx.texture_set.ensure_texture_loaded(ctx, &game_ctx.constants, &tex_tileset_name)?;
        game_ctx.texture_set.ensure_texture_loaded(ctx, &game_ctx.constants, &tex_background_name)?;
        game_ctx.texture_set.ensure_texture_loaded(ctx, &game_ctx.constants, &tex_hud_name)?;

        Ok(Self {
            debugger: LiveDebugger::new(),
            tex_tileset_name,
            tex_background_name,
            tex_hud_name,
            life_bar: 3,
            life_bar_count: 0,
        })
    }

    fn draw_number(&self, x: f32, y: f32, val: usize, align: Alignment, game_ctx: &mut GameContext, ctx: &mut Context) -> GameResult {
        let set = game_ctx.texture_set.tex_map.get_mut(&self.tex_hud_name);
        if set.is_none() {
            return Ok(());
        }

        let batch = set.unwrap();
        let n = val.to_string();
        let align_offset = if align == Alignment::Right { n.len() as f32 * 8.0 } else { 0.0 };

        for (offset, chr) in n.chars().enumerate() {
            let idx = chr as usize - '0' as usize;
            batch.add_rect(x - align_offset + offset as f32 * 8.0, y, &Rect::<usize>::new_size(idx * 8, 56, 8, 8));
        }

        batch.draw(ctx)?;
        Ok(())
    }

    fn draw_hud(&self, state: &GameState, game_ctx: &mut GameContext, ctx: &mut Context) -> GameResult {
        let set = game_ctx.texture_set.tex_map.get_mut(&self.tex_hud_name);
        if set.is_none() {
            return Ok(());
        }
        let batch = set.unwrap();

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
        // life box
        batch.add_rect(16.0, 40.0,
                       &Rect::<usize>::new_size(0, 40, 64, 8));
        // bar
        batch.add_rect(40.0, 40.0,
                       &Rect::<usize>::new_size(0, 32, ((self.life_bar as usize * 40) / state.player().max_life as usize) - 1, 8));
        // life
        batch.add_rect(40.0, 40.0,
                       &Rect::<usize>::new_size(0, 24, ((state.player().life as usize * 40) / state.player().max_life as usize) - 1, 8));

        batch.draw(ctx)?;

        self.draw_number(40.0, 40.0, self.life_bar as usize, Alignment::Right, game_ctx, ctx)?;

        Ok(())
    }

    fn draw_background(&self, state: &GameState, game_ctx: &mut GameContext, ctx: &mut Context) -> GameResult {
        let set = game_ctx.texture_set.tex_map.get_mut(&self.tex_background_name);
        if set.is_none() {
            return Ok(());
        }

        let batch = set.unwrap();

        match state.stage.data.background_type {
            BackgroundType::Stationary => {
                let count_x = game_ctx.canvas_size.0 as usize / batch.width() + 1;
                let count_y = game_ctx.canvas_size.1 as usize / batch.height() + 1;

                for y in 0..count_y {
                    for x in 0..count_x {
                        batch.add((x * batch.width()) as f32, (y * batch.height()) as f32);
                    }
                }
            }
            BackgroundType::MoveDistant => {
                let off_x = state.frame.x as usize / 2 % (batch.width() * 0x200);
                let off_y = state.frame.y as usize / 2 % (batch.height() * 0x200);

                let count_x = game_ctx.canvas_size.0 as usize / batch.width() + 2;
                let count_y = game_ctx.canvas_size.1 as usize / batch.height() + 2;

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
                let offset = (state.tick % 640) as isize;

                batch.add_rect(((game_ctx.canvas_size.0 - 320.0) / 2.0).floor(), 0.0,
                               &Rect::<usize>::new_size(0, 0, 320, 88));

                for x in ((-offset / 2)..(game_ctx.canvas_size.0 as isize)).step_by(320) {
                    batch.add_rect(x as f32, 88.0,
                                   &Rect::<usize>::new_size(0, 88, 320, 35));
                }

                for x in ((-offset % 320)..(game_ctx.canvas_size.0 as isize)).step_by(320) {
                    batch.add_rect(x as f32, 123.0,
                                   &Rect::<usize>::new_size(0, 123, 320, 23));
                }

                for x in ((-offset * 2)..(game_ctx.canvas_size.0 as isize)).step_by(320) {
                    batch.add_rect(x as f32, 146.0,
                                   &Rect::<usize>::new_size(0, 146, 320, 30));
                }

                for x in ((-offset * 4)..(game_ctx.canvas_size.0 as isize)).step_by(320) {
                    batch.add_rect(x as f32, 176.0,
                                   &Rect::<usize>::new_size(0, 176, 320, 64));
                }
            }
        }

        batch.draw(ctx)?;

        Ok(())
    }

    fn draw_tiles(&self, layer: TileLayer, state: &GameState, game_ctx: &mut GameContext, ctx: &mut Context) -> GameResult {
        if let Some(batch) = game_ctx.texture_set.tex_map.get_mut(&self.tex_tileset_name) {
            let mut rect = Rect::<usize>::new(0, 0, 16, 16);

            let tile_start_x = clamp(state.frame.x / 0x200 / 16, 0, state.stage.map.width as isize) as usize;
            let tile_start_y = clamp(state.frame.y / 0x200 / 16, 0, state.stage.map.height as isize) as usize;
            let tile_end_x = clamp((state.frame.x / 0x200 + 8 + game_ctx.canvas_size.0 as isize) / 16 + 1, 0, state.stage.map.width as isize) as usize;
            let tile_end_y = clamp((state.frame.y / 0x200 + 8 + game_ctx.canvas_size.1 as isize) / 16 + 1, 0, state.stage.map.height as isize) as usize;

            for y in tile_start_y..tile_end_y {
                for x in tile_start_x..tile_end_x {
                    let tile = *state.stage.map.tiles
                        .get((y * state.stage.map.width) + x)
                        .unwrap();

                    match layer {
                        TileLayer::Background => {
                            if state.stage.map.attrib[tile as usize] >= 0x20 {
                                continue;
                            }
                        }
                        TileLayer::Foreground => {
                            let attr = state.stage.map.attrib[tile as usize];
                            if attr < 0x40 || attr >= 0x80 {
                                continue;
                            }
                        }
                        TileLayer::All => {}
                    }

                    rect.left = (tile as usize % 16) * 16;
                    rect.top = (tile as usize / 16) * 16;
                    rect.right = rect.left + 16;
                    rect.bottom = rect.top + 16;

                    batch.add_rect((x as f32 * 16.0 - 8.0) - (state.frame.x / 0x200) as f32, (y as f32 * 16.0 - 8.0) - (state.frame.y / 0x200) as f32, &rect);
                }
            }

            batch.draw(ctx)?;
        }
        Ok(())
    }
}

impl Scene for GameScene {
    fn init(&mut self, state: &mut GameState, game_ctx: &mut GameContext, ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn tick(&mut self, state: &mut GameState, game_ctx: &mut GameContext, ctx: &mut Context) -> GameResult {
        state.update_key_trigger();

        /*if state.flags.flag_x01() {
            state.player_mut().tick(state, &game_ctx.constants, ctx)?;
            state.player_mut().flags.0 = 0;
            state.player_mut().tick_map_collisions(state);
            state.frame.update(state.player(), &state.stage, game_ctx.canvas_size);
        }*/

        state.tick = state.tick.wrapping_add(1);
        //state.tick(game_ctx, ctx)?;

        if state.flags.control_enabled() {
            // update health bar
            if self.life_bar < state.player().life as usize {
                self.life_bar = state.player().life as usize;
            }

            if self.life_bar > state.player().life as usize {
                self.life_bar_count += 1;
                if self.life_bar_count > 30 {
                    self.life_bar -= 1;
                }
            } else {
                self.life_bar_count = 0;
            }
        }

        Ok(())
    }

    fn draw(&self, state: &GameState, game_ctx: &mut GameContext, ctx: &mut Context) -> GameResult {
        self.draw_background(state, game_ctx, ctx)?;
        self.draw_tiles(TileLayer::Background, state, game_ctx, ctx)?;
        state.player().draw(state, game_ctx, ctx)?;
        self.draw_tiles(TileLayer::Foreground, state, game_ctx, ctx)?;
        self.draw_hud(state, game_ctx, ctx)?;

        Ok(())
    }

    fn overlay_draw(&mut self, state: &mut GameState, game_ctx: &mut GameContext, ctx: &mut Context, ui: &mut imgui::Ui) -> GameResult {
        self.debugger.run(state, game_ctx, ctx, ui)?;
        Ok(())
    }
}
