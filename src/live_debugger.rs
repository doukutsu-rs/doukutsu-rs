use ggez::{Context, GameResult};
use imgui::{Condition, im_str, Window};

use crate::game_state::GameState;
use crate::GameContext;

pub struct LiveDebugger {
    selected_item: i32,
}

impl LiveDebugger {
    pub fn new() -> Self {
        Self {
            selected_item: 0,
        }
    }

    pub fn run(&mut self, state: &mut GameState, game_ctx: &GameContext, ctx: &mut Context, ui: &mut imgui::Ui) -> GameResult {
        Window::new(im_str!("Live Debugger"))
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.text(format!(
                    "Player position: ({:.1},{:.1})",
                    state.player().x as f32 / 512.0,
                    state.player().y as f32 / 512.0,
                ));
            });

        Window::new(im_str!("Stages"))
            .size([240.0, 320.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.list_box(im_str!("Stage table"), &mut self.selected_item,
                            &[im_str!("Null"), im_str!("Egg Corridor")], 10);
            });

        Ok(())
    }
}
