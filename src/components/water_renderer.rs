use std::cell::RefCell;

use crate::common::{Color, Rect};
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::framework::backend::{BackendShader, SpriteBatchCommand, VertexData};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics;
use crate::graphics::BlendMode;
use crate::map::{WaterParamEntry, WaterParams, WaterRegionType};
use crate::npc::list::NPCList;
use crate::physics::PhysicalEntity;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;

const TENSION: f32 = 0.03;
const DAMPENING: f32 = 0.01;
const SPREAD: f32 = 0.02;

struct DynamicWaterColumn {
    target_height: f32,
    height: f32,
    speed: f32,
}

impl DynamicWaterColumn {
    pub fn new() -> DynamicWaterColumn {
        DynamicWaterColumn { target_height: 8.0, height: 8.0, speed: 0.0 }
    }

    pub fn tick(&mut self, dampening: f32, tension: f32) {
        self.speed += tension * (self.target_height - self.height) - self.speed * dampening;
        self.height += self.speed;
    }
}

pub struct DynamicWater {
    x: u16,
    y: u16,
    end_x: u16,
    columns: Vec<DynamicWaterColumn>,
    color: WaterParamEntry,
}

impl DynamicWater {
    pub fn new(x: u16, y: u16, length: u16, color: WaterParamEntry) -> DynamicWater {
        let mut columns = Vec::new();
        let count = length as usize * 8 + 1;

        for _ in 0..count {
            columns.push(DynamicWaterColumn::new());
        }

        DynamicWater { x, y, end_x: x + length, columns, color }
    }

    pub fn tick(&mut self) {
        for col in &mut self.columns {
            col.tick(DAMPENING, TENSION);
        }

        static mut L_DELTAS: Vec<f32> = Vec::new();
        static mut R_DELTAS: Vec<f32> = Vec::new();

        // we assume tick() is never called from other threads.
        unsafe {
            L_DELTAS.resize(self.columns.len(), 0.0);
            R_DELTAS.resize(self.columns.len(), 0.0);

            for _ in 0..2 {
                for i in 0..self.columns.len() {
                    if i > 0 {
                        L_DELTAS[i] = SPREAD * (self.columns[i].height - self.columns[i - 1].height);
                        self.columns[i - 1].speed += L_DELTAS[i];
                    }

                    if i < self.columns.len() - 1 {
                        R_DELTAS[i] = SPREAD * (self.columns[i].height - self.columns[i + 1].height);
                        self.columns[i + 1].speed += R_DELTAS[i];
                    }
                }

                for i in 0..self.columns.len() {
                    if i > 0 {
                        self.columns[i - 1].height += L_DELTAS[i];
                    }

                    if i < self.columns.len() - 1 {
                        self.columns[i + 1].height += R_DELTAS[i];
                    }
                }
            }
        }
    }
}

pub struct WaterRenderer {
    depth_regions: Vec<(Rect<u16>, WaterParamEntry)>,
    water_surfaces: Vec<DynamicWater>,
    t: RefCell<u32>,
}

impl WaterRenderer {
    pub fn new() -> WaterRenderer {
        WaterRenderer { depth_regions: Vec::new(), water_surfaces: Vec::new(), t: RefCell::new(0) }
    }

    pub fn initialize(&mut self, regions: Vec<(WaterRegionType, Rect<u16>, u8)>, water_params: &WaterParams) {
        for (reg_type, bounds, color_idx) in regions {
            let color = water_params.get_entry(color_idx);

            match reg_type {
                WaterRegionType::WaterLine => {
                    self.water_surfaces.push(DynamicWater::new(bounds.left, bounds.top, bounds.width() + 1, *color));
                }
                WaterRegionType::WaterDepth => {
                    self.depth_regions.push((bounds, *color));
                }
            }
        }
    }
}

