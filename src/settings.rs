use serde::{Deserialize, Serialize};

use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::keyboard::ScanCode;
use crate::input::keyboard_player_controller::KeyboardController;
use crate::input::player_controller::PlayerController;
use crate::input::touch_player_controller::TouchPlayerController;
use crate::player::TargetPlayer;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub seasonal_textures: bool,
    pub original_textures: bool,
    pub shader_effects: bool,
    pub subpixel_coords: bool,
    pub motion_interpolation: bool,
    pub touch_controls: bool,
    pub player1_key_map: PlayerKeyMap,
    pub player2_key_map: PlayerKeyMap,
    #[serde(skip)]
    pub speed: f64,
    #[serde(skip)]
    pub god_mode: bool,
    #[serde(skip)]
    pub infinite_booster: bool,
    #[serde(skip)]
    pub debug_outlines: bool,
}

impl Settings {
    pub fn load(_ctx: &mut Context) -> GameResult<Settings> {
        Ok(Settings::default())
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
            seasonal_textures: true,
            original_textures: false,
            shader_effects: true,
            subpixel_coords: true,
            motion_interpolation: true,
            touch_controls: cfg!(target_os = "android"),
            player1_key_map: p1_default_keymap(),
            player2_key_map: p2_default_keymap(),
            speed: 1.0,
            god_mode: false,
            infinite_booster: false,
            debug_outlines: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
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
}

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
        skip: ScanCode::LControl,
        inventory: ScanCode::Q,
        map: ScanCode::W,
    }
}

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
    }
}
