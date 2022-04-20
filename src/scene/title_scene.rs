use crate::common::{Color, VERSION_BANNER};
use crate::components::background::Background;
use crate::components::nikumaru::NikumaruCounter;
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::input::touch_controls::TouchControlType;
use crate::map::Map;
use crate::menu::save_select_menu::SaveSelectMenu;
use crate::menu::settings_menu::SettingsMenu;
use crate::menu::{Menu, MenuEntry, MenuSelectionResult};
use crate::scene::jukebox_scene::JukeboxScene;
use crate::scene::Scene;
use crate::shared_game_state::{GameDifficulty, MenuCharacter, ReplayState, Season, SharedGameState, TileSize};
use crate::stage::{BackgroundType, NpcType, Stage, StageData, StageTexturePaths, Tileset};

#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
#[allow(unused)]
enum CurrentMenu {
    MainMenu,
    OptionMenu,
    SaveSelectMenu,
    ChallengesMenu,
    ChallengeConfirmMenu,
}

pub struct TitleScene {
    tick: usize,
    controller: CombinedMenuController,
    current_menu: CurrentMenu,
    main_menu: Menu,
    save_select_menu: SaveSelectMenu,
    challenges_menu: Menu,
    confirm_menu: Menu,
    settings_menu: SettingsMenu,
    background: Background,
    frame: Frame,
    nikumaru_rec: NikumaruCounter,
    stage: Stage,
    textures: StageTexturePaths,
}

impl TitleScene {
    pub fn new() -> Self {
        let fake_stage = Stage {
            map: Map { width: 0, height: 0, tiles: vec![], attrib: [0; 0x100], tile_size: TileSize::Tile16x16 },
            data: StageData {
                name: "".to_string(),
                name_jp: "".to_string(),
                map: "".to_string(),
                boss_no: 0,
                tileset: Tileset { name: "0".to_string() },
                pxpack_data: None,
                background: crate::stage::Background::new("bkMoon"),
                background_type: BackgroundType::Outside,
                background_color: Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
                npc1: NpcType::new("0"),
                npc2: NpcType::new("0"),
            },
        };
        let mut textures = StageTexturePaths::new();
        textures.update(&fake_stage);

        let mut settings_menu = SettingsMenu::new();
        settings_menu.on_title = true;

        Self {
            tick: 0,
            controller: CombinedMenuController::new(),
            current_menu: CurrentMenu::MainMenu,
            main_menu: Menu::new(0, 0, 100, 0),
            save_select_menu: SaveSelectMenu::new(),
            challenges_menu: Menu::new(0, 0, 150, 0),
            confirm_menu: Menu::new(0, 0, 150, 0),
            settings_menu,
            background: Background::new(),
            frame: Frame::new(),
            nikumaru_rec: NikumaruCounter::new(),
            stage: fake_stage,
            textures,
        }
    }

    fn draw_text_centered(&self, text: &str, y: f32, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let width = state.font.text_width(text.chars(), &state.constants);
        state.font.draw_text(
            text.chars(),
            ((state.canvas_size.0 - width) / 2.0).floor(),
            y,
            &state.constants,
            &mut state.texture_set,
            ctx,
        )?;

        Ok(())
    }

    pub fn update_menu_cursor(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let minutes = self.nikumaru_rec.tick / (60 * state.settings.timing_mode.get_tps());
        let mut song_id: usize;

        if self.nikumaru_rec.shown && minutes < 3 {
            state.menu_character = MenuCharacter::Sue;
            song_id = 2;
        } else if self.nikumaru_rec.shown && minutes < 4 {
            state.menu_character = MenuCharacter::King;
            song_id = 41;
        } else if self.nikumaru_rec.shown && minutes < 5 {
            state.menu_character = MenuCharacter::Toroko;
            song_id = 40;
        } else if self.nikumaru_rec.shown && minutes < 6 {
            state.menu_character = MenuCharacter::Curly;
            song_id = 36;
        } else {
            state.menu_character = MenuCharacter::Quote;
            song_id = 24;
        }

        if state.settings.soundtrack == "New" && Season::current() == Season::PixelBirthday {
            song_id = 43;
        }

        if song_id != state.sound_manager.current_song() {
            state.sound_manager.play_song(song_id, &state.constants, &state.settings, ctx)?;
        }
        Ok(())
    }

