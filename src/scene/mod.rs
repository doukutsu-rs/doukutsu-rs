use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::ui::Components;
use crate::shared_game_state::SharedGameState;

#[cfg(feature = "editor")]
pub mod editor_scene;
pub mod game_scene;
pub mod jukebox_scene;
pub mod loading_scene;
pub mod no_data_scene;
pub mod title_scene;

/// Implement this trait on any object that represents an interactive game screen.
pub trait Scene: downcast::Any {
    /// Called when the scene is shown.
    fn init(&mut self, _state: &mut SharedGameState, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    /// Called at game tick. Perform any game state updates there.
    fn tick(&mut self, _state: &mut SharedGameState, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    /// Called before draws between two ticks to update previous positions used for interpolation.
    /// DO NOT perform updates of the game state there.
    fn draw_tick(&mut self, _state: &mut SharedGameState) -> GameResult {
        Ok(())
    }

    /// Called during frame rendering operation.
    fn draw(&self, _state: &mut SharedGameState, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    /// Independent draw meant for debug overlay, that lets you mutate the game state.
    fn imgui_draw(
        &mut self,
        _game_ui: &mut Components,
        _state: &mut SharedGameState,
        _ctx: &mut Context,
        _frame: &mut imgui::Ui,
    ) -> GameResult {
        Ok(())
    }
}

downcast::impl_downcast!(dyn Scene);
