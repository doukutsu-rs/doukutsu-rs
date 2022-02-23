use crate::common::{Color, VERSION_BANNER};
use crate::components::background::Background;
use crate::components::nikumaru::NikumaruCounter;
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::input::touch_controls::TouchControlType;
use crate::map::Map;
use crate::menu::save_select_menu::SaveSelectMenu;
use crate::menu::settings_menu::SettingsMenu;
use crate::menu::{Menu, MenuEntry, MenuSelectionResult};
use crate::scene::jukebox_scene::JukeboxScene;
use crate::scene::Scene;
use crate::shared_game_state::{MenuCharacter, SharedGameState, TileSize};
use crate::stage::{BackgroundType, NpcType, Stage, StageData, StageTexturePaths, Tileset};

#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
#[allow(unused)]
enum CurrentMenu {
    MainMenu,
    OptionMenu,
    SaveSelectMenu,
    ChallengesMenu,
    StartGame,
    LoadGame,
}

pub struct TitleScene {
    tick: usize,
    controller: CombinedMenuController,
    current_menu: CurrentMenu,
    main_menu: Menu,
    save_select_menu: SaveSelectMenu,
    challenges_menu: Menu,
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
        let song_id: usize;

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

        if song_id != state.sound_manager.current_song() {
            state.sound_manager.play_song(song_id, &state.constants, &state.settings, ctx)?;
        }
        Ok(())
    }
}

// asset copyright for freeware version
static COPYRIGHT_PIXEL: &str = "2004.12  Studio Pixel";

impl Scene for TitleScene {
    fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if !state.mod_path.is_none() {
            state.mod_path = None;
            state.reload_resources(ctx)?;
        }

        self.controller.add(state.settings.create_player1_controller());
        self.controller.add(state.settings.create_player2_controller());

        self.main_menu.push_entry(MenuEntry::Active("Start Game".to_string()));
        if state.constants.is_cs_plus {
            self.main_menu.push_entry(MenuEntry::Active("Challenges".to_string()));
        } else {
            self.main_menu.push_entry(MenuEntry::Hidden);
        }
        self.main_menu.push_entry(MenuEntry::Active("Options".to_string()));
        if cfg!(feature = "editor") {
            self.main_menu.push_entry(MenuEntry::Active("Editor".to_string()));
        } else {
            self.main_menu.push_entry(MenuEntry::Hidden);
        }
        if state.constants.is_switch {
            self.main_menu.push_entry(MenuEntry::Active("Jukebox".to_string()));
        } else {
            self.main_menu.push_entry(MenuEntry::Hidden);
        }
        self.main_menu.push_entry(MenuEntry::Active("Quit".to_string()));

        self.settings_menu.init(state, ctx)?;

        self.save_select_menu.init(state, ctx)?;

        for mod_info in state.mod_list.mods.iter() {
            self.challenges_menu.push_entry(MenuEntry::Active(mod_info.name.clone()));
        }
        self.challenges_menu.push_entry(MenuEntry::Active("< Back".to_string()));

        self.controller.update(state, ctx)?;
        self.controller.update_trigger();

        self.nikumaru_rec.load_counter(ctx)?;
        self.update_menu_cursor(state, ctx)?;

        Ok(())
    }

    fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        state.touch_controls.control_type = TouchControlType::None;
        self.background.tick()?;
        self.controller.update(state, ctx)?;
        self.controller.update_trigger();

        self.main_menu.update_height();
        self.main_menu.x = ((state.canvas_size.0 - self.main_menu.width as f32) / 2.0).floor() as isize;
        self.main_menu.y = ((state.canvas_size.1 + 70.0 - self.main_menu.height as f32) / 2.0).floor() as isize;

        self.challenges_menu.update_height();
        self.challenges_menu.x = ((state.canvas_size.0 - self.challenges_menu.width as f32) / 2.0).floor() as isize;
        self.challenges_menu.y =
            ((state.canvas_size.1 + 30.0 - self.challenges_menu.height as f32) / 2.0).floor() as isize;

        match self.current_menu {
            CurrentMenu::MainMenu => match self.main_menu.tick(&mut self.controller, state) {
                MenuSelectionResult::Selected(0, _) => {
                    state.mod_path = None;
                    self.save_select_menu.init(state, ctx)?;
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
                self.save_select_menu.tick(
                    &mut || {
                        *cm = CurrentMenu::MainMenu;
                    },
                    &mut self.controller,
                    state,
                    ctx,
                )?;
            }
            CurrentMenu::StartGame => {
                if self.tick == 10 {
                    state.reset_skip_flags();
                    state.start_new_game(ctx)?;
                }
            }
            CurrentMenu::LoadGame => {
                if self.tick == 10 {
                    state.load_or_start_game(ctx)?;
                }
            }
            CurrentMenu::ChallengesMenu => {
                let last_idx = self.challenges_menu.entries.len() - 1;
                match self.challenges_menu.tick(&mut self.controller, state) {
                    MenuSelectionResult::Selected(idx, _) => {
                        if last_idx == idx {
                            self.current_menu = CurrentMenu::MainMenu;
                        } else if let Some(mod_info) = state.mod_list.mods.get(idx) {
                            state.mod_path = Some(mod_info.path.clone());
                            if mod_info.save_slot >= 0 {
                                self.save_select_menu.init(state, ctx)?;
                                self.current_menu = CurrentMenu::SaveSelectMenu;
                            } else {
                                state.reload_resources(ctx)?;
                                state.start_new_game(ctx)?;
                            }
                        }
                    }
                    MenuSelectionResult::Canceled => {
                        self.current_menu = CurrentMenu::MainMenu;
                    }
                    _ => (),
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

        self.background.draw(state, ctx, &self.frame, &self.textures, &self.stage)?;

        {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Title")?;
            batch.add_rect(
                ((state.canvas_size.0 - state.constants.title.logo_rect.width() as f32) / 2.0).floor(),
                40.0,
                &state.constants.title.logo_rect,
            );

            batch.draw(ctx)?;
        }

        self.draw_text_centered(&VERSION_BANNER, state.canvas_size.1 - 15.0, state, ctx)?;
        self.draw_text_centered(COPYRIGHT_PIXEL, state.canvas_size.1 - 30.0, state, ctx)?;

        self.nikumaru_rec.draw(state, ctx, &self.frame)?;

        match self.current_menu {
            CurrentMenu::MainMenu => self.main_menu.draw(state, ctx)?,
            CurrentMenu::ChallengesMenu => self.challenges_menu.draw(state, ctx)?,
            CurrentMenu::OptionMenu => self.settings_menu.draw(state, ctx)?,
            CurrentMenu::SaveSelectMenu => self.save_select_menu.draw(state, ctx)?,
            _ => {}
        }

        Ok(())
    }
}
