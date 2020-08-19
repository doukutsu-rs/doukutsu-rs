use ggez::{Context, GameResult};

use crate::frame::Frame;
use crate::SharedGameState;

pub trait GameEntity {
    fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult;

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult;
}
