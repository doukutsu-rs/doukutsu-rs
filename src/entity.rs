use crate::common::interpolate_fix9_scale;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::game::frame::Frame;
use crate::game::shared_game_state::SharedGameState;

pub trait GameEntity<C> {
    fn tick(&mut self, state: &mut SharedGameState, custom: C) -> GameResult;

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult;
}

pub trait Interpolatable {
    fn position_x(&self) -> i32;

    fn position_y(&self) -> i32;

    fn prev_position_x(&self) -> i32;

    fn prev_position_y(&self) -> i32;

    fn draw_tick(&mut self);

    #[inline]
    fn interpolate_x(&self, frame_delta: f64) -> f32 {
        interpolate_fix9_scale(self.prev_position_x(), self.position_x(), frame_delta)
    }

    #[inline]
    fn interpolate_y(&self, frame_delta: f64) -> f32 {
        interpolate_fix9_scale(self.prev_position_y(), self.position_y(), frame_delta)
    }

    #[inline]
    fn interpolate(&self, frame_delta: f64) -> (f32, f32) {
        (self.interpolate_x(frame_delta), self.interpolate_y(frame_delta))
    }

    #[inline]
    fn interpolate_relative_x(&self, target: &dyn Interpolatable, frame_delta: f64) -> f32 {
        interpolate_fix9_scale(self.prev_position_x() - target.prev_position_x(), self.position_x() - target.position_x(), frame_delta)
    }

    #[inline]
    fn interpolate_relative_y(&self, target: &dyn Interpolatable, frame_delta: f64) -> f32 {
        interpolate_fix9_scale(self.prev_position_y() - target.prev_position_y(), self.position_y() - target.position_y(), frame_delta)
    }

    #[inline]
    fn interpolate_relative(&self, target: &dyn Interpolatable, frame_delta: f64) -> (f32, f32) {
        (self.interpolate_relative_x(target, frame_delta), self.interpolate_relative_y(target, frame_delta))
    }
}