    pub fn open_settings_menu(&mut self) -> GameResult {
        self.current_menu = CurrentMenu::OptionMenu;
        Ok(())
    }
}

static COPYRIGHT_PIXEL: &str = "2004.12  Studio Pixel"; // Freeware
static COPYRIGHT_NICALIS: &str = "@2022 NICALIS INC."; // Nicalis font uses @ for copyright

impl Scene for TitleScene {
    fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if !state.mod_path.is_none() {
            state.mod_path = None;
            state.reload_resources(ctx)?;
        }

        self.controller.add(state.settings.create_player1_controller());
        self.controller.add(state.settings.create_player2_controller());

        self.main_menu.push_entry(MenuEntry::Active(state.t("menus.main_menu.start")));
        if !state.mod_list.mods.is_empty() {
            self.main_menu.push_entry(MenuEntry::Active(state.t("menus.main_menu.challenges")));
        } else {
            self.main_menu.push_entry(MenuEntry::Hidden);
        }
        self.main_menu.push_entry(MenuEntry::Active(state.t("menus.main_menu.options")));
        if cfg!(feature = "editor") {
            self.main_menu.push_entry(MenuEntry::Active(state.t("menus.main_menu.editor")));
        } else {
            self.main_menu.push_entry(MenuEntry::Hidden);
        }
        if state.constants.is_switch {
            self.main_menu.push_entry(MenuEntry::Active(state.t("menus.main_menu.jukebox")));
        } else {
            self.main_menu.push_entry(MenuEntry::Hidden);
        }
        self.main_menu.push_entry(MenuEntry::Active(state.t("menus.main_menu.quit")));

        self.settings_menu.init(state, ctx)?;

        self.save_select_menu.init(state, ctx)?;

        let mut selected: usize = 0;
        let mut mutate_selection = true;

        for mod_info in state.mod_list.mods.iter() {
            if mod_info.satisfies_requirement(&state.mod_requirements) {
                self.challenges_menu.push_entry(MenuEntry::Active(mod_info.name.clone()));
                mutate_selection = false;
            } else {
                self.challenges_menu.push_entry(MenuEntry::Disabled("???".to_owned()));

                if mutate_selection {
                    selected += 1;
                }
            }
        }
        self.challenges_menu.push_entry(MenuEntry::Active(state.t("common.back")));
        self.challenges_menu.selected = selected;

        self.confirm_menu.push_entry(MenuEntry::Disabled("".to_owned()));
        self.confirm_menu.push_entry(MenuEntry::Active(state.t("menus.challenge_menu.start")));
        self.confirm_menu.push_entry(MenuEntry::Disabled(state.t("menus.challenge_menu.no_replay")));
        self.confirm_menu.push_entry(MenuEntry::Hidden);
        self.confirm_menu.push_entry(MenuEntry::Active(state.t("common.back")));
        self.confirm_menu.selected = 1;

        self.controller.update(state, ctx)?;
        self.controller.update_trigger();

        self.nikumaru_rec.load_counter(state, ctx)?;
        self.update_menu_cursor(state, ctx)?;

        state.replay_state = ReplayState::None;
        state.textscript_vm.flags.set_cutscene_skip(false);

