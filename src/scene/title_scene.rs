use crate::common::{FadeState, Rect};
use crate::ggez::{Context, GameResult};
use crate::menu::{Menu, MenuSelectionResult, MenuEntry};
use crate::scene::game_scene::GameScene;
use crate::scene::Scene;
use crate::SharedGameState;
use crate::text_script::TextScriptExecutionState;

pub struct TitleScene {
    tick: usize,
    title_menu: Menu,
}

impl TitleScene {
    pub fn new() -> Self {
        Self {
            tick: 0,
            title_menu: Menu::new(0, 0, 100, 78),
        }
    }

    fn start_game(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let mut next_scene = GameScene::new(state, ctx, 13)?;
        next_scene.player.x = 10 * 16 * 0x200;
        next_scene.player.y = 8 * 16 * 0x200;
        state.fade_state = FadeState::Hidden;
        state.textscript_vm.state = TextScriptExecutionState::Running(200, 0);

        state.next_scene = Some(Box::new(next_scene));

        Ok(())
    }

    fn draw_background(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "bkMoon")?;
        let offset = (self.tick % 640) as isize;

        batch.add_rect(((state.canvas_size.0 - 320.0) / 2.0).floor(), 0.0,
                       &Rect::<usize>::new_size(0, 0, 320, 88));

        for x in ((-offset / 2)..(state.canvas_size.0 as isize)).step_by(320) {
            batch.add_rect(x as f32, 88.0,
                           &Rect::<usize>::new_size(0, 88, 320, 35));
        }

        for x in ((-offset % 320)..(state.canvas_size.0 as isize)).step_by(320) {
            batch.add_rect(x as f32, 123.0,
                           &Rect::<usize>::new_size(0, 123, 320, 23));
        }

        for x in ((-offset * 2)..(state.canvas_size.0 as isize)).step_by(320) {
            batch.add_rect(x as f32, 146.0,
                           &Rect::<usize>::new_size(0, 146, 320, 30));
        }

        for x in ((-offset * 4)..(state.canvas_size.0 as isize)).step_by(320) {
            batch.add_rect(x as f32, 176.0,
                           &Rect::<usize>::new_size(0, 176, 320, 64));
        }

        batch.draw(ctx)?;

        Ok(())
    }

    fn draw_text_centered(&self, text: &str, y: f32, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let width = state.font.text_width(text.chars(), &state.constants);
        state.font.draw_text(text.chars(), ((state.canvas_size.0 - width) / 2.0).floor(), y, &state.constants, &mut state.texture_set, ctx)?;

        Ok(())
    }
}

static ENGINE_VERSION: &str = "doukutsu-rs 0.1.0";
// asset copyright for freeware version
static COPYRIGHT_PIXEL: &str = "2004.12  Studio Pixel";
// asset copyright for Nicalis, why they've even replaced © with @?
static COPYRIGHT_NICALIS: &str = "@2011 NICALIS INC.";
static COPYRIGHT_NICALIS_SWITCH: &str = "@2017 NICALIS INC."; // untested?

impl Scene for TitleScene {
    fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if self.tick == 0 {
            state.sound_manager.play_song(24, &state.constants, ctx)?;
            self.title_menu.push_entry(MenuEntry::Active("Load game".to_string()));
            self.title_menu.push_entry(MenuEntry::Active("New game".to_string()));
            self.title_menu.push_entry(MenuEntry::Disabled("Options".to_string()));
            self.title_menu.push_entry(MenuEntry::Disabled("Editor".to_string()));
            self.title_menu.push_entry(MenuEntry::Active("Quit".to_string()));
        }

        self.title_menu.x = ((state.canvas_size.0 - self.title_menu.width as f32) / 2.0).floor() as isize;
        self.title_menu.y = ((state.canvas_size.1 + 70.0 - self.title_menu.height as f32) / 2.0).floor() as isize;

        match self.title_menu.tick(state) {
            MenuSelectionResult::Selected(0, _) => {
                self.start_game(state, ctx);
            }
            MenuSelectionResult::Selected(1, _) => {
                self.start_game(state, ctx);
            }
            _ => {}
        }

        self.tick += 1;

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.draw_background(state, ctx)?;

        {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Title")?;
            batch.add_rect(((state.canvas_size.0 - state.constants.title.logo_rect.width() as f32) / 2.0).floor(),
                           40.0,
                           &state.constants.title.logo_rect);

            batch.draw(ctx)?;
        }

        self.draw_text_centered(ENGINE_VERSION, state.canvas_size.1 - 15.0, state, ctx)?;

        if state.constants.is_cs_plus {
            self.draw_text_centered(COPYRIGHT_NICALIS, state.canvas_size.1 - 30.0, state, ctx)?;
        } else {
            self.draw_text_centered(COPYRIGHT_PIXEL, state.canvas_size.1 - 30.0, state, ctx)?;
        }

        self.title_menu.draw(state, ctx)?;

        Ok(())
    }
}
