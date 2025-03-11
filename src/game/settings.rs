use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem::{user_create, user_open};
use crate::framework::gamepad::{Axis, AxisDirection, Button, PlayerControllerInputType};
use crate::framework::graphics::VSyncMode;
use crate::framework::keyboard::ScanCode;
use crate::game::profile::SaveFormat;
use crate::game::player::TargetPlayer;
use crate::game::shared_game_state::{CutsceneSkipMode, ScreenShakeIntensity, TimingMode, WindowMode};
use crate::input::combined_player_controller::CombinedPlayerController;
use crate::input::gamepad_player_controller::GamepadController;
use crate::input::keyboard_player_controller::KeyboardController;
use crate::input::player_controller::PlayerController;
use crate::input::touch_player_controller::TouchPlayerController;
use crate::sound::InterpolationMode;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Settings {
    #[serde(default = "current_version")]
    pub version: u32,
    #[serde(default = "default_true")]
    pub seasonal_textures: bool,
    pub original_textures: bool,
    pub shader_effects: bool,
    #[serde(default = "default_true")]
    pub light_cone: bool,
    #[serde(default = "default_true")]
    pub subpixel_coords: bool,
    #[serde(default = "default_true")]
    pub motion_interpolation: bool,
    pub touch_controls: bool,
    #[serde(default = "default_true")]
    pub display_touch_controls: bool,
    pub soundtrack: String,
    #[serde(default = "default_vol")]
    pub bgm_volume: f32,
    #[serde(default = "default_vol")]
    pub sfx_volume: f32,
    #[serde(default = "default_timing")]
    pub timing_mode: TimingMode,
    #[serde(default = "default_pause_on_focus_loss")]
    pub pause_on_focus_loss: bool,
    #[serde(default = "default_interpolation")]
    pub organya_interpolation: InterpolationMode,
    #[serde(default = "default_p1_controller_type")]
    pub player1_controller_type: ControllerType,
    #[serde(default = "default_p2_controller_type")]
    pub player2_controller_type: ControllerType,
    #[serde(default = "p1_default_keymap")]
    pub player1_key_map: PlayerKeyMap,
    #[serde(default = "p2_default_keymap")]
    pub player2_key_map: PlayerKeyMap,
    #[serde(default = "player_default_controller_button_map")]
    pub player1_controller_button_map: PlayerControllerButtonMap,
    #[serde(default = "player_default_controller_button_map")]
    pub player2_controller_button_map: PlayerControllerButtonMap,
    #[serde(default = "default_controller_axis_sensitivity")]
    pub player1_controller_axis_sensitivity: f64,
    #[serde(default = "default_controller_axis_sensitivity")]
    pub player2_controller_axis_sensitivity: f64,
    #[serde(default = "default_rumble")]
    pub player1_rumble: bool,
    #[serde(default = "default_rumble")]
    pub player2_rumble: bool,
    #[serde(skip, default = "default_speed")]
    pub speed: f64,
    #[serde(skip)]
    pub god_mode: bool,
    #[serde(skip)]
    pub infinite_booster: bool,
    #[serde(skip)]
    pub debug_outlines: bool,
    pub fps_counter: bool,
    pub locale: String,
    #[serde(default = "default_window_mode")]
    pub window_mode: WindowMode,
    #[serde(default = "default_vsync")]
    pub vsync_mode: VSyncMode,
    #[serde(default = "default_screen_shake_intensity")]
    pub screen_shake_intensity: ScreenShakeIntensity,
    pub debug_mode: bool,
    #[serde(skip)]
    pub noclip: bool,
    pub more_rust: bool,
    #[serde(default = "default_cutscene_skip_mode")]
    pub cutscene_skip_mode: CutsceneSkipMode,
    #[serde(default = "default_true")]
    pub discord_rpc: bool,
    #[serde(default = "default_true")]
    pub allow_strafe: bool,
    #[serde(default = "default_save_format")]
    pub save_format: SaveFormat,
}