        Ok(())
    }

    fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        state.touch_controls.control_type = TouchControlType::None;
        self.background.tick()?;
        self.controller.update(state, ctx)?;
        self.controller.update_trigger();

        self.main_menu.update_width(state);
        self.main_menu.update_height();
        self.main_menu.x = ((state.canvas_size.0 - self.main_menu.width as f32) / 2.0).floor() as isize;
        self.main_menu.y = ((state.canvas_size.1 + 70.0 - self.main_menu.height as f32) / 2.0).floor() as isize;

        self.challenges_menu.update_width(state);
        self.challenges_menu.update_height();
        self.challenges_menu.x = ((state.canvas_size.0 - self.challenges_menu.width as f32) / 2.0).floor() as isize;
        self.challenges_menu.y =
            ((state.canvas_size.1 + 30.0 - self.challenges_menu.height as f32) / 2.0).floor() as isize;

        match self.current_menu {
            CurrentMenu::MainMenu => match self.main_menu.tick(&mut self.controller, state) {
                MenuSelectionResult::Selected(0, _) => {
                    state.mod_path = None;
                    self.save_select_menu.init(state, ctx)?;
                    self.save_select_menu.set_skip_difficulty_menu(false);
                    self.current_menu = CurrentMenu::SaveSelectMenu;
                }
                MenuSelectionResult::Selected(1, _) => {
                    self.current_menu = CurrentMenu::ChallengesMenu;
                }
                MenuSelectionResult::Selected(2, _) => {
                    self.current_menu = CurrentMenu::OptionMenu;
                }
                MenuSelectionResult::Selected(3, _) => {
                    // this comment is just there because rustfmt removes parenthesis around the match case and breaks compilation
                    #[cfg(feature = "editor")]
                    {
                        use crate::scene::editor_scene::EditorScene;
                        state.next_scene = Some(Box::new(EditorScene::new()));
                    }
                }
                MenuSelectionResult::Selected(4, _) => {
                    state.next_scene = Some(Box::new(JukeboxScene::new()));
                }
                MenuSelectionResult::Selected(5, _) => {
                    state.shutdown();
                }
                _ => {}
            },
            CurrentMenu::OptionMenu => {
                let timing_mode = state.settings.timing_mode;
                let cm = &mut self.current_menu;
                self.settings_menu.tick(
                    &mut || {
                        *cm = CurrentMenu::MainMenu;
                    },
                    &mut self.controller,
                    state,
                    ctx,
                )?;
                if timing_mode != state.settings.timing_mode {
                    self.update_menu_cursor(state, ctx)?;
                }
            }
            CurrentMenu::SaveSelectMenu => {
                let cm = &mut self.current_menu;
                let rm = if state.mod_path.is_none() { CurrentMenu::MainMenu } else { CurrentMenu::ChallengesMenu };
                self.save_select_menu.tick(
                    &mut || {
                        *cm = rm;
                    },
                    &mut self.controller,
                    state,
                    ctx,
                )?;
            }
            CurrentMenu::ChallengesMenu => {
                let last_idx = self.challenges_menu.entries.len() - 1;
                match self.challenges_menu.tick(&mut self.controller, state) {
                    MenuSelectionResult::Selected(idx, _) => {
                        if last_idx == idx {
                            state.mod_path = None;
                            self.nikumaru_rec.load_counter(state, ctx)?;
                            self.current_menu = CurrentMenu::MainMenu;
                        } else if let Some(mod_info) = state.mod_list.mods.get(idx) {
                            state.mod_path = Some(mod_info.path.clone());
                            if mod_info.save_slot >= 0 {
                                self.save_select_menu.init(state, ctx)?;
                                self.save_select_menu.set_skip_difficulty_menu(true);
                                self.nikumaru_rec.load_counter(state, ctx)?;
                                self.current_menu = CurrentMenu::SaveSelectMenu;
                            } else {
                                let mod_name = mod_info.name.clone();
                                self.confirm_menu.width =
                                    (state.font.text_width(mod_name.chars(), &state.constants).max(50.0) + 32.0) as u16;
                                self.confirm_menu.entries[0] = MenuEntry::Disabled(mod_name);
                                if state.has_replay_data(ctx) {
                                    self.confirm_menu.entries[2] =
                                        MenuEntry::Active(state.t("menus.challenge_menu.replay_best"));
                                    self.confirm_menu.entries[3] =
                                        MenuEntry::Active(state.t("menus.challenge_menu.delete_replay"));
                                } else {
                                    self.confirm_menu.entries[2] =
                                        MenuEntry::Disabled(state.t("menus.challenge_menu.no_replay"));
                                    self.confirm_menu.entries[3] = MenuEntry::Hidden;
                                }
                                self.nikumaru_rec.load_counter(state, ctx)?;
                                self.current_menu = CurrentMenu::ChallengeConfirmMenu;
                            }
                        }
                    }
                    MenuSelectionResult::Canceled => {
                        state.mod_path = None;
                        self.nikumaru_rec.load_counter(state, ctx)?;
                        self.current_menu = CurrentMenu::MainMenu;
                    }
                    _ => (),
                }
            }
            CurrentMenu::ChallengeConfirmMenu => match self.confirm_menu.tick(&mut self.controller, state) {
                MenuSelectionResult::Selected(1, _) => {
                    state.difficulty = GameDifficulty::Normal;
                    state.replay_state = ReplayState::Recording;
                    state.reload_resources(ctx)?;
                    state.start_new_game(ctx)?;
                }
                MenuSelectionResult::Selected(2, _) => {
                    state.difficulty = GameDifficulty::Normal;
                    state.replay_state = ReplayState::Playback;
                    state.reload_resources(ctx)?;
                    state.start_new_game(ctx)?;
                }
                MenuSelectionResult::Selected(3, _) => {
                    state.delete_replay_data(ctx)?;
                    self.current_menu = CurrentMenu::ChallengesMenu;
                }
                MenuSelectionResult::Selected(4, _) | MenuSelectionResult::Canceled => {
                    self.current_menu = CurrentMenu::ChallengesMenu;
                }
                _ => (),
            },
        }

        self.confirm_menu.update_width(state);
        self.confirm_menu.update_height();
        self.confirm_menu.x = ((state.canvas_size.0 - self.confirm_menu.width as f32) / 2.0).floor() as isize;
        self.confirm_menu.y = ((state.canvas_size.1 + 30.0 - self.confirm_menu.height as f32) / 2.0).floor() as isize;

        self.tick += 1;

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.background.draw(state, ctx, &self.frame, &self.textures, &self.stage)?;

        if self.current_menu == CurrentMenu::MainMenu {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Title")?;
            batch.add_rect(
                ((state.canvas_size.0 - state.constants.title.logo_rect.width() as f32) / 2.0).floor(),
                40.0,
                &state.constants.title.logo_rect,
            );

            batch.draw(ctx)?;
        } else {
            let window_title = match self.current_menu {
                CurrentMenu::ChallengesMenu => (state.t("menus.main_menu.challenges")),
                CurrentMenu::ChallengeConfirmMenu | CurrentMenu::SaveSelectMenu => (state.t("menus.main_menu.start")),
                CurrentMenu::OptionMenu => (state.t("menus.main_menu.options")),
                CurrentMenu::MainMenu => unreachable!(),
            };
            state.font.draw_colored_text_with_shadow_scaled(
                window_title.chars(),
                state.canvas_size.0 / 2.0 - state.font.text_width(window_title.chars(), &state.constants) / 2.0,
                state.font.line_height(&state.constants), //im sure there is a better way to shift this into place
                1.0,
                (0xff, 0xff, 0xff, 0xff),
                &state.constants,
                &mut state.texture_set,
                ctx,
            )?;
        }

        self.draw_text_centered(&VERSION_BANNER, state.canvas_size.1 - 15.0, state, ctx)?;

        if state.constants.is_cs_plus {
            self.draw_text_centered(COPYRIGHT_NICALIS, state.canvas_size.1 - 30.0, state, ctx)?;
        } else {
            self.draw_text_centered(COPYRIGHT_PIXEL, state.canvas_size.1 - 30.0, state, ctx)?;
        }

        self.nikumaru_rec.draw(state, ctx, &self.frame)?;

        match self.current_menu {
            CurrentMenu::MainMenu => self.main_menu.draw(state, ctx)?,
            CurrentMenu::ChallengesMenu => self.challenges_menu.draw(state, ctx)?,
            CurrentMenu::ChallengeConfirmMenu => self.confirm_menu.draw(state, ctx)?,
            CurrentMenu::OptionMenu => self.settings_menu.draw(state, ctx)?,
            CurrentMenu::SaveSelectMenu => self.save_select_menu.draw(state, ctx)?,
        }

        Ok(())
    }
}
