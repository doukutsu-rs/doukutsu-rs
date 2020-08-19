use ggez::{Context, GameResult};

use crate::live_debugger::LiveDebugger;
use crate::SharedGameState;

pub mod game_scene;
pub mod loading_scene;

pub trait Scene {
    fn init(&mut self, _state: &mut SharedGameState, _ctx: &mut Context) -> GameResult { Ok(()) }

    fn tick(&mut self, _state: &mut SharedGameState, _ctx: &mut Context) -> GameResult { Ok(()) }

    fn draw(&self, _state: &mut SharedGameState, _ctx: &mut Context) -> GameResult { Ok(()) }

    fn debug_overlay_draw(&mut self, dbg: &mut LiveDebugger, _state: &mut SharedGameState, _ctx: &mut Context, ui: &mut imgui::Ui) -> GameResult { Ok(()) }
}
