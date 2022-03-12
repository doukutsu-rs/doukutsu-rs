use std::cell::RefCell;

use crate::common::{Color, Rect};
use crate::framework::backend::{BackendTexture, SpriteBatchCommand};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::graphics;
use crate::player::Player;
use crate::scripting::tsc::text_script::TextScriptExecutionState;
use crate::shared_game_state::{Language, SharedGameState};
use crate::stage::Stage;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MapSystemState {
    Hidden,
    FadeInBox(u16),
    FadeInLine(u16),
    Visible,
    FadeOutBox(u16),
}

pub struct MapSystem {
    texture: RefCell<Option<Box<dyn BackendTexture>>>,
    has_map_data: RefCell<bool>,
    last_size: (u16, u16),
    tick: u16,
    state: MapSystemState,
}

impl MapSystem {
    pub fn new() -> MapSystem {
        MapSystem {
            texture: RefCell::new(None),
            has_map_data: RefCell::new(false),
            last_size: (0, 0),
            tick: 0,
            state: MapSystemState::Hidden,
        }
    }

    fn render_map(&self, state: &mut SharedGameState, ctx: &mut Context, stage: &Stage) -> GameResult {
        if self.texture.borrow().is_none() {
            *self.has_map_data.borrow_mut() = false;
            return Ok(());
        }

        *self.has_map_data.borrow_mut() = true;

        graphics::set_render_target(ctx, self.texture.borrow().as_ref())?;
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;

        for y in 0..stage.map.height {
            for x in 0..stage.map.width {
                const RECTS: [Rect<u16>; 4] = [
                    Rect { left: 240, top: 24, right: 241, bottom: 25 },
                    Rect { left: 241, top: 24, right: 242, bottom: 25 },
                    Rect { left: 242, top: 24, right: 243, bottom: 25 },
                    Rect { left: 243, top: 24, right: 244, bottom: 25 },
                ];

                let attr = stage.map.get_attribute(x as _, y as _);

                let layer = match attr {
                    0 => 0,
                    0x01 | 0x02 | 0x40 | 0x44 | 0x51 | 0x52 | 0x55 | 0x56 | 0x60 | 0x71 | 0x72 | 0x75 | 0x76 | 0x80
                    | 0x81 | 0x82 | 0x83 | 0xA0 | 0xA1 | 0xA2 | 0xA3 => 1,
                    0x43 | 0x50 | 0x53 | 0x54 | 0x57 | 0x63 | 0x70 | 0x73 | 0x74 | 0x77 => 2,
                    _ => 3,
                };

                batch.add_rect(x as _, y as _, &RECTS[layer]);
            }
        }

        batch.draw(ctx)?;
        graphics::set_render_target(ctx, None)?;

        Ok(())
    }

    pub fn tick(
        &mut self,
        state: &mut SharedGameState,
        ctx: &mut Context,
        stage: &Stage,
        players: [&Player; 2],
    ) -> GameResult {
        if state.textscript_vm.state == TextScriptExecutionState::MapSystem {
            if self.state == MapSystemState::Hidden {
                state.control_flags.set_control_enabled(false);
                self.state = MapSystemState::FadeInBox(0);
            }
        } else {
            self.state = MapSystemState::Hidden;
        }

        if self.state == MapSystemState::Hidden {
            self.tick = 0;
            *self.has_map_data.borrow_mut() = false;
            return Ok(());
        }

        self.tick = self.tick.wrapping_add(1);

        let width = (stage.map.width as f32 * state.scale) as u16;
        let height = (stage.map.height as f32 * state.scale) as u16;

        if self.last_size != (width, height) {
            self.last_size = (width, height);
            *self.texture.borrow_mut() = graphics::create_texture_mutable(ctx, width, height).ok();
            *self.has_map_data.borrow_mut() = false;
        }

        match self.state {
            MapSystemState::FadeInBox(tick) => {
                if tick >= 8 {
                    self.state = MapSystemState::FadeInLine(0);
                } else {
                    self.state = MapSystemState::FadeInBox(tick + 1);
                }
            }
            MapSystemState::FadeOutBox(tick) => {
                if tick == 0 {
                    state.control_flags.set_tick_world(true);
                    state.control_flags.set_control_enabled(true);
                    state.textscript_vm.state = TextScriptExecutionState::Ended;
                    self.state = MapSystemState::Hidden;
                } else {
                    self.state = MapSystemState::FadeOutBox(tick - 1);
                }
            }
            MapSystemState::FadeInLine(tick) => {
                if (tick + 2) < stage.map.height {
                    self.state = MapSystemState::FadeInLine(tick + 2);
                } else {
                    self.state = MapSystemState::Visible;
                }

                for player in &players {
                    if player.controller.trigger_jump() || player.controller.trigger_shoot() {
                        self.state = MapSystemState::FadeOutBox(8);
                        break;
                    }
                }
            }
            MapSystemState::Visible => {
                for player in &players {
                    if player.controller.trigger_jump() || player.controller.trigger_shoot() {
                        self.state = MapSystemState::FadeOutBox(8);
                        break;
                    }
                }
            }
            _ => (),
        }

        Ok(())
    }

