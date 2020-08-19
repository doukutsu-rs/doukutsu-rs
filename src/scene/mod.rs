use ggez::{Context, GameResult};

use crate::GameContext;
use crate::game_state::GameState;

pub mod game_scene;
pub mod loading_scene;

pub trait Scene {
    fn init(&mut self, _state: &mut GameState, _game_ctx: &mut GameContext, _ctx: &mut Context) -> GameResult { Ok(()) }

    fn tick(&mut self, _state: &mut GameState, _game_ctx: &mut GameContext, _ctx: &mut Context) -> GameResult { Ok(()) }

    fn draw(&self, _state: &GameState, _game_ctx: &mut GameContext, _ctx: &mut Context) -> GameResult { Ok(()) }

    fn overlay_draw(&mut self, _state: &mut GameState, _game_ctx: &mut GameContext, _ctx: &mut Context, _ui: &mut imgui::Ui) -> GameResult { Ok(()) }
}
