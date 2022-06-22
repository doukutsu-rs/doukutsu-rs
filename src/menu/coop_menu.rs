use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::menu::MenuEntry;
use crate::menu::{Menu, MenuSelectionResult};
use crate::profile::GameProfile;
use crate::shared_game_state::{PlayerCount, SharedGameState};

pub enum CurrentMenu {
    CoopMenu,
    PlayerSkin,
}

pub struct PlayerCountMenu {
    current_menu: CurrentMenu,
    coop_menu: Menu,
    skin_menu: Menu,
}

impl PlayerCountMenu {
    pub fn new() -> PlayerCountMenu {
        PlayerCountMenu {
            coop_menu: Menu::new(0, 0, 130, 0),
            skin_menu: Menu::new(0, 0, 130, 0),
            current_menu: CurrentMenu::CoopMenu,
        }
    }
    pub fn init(&mut self, state: &mut SharedGameState, ctx: &Context) -> GameResult {
        self.coop_menu = Menu::new(0, 0, 130, 0);
        self.skin_menu = Menu::new(0, 0, 130, 0);

        self.coop_menu.push_entry(MenuEntry::Disabled(state.t("menus.coop_menu.title")));
        self.coop_menu.push_entry(MenuEntry::Active(state.t("menus.coop_menu.one")));
        self.coop_menu.push_entry(MenuEntry::Active(state.t("menus.coop_menu.two")));
        self.coop_menu.push_entry(MenuEntry::Active(state.t("common.back")));
    
        self.coop_menu.selected = 1;
    
        self.skin_menu.push_entry(MenuEntry::Disabled(state.t("menus.skin_menu.title")));
        self.skin_menu.push_entry(MenuEntry::PlayerSkin);
        self.skin_menu.push_entry(MenuEntry::Active(state.t("menus.main_menu.start")));
        self.skin_menu.push_entry(MenuEntry::Active(state.t("common.back")));
        self.skin_menu.selected = 1;

        self.update_sizes(state);

        Ok(())
    }

    fn update_sizes(&mut self, state: &SharedGameState) {
        self.coop_menu.update_width(state);
        self.coop_menu.update_height();
        self.coop_menu.x = ((state.canvas_size.0 - self.coop_menu.width as f32) / 2.3).floor() as isize;
        self.coop_menu.y =
            30 + ((state.canvas_size.1 - self.coop_menu.height as f32) / 2.0).floor() as isize;

        self.skin_menu.update_width(state);
        self.skin_menu.update_height();
        self.skin_menu.x = ((state.canvas_size.0 - self.coop_menu.width as f32) / 2.3).floor() as isize;
        self.skin_menu.y =
            30 + ((state.canvas_size.1 - self.coop_menu.height as f32) / 2.0).floor() as isize;
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
                MenuSelectionResult::Selected(3, _) | MenuSelectionResult::Canceled => exit_action(),
                MenuSelectionResult::Selected(1, _) => {
                    state.player_count = PlayerCount::One;
                    state.reload_resources(ctx)?;
                    state.load_or_start_game(ctx)?;  
                } 
                MenuSelectionResult::Selected(2, _) => {
                    if state.constants.is_cs_plus {
                        self.current_menu = CurrentMenu::PlayerSkin;
                    } else {
                        state.player_count = PlayerCount::Two;
                        state.reload_resources(ctx)?;
                        state.load_or_start_game(ctx)?;  
                    }
                }
                _ => (),
            },
            CurrentMenu::PlayerSkin => match self.skin_menu.tick(controller, state) {
                MenuSelectionResult::Selected(3, _) | MenuSelectionResult::Canceled => {
                    self.current_menu = CurrentMenu::CoopMenu;
                }
                MenuSelectionResult::Selected(1, _)  =>{
                    state.player2_skin += 2;
                }
                MenuSelectionResult::Selected(2, _)  =>{
                    state.player_count = PlayerCount::Two;
                    state.reload_resources(ctx)?;
                    state.load_or_start_game(ctx)?;
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
}