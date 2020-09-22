use crate::common::Rect;
use crate::ggez::{Context, GameResult, graphics};
use crate::ggez::graphics::Color;
use crate::menu::{Menu, MenuEntry, MenuSelectionResult};
use crate::scene::Scene;
use crate::shared_game_state::SharedGameState;

#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
enum CurrentMenu {
    MainMenu,
    OptionMenu,
    StartGame,
    LoadGame,
}

pub struct TitleScene {
    tick: usize,
    current_menu: CurrentMenu,
    main_menu: Menu,
    option_menu: Menu,
}

impl TitleScene {
    pub fn new() -> Self {
        Self {
            tick: 0,
            current_menu: CurrentMenu::MainMenu,
            main_menu: Menu::new(0, 0, 100, 5 * 14 + 6),
            option_menu: Menu::new(0, 0, 140, 3 * 14 + 6),
        }
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
            self.main_menu.push_entry(MenuEntry::Active("New game".to_string()));
            self.main_menu.push_entry(MenuEntry::Active("Load game".to_string()));
            self.main_menu.push_entry(MenuEntry::Active("Options".to_string()));
            self.main_menu.push_entry(MenuEntry::Disabled("Editor".to_string()));
            self.main_menu.push_entry(MenuEntry::Active("Quit".to_string()));

            self.option_menu.push_entry(MenuEntry::Toggle("Test toggle".to_string(), false));
            self.option_menu.push_entry(MenuEntry::Toggle("2x Speed hack".to_string(), false));
            self.option_menu.push_entry(MenuEntry::Active("Back".to_string()));
        }

        self.main_menu.x = ((state.canvas_size.0 - self.main_menu.width as f32) / 2.0).floor() as isize;
        self.main_menu.y = ((state.canvas_size.1 + 70.0 - self.main_menu.height as f32) / 2.0).floor() as isize;

        self.option_menu.x = ((state.canvas_size.0 - self.option_menu.width as f32) / 2.0).floor() as isize;
        self.option_menu.y = ((state.canvas_size.1 + 70.0 - self.option_menu.height as f32) / 2.0).floor() as isize;

        match self.current_menu {
            CurrentMenu::MainMenu => {
                match self.main_menu.tick(state) {
                    MenuSelectionResult::Selected(0, _) => {
                        state.reset();
                        state.sound_manager.play_song(0, &state.constants, ctx)?;
                        self.tick = 1;
                        self.current_menu = CurrentMenu::StartGame;
                    }
                    MenuSelectionResult::Selected(1, _) => {
                        state.sound_manager.play_song(0, &state.constants, ctx)?;
                        self.tick = 1;
                        self.current_menu = CurrentMenu::LoadGame;
                    }
                    MenuSelectionResult::Selected(2, _) => {
                        self.current_menu = CurrentMenu::OptionMenu;
                    }
                    MenuSelectionResult::Selected(4, _) => {
                        state.shutdown();
                    }
                    _ => {}
                }
            }
            CurrentMenu::OptionMenu => {
                match self.option_menu.tick(state) {
                    MenuSelectionResult::Selected(0, toggle) => {
                        if let MenuEntry::Toggle(_, value) = toggle {
                            *value = !(*value);
                        }
                    }
                    MenuSelectionResult::Selected(1, toggle) => {
                        if let MenuEntry::Toggle(_, value) = toggle {
                            *value = !(*value);
                            state.set_speed_hack(*value);
                        }
                    }
                    MenuSelectionResult::Selected(2, _) | MenuSelectionResult::Canceled => {
                        self.current_menu = CurrentMenu::MainMenu;
                    }
                    _ => {}
                }
            }
            CurrentMenu::StartGame => {
                if self.tick == 30 {
                    state.start_new_game(ctx)?;
                }
            }
            CurrentMenu::LoadGame => {
                if self.tick == 30 {
                    state.load_or_start_game(ctx)?;
                }
            }
        }

        self.tick += 1;

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if self.current_menu == CurrentMenu::StartGame || self.current_menu == CurrentMenu::LoadGame {
            graphics::clear(ctx, Color::from_rgb(0, 0, 0));
            return Ok(());
        }

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

        match self.current_menu {
            CurrentMenu::MainMenu => { self.main_menu.draw(state, ctx)?; }
            CurrentMenu::OptionMenu => { self.option_menu.draw(state, ctx)?; }
            _ => {}
        }

        Ok(())
    }
}