fn default_true() -> bool {
    true
}

#[inline(always)]
fn current_version() -> u32 {
    26
}

#[inline(always)]
fn default_timing() -> TimingMode {
    TimingMode::_50Hz
}

#[inline(always)]
fn default_window_mode() -> WindowMode {
    WindowMode::Windowed
}

#[inline(always)]
fn default_interpolation() -> InterpolationMode {
    InterpolationMode::Linear
}

#[inline(always)]
fn default_speed() -> f64 {
    1.0
}

#[inline(always)]
fn default_vol() -> f32 {
    1.0
}

#[inline(always)]
fn default_locale() -> String {
    "en".to_string()
}

#[inline(always)]
fn default_vsync() -> VSyncMode {
    VSyncMode::VSync
}

#[inline(always)]
fn default_screen_shake_intensity() -> ScreenShakeIntensity {
    ScreenShakeIntensity::Full
}

#[inline(always)]
fn default_p1_controller_type() -> ControllerType {
    if cfg!(any(target_os = "horizon")) {
        ControllerType::Gamepad(0)
    } else {
        ControllerType::Keyboard
    }
}

#[inline(always)]
fn default_p2_controller_type() -> ControllerType {
    if cfg!(any(target_os = "horizon")) {
        ControllerType::Gamepad(1)
    } else {
        ControllerType::Keyboard
    }
}

#[inline(always)]
fn default_pause_on_focus_loss() -> bool {
    true
}

#[inline(always)]
fn default_rumble() -> bool {
    false
}

#[inline(always)]
fn default_cutscene_skip_mode() -> CutsceneSkipMode {
    CutsceneSkipMode::Hold
}

#[inline(always)]
fn default_save_format() -> SaveFormat {
    SaveFormat::Freeware
}

impl Settings {
    pub fn load(ctx: &Context) -> GameResult<Settings> {
        if let Ok(file) = user_open(ctx, "/settings.json") {
            match serde_json::from_reader::<_, Settings>(file) {
                Ok(settings) => return Ok(settings.upgrade()),
                Err(err) => log::warn!("Failed to deserialize settings: {}", err),
            }
        }

        Ok(Settings::default())
    }

