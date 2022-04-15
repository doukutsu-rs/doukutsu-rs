use std::collections::HashMap;

use itertools::Itertools;

use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::menu::MenuEntry;
use crate::menu::{Menu, MenuSelectionResult};
use crate::scene::title_scene::TitleScene;
use crate::shared_game_state::{Language, SharedGameState, TimingMode};
use crate::sound::InterpolationMode;
use crate::{graphics, VSyncMode};

#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
#[allow(unused)]
enum CurrentMenu {
    MainMenu,
    GraphicsMenu,
    SoundMenu,
    SoundtrackMenu,
    LanguageMenu,
}

pub struct SettingsMenu {
    current: CurrentMenu,
    main: Menu,
    graphics: Menu,
    sound: Menu,
    soundtrack: Menu,
    language: Menu,
    pub on_title: bool,
}

static DISCORD_LINK: &str = "https://discord.gg/fbRsNNB";

impl SettingsMenu {
    pub fn new() -> SettingsMenu {
        let main = Menu::new(0, 0, 220, 0);
        let graphics = Menu::new(0, 0, 180, 0);
        let sound = Menu::new(0, 0, 260, 0);
        let soundtrack = Menu::new(0, 0, 260, 0);
        let language = Menu::new(0, 0, 120, 0);

        SettingsMenu { current: CurrentMenu::MainMenu, main, graphics, sound, soundtrack, language, on_title: false }
    }

    pub fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.graphics.push_entry(MenuEntry::DescriptiveOptions(
            state.t("menus.options_menu.graphics_menu.vsync_mode.entry"),
            state.settings.vsync_mode as usize,
            vec![
                state.t("menus.options_menu.graphics_menu.vsync_mode.uncapped"),
                state.t("menus.options_menu.graphics_menu.vsync_mode.vsync"),
                state.t("menus.options_menu.graphics_menu.vsync_mode.vrr_1x"),
                state.t("menus.options_menu.graphics_menu.vsync_mode.vrr_2x"),
                state.t("menus.options_menu.graphics_menu.vsync_mode.vrr_3x"),
            ],
            vec![
                state.t("menus.options_menu.graphics_menu.vsync_mode.uncapped_desc"),
                state.t("menus.options_menu.graphics_menu.vsync_mode.vsync_desc"),
                state.t("menus.options_menu.graphics_menu.vsync_mode.vrr_1x_desc"),
                state.t("menus.options_menu.graphics_menu.vsync_mode.vrr_2x_desc"),
                state.t("menus.options_menu.graphics_menu.vsync_mode.vrr_3x_desc"),
            ],
        ));
        self.graphics.push_entry(MenuEntry::Toggle(
            state.t("menus.options_menu.graphics_menu.lighting_effects"),
            state.settings.shader_effects,
        ));
        self.graphics.push_entry(MenuEntry::Toggle(
            state.t("menus.options_menu.graphics_menu.weapon_light_cone"),
            state.settings.light_cone,
        ));
        self.graphics.push_entry(MenuEntry::Toggle(
            state.t("menus.options_menu.graphics_menu.motion_interpolation"),
            state.settings.motion_interpolation,
        ));
        self.graphics.push_entry(MenuEntry::Toggle(
            state.t("menus.options_menu.graphics_menu.subpixel_scrolling"),
            state.settings.subpixel_coords,
        ));

        // NS version uses two different maps, therefore we can't dynamically switch between graphics presets.
        if state.constants.supports_og_textures {
            if !state.constants.is_switch || self.on_title {
                self.graphics.push_entry(MenuEntry::Toggle(
                    state.t("menus.options_menu.graphics_menu.original_textures"),
                    state.settings.original_textures,
                ));
            } else {
                self.graphics
                    .push_entry(MenuEntry::Disabled(state.t("menus.options_menu.graphics_menu.original_textures")));
            }
        } else {
            self.graphics.push_entry(MenuEntry::Hidden);
        }

