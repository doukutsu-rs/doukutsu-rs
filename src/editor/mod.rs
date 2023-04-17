use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use imgui::{Image, MouseButton, Window};

use crate::common::{Color, Rect};
use crate::components::background::Background;
use crate::components::tilemap::{TileLayer, Tilemap};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics;
use crate::game::shared_game_state::SharedGameState;
use crate::game::frame::Frame;
use crate::game::stage::{Stage, StageTexturePaths};
use crate::graphics::texture_set::I_MAG;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CurrentTool {
    Move,
    Brush,
    Fill,
    Rectangle,
}

pub struct EditorInstance {
    pub stage: Stage,
    pub stage_id: usize,
    pub frame: Frame,
    pub background: Background,
    pub stage_textures: Rc<RefCell<StageTexturePaths>>,
    pub tilemap: Tilemap,
    pub zoom: f32,
    pub current_tile: u8,
    pub mouse_pos: (f32, f32),
    pub want_capture_mouse: bool,
}

impl EditorInstance {
    pub fn new(stage_id: usize, stage: Stage) -> EditorInstance {
        let stage_textures = {
            let mut textures = StageTexturePaths::new();
            textures.update(&stage);
            Rc::new(RefCell::new(textures))
        };
        let mut frame = Frame::new();
        frame.x = -16 * 0x200;
        frame.y = -48 * 0x200;

        EditorInstance {
            stage,
            stage_id,
            frame,
            background: Background::new(),
            stage_textures,
            tilemap: Tilemap::new(),
            zoom: 2.0,
            current_tile: 0,
            mouse_pos: (0.0, 0.0),
            want_capture_mouse: true,
        }
    }

    pub fn process(&mut self, state: &mut SharedGameState, ctx: &mut Context, ui: &mut imgui::Ui, tool: CurrentTool) {
        self.frame.prev_x = self.frame.x;
        self.frame.prev_y = self.frame.y;
        self.mouse_pos = (ui.io().mouse_pos[0], ui.io().mouse_pos[1]);
        self.want_capture_mouse = ui.io().want_capture_mouse;

        let mut drag = false;

        match tool {
            CurrentTool::Move => {
                if ui.io().want_capture_mouse {
                    return;
                }

                drag |= ui.is_mouse_down(MouseButton::Left) || ui.is_mouse_down(MouseButton::Right);
            }
            CurrentTool::Brush => {
                self.palette_window(state, ctx, ui);

                if ui.io().want_capture_mouse {
                    return;
                }

                drag |= ui.is_mouse_down(MouseButton::Right);

                if !drag && ui.is_mouse_down(MouseButton::Left) {
                    let tile_size = self.stage.map.tile_size.as_int();
                    let halft = tile_size / 2;
                    let stage_mouse_x = (self.frame.x / 0x200) + halft + (self.mouse_pos.0 / self.zoom) as i32;
                    let stage_mouse_y = (self.frame.y / 0x200) + halft + (self.mouse_pos.1 / self.zoom) as i32;
                    let tile_x = stage_mouse_x / tile_size;
                    let tile_y = stage_mouse_y / tile_size;

                    if tile_x >= 0
                        && tile_y >= 0
                        && tile_x < self.stage.map.width as i32
                        && tile_y < self.stage.map.height as i32
                    {
                        self.stage.change_tile(tile_x as usize, tile_y as usize, self.current_tile);
                    }
                }
            }
            CurrentTool::Fill => {
                self.palette_window(state, ctx, ui);
                drag |= ui.is_mouse_down(MouseButton::Right);
            }
            CurrentTool::Rectangle => {
                self.palette_window(state, ctx, ui);
                drag |= ui.is_mouse_down(MouseButton::Right);
            }
        }

        if drag {
            self.frame.x -= (512.0 * ui.io().mouse_delta[0] as f32 / self.zoom) as i32;
            self.frame.y -= (512.0 * ui.io().mouse_delta[1] as f32 / self.zoom) as i32;
            self.frame.prev_x = self.frame.x;
            self.frame.prev_y = self.frame.y;
        }
    }

    fn tile_cursor(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if self.want_capture_mouse {
            return Ok(());
        }

        let tile_size = self.stage.map.tile_size.as_int();
        let halft = tile_size / 2;
        let stage_mouse_x = (self.frame.x / 0x200) + halft + (self.mouse_pos.0 / self.zoom) as i32;
        let stage_mouse_y = (self.frame.y / 0x200) + halft + (self.mouse_pos.1 / self.zoom) as i32;
        let tile_x = stage_mouse_x / tile_size;
        let tile_y = stage_mouse_y / tile_size;
        let frame_x = self.frame.x as f32 / 512.0;
        let frame_y = self.frame.y as f32 / 512.0;

        if tile_x < 0 || tile_y < 0 || tile_x >= self.stage.map.width as i32 || tile_y >= self.stage.map.height as i32 {
            return Ok(());
        }

        let name = &self.stage_textures.deref().borrow().tileset_fg;

        if let Ok(batch) = state.texture_set.get_or_load_batch(ctx, &state.constants, name) {
            let tile_size16 = tile_size as u16;
            let rect = Rect::new_size(
                (self.current_tile as u16 % 16) * tile_size16,
                (self.current_tile as u16 / 16) * tile_size16,
                tile_size16,
                tile_size16,
            );

            batch.add_rect_tinted(
                (tile_x * tile_size - halft) as f32 - frame_x,
                (tile_y * tile_size - halft) as f32 - frame_y,
                (255, 255, 255, 192),
                &rect,
            );

            batch.draw(ctx)?;
        }

        Ok(())
    }

