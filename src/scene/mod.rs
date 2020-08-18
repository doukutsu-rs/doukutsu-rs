use ggez::{Context, GameResult};

use crate::SharedGameState;

pub mod game_scene;
pub mod loading_scene;

pub trait Scene {
    fn init(&mut self, _state: &mut SharedGameState, _ctx: &mut Context) -> GameResult { Ok(()) }

    fn tick(&mut self, _state: &mut SharedGameState, _ctx: &mut Context) -> GameResult { Ok(()) }

    fn draw(&self, _state: &mut SharedGameState, _ctx: &mut Context) -> GameResult { Ok(()) }

    fn overlay_draw(&self, _state: &mut SharedGameState, _ctx: &mut Context) -> GameResult { Ok(()) }
}
