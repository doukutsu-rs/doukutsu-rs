use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics;
use crate::framework::keyboard::ScanCode;
use crate::game::shared_game_state::{MenuCharacter, PlayerCount, SharedGameState};
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::menu::MenuEntry;
use crate::menu::{Menu, MenuSelectionResult};
use crate::scene::title_scene::TitleScene;

use super::coop_menu::PlayerCountMenu;
use super::settings_menu::SettingsMenu;

#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
#[allow(unused)]
enum CurrentMenu {
    PauseMenu,
    CoopMenu,
    SettingsMenu,
    ConfirmMenu,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum PauseMenuEntry {
    Resume,
    Retry,
    AddPlayer2,
    DropPlayer2,
    Settings,
    Title,
    Quit,
}

impl Default for PauseMenuEntry {
    fn default() -> Self {
        PauseMenuEntry::Resume
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ConfirmMenuEntry {
    Empty,
    Yes,
    No,
}

impl Default for ConfirmMenuEntry {
    fn default() -> Self {
        ConfirmMenuEntry::Yes
    }
}

pub struct PauseMenu {
    is_paused: bool,
    current_menu: CurrentMenu,
    settings_menu: SettingsMenu,
    coop_menu: PlayerCountMenu,
    controller: CombinedMenuController,
    pause_menu: Menu<PauseMenuEntry>,
    confirm_menu: Menu<ConfirmMenuEntry>,
    tick: u32,
    should_update_coop_menu: bool,
}

impl PauseMenu {
    pub fn new() -> PauseMenu {
        let main = Menu::new(0, 0, 75, 0);

        PauseMenu {
            is_paused: false,
            current_menu: CurrentMenu::PauseMenu,
            settings_menu: SettingsMenu::new(),
            coop_menu: PlayerCountMenu::new(),
            controller: CombinedMenuController::new(),
            pause_menu: main,
            confirm_menu: Menu::new(0, 0, 75, 0),
            tick: 0,
            should_update_coop_menu: false,
        }
    }

    pub fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.controller.add(state.settings.create_player1_controller());
        self.controller.add(state.settings.create_player2_controller());

        self.pause_menu
            .push_entry(PauseMenuEntry::Resume, MenuEntry::Active(state.loc.t("menus.pause_menu.resume").to_owned()));
        self.pause_menu
            .push_entry(PauseMenuEntry::Retry, MenuEntry::Active(state.loc.t("menus.pause_menu.retry").to_owned()));
        self.pause_menu.push_entry(PauseMenuEntry::AddPlayer2, MenuEntry::Hidden);
        self.pause_menu.push_entry(PauseMenuEntry::DropPlayer2, MenuEntry::Hidden);
        self.pause_menu.push_entry(
            PauseMenuEntry::Settings,
            MenuEntry::Active(state.loc.t("menus.pause_menu.options").to_owned()),
        );
        self.pause_menu
            .push_entry(PauseMenuEntry::Title, MenuEntry::Active(state.loc.t("menus.pause_menu.title").to_owned()));
        self.pause_menu
            .push_entry(PauseMenuEntry::Quit, MenuEntry::Active(state.loc.t("menus.pause_menu.quit").to_owned()));

        self.confirm_menu.push_entry(ConfirmMenuEntry::Empty, MenuEntry::Disabled(String::new()));
        self.confirm_menu.push_entry(ConfirmMenuEntry::Yes, MenuEntry::Active(state.loc.t("common.yes").to_owned()));
        self.confirm_menu.push_entry(ConfirmMenuEntry::No, MenuEntry::Active(state.loc.t("common.no").to_owned()));

        self.confirm_menu.selected = ConfirmMenuEntry::Yes;

        self.update_sizes(state);

        self.settings_menu.init(state, ctx)?;
        self.coop_menu.init(state)?;

        self.controller.update(state, ctx)?;
        self.controller.update_trigger();

        state.menu_character = MenuCharacter::Quote;

        self.update_coop_menu_items(state);

        Ok(())
    }

    fn update_sizes(&mut self, state: &SharedGameState) {
        self.pause_menu.update_width(state);
        self.pause_menu.update_height(state);
        self.pause_menu.x = ((state.canvas_size.0 - self.pause_menu.width as f32) / 2.0).floor() as isize;
        self.pause_menu.y = ((state.canvas_size.1 - self.pause_menu.height as f32) / 2.0).floor() as isize;

        self.confirm_menu.update_width(state);
        self.confirm_menu.update_height(state);
        self.confirm_menu.x = ((state.canvas_size.0 - self.confirm_menu.width as f32) / 2.0).floor() as isize;
        self.confirm_menu.y = ((state.canvas_size.1 - self.confirm_menu.height as f32) / 2.0).floor() as isize;
    }

    fn update_coop_menu_items(&mut self, state: &SharedGameState) {
        if !state.constants.supports_two_player {
            return;
        }

        match state.player_count {
            PlayerCount::One => {
                self.pause_menu.set_entry(
                    PauseMenuEntry::AddPlayer2,
                    MenuEntry::Active(state.loc.t("menus.pause_menu.add_player2").to_owned()),
                );
                self.pause_menu.set_entry(PauseMenuEntry::DropPlayer2, MenuEntry::Hidden);

                if self.pause_menu.selected == PauseMenuEntry::DropPlayer2 {
                    self.pause_menu.selected = PauseMenuEntry::AddPlayer2;
                }
            }
            PlayerCount::Two => {
                self.pause_menu.set_entry(PauseMenuEntry::AddPlayer2, MenuEntry::Hidden);
                self.pause_menu.set_entry(
                    PauseMenuEntry::DropPlayer2,
                    MenuEntry::Active(state.loc.t("menus.pause_menu.drop_player2").to_owned()),
                );

                if self.pause_menu.selected == PauseMenuEntry::AddPlayer2 {
                    self.pause_menu.selected = PauseMenuEntry::DropPlayer2;
                }
            }
        }
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
            state.sound_manager.play_song(0, &state.constants, &state.settings, ctx, false)?;
            state.load_or_start_game(ctx)?;
        }

        if self.should_update_coop_menu {
            self.update_coop_menu_items(state);
            self.should_update_coop_menu = false;
        }

        match self.current_menu {
            CurrentMenu::PauseMenu => match self.pause_menu.tick(&mut self.controller, state) {
                MenuSelectionResult::Selected(PauseMenuEntry::Resume, _) | MenuSelectionResult::Canceled => {
                    // double tap prevention
                    if self.tick >= 3 {
                        self.tick = 0;
                        self.is_paused = false;
                    }
                }
                MenuSelectionResult::Selected(PauseMenuEntry::Retry, _) => {
                    state.stop_noise();
                    state.sound_manager.play_song(0, &state.constants, &state.settings, ctx, false)?;
                    state.load_or_start_game(ctx)?;
                }
                MenuSelectionResult::Selected(PauseMenuEntry::AddPlayer2, _) => {
                    if !state.constants.is_cs_plus {
                        state.player_count = PlayerCount::Two;
                        state.player_count_modified_in_game = true;
                        self.should_update_coop_menu = true;
                    } else {
                        self.current_menu = CurrentMenu::CoopMenu;
                    }
                }
                MenuSelectionResult::Selected(PauseMenuEntry::DropPlayer2, _) => {
                    state.player_count = PlayerCount::One;
                    state.player_count_modified_in_game = true;
                    self.should_update_coop_menu = true;
                }
                MenuSelectionResult::Selected(PauseMenuEntry::Settings, _) => {
                    self.current_menu = CurrentMenu::SettingsMenu;
                }
                MenuSelectionResult::Selected(PauseMenuEntry::Title, _) => {
                    self.confirm_menu.set_entry(
                        ConfirmMenuEntry::Empty,
                        MenuEntry::Disabled(state.loc.t("menus.pause_menu.title_confirm").to_owned()),
                    );
                    self.current_menu = CurrentMenu::ConfirmMenu;
                }
                MenuSelectionResult::Selected(PauseMenuEntry::Quit, _) => {
                    self.confirm_menu.set_entry(
                        ConfirmMenuEntry::Empty,
                        MenuEntry::Disabled(state.loc.t("menus.pause_menu.quit_confirm").to_owned()),
                    );
                    self.current_menu = CurrentMenu::ConfirmMenu;
                }
                _ => (),
            },
            CurrentMenu::CoopMenu => {
                let cm = &mut self.current_menu;
                let should_update = &mut self.should_update_coop_menu;

                self.coop_menu.tick(
                    &mut || {
                        *cm = CurrentMenu::PauseMenu;
                        *should_update = true;
                    },
                    &mut self.controller,
                    state,
                    ctx,
                )?;
            }
            CurrentMenu::SettingsMenu => {
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
                MenuSelectionResult::Selected(ConfirmMenuEntry::Yes, _) => match self.pause_menu.selected {
                    PauseMenuEntry::Title => {
                        state.stop_noise();
                        state.textscript_vm.flags.set_cutscene_skip(false);
                        state.next_scene = Some(Box::new(TitleScene::new()));
                    }
                    PauseMenuEntry::Quit => {
                        ctx.shutdown();
                    }
                    _ => (),
                },
                MenuSelectionResult::Selected(ConfirmMenuEntry::No, _) | MenuSelectionResult::Canceled => {
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
                CurrentMenu::CoopMenu => {
                    self.coop_menu.draw(state, ctx)?;
                }
                CurrentMenu::SettingsMenu => {
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