    fn palette_window(&mut self, state: &mut SharedGameState, ctx: &mut Context, ui: &imgui::Ui) {
        Window::new("Palette")
            .size([260.0, 260.0], imgui::Condition::Always)
            .position(ui.io().display_size, imgui::Condition::FirstUseEver)
            .position_pivot([1.0, 1.0])
            .resizable(false)
            .build(ui, || {
                let name = &self.stage_textures.deref().borrow().tileset_fg;
                let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, name);

                let pos = ui.cursor_screen_pos();
                let tile_size = self.stage.map.tile_size.as_float();

                if let Ok(batch) = batch {
                    let (scale_x, scale_y) = batch.scale();
                    if let Some(tex) = batch.get_texture() {
                        let (width, height) = tex.dimensions();
                        let (width, height) = (width as f32 / scale_x, height as f32 / scale_y);

                        if let Ok(tex_id) = graphics::imgui_texture_id(ctx, tex) {
                            Image::new(tex_id, [width, height]).build(ui);
                        }

                        ui.set_cursor_screen_pos(pos);
                        ui.invisible_button("##tiles", [width, height]);
                    }
                }

                let draw_list = ui.get_window_draw_list();
                let cur_pos1 = [
                    pos[0].floor() + tile_size * (self.current_tile % 16) as f32,
                    pos[1].floor() + tile_size * (self.current_tile / 16) as f32,
                ];
                let cur_pos2 = [cur_pos1[0] + tile_size, cur_pos1[1] + tile_size];
                draw_list.add_rect(cur_pos1, cur_pos2, [1.0, 0.0, 0.0, 1.0]).thickness(2.0).build();

                if ui.is_mouse_down(MouseButton::Left) {
                    let mouse_pos = ui.io().mouse_pos;
                    let x = (mouse_pos[0] - pos[0]) / tile_size;
                    let y = (mouse_pos[1] - pos[1]) / tile_size;

                    if x >= 0.0 && x < 16.0 && y >= 0.0 && y < 16.0 {
                        self.current_tile = (y as u8 * 16 + x as u8) as u8;
                    }
                }
            });
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, tool: CurrentTool) -> GameResult {
        let old_scale = state.scale;
        set_scale(state, self.zoom);

        let paths = self.stage_textures.deref().borrow();
        self.background.draw(state, ctx, &self.frame, &*paths, &self.stage)?;

        self.tilemap.draw(state, ctx, &self.frame, TileLayer::Background, &*paths, &self.stage)?;
        self.tilemap.draw(state, ctx, &self.frame, TileLayer::Middleground, &*paths, &self.stage)?;
        self.tilemap.draw(state, ctx, &self.frame, TileLayer::Foreground, &*paths, &self.stage)?;
        self.tilemap.draw(state, ctx, &self.frame, TileLayer::Snack, &*paths, &self.stage)?;

        self.draw_black_bars(state, ctx)?;

        match tool {
            CurrentTool::Move => (),
            CurrentTool::Brush | CurrentTool::Fill | CurrentTool::Rectangle => {
                self.tile_cursor(state, ctx)?;
            }
        }

        set_scale(state, old_scale);

        Ok(())
    }

    fn draw_black_bars(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let color = Color::from_rgba(0, 0, 0, 128);
        let (x, y) = self.frame.xy_interpolated(state.frame_time);
        let (x, y) = (x * state.scale, y * state.scale);
        let canvas_w_scaled = state.canvas_size.0 as f32 * state.scale;
        let canvas_h_scaled = state.canvas_size.1 as f32 * state.scale;
        let level_width = (self.stage.map.width as f32 - 1.0) * self.stage.map.tile_size.as_float();
        let level_height = (self.stage.map.height as f32 - 1.0) * self.stage.map.tile_size.as_float();
        let left_side = -x;
        let right_side = -x + level_width * state.scale;
        let upper_side = -y;
        let lower_side = -y + level_height * state.scale;

        if left_side > 0.0 {
            let rect = Rect::new(0, upper_side as isize, left_side as isize, lower_side as isize);
            graphics::draw_rect(ctx, rect, color)?;
        }

        if right_side < canvas_w_scaled {
            let rect = Rect::new(
                right_side as isize,
                upper_side as isize,
                (state.canvas_size.0 * state.scale) as isize,
                lower_side as isize,
            );
            graphics::draw_rect(ctx, rect, color)?;
        }

        if upper_side > 0.0 {
            let rect = Rect::new(0, 0, canvas_w_scaled as isize, upper_side as isize);
            graphics::draw_rect(ctx, rect, color)?;
        }

        if lower_side < canvas_h_scaled {
            let rect = Rect::new(0, lower_side as isize, canvas_w_scaled as isize, canvas_h_scaled as isize);
            graphics::draw_rect(ctx, rect, color)?;
        }

        Ok(())
    }
}

fn set_scale(state: &mut SharedGameState, scale: f32) {
    state.scale = scale;

    unsafe {
        I_MAG = state.scale;
        state.canvas_size = (state.screen_size.0 / state.scale, state.screen_size.1 / state.scale);
    }
}