    fn upgrade(mut self) -> Self {
        let initial_version = self.version;

        if self.version == 2 {
            self.version = 3;
            self.light_cone = true;
        }

        if self.version == 3 {
            self.version = 4;
            self.timing_mode = default_timing();
        }

        if self.version == 4 {
            self.version = 5;
            self.bgm_volume = default_vol();
            self.sfx_volume = default_vol();
        }

        if self.version == 5 {
            self.version = 6;
            self.player1_key_map.strafe = ScanCode::LShift;
            self.player2_key_map.strafe = ScanCode::RShift;
        }

        if self.version == 6 {
            self.version = 7;
            self.locale = default_locale();
        }

        if self.version == 7 {
            self.version = 8;
            self.vsync_mode = default_vsync();
        }

        if self.version == 8 {
            self.version = 9;
            self.debug_mode = false;
        }

        if self.version == 9 {
            self.version = 10;
            self.screen_shake_intensity = default_screen_shake_intensity();
        }

        if self.version == 10 {
            self.version = 11;
            self.window_mode = default_window_mode();
        }

        if self.version == 11 {
            self.version = 12;
            self.player1_controller_type = default_p1_controller_type();
            self.player2_controller_type = default_p2_controller_type();
            self.player1_controller_button_map = player_default_controller_button_map();
            self.player2_controller_button_map = player_default_controller_button_map();
            self.player1_controller_axis_sensitivity = default_controller_axis_sensitivity();
            self.player2_controller_axis_sensitivity = default_controller_axis_sensitivity();
        }

        if self.version == 12 {
            self.version = 13;

            if self.player1_key_map.skip == ScanCode::E {
                self.player1_key_map.skip = ScanCode::Q;
            }

            if self.player2_key_map.skip == ScanCode::U {
                self.player2_key_map.skip = ScanCode::T;
            }

            // reset controller mappings since we've updated enums
            self.player1_controller_button_map = player_default_controller_button_map();
            self.player2_controller_button_map = player_default_controller_button_map();
        }

        if self.version == 13 {
            self.version = 14;

            // reset controller mappings again since we have new enums
            self.player1_controller_button_map = player_default_controller_button_map();
            self.player2_controller_button_map = player_default_controller_button_map();
        }

        if self.version == 14 {
            self.version = 15;
            self.pause_on_focus_loss = default_pause_on_focus_loss();
        }

        if self.version == 15 {
            self.version = 16;

            self.player1_key_map.menu_ok = self.player1_key_map.jump;
            self.player1_key_map.menu_back = self.player1_key_map.shoot;
            self.player1_controller_button_map.menu_ok = self.player1_controller_button_map.jump;
            self.player1_controller_button_map.menu_back = self.player1_controller_button_map.shoot;

            self.player2_key_map.menu_ok = self.player2_key_map.jump;
            self.player2_key_map.menu_back = self.player2_key_map.shoot;
            self.player2_controller_button_map.menu_ok = self.player2_controller_button_map.jump;
            self.player2_controller_button_map.menu_back = self.player2_controller_button_map.shoot;
        }

        if self.version == 16 {
            self.version = 17;

            if self.player1_controller_button_map.shoot == PlayerControllerInputType::ButtonInput(Button::East) {
                self.player1_controller_button_map.shoot = PlayerControllerInputType::ButtonInput(Button::West);
            }

            if self.player2_controller_button_map.shoot == PlayerControllerInputType::ButtonInput(Button::East) {
                self.player2_controller_button_map.shoot = PlayerControllerInputType::ButtonInput(Button::West);
            }

            if self.player1_controller_button_map.map == PlayerControllerInputType::ButtonInput(Button::West) {
                self.player1_controller_button_map.map = PlayerControllerInputType::ButtonInput(Button::East);
            }

            if self.player2_controller_button_map.map == PlayerControllerInputType::ButtonInput(Button::West) {
                self.player2_controller_button_map.map = PlayerControllerInputType::ButtonInput(Button::East);
            }
        }

        if self.version == 17 {
            self.version = 18;
            self.player1_rumble = default_rumble();
            self.player2_rumble = default_rumble();
        }

        if self.version == 18 {
            self.version = 19;
            self.more_rust = false;
        }

        if self.version == 19 {
            self.version = 20;
            self.cutscene_skip_mode = CutsceneSkipMode::Hold;
        }

        if self.version == 20 {
            self.version = 21;

            self.locale = match self.locale.as_str() {
                "English" => "en".to_string(),
                "Japanese" => "jp".to_string(),
                _ => default_locale(),
            };
        }

        if self.version == 21 {
            self.version = 22;
            self.discord_rpc = true;
        }

        if self.version == 22 {
            self.version = 23;
            self.display_touch_controls = true;
        }

        if self.version == 23 {
            self.version = 24;
            self.allow_strafe = true;
        }

        if self.version == 24 {
            self.version = 25;
            self.soundtrack = match self.soundtrack.as_str() {
                "Organya" => "organya".to_owned(),
                "Remastered" => "remastered".to_owned(),
                "New" => "new".to_owned(),
                "Famitracks" => "famitracks".to_owned(),
                "Ridiculon" => "ridiculon".to_owned(),
                _ => self.soundtrack.clone(),
            }
        }

        if self.version == 25 {
            self.version = 26;
            self.save_format = default_save_format();
        }

        if self.version != initial_version {
            log::info!("Upgraded configuration file from version {} to {}.", initial_version, self.version);
        }

        self
    }

    pub fn save(&self, ctx: &Context) -> GameResult {
        let file = user_create(ctx, "/settings.json")?;
        serde_json::to_writer_pretty(file, self)?;

        Ok(())
    }

