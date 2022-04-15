use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::menu::MenuEntry;
use crate::menu::{Menu, MenuSelectionResult};
use crate::profile::GameProfile;
use crate::shared_game_state::{GameDifficulty, SharedGameState};

#[derive(Clone, Copy)]
pub struct MenuSaveInfo {
    pub current_map: u32,
    pub max_life: u16,
    pub life: u16,
    pub weapon_count: usize,
    pub weapon_id: [u32; 8],
    pub difficulty: u8,
}

impl Default for MenuSaveInfo {
    fn default() -> Self {
        MenuSaveInfo { current_map: 0, max_life: 0, life: 0, weapon_count: 0, weapon_id: [0; 8], difficulty: 0 }
    }
}
pub enum CurrentMenu {
    SaveMenu,
    DifficultyMenu,
    DeleteConfirm,
    LoadConfirm,
}
pub struct SaveSelectMenu {
    pub saves: [MenuSaveInfo; 3],
    current_menu: CurrentMenu,
    save_menu: Menu,
    save_detailed: Menu,
    difficulty_menu: Menu,
    delete_confirm: Menu,
    load_confirm: Menu,
    skip_difficulty_menu: bool,
}

impl SaveSelectMenu {
    pub fn new() -> SaveSelectMenu {
        SaveSelectMenu {
            saves: [MenuSaveInfo::default(); 3],
            current_menu: CurrentMenu::SaveMenu,
            save_menu: Menu::new(0, 0, 230, 0),
            save_detailed: Menu::new(0, 0, 230, 0),
            difficulty_menu: Menu::new(0, 0, 130, 0),
            delete_confirm: Menu::new(0, 0, 75, 0),
            load_confirm: Menu::new(0, 0, 75, 0),
            skip_difficulty_menu: false,
        }
    }

    pub fn init(&mut self, state: &mut SharedGameState, ctx: &Context) -> GameResult {
        self.save_menu = Menu::new(0, 0, 230, 0);
        self.save_detailed = Menu::new(0, 0, 230, 0);
        self.difficulty_menu = Menu::new(0, 0, 130, 0);
        self.delete_confirm = Menu::new(0, 0, 75, 0);
        self.load_confirm = Menu::new(0, 0, 75, 0);
        self.skip_difficulty_menu = false;

        for (iter, save) in self.saves.iter_mut().enumerate() {
            if let Ok(data) = filesystem::user_open(ctx, state.get_save_filename(iter + 1).unwrap_or("".to_string())) {
                let loaded_save = GameProfile::load_from_save(data)?;

                save.current_map = loaded_save.current_map;
                save.max_life = loaded_save.max_life;
                save.life = loaded_save.life;
                save.weapon_count = loaded_save.weapon_data.iter().filter(|weapon| weapon.weapon_id != 0).count();
                save.weapon_id = loaded_save.weapon_data.map(|weapon| weapon.weapon_id);
                save.difficulty = loaded_save.difficulty;

                self.save_menu.push_entry(MenuEntry::SaveData(*save));
            } else {
                self.save_menu.push_entry(MenuEntry::NewSave);
            }
        }

        self.save_menu.push_entry(MenuEntry::Active(state.t("common.back")));

        self.difficulty_menu.push_entry(MenuEntry::Disabled(state.t("menus.difficulty_menu.title")));
        self.difficulty_menu.push_entry(MenuEntry::Active(state.t("menus.difficulty_menu.easy")));
        self.difficulty_menu.push_entry(MenuEntry::Active(state.t("menus.difficulty_menu.normal")));
        self.difficulty_menu.push_entry(MenuEntry::Active(state.t("menus.difficulty_menu.hard")));
        self.difficulty_menu.push_entry(MenuEntry::Active(state.t("common.back")));

        self.difficulty_menu.selected = 2;

        self.delete_confirm.push_entry(MenuEntry::Disabled(state.t("menus.save_menu.delete_confirm")));
        self.delete_confirm.push_entry(MenuEntry::Active(state.t("common.yes")));
        self.delete_confirm.push_entry(MenuEntry::Active(state.t("common.no")));

        self.delete_confirm.selected = 2;

        self.load_confirm.push_entry(MenuEntry::Active(state.t("menus.main_menu.start")));
        self.load_confirm.push_entry(MenuEntry::Active(state.t("menus.save_menu.delete_confirm")));
        self.load_confirm.push_entry(MenuEntry::Active(state.t("common.back")));

        self.save_detailed.draw_cursor = false;
        if let MenuEntry::SaveData(save) = self.save_menu.entries[0] {
            self.save_detailed.push_entry(MenuEntry::SaveDataSingle(save));
        }

        self.update_sizes(state);

        Ok(())
    }

    pub fn set_skip_difficulty_menu(&mut self, skip: bool) {
        self.skip_difficulty_menu = skip;
    }

