use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics;
use crate::framework::keyboard::ScanCode;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::menu::MenuEntry;
use crate::menu::{Menu, MenuSelectionResult};
use crate::scene::title_scene::TitleScene;
use crate::shared_game_state::{MenuCharacter, SharedGameState};

use super::settings_menu::SettingsMenu;

#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
#[allow(unused)]
enum CurrentMenu {
    PauseMenu,
    OptionsMenu,
    ConfirmMenu,
}

pub struct PauseMenu {
    is_paused: bool,
    current_menu: CurrentMenu,
    settings_menu: SettingsMenu,
    controller: CombinedMenuController,
    pause_menu: Menu,
    confirm_menu: Menu,
    tick: u32,
}

impl PauseMenu {
    pub fn new() -> PauseMenu {
        let main = Menu::new(0, 0, 75, 0);

        PauseMenu {
            is_paused: false,
            current_menu: CurrentMenu::PauseMenu,
            settings_menu: SettingsMenu::new(),
            controller: CombinedMenuController::new(),
            pause_menu: main,
            confirm_menu: Menu::new(0, 0, 75, 0),
            tick: 0,
        }
    }

    pub fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.controller.add(state.settings.create_player1_controller());
        self.controller.add(state.settings.create_player2_controller());

        self.pause_menu.push_entry(MenuEntry::Active(state.t("menus.pause_menu.resume")));
        self.pause_menu.push_entry(MenuEntry::Active(state.t("menus.pause_menu.retry")));
        self.pause_menu.push_entry(MenuEntry::Active(state.t("menus.pause_menu.options")));
        self.pause_menu.push_entry(MenuEntry::Active(state.t("menus.pause_menu.title")));
        self.pause_menu.push_entry(MenuEntry::Active(state.t("menus.pause_menu.quit")));

        self.confirm_menu.push_entry(MenuEntry::Disabled("".to_owned()));
        self.confirm_menu.push_entry(MenuEntry::Active(state.t("common.yes")));
        self.confirm_menu.push_entry(MenuEntry::Active(state.t("common.no")));

        self.confirm_menu.selected = 1;

        self.update_sizes(state);

        self.settings_menu.init(state, ctx)?;

        self.controller.update(state, ctx)?;
        self.controller.update_trigger();

        state.menu_character = MenuCharacter::Quote;

        Ok(())
    }

    fn update_sizes(&mut self, state: &SharedGameState) {
        self.pause_menu.update_width(state);
        self.pause_menu.update_height();
        self.pause_menu.x = ((state.canvas_size.0 - self.pause_menu.width as f32) / 2.0).floor() as isize;
        self.pause_menu.y = ((state.canvas_size.1 - self.pause_menu.height as f32) / 2.0).floor() as isize;

        self.confirm_menu.update_width(state);
        self.confirm_menu.update_height();
        self.confirm_menu.x = ((state.canvas_size.0 - self.confirm_menu.width as f32) / 2.0).floor() as isize;
        self.confirm_menu.y = ((state.canvas_size.1 - self.confirm_menu.height as f32) / 2.0).floor() as isize;
    }

    pub fn pause(&mut self, state: &mut SharedGameState) {
        self.is_paused = true;
        state.sound_manager.play_sfx(5);
    }

    pub fn is_paused(&mut self) -> bool {
        self.is_paused
    }

    pub fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.update_sizes(state);

        self.controller.update(state, ctx)?;
        self.controller.update_trigger();

        // Shortcut for quick restart
        if ctx.keyboard_context.is_key_pressed(ScanCode::F2) {
            state.stop_noise();
            state.sound_manager.play_song(0, &state.constants, &state.settings, ctx)?;
            state.load_or_start_game(ctx)?;
        }

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
                    state.stop_noise();
                    state.sound_manager.play_song(0, &state.constants, &state.settings, ctx)?;
                    state.load_or_start_game(ctx)?;
                }
                MenuSelectionResult::Selected(2, _) => {
                    self.current_menu = CurrentMenu::OptionsMenu;
                }
                MenuSelectionResult::Selected(3, _) => {
                    self.confirm_menu.entries[0] = MenuEntry::Disabled(state.t("menus.pause_menu.title_confirm"));
                    self.current_menu = CurrentMenu::ConfirmMenu;
                }
                MenuSelectionResult::Selected(4, _) => {
                    self.confirm_menu.entries[0] = MenuEntry::Disabled(state.t("menus.pause_menu.quit_confirm"));
                    self.current_menu = CurrentMenu::ConfirmMenu;
                }
                _ => (),
            },
            CurrentMenu::OptionsMenu => {
                let cm = &mut self.current_menu;
                self.settings_menu.tick(
                    &mut || {
                        *cm = CurrentMenu::PauseMenu;
                    },
                    &mut self.controller,
                    state,
                    ctx,
                )?;
            }
            CurrentMenu::ConfirmMenu => match self.confirm_menu.tick(&mut self.controller, state) {
                MenuSelectionResult::Selected(1, _) => match self.pause_menu.selected {
                    3 => {
                        state.stop_noise();
                        state.textscript_vm.flags.set_cutscene_skip(false);
                        state.next_scene = Some(Box::new(TitleScene::new()));
                    }
                    4 => {
                        state.shutdown();
                    }
                    _ => (),
                },
                MenuSelectionResult::Selected(2, _) | MenuSelectionResult::Canceled => {
                    self.current_menu = CurrentMenu::PauseMenu;
                }
                _ => (),
            },
        }

        self.tick += 1;

        Ok(())
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if self.is_paused {
            let clip_y = ((self.tick as f32 + state.frame_time as f32 - 2.0) * state.scale * 10.0)
                .clamp(0.0, state.screen_size.1) as isize;
            let clip_rect = crate::common::Rect::new_size(
                0,
                (state.screen_size.1 / 2.0) as isize - clip_y,
                state.screen_size.0 as isize,
                clip_y * 2,
            );

            match self.current_menu {
                CurrentMenu::PauseMenu => {
                    graphics::set_clip_rect(ctx, Some(clip_rect))?;
                    self.pause_menu.draw(state, ctx)?;
                    graphics::set_clip_rect(ctx, None)?;
                }
                CurrentMenu::OptionsMenu => {
                    self.settings_menu.draw(state, ctx)?;
                }
                CurrentMenu::ConfirmMenu => {
                    graphics::set_clip_rect(ctx, Some(clip_rect))?;
                    self.confirm_menu.draw(state, ctx)?;
                    graphics::set_clip_rect(ctx, None)?;
                }
            }
        }

        Ok(())
    }
}