    pub fn create_player1_controller(&self) -> Box<dyn PlayerController> {
        if self.touch_controls {
            return Box::new(TouchPlayerController::new());
        }

        match self.player1_controller_type {
            ControllerType::Keyboard => Box::new(KeyboardController::new(TargetPlayer::Player1)),
            ControllerType::Gamepad(index) => {
                let keyboard_controller = Box::new(KeyboardController::new(TargetPlayer::Player1));

                let mut gamepad_controller = Box::new(GamepadController::new(index, TargetPlayer::Player1));
                gamepad_controller.set_rumble_enabled(self.player1_rumble);

                let mut combined_player_controller = CombinedPlayerController::new();
                combined_player_controller.add(keyboard_controller);
                combined_player_controller.add(gamepad_controller);

                Box::new(combined_player_controller)
            }
        }
    }

    pub fn create_player2_controller(&self) -> Box<dyn PlayerController> {
        match self.player2_controller_type {
            ControllerType::Keyboard => Box::new(KeyboardController::new(TargetPlayer::Player2)),
            ControllerType::Gamepad(index) => {
                let keyboard_controller = Box::new(KeyboardController::new(TargetPlayer::Player2));

                let mut gamepad_controller = Box::new(GamepadController::new(index, TargetPlayer::Player2));
                gamepad_controller.set_rumble_enabled(self.player2_rumble);

                let mut combined_player_controller = CombinedPlayerController::new();
                combined_player_controller.add(keyboard_controller);
                combined_player_controller.add(gamepad_controller);

                Box::new(combined_player_controller)
            }
        }
    }

