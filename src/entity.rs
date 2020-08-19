use ggez::{Context, GameResult};

use crate::engine_constants::EngineConstants;
use crate::game_state::GameState;
use crate::GameContext;

pub trait GameEntity {
    fn init(&mut self, _state: &GameState, _game_ctx: &mut GameContext, _ctx: &mut Context) -> GameResult { Ok(()) }

    fn tick(&mut self, state: &GameState, constants: &EngineConstants, ctx: &mut Context) -> GameResult;

    fn draw(&self, state: &GameState, game_ctx: &mut GameContext, ctx: &mut Context) -> GameResult;
}