        if state.constants.is_cs_plus {
            self.graphics.push_entry(MenuEntry::Toggle(
                state.t("menus.options_menu.graphics_menu.seasonal_textures"),
                state.settings.seasonal_textures,
            ));
        } else {
            self.graphics.push_entry(MenuEntry::Hidden);
        }

        self.graphics.push_entry(MenuEntry::Disabled(format!(
            "{} {}",
            state.t("menus.options_menu.graphics_menu.renderer"),
            ctx.renderer.as_ref().unwrap().renderer_name()
        )));

        self.graphics.push_entry(MenuEntry::Active(state.t("common.back")));

        self.main.push_entry(MenuEntry::Active(state.t("menus.options_menu.graphics")));
        self.main.push_entry(MenuEntry::Active(state.t("menus.options_menu.sound")));

        self.language.push_entry(MenuEntry::Disabled(state.t("menus.options_menu.language")));
        for language in Language::values() {
            self.language.push_entry(MenuEntry::Active(language.to_string()));
        }
        self.language.push_entry(MenuEntry::Active(state.t("common.back")));

        if self.on_title {
            self.main.push_entry(MenuEntry::Active(state.t("menus.options_menu.language")));
        } else {
            self.main.push_entry(MenuEntry::Disabled(state.t("menus.options_menu.language")));
        }

        self.main.push_entry(MenuEntry::Options(
            state.t("menus.options_menu.game_timing.entry"),
            if state.settings.timing_mode == TimingMode::_50Hz { 0 } else { 1 },
            vec![state.t("menus.options_menu.game_timing.50tps"), state.t("menus.options_menu.game_timing.60tps")],
        ));

        self.main.push_entry(MenuEntry::Active(DISCORD_LINK.to_owned()));

        self.main.push_entry(MenuEntry::Active(state.t("common.back")));

        self.sound.push_entry(MenuEntry::OptionsBar(
            state.t("menus.options_menu.sound_menu.music_volume"),
            state.settings.bgm_volume,
        ));
        self.sound.push_entry(MenuEntry::OptionsBar(
            state.t("menus.options_menu.sound_menu.effects_volume"),
            state.settings.sfx_volume,
        ));

        self.sound.push_entry(MenuEntry::DescriptiveOptions(
            state.t("menus.options_menu.sound_menu.bgm_interpolation.entry"),
            state.settings.organya_interpolation as usize,
            vec![
                state.t("menus.options_menu.sound_menu.bgm_interpolation.nearest"),
                state.t("menus.options_menu.sound_menu.bgm_interpolation.linear"),
                state.t("menus.options_menu.sound_menu.bgm_interpolation.cosine"),
                state.t("menus.options_menu.sound_menu.bgm_interpolation.cubic"),
                state.t("menus.options_menu.sound_menu.bgm_interpolation.linear_lp"),
            ],
            vec![
                state.t("menus.options_menu.sound_menu.bgm_interpolation.nearest_desc"),
                state.t("menus.options_menu.sound_menu.bgm_interpolation.linear_desc"),
                state.t("menus.options_menu.sound_menu.bgm_interpolation.cosine_desc"),
                state.t("menus.options_menu.sound_menu.bgm_interpolation.cubic_desc"),
                state.t("menus.options_menu.sound_menu.bgm_interpolation.linear_lp_desc"),
            ],
        ));
        self.sound.push_entry(MenuEntry::DisabledWhite("".to_owned()));
        self.sound.push_entry(MenuEntry::Active(state.tt(
            "menus.options_menu.sound_menu.soundtrack",
            HashMap::from([("soundtrack".to_owned(), state.settings.soundtrack.to_owned())]),
        )));
        self.sound.push_entry(MenuEntry::Active(state.t("common.back")));

        let mut soundtrack_entries =
            state.constants.soundtracks.iter().filter(|s| s.available).map(|s| s.name.to_owned()).collect_vec();
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

        for soundtrack in &soundtrack_entries {
            self.soundtrack.push_entry(MenuEntry::Active(soundtrack.to_string()));
        }

