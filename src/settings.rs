use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem::{user_create, user_open};
use crate::framework::keyboard::ScanCode;
use crate::input::keyboard_player_controller::KeyboardController;
use crate::input::player_controller::PlayerController;
use crate::input::touch_player_controller::TouchPlayerController;
use crate::player::TargetPlayer;
use crate::shared_game_state::TimingMode;
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
    pub soundtrack: String,
    #[serde(default = "default_vol")]
    pub bgm_volume: f32,
    #[serde(default = "default_vol")]
    pub sfx_volume: f32,
    #[serde(default = "default_timing")]
    pub timing_mode: TimingMode,
    #[serde(default = "default_interpolation")]
    pub organya_interpolation: InterpolationMode,
    #[serde(default = "p1_default_keymap")]
    pub player1_key_map: PlayerKeyMap,
    #[serde(default = "p2_default_keymap")]
    pub player2_key_map: PlayerKeyMap,
    #[serde(skip, default = "default_speed")]
    pub speed: f64,
    #[serde(skip)]
    pub god_mode: bool,
    #[serde(skip)]
    pub infinite_booster: bool,
    #[serde(skip)]
    pub debug_outlines: bool,
    pub fps_counter: bool,
}

fn default_true() -> bool {
    true
}

#[inline(always)]
fn current_version() -> u32 {
    6
}

#[inline(always)]
fn default_timing() -> TimingMode {
    TimingMode::_50Hz
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

        Box::new(KeyboardController::new(TargetPlayer::Player1))
    }

    pub fn create_player2_controller(&self) -> Box<dyn PlayerController> {
        Box::new(KeyboardController::new(TargetPlayer::Player2))
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
            soundtrack: "Organya".to_string(),
            bgm_volume: 1.0,
            sfx_volume: 1.0,
            timing_mode: default_timing(),
            organya_interpolation: InterpolationMode::Linear,
            player1_key_map: p1_default_keymap(),
            player2_key_map: p2_default_keymap(),
            speed: 1.0,
            god_mode: false,
            infinite_booster: false,
            debug_outlines: false,
            fps_counter: false,
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
}

#[inline(always)]
fn p1_default_keymap() -> PlayerKeyMap {
    PlayerKeyMap {
        left: ScanCode::Left,
        up: ScanCode::Up,
        right: ScanCode::Right,
        down: ScanCode::Down,
        prev_weapon: ScanCode::A,
        next_weapon: ScanCode::S,
        jump: ScanCode::Z,
        shoot: ScanCode::X,
        skip: ScanCode::E,
        inventory: ScanCode::Q,
        map: ScanCode::W,
        strafe: ScanCode::LShift,
    }
}

#[inline(always)]
fn p2_default_keymap() -> PlayerKeyMap {
    PlayerKeyMap {
        left: ScanCode::Comma,
        up: ScanCode::L,
        right: ScanCode::Slash,
        down: ScanCode::Period,
        prev_weapon: ScanCode::G,
        next_weapon: ScanCode::H,
        jump: ScanCode::B,
        shoot: ScanCode::N,
        skip: ScanCode::U,
        inventory: ScanCode::T,
        map: ScanCode::Y,
        strafe: ScanCode::RShift,
    }
}
