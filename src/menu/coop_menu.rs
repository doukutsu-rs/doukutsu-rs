use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::game::shared_game_state::{PlayerCount, SharedGameState};
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::menu::MenuEntry;
use crate::menu::{Menu, MenuSelectionResult};

pub enum CurrentMenu {
    CoopMenu,
    PlayerSkin,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CoopMenuEntry {
    Title,
    One,
    Two,
    Back,
}

impl Default for CoopMenuEntry {
    fn default() -> Self {
        CoopMenuEntry::One
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SkinMenuEntry {
    Title,
    Skin,
    Start,
    Add,
    Back,
}

impl Default for SkinMenuEntry {
    fn default() -> Self {
        SkinMenuEntry::Skin
    }
}

pub struct PlayerCountMenu {
    current_menu: CurrentMenu,
    coop_menu: Menu<CoopMenuEntry>,
    skin_menu: Menu<SkinMenuEntry>,
    pub on_title: bool,
}

impl PlayerCountMenu {
    pub fn new() -> PlayerCountMenu {
        PlayerCountMenu {
            coop_menu: Menu::new(0, 0, 130, 0),
            skin_menu: Menu::new(0, 0, 130, 0),
            current_menu: CurrentMenu::CoopMenu,
            on_title: false,
        }
    }
    pub fn init(&mut self, state: &mut SharedGameState) -> GameResult {
        self.coop_menu = Menu::new(0, 0, 130, 0);
        self.skin_menu = Menu::new(0, 0, 130, 0);

        self.coop_menu
            .push_entry(CoopMenuEntry::Title, MenuEntry::Disabled(state.loc.t("menus.coop_menu.title").to_owned()));
        self.coop_menu.push_entry(CoopMenuEntry::One, MenuEntry::Active(state.loc.t("menus.coop_menu.one").to_owned()));
        self.coop_menu.push_entry(CoopMenuEntry::Two, MenuEntry::Active(state.loc.t("menus.coop_menu.two").to_owned()));
        self.coop_menu.push_entry(CoopMenuEntry::Back, MenuEntry::Active(state.loc.t("common.back").to_owned()));

        self.coop_menu.selected = CoopMenuEntry::One;

        self.skin_menu
            .push_entry(SkinMenuEntry::Title, MenuEntry::Disabled(state.loc.t("menus.skin_menu.title").to_owned()));
        self.skin_menu.push_entry(SkinMenuEntry::Skin, MenuEntry::PlayerSkin);

        if self.on_title {
            self.skin_menu
                .push_entry(SkinMenuEntry::Start, MenuEntry::Active(state.loc.t("menus.main_menu.start").to_owned()));
        } else {
            self.skin_menu.push_entry(
                SkinMenuEntry::Add,
                MenuEntry::Active(state.loc.t("menus.pause_menu.add_player2").to_owned()),
            );
        }

        self.skin_menu.push_entry(SkinMenuEntry::Back, MenuEntry::Active(state.loc.t("common.back").to_owned()));

        self.skin_menu.selected = SkinMenuEntry::Skin;

        if !self.on_title && state.constants.is_cs_plus {
            self.current_menu = CurrentMenu::PlayerSkin;
        }

        self.update_sizes(state);

        Ok(())
    }

    fn update_sizes(&mut self, state: &SharedGameState) {
        self.coop_menu.update_width(state);
        self.coop_menu.update_height(state);
        self.coop_menu.x = ((state.canvas_size.0 - self.coop_menu.width as f32) / 2.0).floor() as isize;
        self.coop_menu.y = 30 + ((state.canvas_size.1 - self.coop_menu.height as f32) / 2.0).floor() as isize;

        self.skin_menu.update_width(state);
        self.skin_menu.update_height(state);
        self.skin_menu.x = ((state.canvas_size.0 - self.coop_menu.width as f32) / 2.0).floor() as isize;
        self.skin_menu.y = 30 + ((state.canvas_size.1 - self.coop_menu.height as f32) / 2.0).floor() as isize;
    }

    pub fn tick(
        &mut self,
        exit_action: &mut dyn FnMut(),
        controller: &mut CombinedMenuController,
        state: &mut SharedGameState,
        ctx: &mut Context,
    ) -> GameResult {
        self.update_sizes(state);
        match self.current_menu {
            CurrentMenu::CoopMenu => match self.coop_menu.tick(controller, state) {
                MenuSelectionResult::Selected(CoopMenuEntry::Back, _) | MenuSelectionResult::Canceled => exit_action(),
                MenuSelectionResult::Selected(CoopMenuEntry::One, _) => {
                    self.start_game(PlayerCount::One, state, ctx)?;
                }
                MenuSelectionResult::Selected(CoopMenuEntry::Two, _) => {
                    if state.constants.is_cs_plus {
                        self.current_menu = CurrentMenu::PlayerSkin;
                    } else {
                        self.start_game(PlayerCount::Two, state, ctx)?;
                    }
                }
                _ => (),
            },
            CurrentMenu::PlayerSkin => match self.skin_menu.tick(controller, state) {
                MenuSelectionResult::Selected(SkinMenuEntry::Back, _) | MenuSelectionResult::Canceled => {
                    if self.on_title {
                        self.current_menu = CurrentMenu::CoopMenu;
                    } else {
                        exit_action();
                    }
                }
                MenuSelectionResult::Selected(SkinMenuEntry::Skin, _) => {
                    state.player2_skin_location.offset += 2;

                    let current_skin_spritesheet_name =
                        state.constants.player_skin_paths[state.player2_skin_location.texture_index as usize].as_str();

                    if let Some(tex_size) = state.constants.tex_sizes.get(current_skin_spritesheet_name) {
                        // TODO: should probably have a way to figure out the height from the spritesheet ahead of time

                        if state.player2_skin_location.offset * 2 * 16 >= tex_size.1 {
                            state.player2_skin_location.offset = 0;

                            if (state.player2_skin_location.texture_index as usize)
                                == state.constants.player_skin_paths.len() - 1
                            {
                                state.player2_skin_location.texture_index = 0;
                            } else {
                                state.player2_skin_location.texture_index += 1;
                            }
                        }
                    }
                }
                MenuSelectionResult::Selected(SkinMenuEntry::Start, _) => {
                    state.player_count = PlayerCount::Two;
                    state.reload_resources(ctx)?;
                    state.load_or_start_game(ctx)?;
                }
                MenuSelectionResult::Selected(SkinMenuEntry::Add, _) => {
                    state.player_count = PlayerCount::Two;
                    state.player_count_modified_in_game = true;
                    exit_action();
                }
                _ => (),
            },
        }
        Ok(())
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        match self.current_menu {
            CurrentMenu::CoopMenu => {
                self.coop_menu.draw(state, ctx)?;
            }
            CurrentMenu::PlayerSkin => {
                self.skin_menu.draw(state, ctx)?;
            }
        }
        Ok(())
    }

    fn start_game(&mut self, player_count: PlayerCount, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        state.player_count = player_count;
        state.reload_resources(ctx)?;
        state.load_or_start_game(ctx)?;
        Ok(())
    }
}