    pub fn draw(
        &self,
        state: &mut SharedGameState,
        ctx: &mut Context,
        stage: &Stage,
        players: [&Player; 2],
    ) -> GameResult {
        if self.state == MapSystemState::Hidden {
            return Ok(());
        }

        if !*self.has_map_data.borrow() {
            self.render_map(state, ctx, stage)?;
        }

        let (scr_w, scr_h) = (state.canvas_size.0 * state.scale, state.canvas_size.1 * state.scale);
        let text_height = state.font.line_height(&state.constants);
        let rect_black_bar = Rect::new_size(
            0,
            (7.0 * state.scale) as _,
            state.screen_size.0 as _,
            ((text_height + 4.0) * state.scale) as _,
        );

        if !state.constants.is_switch {
            graphics::draw_rect(ctx, rect_black_bar, Color::new(0.0, 0.0, 0.0, 1.0))?;
        }

        let map_name = if state.settings.locale == Language::Japanese {
            stage.data.name_jp.chars()
        } else {
            stage.data.name.chars()
        };

        let map_name_width = state.font.text_width(map_name.clone(), &state.constants);
        let map_name_off_x = (state.canvas_size.0 - map_name_width) / 2.0;

        state.font.draw_text(map_name, map_name_off_x, 9.0, &state.constants, &mut state.texture_set, ctx)?;

        let mut map_rect = Rect::new(0.0, 0.0, self.last_size.0 as f32, self.last_size.1 as f32);

        match self.state {
            MapSystemState::FadeInBox(tick) | MapSystemState::FadeOutBox(tick) => {
                let width = (state.scale * tick as f32 * stage.map.width as f32 / 16.0) as isize;
                let height = (state.scale * tick as f32 * stage.map.height as f32 / 16.0) as isize;

                let rect = Rect::new_size(
                    (scr_w / 2.0) as isize - width,
                    (scr_h / 2.0) as isize - height,
                    width * 2,
                    height * 2,
                );

                graphics::draw_rect(ctx, rect, Color::new(0.0, 0.0, 0.0, 1.0))?;

                return Ok(());
            }
            MapSystemState::FadeInLine(line) => {
                map_rect.bottom = state.scale * (line as f32 + 1.0);
            }
            _ => (),
        }

        let width_border = state.scale * (stage.map.width as f32 + 2.0);
        let height_border = state.scale * (stage.map.height as f32 + 2.0);

        let rect = Rect::new_size(
            ((scr_w - width_border) / 2.0) as isize,
            ((scr_h - height_border) / 2.0) as isize,
            width_border as isize,
            height_border as isize,
        );

        graphics::draw_rect(ctx, rect, Color::new(0.0, 0.0, 0.0, 1.0))?;

        if let Some(tex) = self.texture.borrow_mut().as_mut() {
            let width = state.scale * stage.map.width as f32;
            let height = state.scale * stage.map.height as f32;

            tex.clear();
            tex.add(SpriteBatchCommand::DrawRect(
                map_rect,
                Rect::new_size((scr_w - width) / 2.0, (scr_h - height) / 2.0, map_rect.width(), map_rect.height()),
            ));
            tex.draw()?;
        }

        if (self.tick & 8) != 0 {
            const PLAYER_RECT: Rect<u16> = Rect { left: 0, top: 57, right: 1, bottom: 58 };

            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;
            let x_offset = (state.canvas_size.0 - stage.map.width as f32) / 2.0;
            let y_offset = (state.canvas_size.1 - stage.map.height as f32) / 2.0;
            let tile_div = stage.map.tile_size.as_int() * 0x200;

            for player in &players {
                if !player.cond.alive() {
                    continue;
                }

                let plr_x = x_offset + (player.x / tile_div) as f32;
                let plr_y = y_offset + (player.y / tile_div) as f32;

                batch.add_rect(plr_x, plr_y, &PLAYER_RECT);
            }

            batch.draw(ctx)?;
        }

        Ok(())
    }
}
