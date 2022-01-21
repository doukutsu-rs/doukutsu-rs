use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::menu::MenuEntry;
use crate::menu::{Menu, MenuSelectionResult};
use crate::scene::title_scene::TitleScene;
use crate::shared_game_state::SharedGameState;

use super::settings_menu::SettingsMenu;

#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
#[allow(unused)]
enum CurrentMenu {
    PauseMenu,
    OptionsMenu,
}

pub struct PauseMenu {
    is_paused: bool,
    current_menu: CurrentMenu,
    option_menu: SettingsMenu,
    controller: CombinedMenuController,
    pause_menu: Menu,
    tick: u32,
}

impl PauseMenu {
    pub fn new() -> PauseMenu {
        let main = Menu::new(0, 0, 75, 0);

        PauseMenu {
            is_paused: false,
            current_menu: CurrentMenu::PauseMenu,
            option_menu: SettingsMenu::new(),
            controller: CombinedMenuController::new(),
            pause_menu: main,
            tick: 0,
        }
    }

    pub fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.controller.add(state.settings.create_player1_controller());
        self.controller.add(state.settings.create_player2_controller());

        self.pause_menu.push_entry(MenuEntry::Active("Resume".to_owned()));
        self.pause_menu.push_entry(MenuEntry::Active("Retry".to_owned()));
        self.pause_menu.push_entry(MenuEntry::Active("Options".to_owned()));
        self.pause_menu.push_entry(MenuEntry::Active("Main Menu".to_owned()));
        self.pause_menu.push_entry(MenuEntry::Disabled(" --- ".to_owned()));
        self.pause_menu.push_entry(MenuEntry::Active("Quit".to_owned()));

        self.update_sizes(state);

        self.option_menu.init(state, ctx)?;

        self.controller.update(state, ctx)?;
        self.controller.update_trigger();

        Ok(())
    }

    fn update_sizes(&mut self, state: &SharedGameState) {
        self.pause_menu.update_height();
        self.pause_menu.x = ((state.canvas_size.0 - self.pause_menu.width as f32) / 2.0).floor() as isize;
        self.pause_menu.y = ((state.canvas_size.1 - self.pause_menu.height as f32) / 2.0).floor() as isize;
    }

    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    pub fn is_paused(&mut self) -> bool {
        self.is_paused
    }

    pub fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.update_sizes(state);

        self.controller.update(state, ctx)?;
        self.controller.update_trigger();

        self.is_paused = true;

        match self.current_menu {
            CurrentMenu::PauseMenu => match self.pause_menu.tick(&mut self.controller, state) {
                MenuSelectionResult::Selected(0, _) | MenuSelectionResult::Canceled => {
                    // double tap prevention
                    if self.tick >= 3 {
                        self.tick = 0;
                        self.is_paused = false;
                    }
                }
                MenuSelectionResult::Selected(1, _) => {
                    state.load_or_start_game(ctx)?;
                }
                MenuSelectionResult::Selected(2, _) => {
                    self.current_menu = CurrentMenu::OptionsMenu;
                }
                MenuSelectionResult::Selected(3, _) => {
                    state.next_scene = Some(Box::new(TitleScene::new()));
                }
                MenuSelectionResult::Selected(5, _) => {
                    state.shutdown();
                }
                _ => (),
            },
            CurrentMenu::OptionsMenu => {
                let cm = &mut self.current_menu;
                self.option_menu.tick(
                    &mut || {
                        *cm = CurrentMenu::PauseMenu;
                    },
                    &mut self.controller,
                    state,
                    ctx,
                )?;
            }
        }

        self.tick += 1;

        Ok(())
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if self.is_paused {
            match self.current_menu {
                CurrentMenu::PauseMenu => {
                    self.pause_menu.draw(state, ctx)?;
                }
                CurrentMenu::OptionsMenu => {
                    self.option_menu.draw(state, ctx)?;
                }
            }
        }

        Ok(())
    }
}