    fn update_sizes(&mut self, state: &SharedGameState) {
        self.save_menu.update_width(state);
        self.save_menu.update_height();
        self.save_menu.x = ((state.canvas_size.0 - self.save_menu.width as f32) / 2.0).floor() as isize;
        self.save_menu.y = ((state.canvas_size.1 - self.save_menu.height as f32) / 2.0).floor() as isize;

        self.difficulty_menu.update_width(state);
        self.difficulty_menu.update_height();
        self.difficulty_menu.x = ((state.canvas_size.0 - self.difficulty_menu.width as f32) / 2.0).floor() as isize;
        self.difficulty_menu.y =
            30 + ((state.canvas_size.1 - self.difficulty_menu.height as f32) / 2.0).floor() as isize;

        self.delete_confirm.update_width(state);
        self.delete_confirm.update_height();
        self.delete_confirm.x = ((state.canvas_size.0 - self.delete_confirm.width as f32) / 2.0).floor() as isize;
        self.delete_confirm.y = 30 + ((state.canvas_size.1 - self.delete_confirm.height as f32) / 2.0).floor() as isize;

        self.load_confirm.update_width(state);
        self.load_confirm.update_height();
        self.load_confirm.x = ((state.canvas_size.0 - self.load_confirm.width as f32) / 2.0).floor() as isize;
        self.load_confirm.y = 30 + ((state.canvas_size.1 - self.load_confirm.height as f32) / 2.0).floor() as isize;

        self.save_detailed.update_width(state);
        self.save_detailed.update_height();
        self.save_detailed.x = ((state.canvas_size.0 - self.save_detailed.width as f32) / 2.0).floor() as isize;
        self.save_detailed.y = -40 + ((state.canvas_size.1 - self.save_detailed.height as f32) / 2.0).floor() as isize;
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
            CurrentMenu::SaveMenu => match self.save_menu.tick(controller, state) {
                MenuSelectionResult::Selected(3, _) | MenuSelectionResult::Canceled => exit_action(),
                MenuSelectionResult::Selected(slot, _) => {
                    state.save_slot = slot + 1;

                    if let Ok(_) =
                        filesystem::user_open(ctx, state.get_save_filename(state.save_slot).unwrap_or("".to_string()))
                    {
                        if let MenuEntry::SaveData(save) = self.save_menu.entries[slot] {
                            self.save_detailed.entries.clear();
                            self.save_detailed.push_entry(MenuEntry::SaveDataSingle(save));
                        }
                        self.current_menu = CurrentMenu::LoadConfirm;
                        self.load_confirm.selected = 0;
                    } else if self.skip_difficulty_menu {
                        state.reload_resources(ctx)?;
                        state.load_or_start_game(ctx)?;
                    } else {
                        self.difficulty_menu.selected = 2;
                        self.current_menu = CurrentMenu::DifficultyMenu;
                    }
                }
                _ => (),
            },
            CurrentMenu::DifficultyMenu => match self.difficulty_menu.tick(controller, state) {
                MenuSelectionResult::Selected(4, _) | MenuSelectionResult::Canceled => {
                    self.current_menu = CurrentMenu::SaveMenu;
                }
                MenuSelectionResult::Selected(1, _) => {
                    state.difficulty = GameDifficulty::Easy;
                    state.reload_resources(ctx)?;
                    state.load_or_start_game(ctx)?;
                }
                MenuSelectionResult::Selected(2, _) => {
                    state.difficulty = GameDifficulty::Normal;
                    state.reload_resources(ctx)?;
                    state.load_or_start_game(ctx)?;
                }
                MenuSelectionResult::Selected(3, _) => {
                    state.difficulty = GameDifficulty::Hard;
                    state.reload_resources(ctx)?;
                    state.load_or_start_game(ctx)?;
                }
                _ => (),
            },
            CurrentMenu::DeleteConfirm => match self.delete_confirm.tick(controller, state) {
                MenuSelectionResult::Selected(1, _) => {
                    state.sound_manager.play_sfx(17); // Player Death sfx
                    filesystem::user_delete(
                        ctx,
                        state.get_save_filename(self.save_menu.selected + 1).unwrap_or("".to_string()),
                    )?;
                    self.save_menu.entries[self.save_menu.selected] = MenuEntry::NewSave;
                    self.current_menu = CurrentMenu::SaveMenu;
                }
                MenuSelectionResult::Selected(2, _) | MenuSelectionResult::Canceled => {
                    self.current_menu = CurrentMenu::LoadConfirm;
                    self.load_confirm.selected = 0;
                }
                _ => (),
            },
            CurrentMenu::LoadConfirm => match self.load_confirm.tick(controller, state) {
                MenuSelectionResult::Selected(0, _) => {
                    state.reload_resources(ctx)?;
                    state.load_or_start_game(ctx)?;
                }
                MenuSelectionResult::Selected(1, _) => {
                    self.current_menu = CurrentMenu::DeleteConfirm;
                    self.delete_confirm.selected = 2;
                }
                MenuSelectionResult::Selected(2, _) | MenuSelectionResult::Canceled => {
                    self.current_menu = CurrentMenu::SaveMenu;
                }
                _ => (),
            },
        }

        Ok(())
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        match self.current_menu {
            CurrentMenu::SaveMenu => {
                self.save_menu.draw(state, ctx)?;
            }
            CurrentMenu::DifficultyMenu => {
                self.difficulty_menu.draw(state, ctx)?;
            }
            CurrentMenu::DeleteConfirm => {
                self.save_detailed.draw(state, ctx)?;
                self.delete_confirm.draw(state, ctx)?;
            }
            CurrentMenu::LoadConfirm => {
                self.save_detailed.draw(state, ctx)?;
                self.load_confirm.draw(state, ctx)?;
            }
        }
        Ok(())
    }
}