        self.soundtrack.width = soundtrack_entries
            .into_iter()
            .map(|str| state.font.text_width(str.chars(), &state.constants))
            .max_by(|a, b| a.abs().partial_cmp(&b.abs()).unwrap())
            .unwrap_or(self.soundtrack.width as f32) as u16
            + 32;

        self.soundtrack.push_entry(MenuEntry::Active(state.t("common.back")));

        self.update_sizes(state);

        Ok(())
    }

    fn update_sizes(&mut self, state: &SharedGameState) {
        self.main.update_width(state);
        self.main.update_height();
        self.main.x = ((state.canvas_size.0 - self.main.width as f32) / 2.0).floor() as isize;
        self.main.y = 30 + ((state.canvas_size.1 - self.main.height as f32) / 2.0).floor() as isize;

        self.graphics.update_width(state);
        self.graphics.update_height();
        self.graphics.x = ((state.canvas_size.0 - self.graphics.width as f32) / 2.0).floor() as isize;
        self.graphics.y = 30 + ((state.canvas_size.1 - self.graphics.height as f32) / 2.0).floor() as isize;

        self.sound.update_width(state);
        self.sound.update_height();
        self.sound.x = ((state.canvas_size.0 - self.sound.width as f32) / 2.0).floor() as isize;
        self.sound.y = 30 + ((state.canvas_size.1 - self.sound.height as f32) / 2.0).floor() as isize;

        self.soundtrack.update_width(state);
        self.soundtrack.update_height();
        self.soundtrack.x = ((state.canvas_size.0 - self.soundtrack.width as f32) / 2.0).floor() as isize;
        self.soundtrack.y = ((state.canvas_size.1 - self.soundtrack.height as f32) / 2.0).floor() as isize;

        self.language.update_width(state);
        self.language.update_height();
        self.language.x = ((state.canvas_size.0 - self.language.width as f32) / 2.0).floor() as isize;
        self.language.y = ((state.canvas_size.1 - self.language.height as f32) / 2.0).floor() as isize;
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
                MenuSelectionResult::Selected(2, _) => {
                    self.language.selected = (state.settings.locale as usize) + 1;
                    self.current = CurrentMenu::LanguageMenu;
                }
                MenuSelectionResult::Selected(3, toggle) => {
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
                MenuSelectionResult::Selected(4, _) => {
                    if let Err(e) = webbrowser::open(DISCORD_LINK) {
                        log::warn!("Error opening web browser: {}", e);
                    }
                }
                MenuSelectionResult::Selected(5, _) | MenuSelectionResult::Canceled => exit_action(),
                _ => (),
            },
            CurrentMenu::GraphicsMenu => match self.graphics.tick(controller, state) {
                MenuSelectionResult::Selected(0, toggle) | MenuSelectionResult::Right(0, toggle, _) => {
                    if let MenuEntry::DescriptiveOptions(_, value, _, _) = toggle {
                        let (new_mode, new_value) = match *value {
                            0 => (VSyncMode::VSync, 1),
                            1 => (VSyncMode::VRRTickSync1x, 2),
                            2 => (VSyncMode::VRRTickSync2x, 3),
                            3 => (VSyncMode::VRRTickSync3x, 4),
                            _ => (VSyncMode::Uncapped, 0),
                        };

                        *value = new_value;
                        state.settings.vsync_mode = new_mode;
                        graphics::set_vsync_mode(ctx, new_mode)?;

                        let _ = state.settings.save(ctx);
                    }
                }
                MenuSelectionResult::Left(0, toggle, _) => {
                    if let MenuEntry::DescriptiveOptions(_, value, _, _) = toggle {
                        let (new_mode, new_value) = match *value {
                            0 => (VSyncMode::VRRTickSync3x, 4),
                            1 => (VSyncMode::Uncapped, 0),
                            2 => (VSyncMode::VSync, 1),
                            3 => (VSyncMode::VRRTickSync1x, 2),
                            _ => (VSyncMode::VRRTickSync2x, 3),
                        };

                        *value = new_value;
                        state.settings.vsync_mode = new_mode;
                        graphics::set_vsync_mode(ctx, new_mode)?;

                        let _ = state.settings.save(ctx);
                    }
                }
                MenuSelectionResult::Selected(1, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.shader_effects = !state.settings.shader_effects;
                        let _ = state.settings.save(ctx);

                        *value = state.settings.shader_effects;
                    }
                }
                MenuSelectionResult::Selected(2, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.light_cone = !state.settings.light_cone;
                        let _ = state.settings.save(ctx);

                        *value = state.settings.light_cone;
                    }
                }
                MenuSelectionResult::Selected(3, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.motion_interpolation = !state.settings.motion_interpolation;
                        let _ = state.settings.save(ctx);

                        *value = state.settings.motion_interpolation;
                    }
                }
                MenuSelectionResult::Selected(4, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.subpixel_coords = !state.settings.subpixel_coords;
                        let _ = state.settings.save(ctx);

                        *value = state.settings.subpixel_coords;
                    }
                }
                MenuSelectionResult::Selected(5, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.original_textures = !state.settings.original_textures;
                        if self.on_title {
                            state.reload_resources(ctx)?;
                        } else {
                            state.reload_graphics();
                        }
                        let _ = state.settings.save(ctx);

                        *value = state.settings.original_textures;
                    }
                }
                MenuSelectionResult::Selected(6, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.seasonal_textures = !state.settings.seasonal_textures;
                        state.reload_graphics();
                        let _ = state.settings.save(ctx);

                        *value = state.settings.seasonal_textures;
                    }
                }
                MenuSelectionResult::Selected(8, _) | MenuSelectionResult::Canceled => {
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
                MenuSelectionResult::Selected(4, _) => {
                    let mut active_soundtrack_index = 0;

                    for (idx, entry) in self.soundtrack.entries.iter().enumerate() {
                        if let MenuEntry::Active(soundtrack) = entry {
                            if soundtrack == &state.settings.soundtrack {
                                active_soundtrack_index = idx;
                                let _ = state.settings.save(ctx);
                                break;
                            }
                        }
                    }

                    self.soundtrack.selected = active_soundtrack_index;

                    self.current = CurrentMenu::SoundtrackMenu
                }
                MenuSelectionResult::Selected(5, _) | MenuSelectionResult::Canceled => {
                    self.current = CurrentMenu::MainMenu
                }
                _ => (),
            },
            CurrentMenu::LanguageMenu => {
                let last = self.language.entries.len() - 1;

                match self.language.tick(controller, state) {
                    MenuSelectionResult::Selected(idx, entry) => {
                        if let (true, MenuEntry::Active(_)) = (idx != last, entry) {
                            let new_locale = Language::from_primitive(idx.saturating_sub(1));
                            if new_locale == state.settings.locale {
                                self.current = CurrentMenu::MainMenu;
                            } else {
                                state.settings.locale = new_locale;
                                state.reload_fonts(ctx);

                                let _ = state.settings.save(ctx);

                                let mut new_menu = TitleScene::new();
                                new_menu.open_settings_menu()?;
                                state.next_scene = Some(Box::new(new_menu));
                            }
                        }

                        self.current = CurrentMenu::MainMenu;
                    }
                    MenuSelectionResult::Canceled => {
                        self.current = CurrentMenu::MainMenu;
                    }
                    _ => {}
                }
            }
            CurrentMenu::SoundtrackMenu => {
                let last = self.soundtrack.entries.len() - 1;
                match self.soundtrack.tick(controller, state) {
                    MenuSelectionResult::Selected(idx, entry) => {
                        if let (true, MenuEntry::Active(name)) = (idx != last, entry) {
                            state.settings.soundtrack = name.to_owned();
                            let _ = state.settings.save(ctx);
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
            CurrentMenu::LanguageMenu => self.language.draw(state, ctx)?,
        }

        Ok(())
    }
}
