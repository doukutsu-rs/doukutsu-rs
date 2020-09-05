use crate::ggez::{Context, GameResult};

use crate::frame::Frame;
use crate::SharedGameState;

pub trait GameEntity<C> {
    fn tick(&mut self, state: &mut SharedGameState, custom: C) -> GameResult;

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult;
}