impl GameEntity<(&[&Player], &NPCList)> for WaterRenderer {
    fn tick(&mut self, state: &mut SharedGameState, (players, npc_list): (&[&Player], &NPCList)) -> GameResult<()> {
        for surf in &mut self.water_surfaces {
            let line_x = surf.x as f32 * 16.0;
            let line_y = surf.y as f32 * 16.0;

            let mut tick_object = |obj: &dyn PhysicalEntity| {
                let obj_x = obj.x() as f32 / 512.0 + 8.0;
                let obj_y = obj.y() as f32 / 512.0 + 8.0;

                if (obj.vel_y() > 0x80 || obj.vel_y() < -0x80)
                    && obj_x > line_x
                    && obj_x < surf.end_x as f32 * 16.0
                    && obj_y > line_y - 5.0
                    && obj_y < line_y + 4.0
                {
                    let col_idx_center = (((obj_x - line_x) / 2.0) as i32).clamp(0, surf.columns.len() as i32);
                    let col_idx_left = (col_idx_center - (obj.hit_bounds().left as i32 / (8 * 0x200)))
                        .clamp(0, surf.columns.len() as i32) as usize;
                    let col_idx_right = (col_idx_center + (obj.hit_bounds().left as i32 / (8 * 0x200)))
                        .clamp(0, surf.columns.len() as i32) as usize;

                    for col in &mut surf.columns[col_idx_left..=col_idx_right] {
                        col.speed = (obj.vel_y() as f32 / 1024.0) * (obj.hit_rect_size() as f32 * 0.25).clamp(0.1, 1.0);
                    }
                }
            };

            for player in players {
                tick_object(*player);
            }

            for npc in npc_list.iter_alive() {
                static NO_COLL_NPCS: [u16; 3] = [0, 3, 4];
                if NO_COLL_NPCS.contains(&npc.npc_type) {
                    continue;
                }

                tick_object(npc);
            }

            surf.tick();
        }

        let mut t_ref = self.t.borrow_mut();
        *t_ref = t_ref.wrapping_add(1);

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult<()> {
        if !graphics::supports_vertex_draw(ctx)? {
            return Ok(());
        }

        graphics::set_render_target(ctx, state.lightmap_canvas.as_ref())?;
        graphics::clear(ctx, Color::from_rgba(0, 0, 0, 0));
        graphics::set_blend_mode(ctx, BlendMode::None)?;

        let (o_x, o_y) = frame.xy_interpolated(state.frame_time);
        let uv = (0.5, 0.5);
        let t = *self.t.borrow_mut() as f32 + state.frame_time as f32;
        let shader = BackendShader::WaterFill(state.scale, t, (o_x, o_y));

        for (region, color) in &self.depth_regions {
            let color_mid_rgba = color.color_middle.to_rgba();
            let color_btm_rgba = color.color_bottom.to_rgba();
            let mut vertices = vec![];
            vertices.reserve(6);

            let left = (region.left as f32 * 16.0 - o_x - 8.0) * state.scale;
            let top = (region.top as f32 * 16.0 - o_y - 8.0) * state.scale;
            let right = (region.right as f32 * 16.0 - o_x + 8.0) * state.scale;
            let bottom = (region.bottom as f32 * 16.0 - o_y + 8.0) * state.scale;

            vertices.push(VertexData { position: (left, bottom), uv, color: color_btm_rgba });
            vertices.push(VertexData { position: (left, top), uv, color: color_mid_rgba });
            vertices.push(VertexData { position: (right, top), uv, color: color_mid_rgba });
            vertices.push(VertexData { position: (left, bottom), uv, color: color_btm_rgba });
            vertices.push(VertexData { position: (right, top), uv, color: color_mid_rgba });
            vertices.push(VertexData { position: (right, bottom), uv, color: color_btm_rgba });

            graphics::draw_triangle_list(ctx, vertices, None, shader)?;
        }

        for surf in &self.water_surfaces {
            let pos_x = surf.x as f32 * 16.0;
            let pos_y = surf.y as f32 * 16.0;
            let color_top_rgba = surf.color.color_top.to_rgba();
            let color_mid_rgba = surf.color.color_middle.to_rgba();
            let color_btm_rgba = surf.color.color_bottom.to_rgba();

            if (pos_x - o_x - 16.0) > state.canvas_size.0
                || (pos_x - o_x + 16.0 + surf.end_x as f32 * 16.0) < 0.0
                || (pos_y - o_y - 16.0) > state.canvas_size.1
                || (pos_y - o_y + 16.0) < 0.0
            {
                continue;
            }

            let mut vertices = vec![];
            vertices.reserve(12 * surf.columns.len());

            let bottom = (pos_y - o_y + 8.0) * state.scale;
            for i in 1..surf.columns.len() {
                let x_right = (pos_x - 8.0 - o_x + i as f32 * 2.0) * state.scale;
                let x_left = x_right - 2.0 * state.scale;
                let top_left = (pos_y - o_y - 13.0 + surf.columns[i - 1].height) * state.scale;
                let top_right = (pos_y - o_y - 13.0 + surf.columns[i].height) * state.scale;
                let middle_left = top_left + 6.0 * state.scale;
                let middle_right = top_left + 6.0 * state.scale;

                vertices.push(VertexData { position: (x_left, middle_left), uv, color: color_mid_rgba });
                vertices.push(VertexData { position: (x_left, top_left), uv, color: color_top_rgba });
                vertices.push(VertexData { position: (x_right, top_right), uv, color: color_top_rgba });
                vertices.push(VertexData { position: (x_left, middle_left), uv, color: color_mid_rgba });
                vertices.push(VertexData { position: (x_right, top_right), uv, color: color_top_rgba });
                vertices.push(VertexData { position: (x_right, middle_right), uv, color: color_mid_rgba });

                vertices.push(VertexData { position: (x_left, bottom), uv, color: color_btm_rgba });
                vertices.push(VertexData { position: (x_left, middle_left), uv, color: color_mid_rgba });
                vertices.push(VertexData { position: (x_right, middle_right), uv, color: color_mid_rgba });
                vertices.push(VertexData { position: (x_left, bottom), uv, color: color_btm_rgba });
                vertices.push(VertexData { position: (x_right, middle_right), uv, color: color_mid_rgba });
                vertices.push(VertexData { position: (x_right, bottom), uv, color: color_btm_rgba });
            }

            graphics::draw_triangle_list(ctx, vertices, None, shader)?;
        }

        graphics::set_blend_mode(ctx, BlendMode::Alpha)?;
        graphics::set_render_target(ctx, None)?;

        {
            let canvas = state.lightmap_canvas.as_mut().unwrap();
            let rect = Rect { left: 0.0, top: 0.0, right: state.screen_size.0, bottom: state.screen_size.1 };

            canvas.clear();
            canvas.add(SpriteBatchCommand::DrawRect(rect, rect));
            canvas.draw()?;
        }

        Ok(())
    }
}
