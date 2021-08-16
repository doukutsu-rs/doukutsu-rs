use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::menu::{Menu, MenuSelectionResult};
use crate::menu::MenuEntry;
use crate::shared_game_state::{SharedGameState, TimingMode};
use crate::sound::InterpolationMode;

#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
#[allow(unused)]
enum CurrentMenu {
    MainMenu,
    GraphicsMenu,
    SoundMenu,
}

pub struct SettingsMenu {
    current: CurrentMenu,
    main: Menu,
    graphics: Menu,
    sound: Menu,
}

static DISCORD_LINK: &str = "https://discord.gg/fbRsNNB";

impl SettingsMenu {
    pub fn new() -> SettingsMenu {
        let main = Menu::new(0, 0, 200, 0);
        let graphics = Menu::new(0, 0, 180, 0);
        let sound = Menu::new(0, 0, 260, 0);

        SettingsMenu { current: CurrentMenu::MainMenu, main, graphics, sound }
    }

    pub fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.graphics.push_entry(MenuEntry::Toggle("Lighting effects:".to_string(), state.settings.shader_effects));
        self.graphics
            .push_entry(MenuEntry::Toggle("Motion interpolation:".to_string(), state.settings.motion_interpolation));
        self.graphics.push_entry(MenuEntry::Toggle("Subpixel scrolling:".to_string(), state.settings.subpixel_coords));

        if state.constants.supports_og_textures {
            self.graphics
                .push_entry(MenuEntry::Toggle("Original textures".to_string(), state.settings.original_textures));
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
            if state.timing_mode == TimingMode::_50Hz { 0 } else { 1 },
            vec!["50tps (freeware)".to_owned(), "60tps (CS+)".to_owned()],
        ));

        self.main.push_entry(MenuEntry::Active(DISCORD_LINK.to_owned()));

        self.main.push_entry(MenuEntry::Active("< Back".to_owned()));

        self.sound.push_entry(MenuEntry::DisabledWhite("BGM Interpolation:".to_owned()));
        self.sound.push_entry(MenuEntry::Options(
            "".to_owned(),
            state.settings.organya_interpolation as usize,
            vec![
                "Nearest (fastest, lowest quality)".to_owned(),
                "Linear (fast, similar to freeware on Vista+)".to_owned(),
                "Cosine".to_owned(),
                "Cubic".to_owned(),
                "Polyphase (slowest, similar to freeware on XP)".to_owned()
            ],
        ));
        self.sound.push_entry(MenuEntry::Disabled(format!("Soundtrack: {}", state.settings.soundtrack)));
        self.sound.push_entry(MenuEntry::Active("< Back".to_owned()));

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
                        match state.timing_mode {
                            TimingMode::_50Hz => {
                                state.timing_mode = TimingMode::_60Hz;
                                *value = 1;
                            }
                            TimingMode::_60Hz => {
                                state.timing_mode = TimingMode::_50Hz;
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
                        state.settings.motion_interpolation = !state.settings.motion_interpolation;
                        let _ = state.settings.save(ctx);

                        *value = state.settings.motion_interpolation;
                    }
                }
                MenuSelectionResult::Selected(2, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.subpixel_coords = !state.settings.subpixel_coords;
                        let _ = state.settings.save(ctx);

                        *value = state.settings.subpixel_coords;
                    }
                }
                MenuSelectionResult::Selected(3, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.original_textures = !state.settings.original_textures;
                        state.reload_textures();
                        let _ = state.settings.save(ctx);

                        *value = state.settings.original_textures;
                    }
                }
                MenuSelectionResult::Selected(4, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        state.settings.seasonal_textures = !state.settings.seasonal_textures;
                        state.reload_textures();
                        let _ = state.settings.save(ctx);

                        *value = state.settings.seasonal_textures;
                    }
                }
                MenuSelectionResult::Selected(6, _) | MenuSelectionResult::Canceled => {
                    self.current = CurrentMenu::MainMenu
                }
                _ => (),
            },
            CurrentMenu::SoundMenu => match self.sound.tick(controller, state) {
                MenuSelectionResult::Selected(1, toggle) => {
                    if let MenuEntry::Options(_, value, _) = toggle {
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
                MenuSelectionResult::Selected(3, _) | MenuSelectionResult::Canceled => {
                    self.current = CurrentMenu::MainMenu
                }
                _ => (),
            }
        }
        Ok(())
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        match self.current {
            CurrentMenu::MainMenu => self.main.draw(state, ctx)?,
            CurrentMenu::GraphicsMenu => self.graphics.draw(state, ctx)?,
            CurrentMenu::SoundMenu => self.sound.draw(state, ctx)?,
        }

        Ok(())
    }
}
