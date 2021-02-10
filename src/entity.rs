use crate::framework::context::Context;
use crate::framework::error::GameResult;

use crate::frame::Frame;
use crate::shared_game_state::SharedGameState;

pub trait GameEntity<C> {
    fn tick(&mut self, state: &mut SharedGameState, custom: C) -> GameResult;

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult;
}