    pub fn get_gamepad_axis_sensitivity(&self, id: u32) -> f64 {
        if self.player1_controller_type == ControllerType::Gamepad(id) {
            self.player1_controller_axis_sensitivity
        } else if self.player2_controller_type == ControllerType::Gamepad(id) {
            self.player2_controller_axis_sensitivity
        } else {
            default_controller_axis_sensitivity()
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            version: current_version(),
            seasonal_textures: true,
            original_textures: false,
            shader_effects: false,
            light_cone: true,
            subpixel_coords: true,
            motion_interpolation: true,
            touch_controls: cfg!(target_os = "android"),
            display_touch_controls: true,
            soundtrack: "Organya".to_string(),
            bgm_volume: 1.0,
            sfx_volume: 1.0,
            timing_mode: default_timing(),
            pause_on_focus_loss: default_pause_on_focus_loss(),
            organya_interpolation: InterpolationMode::Linear,
            player1_controller_type: default_p1_controller_type(),
            player2_controller_type: default_p2_controller_type(),
            player1_key_map: p1_default_keymap(),
            player2_key_map: p2_default_keymap(),
            player1_controller_button_map: player_default_controller_button_map(),
            player2_controller_button_map: player_default_controller_button_map(),
            player1_controller_axis_sensitivity: default_controller_axis_sensitivity(),
            player2_controller_axis_sensitivity: default_controller_axis_sensitivity(),
            player1_rumble: default_rumble(),
            player2_rumble: default_rumble(),
            speed: 1.0,
            god_mode: false,
            infinite_booster: false,
            debug_outlines: false,
            fps_counter: false,
            locale: default_locale(),
            window_mode: WindowMode::Windowed,
            vsync_mode: VSyncMode::VSync,
            screen_shake_intensity: ScreenShakeIntensity::Full,
            debug_mode: false,
            noclip: false,
            more_rust: false,
            cutscene_skip_mode: CutsceneSkipMode::Hold,
            discord_rpc: true,
            allow_strafe: true,
            save_format: default_save_format()
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PlayerKeyMap {
    pub left: ScanCode,
    pub up: ScanCode,
    pub right: ScanCode,
    pub down: ScanCode,
    pub prev_weapon: ScanCode,
    pub next_weapon: ScanCode,
    pub jump: ScanCode,
    pub shoot: ScanCode,
    pub skip: ScanCode,
    pub inventory: ScanCode,
    pub map: ScanCode,
    pub strafe: ScanCode,
    pub menu_ok: ScanCode,
    pub menu_back: ScanCode,
}

#[inline(always)]
pub fn p1_default_keymap() -> PlayerKeyMap {
    PlayerKeyMap {
        left: ScanCode::Left,
        up: ScanCode::Up,
        right: ScanCode::Right,
        down: ScanCode::Down,
        prev_weapon: ScanCode::A,
        next_weapon: ScanCode::S,
        jump: ScanCode::Z,
        shoot: ScanCode::X,
        skip: ScanCode::Q,
        inventory: ScanCode::Q,
        map: ScanCode::W,
        strafe: ScanCode::LShift,
        menu_ok: ScanCode::Z,
        menu_back: ScanCode::X,
    }
}

#[inline(always)]
pub fn p2_default_keymap() -> PlayerKeyMap {
    PlayerKeyMap {
        left: ScanCode::Comma,
        up: ScanCode::L,
        right: ScanCode::Slash,
        down: ScanCode::Period,
        prev_weapon: ScanCode::G,
        next_weapon: ScanCode::H,
        jump: ScanCode::B,
        shoot: ScanCode::N,
        skip: ScanCode::T,
        inventory: ScanCode::T,
        map: ScanCode::Y,
        strafe: ScanCode::RShift,
        menu_ok: ScanCode::B,
        menu_back: ScanCode::N,
    }
}

#[derive(serde::Serialize, serde::Deserialize, Eq, PartialEq, Copy, Clone)]
pub enum ControllerType {
    Keyboard,
    Gamepad(u32),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PlayerControllerButtonMap {
    pub left: PlayerControllerInputType,
    pub up: PlayerControllerInputType,
    pub right: PlayerControllerInputType,
    pub down: PlayerControllerInputType,
    pub prev_weapon: PlayerControllerInputType,
    pub next_weapon: PlayerControllerInputType,
    pub jump: PlayerControllerInputType,
    pub shoot: PlayerControllerInputType,
    pub skip: PlayerControllerInputType,
    pub inventory: PlayerControllerInputType,
    pub map: PlayerControllerInputType,
    pub strafe: PlayerControllerInputType,
    pub menu_ok: PlayerControllerInputType,
    pub menu_back: PlayerControllerInputType,
}

#[inline(always)]
pub fn player_default_controller_button_map() -> PlayerControllerButtonMap {
    PlayerControllerButtonMap {
        left: PlayerControllerInputType::Either(Button::DPadLeft, Axis::LeftX, AxisDirection::Left),
        up: PlayerControllerInputType::Either(Button::DPadUp, Axis::LeftY, AxisDirection::Up),
        right: PlayerControllerInputType::Either(Button::DPadRight, Axis::LeftX, AxisDirection::Right),
        down: PlayerControllerInputType::Either(Button::DPadDown, Axis::LeftY, AxisDirection::Down),
        prev_weapon: PlayerControllerInputType::ButtonInput(Button::LeftShoulder),
        next_weapon: PlayerControllerInputType::ButtonInput(Button::RightShoulder),
        jump: PlayerControllerInputType::ButtonInput(Button::East),
        shoot: PlayerControllerInputType::ButtonInput(Button::South),
        skip: PlayerControllerInputType::ButtonInput(Button::West),
        strafe: PlayerControllerInputType::AxisInput(Axis::TriggerRight, AxisDirection::Either),
        inventory: PlayerControllerInputType::ButtonInput(Button::West),
        map: PlayerControllerInputType::ButtonInput(Button::North),
        menu_ok: PlayerControllerInputType::ButtonInput(Button::South),
        menu_back: PlayerControllerInputType::ButtonInput(Button::East),
    }
}

#[inline(always)]
pub fn default_controller_axis_sensitivity() -> f64 {
    0.3
}
