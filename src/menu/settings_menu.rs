use itertools::Itertools;

use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::menu::MenuEntry;
use crate::menu::{Menu, MenuSelectionResult};
use crate::shared_game_state::{SharedGameState, TimingMode};
use crate::sound::InterpolationMode;

#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
#[allow(unused)]
enum CurrentMenu {
    MainMenu,
    GraphicsMenu,
    SoundMenu,
    SoundtrackMenu,
}

pub struct SettingsMenu {
    current: CurrentMenu,
    main: Menu,
    graphics: Menu,
    sound: Menu,
    soundtrack: Menu,
    pub on_title: bool,
}

static DISCORD_LINK: &str = "https://discord.gg/fbRsNNB";

impl SettingsMenu {
    pub fn new() -> SettingsMenu {
        let main = Menu::new(0, 0, 220, 0);
        let graphics = Menu::new(0, 0, 180, 0);
        let sound = Menu::new(0, 0, 260, 0);
        let soundtrack = Menu::new(0, 0, 260, 0);

        SettingsMenu { current: CurrentMenu::MainMenu, main, graphics, sound, soundtrack, on_title: false }
    }

    pub fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.graphics.push_entry(MenuEntry::Toggle("Lighting effects:".to_string(), state.settings.shader_effects));
        self.graphics.push_entry(MenuEntry::Toggle("Weapon light cone:".to_string(), state.settings.light_cone));
        self.graphics
            .push_entry(MenuEntry::Toggle("Motion interpolation:".to_string(), state.settings.motion_interpolation));
        self.graphics.push_entry(MenuEntry::Toggle("Subpixel scrolling:".to_string(), state.settings.subpixel_coords));

        // NS version uses two different maps, therefore we can't dynamically switch between graphics presets.
        if state.constants.supports_og_textures {
            if !state.constants.is_switch || self.on_title {
                self.graphics
                    .push_entry(MenuEntry::Toggle("Original textures".to_string(), state.settings.original_textures));
            } else {
                self.graphics.push_entry(MenuEntry::Disabled("Original textures".to_string()));
            }
        } else {
            self.graphics.push_entry(MenuEntry::Hidden);
        }

        if state.constants.is_cs_plus {
            self.graphics
                .push_entry(MenuEntry::Toggle("Seasonal textures".to_string(), state.settings.seasonal_textures));
        } else {
            self.graphics.push_entry(MenuEntry::Hidden);
        }

        self.graphics
            .push_entry(MenuEntry::Disabled(format!("Renderer: {}", ctx.renderer.as_ref().unwrap().renderer_name())));

        self.graphics.push_entry(MenuEntry::Active("< Back".to_owned()));

        self.main.push_entry(MenuEntry::Active("Graphics...".to_owned()));
        self.main.push_entry(MenuEntry::Active("Sound...".to_owned()));

        self.main.push_entry(MenuEntry::Options(
            "Game timing:".to_owned(),
            if state.settings.timing_mode == TimingMode::_50Hz { 0 } else { 1 },
            vec!["50tps (freeware)".to_owned(), "60tps (CS+)".to_owned()],
        ));

        self.main.push_entry(MenuEntry::Active(DISCORD_LINK.to_owned()));

        self.main.push_entry(MenuEntry::Active("< Back".to_owned()));

        self.sound.push_entry(MenuEntry::OptionsBar("Music Volume".to_owned(), state.settings.bgm_volume));
        self.sound.push_entry(MenuEntry::OptionsBar("Effects Volume".to_owned(), state.settings.sfx_volume));

        self.sound.push_entry(MenuEntry::DescriptiveOptions(
            "BGM Interpolation:".to_owned(),
            state.settings.organya_interpolation as usize,
            vec![
                "Nearest".to_owned(),
                "Linear".to_owned(),
                "Cosine".to_owned(),
                "Cubic".to_owned(),
                "Linear+LP".to_owned(),
            ],
            vec![
                "(Fastest, lowest quality)".to_owned(),
                "(Fast, similar to freeware on Vista+)".to_owned(),
                "(Cosine interpolation)".to_owned(),
                "(Cubic interpolation)".to_owned(),
                "(Slowest, similar to freeware on XP)".to_owned(),
            ],
        ));
        self.sound.push_entry(MenuEntry::DisabledWhite("".to_owned()));
        self.sound.push_entry(MenuEntry::Active(format!("Soundtrack: {}", state.settings.soundtrack)));
        self.sound.push_entry(MenuEntry::Active("< Back".to_owned()));

