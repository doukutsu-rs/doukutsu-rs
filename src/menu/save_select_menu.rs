use pelite::pe::imports::Import;

use crate::common::Version;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::game::profile::{GameProfile, SaveContainer, SaveFormat, SaveParams, SaveSlot, SWITCH_VER_1_2, SWITCH_VER_1_3};
use crate::game::shared_game_state::{GameDifficulty, SharedGameState};
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::menu::coop_menu::PlayerCountMenu;
use crate::menu::MenuEntry;
use crate::menu::{Menu, MenuSelectionResult};
use crate::util::file_picker::{open_file_picker, FilePickerParams};

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

#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
#[allow(unused)]
pub enum CurrentMenu {
    SaveMenu,
    DifficultyMenu,
    PlayerCountMenu,
    DeleteConfirm,
    LoadConfirm,
    ImportExport,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SaveMenuEntry {
    Load(usize),
    New(usize),
    ImportExport,
    Back,
}

impl Default for SaveMenuEntry {
    fn default() -> Self {
        SaveMenuEntry::Load(0)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DifficultyMenuEntry {
    Title,
    Difficulty(GameDifficulty),
    Back,
}

impl Default for DifficultyMenuEntry {
    fn default() -> Self {
        DifficultyMenuEntry::Difficulty(GameDifficulty::Normal)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DeleteConfirmMenuEntry {
    Title,
    Yes,
    No,
}

impl Default for DeleteConfirmMenuEntry {
    fn default() -> Self {
        DeleteConfirmMenuEntry::No
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LoadConfirmMenuEntry {
    Start,
    Delete,
    Back,
}

impl Default for LoadConfirmMenuEntry {
    fn default() -> Self {
        LoadConfirmMenuEntry::Start
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ImportExportMenuEntry {
    Format,
    Import,
    Export,
    Back,
}

#[derive(Clone, Copy)]
enum ImportExportLocation {
    Filesystem,
    // TODO: add some Switch emulators
}

impl Default for ImportExportLocation {
    fn default() -> Self {
        Self::Filesystem
    }
}

#[derive(Clone)]
pub struct MenuExportInfo {
    pub location: ImportExportLocation,
    pub format: Option<SaveFormat>,
    pub picker_params: FilePickerParams,
    pub save_params: SaveParams,
}

impl Default for MenuExportInfo {
    fn default() -> Self {
        Self {
            location: ImportExportLocation::Filesystem,
            format: None,
            picker_params: FilePickerParams::new(),
            save_params: SaveParams::default()
        }
    }
}

impl MenuExportInfo {
    fn fpicker_from_format(state: &SharedGameState, mut format: Option<SaveFormat>, is_export: bool) -> FilePickerParams {
        if is_export && format.is_none() {
            format = Some(state.settings.save_format);
        }

        let mut params = match format {
            Some(SaveFormat::Freeware) => {
                FilePickerParams::new()
                    .pick_dirs(true)
            }
            Some(SaveFormat::Plus) | Some(SaveFormat::Switch(_)) => {
                let filename = SaveContainer::get_save_filename(format.unwrap(), None).split_off(1);

                FilePickerParams::new()
                    .file_name(Some(filename))
                    .filter(state.loc.ts("menus.save_manage_menu.file_filters.plus"), vec![
                        "dat".to_owned()
                    ])
            }
            Some(SaveFormat::Generic) => {
                let filename = SaveContainer::get_save_filename(format.unwrap(), None).split_off(1);

                FilePickerParams::new()
                    .file_name(Some(filename))
                    .filter(state.loc.ts("menus.save_manage_menu.file_filters.generic"), vec![
                        "json".to_owned()
                    ])
            }
            None => FilePickerParams::new()
        };

        if is_export && format != Some(SaveFormat::Freeware) {
            params = params.save(true);
        }

        if let Some(fs_container) = &state.fs_container {
            params = params.starting_dir(Some(fs_container.user_path.clone()));
        }

        params
    }
}


impl ImportExportMenuEntry {
    fn format_from_value(val: usize) -> Option<SaveFormat> {
        match val {
            0 => None, // Auto
            1 => Some(SaveFormat::Freeware),
            2 => Some(SaveFormat::Plus),
            3 => Some(SaveFormat::Switch(SWITCH_VER_1_2)),
            4 => Some(SaveFormat::Switch(SWITCH_VER_1_3)),
            _ => unreachable!()
        }
    }
}

impl Default for ImportExportMenuEntry {
    fn default() -> Self {
        ImportExportMenuEntry::Format
    }
}

pub struct SaveSelectMenu {
    pub saves: [MenuSaveInfo; 3],
    pub export_info: MenuExportInfo,
    current_menu: CurrentMenu,
    save_menu: Menu<SaveMenuEntry>,
    save_detailed: Menu<usize>,
    difficulty_menu: Menu<DifficultyMenuEntry>,
    coop_menu: PlayerCountMenu,
    delete_confirm: Menu<DeleteConfirmMenuEntry>,
    load_confirm: Menu<LoadConfirmMenuEntry>,
    skip_difficulty_menu: bool,
    import_export_menu: Menu<ImportExportMenuEntry>,
}

impl SaveSelectMenu {
    pub fn new() -> SaveSelectMenu {
        SaveSelectMenu {
            saves: [MenuSaveInfo::default(); 3],
            export_info: MenuExportInfo::default(),
            current_menu: CurrentMenu::SaveMenu,
            save_menu: Menu::new(0, 0, 230, 0),
            coop_menu: PlayerCountMenu::new(),
            save_detailed: Menu::new(0, 0, 230, 0),
            difficulty_menu: Menu::new(0, 0, 130, 0),
            delete_confirm: Menu::new(0, 0, 75, 0),
            load_confirm: Menu::new(0, 0, 75, 0),
            skip_difficulty_menu: false,
            import_export_menu: Menu::new(0, 0, 75, 0),
        }
    }

    pub fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.save_menu = Menu::new(0, 0, 230, 0);
        self.save_detailed = Menu::new(0, 0, 230, 0);
        self.coop_menu.on_title = true;
        self.coop_menu.init(state)?;
        self.difficulty_menu = Menu::new(0, 0, 130, 0);
        self.delete_confirm = Menu::new(0, 0, 75, 0);
        self.load_confirm = Menu::new(0, 0, 75, 0);
        self.import_export_menu = Menu::new(0, 0, 75, 0);
        self.skip_difficulty_menu = false;

        let mut should_mutate_selection = true;

        let save_container = SaveContainer::load(ctx, state)?;
        for (iter, save) in self.saves.iter_mut().enumerate() {
            if let Some(slot) = state.get_save_slot(iter + 1) {
                if let Some(loaded_profile) = save_container.get_profile(slot) {
                    log::trace!("Loading save select menu. Iter - {}. {}", iter, loaded_profile.is_empty());
                    save.current_map = loaded_profile.current_map;
                    save.max_life = loaded_profile.max_life;
                    save.life = loaded_profile.life;
                    save.weapon_count = loaded_profile.weapon_data.iter().filter(|weapon| weapon.weapon_id != 0).count();
                    save.weapon_id = loaded_profile.weapon_data.map(|weapon| weapon.weapon_id);
                    save.difficulty = loaded_profile.difficulty;

                    self.save_menu.push_entry(SaveMenuEntry::Load(iter), MenuEntry::SaveData(*save));

                    if should_mutate_selection {
                        should_mutate_selection = false;
                        self.save_menu.selected = SaveMenuEntry::Load(iter);
                    }
                } else {
                    self.save_menu.push_entry(SaveMenuEntry::New(iter), MenuEntry::NewSave);

                    if should_mutate_selection {
                        should_mutate_selection = false;
                        self.save_menu.selected = SaveMenuEntry::New(iter);
                    }
                }
            } else {
                self.save_menu.push_entry(SaveMenuEntry::New(iter), MenuEntry::NewSave);

                if should_mutate_selection {
                    should_mutate_selection = false;
                    self.save_menu.selected = SaveMenuEntry::New(iter);
                }
            }
        }

        self.save_menu.push_entry(SaveMenuEntry::ImportExport, MenuEntry::Active(state.loc.ts("menus.save_manage_menu.import_export_save")));
        self.save_menu.push_entry(SaveMenuEntry::Back, MenuEntry::Active(state.loc.t("common.back").to_owned()));

        self.difficulty_menu.push_entry(
            DifficultyMenuEntry::Title,
            MenuEntry::Disabled(state.loc.t("menus.difficulty_menu.title").to_owned()),
        );
        self.difficulty_menu.push_entry(
            DifficultyMenuEntry::Difficulty(GameDifficulty::Easy),
            MenuEntry::Active(state.loc.t("menus.difficulty_menu.easy").to_owned()),
        );
        self.difficulty_menu.push_entry(
            DifficultyMenuEntry::Difficulty(GameDifficulty::Normal),
            MenuEntry::Active(state.loc.t("menus.difficulty_menu.normal").to_owned()),
        );
        self.difficulty_menu.push_entry(
            DifficultyMenuEntry::Difficulty(GameDifficulty::Hard),
            MenuEntry::Active(state.loc.t("menus.difficulty_menu.hard").to_owned()),
        );
        self.difficulty_menu
            .push_entry(DifficultyMenuEntry::Back, MenuEntry::Active(state.loc.t("common.back").to_owned()));

        self.difficulty_menu.selected = DifficultyMenuEntry::Difficulty(GameDifficulty::Normal);

        self.delete_confirm.push_entry(
            DeleteConfirmMenuEntry::Title,
            MenuEntry::Disabled(state.loc.t("menus.save_menu.delete_confirm").to_owned()),
        );
        self.delete_confirm
            .push_entry(DeleteConfirmMenuEntry::Yes, MenuEntry::Active(state.loc.t("common.yes").to_owned()));
        self.delete_confirm
            .push_entry(DeleteConfirmMenuEntry::No, MenuEntry::Active(state.loc.t("common.no").to_owned()));

        self.delete_confirm.selected = DeleteConfirmMenuEntry::No;

        self.load_confirm.push_entry(
            LoadConfirmMenuEntry::Start,
            MenuEntry::Active(state.loc.t("menus.main_menu.start").to_owned()),
        );
        self.load_confirm.push_entry(
            LoadConfirmMenuEntry::Delete,
            MenuEntry::Active(state.loc.t("menus.save_menu.delete_confirm").to_owned()),
        );
        self.load_confirm
            .push_entry(LoadConfirmMenuEntry::Back, MenuEntry::Active(state.loc.t("common.back").to_owned()));

        self.save_detailed.draw_cursor = false;

        if let (_, MenuEntry::SaveData(save)) = self.save_menu.entries[0] {
            self.save_detailed.push_entry(0, MenuEntry::SaveDataSingle(save));
        }

        self.export_info.format = None;

        self.import_export_menu.push_entry(
            ImportExportMenuEntry::Format,
            MenuEntry::Options(
                state.loc.ts("menus.save_manage_menu.save_format.entry"),
                0,
                vec![
                    state.loc.ts("menus.save_manage_menu.save_format.auto"),
                    state.loc.ts("menus.save_manage_menu.save_format.freeware"),
                    state.loc.ts("menus.save_manage_menu.save_format.plus"),
                    state.loc.tt(
                        "menus.save_manage_menu.save_format.switch",
                        &[("version", "v1.2")]
                    ),
                    state.loc.tt(
                        "menus.save_manage_menu.save_format.switch",
                        &[("version", "v1.3")]
                    ),
                ]
            )
        );
        self.import_export_menu.push_entry(
            ImportExportMenuEntry::Import,
            MenuEntry::Active(state.loc.ts("menus.save_manage_menu.action_type.import"))
        );
        self.import_export_menu.push_entry(
            ImportExportMenuEntry::Export,
            MenuEntry::Active(state.loc.ts("menus.save_manage_menu.action_type.export"))
        );
        self.import_export_menu.push_entry(ImportExportMenuEntry::Back, MenuEntry::Active(state.loc.ts("common.back")));

        self.update_sizes(state);

        Ok(())
    }

    pub fn set_skip_difficulty_menu(&mut self, skip: bool) {
        self.skip_difficulty_menu = skip;
    }

    fn update_sizes(&mut self, state: &SharedGameState) {
        self.save_menu.update_width(state);
        self.save_menu.update_height(state);
        self.save_menu.x = ((state.canvas_size.0 - self.save_menu.width as f32) / 2.0).floor() as isize;
        self.save_menu.y = ((state.canvas_size.1 - self.save_menu.height as f32) / 2.0).floor() as isize;

        self.difficulty_menu.update_width(state);
        self.difficulty_menu.update_height(state);
        self.difficulty_menu.x = ((state.canvas_size.0 - self.difficulty_menu.width as f32) / 2.0).floor() as isize;
        self.difficulty_menu.y =
            30 + ((state.canvas_size.1 - self.difficulty_menu.height as f32) / 2.0).floor() as isize;

        self.delete_confirm.update_width(state);
        self.delete_confirm.update_height(state);
        self.delete_confirm.x = ((state.canvas_size.0 - self.delete_confirm.width as f32) / 2.0).floor() as isize;
        self.delete_confirm.y = 30 + ((state.canvas_size.1 - self.delete_confirm.height as f32) / 2.0).floor() as isize;

        self.load_confirm.update_width(state);
        self.load_confirm.update_height(state);
        self.load_confirm.x = ((state.canvas_size.0 - self.load_confirm.width as f32) / 2.0).floor() as isize;
        self.load_confirm.y = 30 + ((state.canvas_size.1 - self.load_confirm.height as f32) / 2.0).floor() as isize;

        self.save_detailed.update_width(state);
        self.save_detailed.update_height(state);
        self.save_detailed.x = ((state.canvas_size.0 - self.save_detailed.width as f32) / 2.0).floor() as isize;
        self.save_detailed.y = -40 + ((state.canvas_size.1 - self.save_detailed.height as f32) / 2.0).floor() as isize;

        self.import_export_menu.update_width(state);
        self.import_export_menu.update_height(state);
        self.import_export_menu.x = ((state.canvas_size.0 - self.import_export_menu.width as f32) / 2.0).floor() as isize;
        self.import_export_menu.y = ((state.canvas_size.1 - self.import_export_menu.height as f32) / 2.0).floor() as isize;
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
                MenuSelectionResult::Selected(SaveMenuEntry::Back, _) | MenuSelectionResult::Canceled => exit_action(),
                MenuSelectionResult::Selected(SaveMenuEntry::New(slot), _) => {
                    state.save_slot = slot + 1;

                    if self.skip_difficulty_menu {
                        self.confirm_save_slot(state, ctx)?;
                    } else {
                        self.difficulty_menu.selected = DifficultyMenuEntry::Difficulty(GameDifficulty::Normal);
                        self.current_menu = CurrentMenu::DifficultyMenu;
                    }
                }
                MenuSelectionResult::Selected(SaveMenuEntry::Load(slot), _) => {
                    state.save_slot = slot + 1;

                    if let (_, MenuEntry::SaveData(save)) = self.save_menu.entries[slot] {
                        self.save_detailed.entries.clear();
                        self.save_detailed.push_entry(0, MenuEntry::SaveDataSingle(save));
                    }

                    self.current_menu = CurrentMenu::LoadConfirm;
                    self.load_confirm.selected = LoadConfirmMenuEntry::Start;
                }
                MenuSelectionResult::Selected(SaveMenuEntry::ImportExport, _) => {
                    self.current_menu = CurrentMenu::ImportExport;
                    self.import_export_menu.selected = ImportExportMenuEntry::Back;
                }
                _ => (),
            },
            CurrentMenu::DifficultyMenu => match self.difficulty_menu.tick(controller, state) {
                MenuSelectionResult::Selected(DifficultyMenuEntry::Back, _) | MenuSelectionResult::Canceled => {
                    self.current_menu = CurrentMenu::SaveMenu;
                }
                MenuSelectionResult::Selected(DifficultyMenuEntry::Difficulty(difficulty), _) => {
                    state.difficulty = difficulty;
                    self.confirm_save_slot(state, ctx)?;
                }
                _ => (),
            },
            CurrentMenu::PlayerCountMenu => {
                let cm = &mut self.current_menu;
                let rm = CurrentMenu::SaveMenu;
                self.coop_menu.tick(
                    &mut || {
                        *cm = rm;
                    },
                    controller,
                    state,
                    ctx,
                )?;
            }
            CurrentMenu::DeleteConfirm => match self.delete_confirm.tick(controller, state) {
                MenuSelectionResult::Selected(DeleteConfirmMenuEntry::Yes, _) => {
                    match self.save_menu.selected {
                        SaveMenuEntry::Load(slot) => {
                            state.sound_manager.play_sfx(17); // Player Death sfx
                            let mut save = SaveContainer::load(ctx, state)?;
                            save.delete_profile(&ctx, state.get_save_slot(slot + 1).unwrap());
                            save.save(ctx, state, SaveParams::default())?;
                        }
                        _ => (),
                    }

                    self.save_menu.set_entry(self.save_menu.selected, MenuEntry::NewSave);
                    if let SaveMenuEntry::Load(slot) = self.save_menu.selected {
                        self.save_menu.set_id(self.save_menu.selected, SaveMenuEntry::New(slot));
                        self.save_menu.selected = SaveMenuEntry::New(slot);
                    }

                    self.current_menu = CurrentMenu::SaveMenu;
                }
                MenuSelectionResult::Selected(DeleteConfirmMenuEntry::No, _) | MenuSelectionResult::Canceled => {
                    self.current_menu = CurrentMenu::LoadConfirm;
                    self.load_confirm.selected = LoadConfirmMenuEntry::Start;
                }
                _ => (),
            },
            CurrentMenu::LoadConfirm => match self.load_confirm.tick(controller, state) {
                MenuSelectionResult::Selected(LoadConfirmMenuEntry::Start, _) => {
                    self.confirm_save_slot(state, ctx)?;
                }
                MenuSelectionResult::Selected(LoadConfirmMenuEntry::Delete, _) => {
                    self.current_menu = CurrentMenu::DeleteConfirm;
                    self.delete_confirm.selected = DeleteConfirmMenuEntry::No;
                }
                MenuSelectionResult::Selected(LoadConfirmMenuEntry::Back, _) | MenuSelectionResult::Canceled => {
                    self.current_menu = CurrentMenu::SaveMenu;
                }
                _ => (),
            },
            CurrentMenu::ImportExport => match self.import_export_menu.tick(controller, state) {
                MenuSelectionResult::Selected(ImportExportMenuEntry::Format, toggle)
                | MenuSelectionResult::Right(ImportExportMenuEntry::Format, toggle, _) => {
                    if let MenuEntry::Options(_, value, _) = toggle {
                        *value = match *value {
                            0..4 => *value + 1,
                            4 => 0,
                            _ => unreachable!(),
                        };

                        self.export_info.format = ImportExportMenuEntry::format_from_value(*value);
                    }
                }
                MenuSelectionResult::Left(ImportExportMenuEntry::Format, toggle, _) => {
                    if let MenuEntry::Options(_, value, _) = toggle {
                        *value = match *value {
                            1..=4 => *value - 1,
                            0 => 4,
                            _ => unreachable!(),
                        };

                        self.export_info.format = ImportExportMenuEntry::format_from_value(*value);
                    }
                }
                MenuSelectionResult::Selected(ImportExportMenuEntry::Import, _) => {
                    let picker_params = MenuExportInfo::fpicker_from_format(state, self.export_info.format, false);
                    let selection = open_file_picker(&picker_params);
                    let out_path = selection.and_then(|location| location.first().cloned());

                    if out_path.is_none() {
                        // Export path is not selected, so we break export operation
                        return Ok(());
                    }

                    log::trace!("{:?}", out_path);

                    let mut save_container = SaveContainer::load(ctx, state)?;
                    save_container.import(state, ctx, self.export_info.format, self.export_info.save_params.clone(), out_path.unwrap().clone())?;
                    save_container.save(ctx, state, self.export_info.save_params.clone())?;
                }
                MenuSelectionResult::Selected(ImportExportMenuEntry::Export, _) => {
                    let picker_params = MenuExportInfo::fpicker_from_format(state, self.export_info.format, true);
                    let selection = open_file_picker(&picker_params);
                    let out_path = selection.and_then(|location| location.first().cloned());

                    if out_path.is_none() {
                        // Export path is not selected, so we break export operation
                        return Ok(());
                    }

                    log::trace!("{:?}", out_path);


                    let format = self.export_info.format.unwrap_or(state.settings.save_format);

                    let mut save_container = SaveContainer::load(ctx, state)?;
                    save_container.export(state, ctx, format, self.export_info.save_params.clone(), out_path.unwrap().clone())?;
                }
                MenuSelectionResult::Selected(ImportExportMenuEntry::Back, _) | MenuSelectionResult::Canceled => {
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
            CurrentMenu::PlayerCountMenu => {
                self.coop_menu.draw(state, ctx)?;
            }
            CurrentMenu::DeleteConfirm => {
                self.save_detailed.draw(state, ctx)?;
                self.delete_confirm.draw(state, ctx)?;
            }
            CurrentMenu::LoadConfirm => {
                self.save_detailed.draw(state, ctx)?;
                self.load_confirm.draw(state, ctx)?;
            }
            CurrentMenu::ImportExport => {
                self.import_export_menu.draw(state, ctx)?;
            }
        }
        Ok(())
    }

    fn confirm_save_slot(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if state.constants.supports_two_player {
            self.current_menu = CurrentMenu::PlayerCountMenu;
        } else {
            state.reload_resources(ctx)?;
            state.load_or_start_game(ctx)?;
        }

        Ok(())
    }
}
