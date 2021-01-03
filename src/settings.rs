use ggez::{Context, GameResult};
use serde::{Deserialize, Serialize};
use winit::event::VirtualKeyCode;

use crate::input::keyboard_player_controller::KeyboardController;
use crate::input::player_controller::PlayerController;
use crate::player::TargetPlayer;
use crate::input::touch_player_controller::TouchPlayerController;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub seasonal_textures: bool,
    pub original_textures: bool,
    pub shader_effects: bool,
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
    pub left: VirtualKeyCode,
    pub up: VirtualKeyCode,
    pub right: VirtualKeyCode,
    pub down: VirtualKeyCode,
    pub prev_weapon: VirtualKeyCode,
    pub next_weapon: VirtualKeyCode,
    pub jump: VirtualKeyCode,
    pub shoot: VirtualKeyCode,
    pub skip: VirtualKeyCode,
    pub inventory: VirtualKeyCode,
    pub map: VirtualKeyCode,
}

fn p1_default_keymap() -> PlayerKeyMap {
    PlayerKeyMap {
        left: VirtualKeyCode::Left,
        up: VirtualKeyCode::Up,
        right: VirtualKeyCode::Right,
        down: VirtualKeyCode::Down,
        prev_weapon: VirtualKeyCode::A,
        next_weapon: VirtualKeyCode::S,
        jump: VirtualKeyCode::Z,
        shoot: VirtualKeyCode::X,
        skip: VirtualKeyCode::LControl,
        inventory: VirtualKeyCode::Q,
        map: VirtualKeyCode::W,
    }
}

fn p2_default_keymap() -> PlayerKeyMap {
    PlayerKeyMap {
        left: VirtualKeyCode::Comma,
        up: VirtualKeyCode::L,
        right: VirtualKeyCode::Slash,
        down: VirtualKeyCode::Period,
        prev_weapon: VirtualKeyCode::G,
        next_weapon: VirtualKeyCode::H,
        jump: VirtualKeyCode::B,
        shoot: VirtualKeyCode::N,
        skip: VirtualKeyCode::RControl,
        inventory: VirtualKeyCode::T,
        map: VirtualKeyCode::Y,
    }
}
