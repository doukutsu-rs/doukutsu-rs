use imgui::MenuItem;

use crate::framework::ui::Components;
use crate::scene::title_scene::TitleScene;
use crate::{Context, GameResult, Scene, SharedGameState};

pub struct EditorScene {}

impl EditorScene {
    pub fn new() -> Self {
        EditorScene {}
    }

    fn exit_editor(&mut self, state: &mut SharedGameState) {
        state.next_scene = Some(Box::new(TitleScene::new()));
    }
}

impl Scene for EditorScene {
    fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        state.sound_manager.play_song(0, &state.constants, &state.settings, ctx)?;

        Ok(())
    }

    fn draw(&self, _state: &mut SharedGameState, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn imgui_draw(
        &mut self,
        _game_ui: &mut Components,
        state: &mut SharedGameState,
        _ctx: &mut Context,
        ui: &mut imgui::Ui,
    ) -> GameResult {
        if let Some(menu_bar) = ui.begin_main_menu_bar() {
            if let Some(menu) = ui.begin_menu("File") {
                MenuItem::new("Open stage").shortcut("Ctrl+O").build(ui);
                ui.separator();

                if MenuItem::new("Exit editor").build(ui) {
                    self.exit_editor(state);
                }

                menu.end();
            }
            menu_bar.end();
        }

        Ok(())
    }
}
