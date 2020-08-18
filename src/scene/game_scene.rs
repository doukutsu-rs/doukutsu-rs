use ggez::{Context, GameResult};
use ggez::nalgebra::clamp;
use log::info;

use crate::common::Rect;
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::player::Player;
use crate::scene::Scene;
use crate::SharedGameState;
use crate::stage::{BackgroundType, Stage};
use crate::str;
use num_traits::abs;

pub struct GameScene {
    pub tick: usize,
    pub stage: Stage,
    pub frame: Frame,
    pub player: Player,
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
    pub fn new(state: &mut SharedGameState, ctx: &mut Context, id: usize) -> GameResult<GameScene> {
        let stage = Stage::load(ctx, &state.stages[id])?;
        info!("Loaded stage: {}", stage.data.name);
        info!("Map size: {}x{}", stage.map.width, stage.map.height);

        let tex_tileset_name = ["Stage/", &stage.data.tileset.filename()].join("");
        let tex_background_name = stage.data.background.filename();
        let tex_hud_name = str!("TextBox");

        state.texture_set.ensure_texture_loaded(ctx, &tex_tileset_name)?;
        state.texture_set.ensure_texture_loaded(ctx, &tex_background_name)?;
        state.texture_set.ensure_texture_loaded(ctx, &tex_hud_name)?;

        Ok(GameScene {
            tick: 0,
            stage,
            player: Player::new(state, ctx)?,
            frame: Frame {
                x: 0,
                y: 0,
                wait: 16,
            },
            tex_tileset_name,
            tex_background_name,
            tex_hud_name,
            life_bar: 3,
            life_bar_count: 0,
        })
    }

    fn draw_number(&self, x: f32, y: f32, val: usize, align: Alignment, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let set = state.texture_set.tex_map.get_mut(&self.tex_hud_name);
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

    fn draw_hud(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let set = state.texture_set.tex_map.get_mut(&self.tex_hud_name);
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
                       &Rect::<usize>::new_size(0, 32, ((self.life_bar as usize * 40) / self.player.max_life as usize) - 1, 8));
        // life
        batch.add_rect(40.0, 40.0,
                       &Rect::<usize>::new_size(0, 24, ((self.player.life as usize * 40) / self.player.max_life as usize) - 1, 8));

        batch.draw(ctx)?;

        self.draw_number(40.0, 40.0, self.life_bar as usize, Alignment::Right, state, ctx)?;

        Ok(())
    }

    fn draw_background(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let set = state.texture_set.tex_map.get_mut(&self.tex_background_name);
        if set.is_none() {
            return Ok(());
        }

        let batch = set.unwrap();

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

    fn draw_tiles(&self, state: &mut SharedGameState, ctx: &mut Context, layer: TileLayer) -> GameResult {
        let set = state.texture_set.tex_map.get_mut(&self.tex_tileset_name);
        if set.is_none() {
            return Ok(());
        }

        let mut rect = Rect::<usize>::new(0, 0, 16, 16);
        let batch = set.unwrap();

        let tile_start_x = clamp(self.frame.x / 0x200 / 16, 0, self.stage.map.width as isize) as usize;
        let tile_start_y = clamp(self.frame.y / 0x200 / 16, 0, self.stage.map.height as isize) as usize;
        let tile_end_x = clamp((self.frame.x / 0x200 + 8 + state.canvas_size.0 as isize) / 16 + 1, 0, self.stage.map.width as isize) as usize;
        let tile_end_y = clamp((self.frame.y / 0x200 + 8 + state.canvas_size.1 as isize) / 16 + 1, 0, self.stage.map.height as isize) as usize;

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
                    }
                    TileLayer::Foreground => {
                        let attr = self.stage.map.attrib[tile as usize];
                        if attr < 0x40 || attr >= 0x80 {
                            continue;
                        }
                    }
                    _ => {}
                }

                rect.left = (tile as usize % 16) * 16;
                rect.top = (tile as usize / 16) * 16;
                rect.right = rect.left + 16;
                rect.bottom = rect.top + 16;

                batch.add_rect((x as f32 * 16.0 - 8.0) - (self.frame.x / 0x200) as f32, (y as f32 * 16.0 - 8.0) - (self.frame.y / 0x200) as f32, &rect);
            }
        }

        batch.draw(ctx)?;
        Ok(())
    }
}

impl Scene for GameScene {
    fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        state.sound_manager.play_song(ctx)?;
        self.player.x = 700 * 0x200;
        self.player.y = 1000 * 0x200;
        //self.player.equip.set_booster_2_0(true);
        state.flags.set_flag_x01(true);
        state.flags.set_control_enabled(true);
        Ok(())
    }

    fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        state.update_key_trigger();

        if state.flags.flag_x01() {
            self.player.tick(state, ctx)?;

            self.player.flags.0 = 0;
            self.player.tick_map_collisions(state, &self.stage);

            self.frame.update(state, &self.player, &self.stage);
        }

        if state.flags.control_enabled() {
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

        self.tick = self.tick.wrapping_add(1);
        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.draw_background(state, ctx)?;
        self.draw_tiles(state, ctx, TileLayer::Background)?;
        self.player.draw(state, ctx, &self.frame)?;
        self.draw_tiles(state, ctx, TileLayer::Foreground)?;

        self.draw_hud(state, ctx)?;
        self.draw_number(16.0, 50.0, abs(self.player.x) as usize, Alignment::Left, state, ctx)?;
        self.draw_number(16.0, 58.0, abs(self.player.y) as usize, Alignment::Left, state, ctx)?;

        Ok(())
    }
}
