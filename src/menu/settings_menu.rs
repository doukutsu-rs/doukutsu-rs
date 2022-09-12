use std::collections::HashMap;

use itertools::Itertools;

use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::menu::MenuEntry;
use crate::menu::{Menu, MenuSelectionResult};
use crate::scene::title_scene::TitleScene;
use crate::shared_game_state::{CutsceneSkipMode, ScreenShakeIntensity, SharedGameState, TimingMode, WindowMode};
use crate::sound::InterpolationMode;
use crate::{graphics, VSyncMode};

use super::controls_menu::ControlsMenu;

#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
#[allow(unused)]
enum CurrentMenu {
    MainMenu,
    GraphicsMenu,
    SoundMenu,
    ControlsMenu,
    SoundtrackMenu,
    LanguageMenu,
    BehaviorMenu,
    LinksMenu,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum MainMenuEntry {
    Graphics,
    Sound,
    Controls,
    Language,
    Behavior,
    Links,
    Back,
}

impl Default for MainMenuEntry {
    fn default() -> Self {
        MainMenuEntry::Graphics
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum GraphicsMenuEntry {
    WindowMode,
    VSyncMode,
    LightingEffects,
    WeaponLightCone,
    ScreenShake,
    MotionInterpolation,
    SubpixelScrolling,
    OriginalTextures,
    SeasonalTextures,
    Renderer,
    Back,
}

impl Default for GraphicsMenuEntry {
    fn default() -> Self {
        GraphicsMenuEntry::WindowMode
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum SoundMenuEntry {
    MusicVolume,
    EffectsVolume,
    BGMInterpolation,
    Soundtrack,
    Back,
}

impl Default for SoundMenuEntry {
    fn default() -> Self {
        SoundMenuEntry::MusicVolume
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum SoundtrackMenuEntry {
    Soundtrack(usize),
    Back,
}

impl Default for SoundtrackMenuEntry {
    fn default() -> Self {
        SoundtrackMenuEntry::Soundtrack(0)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum LanguageMenuEntry {
    Title,
    Language(String),
    Back,
}

impl Default for LanguageMenuEntry {
    fn default() -> Self {
        LanguageMenuEntry::Back
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum BehaviorMenuEntry {
    GameTiming,
    PauseOnFocusLoss,
    CutsceneSkipMode,
    Back,
}

impl Default for BehaviorMenuEntry {
    fn default() -> Self {
        BehaviorMenuEntry::GameTiming
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum LinksMenuEntry {
    Title,
    Link(&'static str),
    Back,
}

impl Default for LinksMenuEntry {
    fn default() -> Self {
        LinksMenuEntry::Link(DISCORD_LINK)
    }
}

pub struct SettingsMenu {
    current: CurrentMenu,
    main: Menu<MainMenuEntry>,
    graphics: Menu<GraphicsMenuEntry>,
    sound: Menu<SoundMenuEntry>,
    soundtrack: Menu<SoundtrackMenuEntry>,
    language: Menu<LanguageMenuEntry>,
    behavior: Menu<BehaviorMenuEntry>,
    links: Menu<LinksMenuEntry>,
    controls_menu: ControlsMenu,
    pub on_title: bool,
}

static DISCORD_LINK: &str = "https://discord.gg/fbRsNNB";
static GITHUB_LINK: &str = "https://github.com/doukutsu-rs/doukutsu-rs";
static DOCS_LINK: &str = "https://doukutsu-rs.gitbook.io/docs/";
static TRIBUTE_LINK: &str = "https://www.cavestory.org/";
static GENERAL_LINK: &str = "https://discord.gg/cavestory";
static MODDING_LINK: &str = "https://discord.gg/xRsWpz6";
static GETPLUS_LINK: &str = "https://www.nicalis.com/games/cavestory+";

impl SettingsMenu {
    pub fn new() -> SettingsMenu {
        let main = Menu::new(0, 0, 220, 0);
        let graphics = Menu::new(0, 0, 180, 0);
        let sound = Menu::new(0, 0, 260, 0);
        let soundtrack = Menu::new(0, 0, 260, 0);
        let language = Menu::new(0, 0, 120, 0);
        let behavior = Menu::new(0, 0, 220, 0);
        let links = Menu::new(0, 0, 220, 0);

        let controls_menu = ControlsMenu::new();

        SettingsMenu {
            current: CurrentMenu::MainMenu,
            main,
            graphics,
            sound,
            soundtrack,
            language,
            behavior,
            links,
            controls_menu,
            on_title: false,
        }
    }

    pub fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        #[cfg(not(target_os = "android"))]
        self.graphics.push_entry(
            GraphicsMenuEntry::WindowMode,
            MenuEntry::Options(
                state.t("menus.options_menu.graphics_menu.window_mode.entry"),
                state.settings.window_mode as usize,
                vec![
                    state.t("menus.options_menu.graphics_menu.window_mode.windowed"),
                    state.t("menus.options_menu.graphics_menu.window_mode.fullscreen"),
                ],
            ),
        );

        self.graphics.push_entry(
            GraphicsMenuEntry::VSyncMode,
            MenuEntry::DescriptiveOptions(
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
            ),
        );
        self.graphics.push_entry(
            GraphicsMenuEntry::LightingEffects,
            MenuEntry::Toggle(
                state.t("menus.options_menu.graphics_menu.lighting_effects"),
                state.settings.shader_effects,
            ),
        );
        self.graphics.push_entry(
            GraphicsMenuEntry::WeaponLightCone,
            MenuEntry::Toggle(state.t("menus.options_menu.graphics_menu.weapon_light_cone"), state.settings.light_cone),
        );
        self.graphics.push_entry(
            GraphicsMenuEntry::ScreenShake,
            MenuEntry::Options(
                state.t("menus.options_menu.graphics_menu.screen_shake.entry"),
                state.settings.screen_shake_intensity as usize,
                vec![
                    state.t("menus.options_menu.graphics_menu.screen_shake.full"),
                    state.t("menus.options_menu.graphics_menu.screen_shake.half"),
                    state.t("menus.options_menu.graphics_menu.screen_shake.off"),
                ],
            ),
        );
        self.graphics.push_entry(
            GraphicsMenuEntry::MotionInterpolation,
            MenuEntry::Toggle(
                state.t("menus.options_menu.graphics_menu.motion_interpolation"),
                state.settings.motion_interpolation,
            ),
        );
        self.graphics.push_entry(
            GraphicsMenuEntry::SubpixelScrolling,
            MenuEntry::Toggle(
                state.t("menus.options_menu.graphics_menu.subpixel_scrolling"),
                state.settings.subpixel_coords,
            ),
        );

        // NS version uses two different maps, therefore we can't dynamically switch between graphics presets.
        if state.constants.supports_og_textures {
            if !state.constants.is_switch || self.on_title {
                self.graphics.push_entry(
                    GraphicsMenuEntry::OriginalTextures,
                    MenuEntry::Toggle(
                        state.t("menus.options_menu.graphics_menu.original_textures"),
                        state.settings.original_textures,
                    ),
                );
            } else {
                self.graphics.push_entry(
                    GraphicsMenuEntry::OriginalTextures,
                    MenuEntry::Disabled(state.t("menus.options_menu.graphics_menu.original_textures")),
                );
            }
        }

        if state.constants.is_cs_plus {
            self.graphics.push_entry(
                GraphicsMenuEntry::SeasonalTextures,
                MenuEntry::Toggle(
                    state.t("menus.options_menu.graphics_menu.seasonal_textures"),
                    state.settings.seasonal_textures,
                ),
            );
        }

        self.graphics.push_entry(
            GraphicsMenuEntry::Renderer,
            MenuEntry::Disabled(format!(
                "{} {}",
                state.t("menus.options_menu.graphics_menu.renderer"),
                ctx.renderer.as_ref().unwrap().renderer_name()
            )),
        );

        self.graphics.push_entry(GraphicsMenuEntry::Back, MenuEntry::Active(state.t("common.back")));

        self.main.push_entry(MainMenuEntry::Graphics, MenuEntry::Active(state.t("menus.options_menu.graphics")));
        self.main.push_entry(MainMenuEntry::Sound, MenuEntry::Active(state.t("menus.options_menu.sound")));

        #[cfg(not(target_os = "android"))]
        self.main.push_entry(MainMenuEntry::Controls, MenuEntry::Active(state.t("menus.options_menu.controls")));

        self.language.push_entry(LanguageMenuEntry::Title, MenuEntry::Disabled(state.t("menus.options_menu.language")));

        for locale in &state.constants.locales {
            self.language
                .push_entry(LanguageMenuEntry::Language(locale.code.clone()), MenuEntry::Active(locale.name.clone()));
        }

        self.language.push_entry(LanguageMenuEntry::Back, MenuEntry::Active(state.t("common.back")));

        if self.on_title {
            self.main.push_entry(MainMenuEntry::Language, MenuEntry::Active(state.t("menus.options_menu.language")));
        }

        self.main.push_entry(MainMenuEntry::Behavior, MenuEntry::Active(state.t("menus.options_menu.behavior")));

        self.main.push_entry(MainMenuEntry::Links, MenuEntry::Active(state.t("menus.options_menu.links")));

        self.links.push_entry(LinksMenuEntry::Title, MenuEntry::Disabled(state.t("menus.options_menu.links")));
        self.links.push_entry(LinksMenuEntry::Link(DISCORD_LINK), MenuEntry::Active("doukutsu-rs Discord".to_owned()));
        self.links.push_entry(LinksMenuEntry::Link(GITHUB_LINK), MenuEntry::Active("doukutsu-rs GitHub".to_owned()));
        self.links.push_entry(LinksMenuEntry::Link(DOCS_LINK), MenuEntry::Active("doukutsu-rs Docs".to_owned()));
        self.links.push_entry(LinksMenuEntry::Link(TRIBUTE_LINK), MenuEntry::Active("Cave Story Tribute Website".to_owned()));
        self.links.push_entry(LinksMenuEntry::Link(GENERAL_LINK), MenuEntry::Active("Cave Story Discord".to_owned()));
        self.links.push_entry(LinksMenuEntry::Link(MODDING_LINK),MenuEntry::Active("Cave Story Modding Community".to_owned()));
        self.links.push_entry(LinksMenuEntry::Link(GETPLUS_LINK), MenuEntry::Active("Get Cave Story+".to_owned()));

        self.main.push_entry(MainMenuEntry::Back, MenuEntry::Active(state.t("common.back")));

        self.sound.push_entry(
            SoundMenuEntry::MusicVolume,
            MenuEntry::OptionsBar(state.t("menus.options_menu.sound_menu.music_volume"), state.settings.bgm_volume),
        );
        self.sound.push_entry(
            SoundMenuEntry::EffectsVolume,
            MenuEntry::OptionsBar(state.t("menus.options_menu.sound_menu.effects_volume"), state.settings.sfx_volume),
        );

        self.sound.push_entry(
            SoundMenuEntry::BGMInterpolation,
            MenuEntry::DescriptiveOptions(
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
            ),
        );
        self.sound.push_entry(
            SoundMenuEntry::Soundtrack,
            MenuEntry::Active(state.tt(
                "menus.options_menu.sound_menu.soundtrack",
                HashMap::from([("soundtrack".to_owned(), state.settings.soundtrack.to_owned())]),
            )),
        );
        self.sound.push_entry(SoundMenuEntry::Back, MenuEntry::Active(state.t("common.back")));

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

        for (idx, soundtrack) in soundtrack_entries.iter().enumerate() {
            self.soundtrack.push_entry(SoundtrackMenuEntry::Soundtrack(idx), MenuEntry::Active(soundtrack.to_string()));
        }

        self.soundtrack.width = soundtrack_entries
            .into_iter()
            .map(|str| state.font.text_width(str.chars(), &state.constants))
            .max_by(|a, b| a.abs().partial_cmp(&b.abs()).unwrap())
            .unwrap_or(self.soundtrack.width as f32) as u16
            + 32;

        self.soundtrack.push_entry(SoundtrackMenuEntry::Back, MenuEntry::Active(state.t("common.back")));

        self.behavior.push_entry(
            BehaviorMenuEntry::GameTiming,
            MenuEntry::Options(
                state.t("menus.options_menu.behavior_menu.game_timing.entry"),
                if state.settings.timing_mode == TimingMode::_50Hz { 0 } else { 1 },
                vec![
                    state.t("menus.options_menu.behavior_menu.game_timing.50tps"),
                    state.t("menus.options_menu.behavior_menu.game_timing.60tps"),
                ],
            ),
        );

        self.behavior.push_entry(
            BehaviorMenuEntry::PauseOnFocusLoss,
            MenuEntry::Toggle(
                state.t("menus.options_menu.behavior_menu.pause_on_focus_loss"),
                state.settings.pause_on_focus_loss,
            ),
        );

        self.behavior.push_entry(
            BehaviorMenuEntry::CutsceneSkipMode,
            MenuEntry::Options(
                state.t("menus.options_menu.behavior_menu.cutscene_skip_method.entry"),
                if state.settings.cutscene_skip_mode == CutsceneSkipMode::Hold { 0 } else { 1 },
                vec![
                    state.t("menus.options_menu.behavior_menu.cutscene_skip_method.hold"),
                    state.t("menus.options_menu.behavior_menu.cutscene_skip_method.fastforward"),
                ],
            ),
        );

        self.behavior.push_entry(BehaviorMenuEntry::Back, MenuEntry::Active(state.t("common.back")));

        self.links.push_entry(LinksMenuEntry::Back, MenuEntry::Active(state.t("common.back")));

        self.controls_menu.init(state, ctx)?;

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
        self.graphics.y = 20 + ((state.canvas_size.1 - self.graphics.height as f32) / 2.0).floor() as isize;

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

        self.behavior.update_width(state);
        self.behavior.update_height();
        self.behavior.x = ((state.canvas_size.0 - self.behavior.width as f32) / 2.0).floor() as isize;
        self.behavior.y = 30 + ((state.canvas_size.1 - self.behavior.height as f32) / 2.0).floor() as isize;

        self.links.update_width(state);
        self.links.update_height();
        self.links.x = ((state.canvas_size.0 - self.links.width as f32) / 2.0).floor() as isize;
        self.links.y = 30 + ((state.canvas_size.1 - self.links.height as f32) / 2.0).floor() as isize;
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
                MenuSelectionResult::Selected(MainMenuEntry::Graphics, _) => {
                    self.current = CurrentMenu::GraphicsMenu;
                }
                MenuSelectionResult::Selected(MainMenuEntry::Sound, _) => {
                    self.current = CurrentMenu::SoundMenu;
                }
                MenuSelectionResult::Selected(MainMenuEntry::Controls, _) => {
                    self.current = CurrentMenu::ControlsMenu;
                }
                MenuSelectionResult::Selected(MainMenuEntry::Language, _) => {
                    self.current = CurrentMenu::LanguageMenu;
                }
                MenuSelectionResult::Selected(MainMenuEntry::Behavior, _) => {
                    self.current = CurrentMenu::BehaviorMenu;
                }
                MenuSelectionResult::Selected(MainMenuEntry::Links, _) => {
                    self.current = CurrentMenu::LinksMenu;
                }
                MenuSelectionResult::Selected(MainMenuEntry::Back, _) | MenuSelectionResult::Canceled => exit_action(),
                _ => (),
            },
            CurrentMenu::GraphicsMenu => match self.graphics.tick(controller, state) {
                MenuSelectionResult::Selected(GraphicsMenuEntry::WindowMode, toggle)
                | MenuSelectionResult::Right(GraphicsMenuEntry::WindowMode, toggle, _)
                | MenuSelectionResult::Left(GraphicsMenuEntry::WindowMode, toggle, _) => {
                    if let MenuEntry::Options(_, value, _) = toggle {
                        let (new_mode, new_value) = match *value {
                            0 => (WindowMode::Fullscreen, 1),
                            1 => (WindowMode::Windowed, 0),
                            _ => unreachable!(),
                        };

                        *value = new_value;
                        state.settings.window_mode = new_mode;

                        let _ = state.settings.save(ctx);
                    }
                }
                MenuSelectionResult::Selected(GraphicsMenuEntry::VSyncMode, toggle)
                | MenuSelectionResult::Right(GraphicsMenuEntry::VSyncMode, toggle, _) => {
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
                MenuSelectionResult::Left(GraphicsMenuEntry::VSyncMode, toggle, _) => {
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
                MenuSelectionResult::Selected(GraphicsMenuEntry::LightingEffects, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.shader_effects = !state.settings.shader_effects;
                        let _ = state.settings.save(ctx);

                        *value = state.settings.shader_effects;
                    }
                }
                MenuSelectionResult::Selected(GraphicsMenuEntry::WeaponLightCone, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.light_cone = !state.settings.light_cone;
                        let _ = state.settings.save(ctx);

                        *value = state.settings.light_cone;
                    }
                }
                MenuSelectionResult::Selected(GraphicsMenuEntry::ScreenShake, toggle)
                | MenuSelectionResult::Right(GraphicsMenuEntry::ScreenShake, toggle, _) => {
                    if let MenuEntry::Options(_, value, _) = toggle {
                        let (new_intensity, new_value) = match *value {
                            0 => (ScreenShakeIntensity::Half, 1),
                            1 => (ScreenShakeIntensity::Off, 2),
                            _ => (ScreenShakeIntensity::Full, 0),
                        };

                        *value = new_value;
                        state.settings.screen_shake_intensity = new_intensity;

                        let _ = state.settings.save(ctx);
                    }
                }
                MenuSelectionResult::Left(GraphicsMenuEntry::ScreenShake, toggle, _) => {
                    if let MenuEntry::Options(_, value, _) = toggle {
                        let (new_intensity, new_value) = match *value {
                            0 => (ScreenShakeIntensity::Off, 2),
                            1 => (ScreenShakeIntensity::Full, 0),
                            _ => (ScreenShakeIntensity::Half, 1),
                        };

                        *value = new_value;
                        state.settings.screen_shake_intensity = new_intensity;

                        let _ = state.settings.save(ctx);
                    }
                }
                MenuSelectionResult::Selected(GraphicsMenuEntry::MotionInterpolation, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.motion_interpolation = !state.settings.motion_interpolation;
                        let _ = state.settings.save(ctx);

                        *value = state.settings.motion_interpolation;
                    }
                }
                MenuSelectionResult::Selected(GraphicsMenuEntry::SubpixelScrolling, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.subpixel_coords = !state.settings.subpixel_coords;
                        let _ = state.settings.save(ctx);

                        *value = state.settings.subpixel_coords;
                    }
                }
                MenuSelectionResult::Selected(GraphicsMenuEntry::OriginalTextures, toggle) => {
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
                MenuSelectionResult::Selected(GraphicsMenuEntry::SeasonalTextures, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.seasonal_textures = !state.settings.seasonal_textures;
                        state.reload_graphics();
                        let _ = state.settings.save(ctx);

                        *value = state.settings.seasonal_textures;
                    }
                }
                MenuSelectionResult::Selected(GraphicsMenuEntry::Back, _) | MenuSelectionResult::Canceled => {
                    self.current = CurrentMenu::MainMenu
                }
                _ => (),
            },
            CurrentMenu::SoundMenu => match self.sound.tick(controller, state) {
                MenuSelectionResult::Left(SoundMenuEntry::MusicVolume, bgm, direction)
                | MenuSelectionResult::Right(SoundMenuEntry::MusicVolume, bgm, direction) => {
                    if let MenuEntry::OptionsBar(_, value) = bgm {
                        *value = (*value * 10.0 + (direction as f32)).clamp(0.0, 10.0) / 10.0;
                        state.settings.bgm_volume = *value;
                        state.sound_manager.set_song_volume(*value);

                        let _ = state.settings.save(ctx);
                    }
                }
                MenuSelectionResult::Left(SoundMenuEntry::EffectsVolume, sfx, direction)
                | MenuSelectionResult::Right(SoundMenuEntry::EffectsVolume, sfx, direction) => {
                    if let MenuEntry::OptionsBar(_, value) = sfx {
                        *value = (*value * 10.0 + (direction as f32)).clamp(0.0, 10.0) / 10.0;
                        state.settings.sfx_volume = *value;
                        state.sound_manager.set_sfx_volume(*value);

                        let _ = state.settings.save(ctx);
                    }
                }
                 MenuSelectionResult::Selected(SoundMenuEntry::BGMInterpolation, toggle) 
                | MenuSelectionResult::Right(SoundMenuEntry::BGMInterpolation, toggle, _) => {
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
                MenuSelectionResult::Left(SoundMenuEntry::BGMInterpolation, toggle, _) => {
                    if let MenuEntry::DescriptiveOptions(_, value, _, _) = toggle {
                        let (new_mode, new_value) = match *value {
                            0 => (InterpolationMode::Polyphase, 4),
                            1 => (InterpolationMode::Nearest, 0),
                            2 => (InterpolationMode::Linear, 1),
                            3 => (InterpolationMode::Cosine, 2),
                            _ => (InterpolationMode::Cubic, 3),
                        };

                        *value = new_value;
                        state.settings.organya_interpolation = new_mode;
                        state.sound_manager.set_org_interpolation(new_mode);
                        let _ = state.settings.save(ctx);
                    }
                }
                MenuSelectionResult::Selected(SoundMenuEntry::Soundtrack, _) => {
                    let mut active_soundtrack = SoundtrackMenuEntry::Soundtrack(0);

                    for (id, entry) in &self.soundtrack.entries {
                        if let MenuEntry::Active(soundtrack) = entry {
                            if soundtrack == &state.settings.soundtrack {
                                active_soundtrack = *id;
                                let _ = state.settings.save(ctx);
                                break;
                            }
                        }
                    }

                    self.soundtrack.selected = active_soundtrack;

                    self.current = CurrentMenu::SoundtrackMenu
                }
                MenuSelectionResult::Selected(SoundMenuEntry::Back, _) | MenuSelectionResult::Canceled => {
                    self.current = CurrentMenu::MainMenu
                }
                _ => (),
            },
            CurrentMenu::ControlsMenu => {
                let cm = &mut self.current;
                self.controls_menu.tick(
                    &mut || {
                        *cm = CurrentMenu::MainMenu;
                    },
                    controller,
                    state,
                    ctx,
                )?;
            }
            CurrentMenu::LanguageMenu => match self.language.tick(controller, state) {
                MenuSelectionResult::Selected(LanguageMenuEntry::Language(new_locale), entry) => {
                    if let MenuEntry::Active(_) = entry {
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
                MenuSelectionResult::Selected(LanguageMenuEntry::Back, _) | MenuSelectionResult::Canceled => {
                    self.current = CurrentMenu::MainMenu;
                }
                _ => {}
            },
            CurrentMenu::SoundtrackMenu => match self.soundtrack.tick(controller, state) {
                MenuSelectionResult::Selected(SoundtrackMenuEntry::Soundtrack(_), entry) => {
                    if let MenuEntry::Active(name) = entry {
                        state.settings.soundtrack = name.to_owned();
                        let _ = state.settings.save(ctx);

                        self.sound.set_entry(
                            SoundMenuEntry::Soundtrack,
                            MenuEntry::Active(format!("Soundtrack: {}", state.settings.soundtrack)),
                        );
                        state.sound_manager.reload_songs(&state.constants, &state.settings, ctx)?;
                    }

                    self.current = CurrentMenu::SoundMenu;
                }
                MenuSelectionResult::Selected(SoundtrackMenuEntry::Back, _) | MenuSelectionResult::Canceled => {
                    self.current = CurrentMenu::SoundMenu;
                }
                _ => (),
            },
            CurrentMenu::BehaviorMenu => match self.behavior.tick(controller, state) {
                MenuSelectionResult::Selected(BehaviorMenuEntry::GameTiming, toggle) => {
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
                MenuSelectionResult::Selected(BehaviorMenuEntry::PauseOnFocusLoss, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.pause_on_focus_loss = !state.settings.pause_on_focus_loss;
                        let _ = state.settings.save(ctx);

                        *value = state.settings.pause_on_focus_loss;
                    }
                }
                MenuSelectionResult::Selected(BehaviorMenuEntry::CutsceneSkipMode, toggle) => {
                    if let MenuEntry::Options(_, value, _) = toggle {
                        match state.settings.cutscene_skip_mode {
                            CutsceneSkipMode::Hold => {
                                state.settings.cutscene_skip_mode = CutsceneSkipMode::FastForward;
                                *value = 1;
                            }
                            CutsceneSkipMode::FastForward => {
                                state.settings.cutscene_skip_mode = CutsceneSkipMode::Hold;
                                *value = 0;
                            }
                        }
                        let _ = state.settings.save(ctx);
                    }
                }
                MenuSelectionResult::Selected(BehaviorMenuEntry::Back, _) | MenuSelectionResult::Canceled => {
                    self.current = CurrentMenu::MainMenu;
                }
                _ => (),
            },
            CurrentMenu::LinksMenu => match self.links.tick(controller, state) {
                MenuSelectionResult::Selected(LinksMenuEntry::Link(url), _) => {
                    if let Err(e) = webbrowser::open(&url) {
                        log::warn!("Error opening web browser: {}", e);
                    }
                }
                MenuSelectionResult::Selected(LinksMenuEntry::Back, _) | MenuSelectionResult::Canceled => {
                    self.current = CurrentMenu::MainMenu;
                }
                _ => (),
            },
        }
        Ok(())
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        match self.current {
            CurrentMenu::MainMenu => self.main.draw(state, ctx)?,
            CurrentMenu::GraphicsMenu => self.graphics.draw(state, ctx)?,
            CurrentMenu::SoundMenu => self.sound.draw(state, ctx)?,
            CurrentMenu::SoundtrackMenu => self.soundtrack.draw(state, ctx)?,
            CurrentMenu::ControlsMenu => self.controls_menu.draw(state, ctx)?,
            CurrentMenu::LanguageMenu => self.language.draw(state, ctx)?,
            CurrentMenu::BehaviorMenu => self.behavior.draw(state, ctx)?,
            CurrentMenu::LinksMenu => self.links.draw(state, ctx)?,
        }

        Ok(())
    }
}
