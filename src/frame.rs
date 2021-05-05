use crate::common::{fix9_scale, interpolate_fix9_scale};
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;
use crate::stage::Stage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum UpdateTarget {
    Player,
    NPC(u16),
    Boss(u16),
}

pub struct Frame {
    pub x: i32,
    pub y: i32,
    pub prev_x: i32,
    pub prev_y: i32,
    pub update_target: UpdateTarget,
    pub target_x: i32,
    pub target_y: i32,
    pub wait: i32,
}

impl Frame {
    pub fn xy_interpolated(&self, frame_time: f64) -> (f32, f32) {
        if self.prev_x == self.x && self.prev_y == self.y {
            return (fix9_scale(self.x), fix9_scale(self.y));
        }

        let x = interpolate_fix9_scale(self.prev_x, self.x, frame_time);
        let y = interpolate_fix9_scale(self.prev_y, self.y, frame_time);

        (x, y)
    }

    pub fn immediate_update(&mut self, state: &mut SharedGameState, stage: &Stage) {
        if (stage.map.width as usize).saturating_sub(1) * 16 < state.canvas_size.0 as usize {
            self.x = -(((state.canvas_size.0 as i32 - ((stage.map.width - 1) * 16) as i32) * 0x200) / 2);
        } else {
            self.x = self.target_x - (state.canvas_size.0 as i32 * 0x200 / 2);

            if self.x < 0 {
                self.x = 0;
            }

            let max_x = (((stage.map.width as i32 - 1) * 16) - state.canvas_size.0 as i32) * 0x200;
            if self.x > max_x {
                self.x = max_x;
            }
        }

        if (stage.map.height as usize).saturating_sub(1) * 16 < state.canvas_size.1 as usize {
            self.y = -(((state.canvas_size.1 as i32 - ((stage.map.height - 1) * 16) as i32) * 0x200) / 2);
        } else {
            self.y = self.target_y - (state.canvas_size.1 as i32 * 0x200 / 2);

            if self.y < 0 {
                self.y = 0;
            }

            let max_y = (((stage.map.height as i32 - 1) * 16) - state.canvas_size.1 as i32) * 0x200;
            if self.y > max_y {
                self.y = max_y;
            }
        }

        self.prev_x = self.x;
        self.prev_y = self.y;
    }

    pub fn update(&mut self, state: &mut SharedGameState, stage: &Stage) {
        if (stage.map.width as usize).saturating_sub(1) * 16 < state.canvas_size.0 as usize {
            self.x = -(((state.canvas_size.0 as i32 - ((stage.map.width - 1) * 16) as i32) * 0x200) / 2);
        } else {
            self.x += (self.target_x - (state.canvas_size.0 as i32 * 0x200 / 2) - self.x) / self.wait;

            if self.x < 0 {
                self.x = 0;
            }

            let max_x = (((stage.map.width as i32 - 1) * 16) - state.canvas_size.0 as i32) * 0x200;
            if self.x > max_x {
                self.x = max_x;
            }
        }

        if (stage.map.height as usize).saturating_sub(1) * 16 < state.canvas_size.1 as usize {
            self.y = -(((state.canvas_size.1 as i32 - ((stage.map.height - 1) * 16) as i32) * 0x200) / 2);
        } else {
            self.y += (self.target_y - (state.canvas_size.1 as i32 * 0x200 / 2) - self.y) / self.wait;

            if self.y < 0 {
                self.y = 0;
            }

            let max_y = (((stage.map.height as i32 - 1) * 16) - state.canvas_size.1 as i32) * 0x200;
            if self.y > max_y {
                self.y = max_y;
            }
        }

        if state.quake_counter > 0 {
            state.quake_counter -= 1;

            self.x += state.effect_rng.range(-0x300..0x300) as i32;
            self.y += state.effect_rng.range(-0x300..0x300) as i32;
        }
    }
}