        let mut soundtrack_entries = state.constants.soundtracks.keys().map(|s| s.to_owned()).collect_vec();
        soundtrack_entries.push("Organya".to_owned());

        if let Ok(dir) = filesystem::read_dir(ctx, "/Soundtracks/") {
            for entry in dir {
                if filesystem::is_dir(ctx, &entry) {
                    let filename = entry.file_name().unwrap().to_string_lossy().to_string();

                    if !soundtrack_entries.contains(&filename) {
                        soundtrack_entries.push(filename);
                    }
                }
            }
        }

        soundtrack_entries.sort();

        for soundtrack in soundtrack_entries {
            self.soundtrack.push_entry(MenuEntry::Active(soundtrack));
        }

        self.soundtrack.push_entry(MenuEntry::Active("< Back".to_owned()));

        self.update_sizes(state);

        Ok(())
    }

    fn update_sizes(&mut self, state: &SharedGameState) {
        self.main.update_height();
        self.main.x = ((state.canvas_size.0 - self.main.width as f32) / 2.0).floor() as isize;
        self.main.y = 30 + ((state.canvas_size.1 - self.main.height as f32) / 2.0).floor() as isize;

        self.graphics.update_height();
        self.graphics.x = ((state.canvas_size.0 - self.graphics.width as f32) / 2.0).floor() as isize;
        self.graphics.y = 30 + ((state.canvas_size.1 - self.graphics.height as f32) / 2.0).floor() as isize;

        self.sound.update_height();
        self.sound.x = ((state.canvas_size.0 - self.sound.width as f32) / 2.0).floor() as isize;
        self.sound.y = 30 + ((state.canvas_size.1 - self.sound.height as f32) / 2.0).floor() as isize;

        self.soundtrack.update_height();
        self.soundtrack.x = ((state.canvas_size.0 - self.soundtrack.width as f32) / 2.0).floor() as isize;
        self.soundtrack.y = ((state.canvas_size.1 - self.soundtrack.height as f32) / 2.0).floor() as isize;
    }

    pub fn tick(
        &mut self,
        exit_action: &mut dyn FnMut(),
        controller: &mut CombinedMenuController,
        state: &mut SharedGameState,
        ctx: &mut Context,
    ) -> GameResult {
        self.update_sizes(state);

        match self.current {
            CurrentMenu::MainMenu => match self.main.tick(controller, state) {
                MenuSelectionResult::Selected(0, _) => {
                    self.current = CurrentMenu::GraphicsMenu;
                }
                MenuSelectionResult::Selected(1, _) => {
                    self.current = CurrentMenu::SoundMenu;
                }
                MenuSelectionResult::Selected(2, toggle) => {
                    if let MenuEntry::Options(_, value, _) = toggle {
                        match state.settings.timing_mode {
                            TimingMode::_50Hz => {
                                state.settings.timing_mode = TimingMode::_60Hz;
                                *value = 1;
                            }
                            TimingMode::_60Hz => {
                                state.settings.timing_mode = TimingMode::_50Hz;
                                *value = 0;
                            }
                            _ => {}
                        }
                        let _ = state.settings.save(ctx);
                    }
                }
                MenuSelectionResult::Selected(3, _) => {
                    if let Err(e) = webbrowser::open(DISCORD_LINK) {
                        log::warn!("Error opening web browser: {}", e);
                    }
                }
                MenuSelectionResult::Selected(4, _) | MenuSelectionResult::Canceled => exit_action(),
                _ => (),
            },
            CurrentMenu::GraphicsMenu => match self.graphics.tick(controller, state) {
                MenuSelectionResult::Selected(0, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.shader_effects = !state.settings.shader_effects;
                        let _ = state.settings.save(ctx);

                        *value = state.settings.shader_effects;
                    }
                }
                MenuSelectionResult::Selected(1, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.light_cone = !state.settings.light_cone;
                        let _ = state.settings.save(ctx);

                        *value = state.settings.light_cone;
                    }
                }
                MenuSelectionResult::Selected(2, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.motion_interpolation = !state.settings.motion_interpolation;
                        let _ = state.settings.save(ctx);

                        *value = state.settings.motion_interpolation;
                    }
                }
                MenuSelectionResult::Selected(3, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.subpixel_coords = !state.settings.subpixel_coords;
                        let _ = state.settings.save(ctx);

                        *value = state.settings.subpixel_coords;
                    }
                }
                MenuSelectionResult::Selected(4, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.original_textures = !state.settings.original_textures;
                        state.reload_resources(ctx)?;
                        let _ = state.settings.save(ctx);

                        *value = state.settings.original_textures;
                    }
                }
                MenuSelectionResult::Selected(5, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.seasonal_textures = !state.settings.seasonal_textures;
                        state.reload_graphics();
                        let _ = state.settings.save(ctx);

                        *value = state.settings.seasonal_textures;
                    }
                }
                MenuSelectionResult::Selected(7, _) | MenuSelectionResult::Canceled => {
                    self.current = CurrentMenu::MainMenu
                }
                _ => (),
            },
            CurrentMenu::SoundMenu => match self.sound.tick(controller, state) {
                MenuSelectionResult::Left(0, bgm, direction) | MenuSelectionResult::Right(0, bgm, direction) => {
                    if let MenuEntry::OptionsBar(_, value) = bgm {
                        *value = (*value + (direction as f32 * 0.1)).clamp(0.0, 1.0);
                        state.settings.bgm_volume = *value;
                        state.sound_manager.set_song_volume(*value);

                        let _ = state.settings.save(ctx);
                    }
                }
                MenuSelectionResult::Left(1, sfx, direction) | MenuSelectionResult::Right(1, sfx, direction) => {
                    if let MenuEntry::OptionsBar(_, value) = sfx {
                        *value = (*value + (direction as f32 * 0.1)).clamp(0.0, 1.0);
                        state.settings.sfx_volume = *value;
                        state.sound_manager.set_sfx_volume(*value);

                        let _ = state.settings.save(ctx);
                    }
                }
                MenuSelectionResult::Selected(2, toggle) => {
                    if let MenuEntry::DescriptiveOptions(_, value, _, _) = toggle {
                        let (new_mode, new_value) = match *value {
                            0 => (InterpolationMode::Linear, 1),
                            1 => (InterpolationMode::Cosine, 2),
                            2 => (InterpolationMode::Cubic, 3),
                            3 => (InterpolationMode::Polyphase, 4),
                            _ => (InterpolationMode::Nearest, 0),
                        };

                        *value = new_value;
                        state.settings.organya_interpolation = new_mode;
                        state.sound_manager.set_org_interpolation(new_mode);

                        let _ = state.settings.save(ctx);
                    }
                }
                MenuSelectionResult::Selected(4, _) => self.current = CurrentMenu::SoundtrackMenu,
                MenuSelectionResult::Selected(5, _) | MenuSelectionResult::Canceled => {
                    self.current = CurrentMenu::MainMenu
                }
                _ => (),
            },
            CurrentMenu::SoundtrackMenu => {
                let last = self.soundtrack.entries.len() - 1;
                match self.soundtrack.tick(controller, state) {
                    MenuSelectionResult::Selected(idx, entry) => {
                        if let (true, MenuEntry::Active(name)) = (idx != last, entry) {
                            state.settings.soundtrack = name.to_owned();
                            self.sound.entries[4] =
                                MenuEntry::Active(format!("Soundtrack: {}", state.settings.soundtrack));
                            state.sound_manager.reload_songs(&state.constants, &state.settings, ctx)?;
                        }

                        self.current = CurrentMenu::SoundMenu;
                    }
                    MenuSelectionResult::Canceled => {
                        self.current = CurrentMenu::SoundMenu;
                    }
                    _ => (),
                }
            }
        }
        Ok(())
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        match self.current {
            CurrentMenu::MainMenu => self.main.draw(state, ctx)?,
            CurrentMenu::GraphicsMenu => self.graphics.draw(state, ctx)?,
            CurrentMenu::SoundMenu => self.sound.draw(state, ctx)?,
            CurrentMenu::SoundtrackMenu => self.soundtrack.draw(state, ctx)?,
        }

        Ok(())
    }
}
