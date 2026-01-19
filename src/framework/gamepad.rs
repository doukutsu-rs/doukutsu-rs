use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::framework::backend::BackendGamepad;
use crate::framework::error::GameResult;
use crate::game::shared_game_state::SharedGameState;
use crate::{common::Rect, engine_constants::EngineConstants, framework::context::Context};

const QUAKE_RUMBLE_LOW_FREQ: u16 = 0x3000;
const QUAKE_RUMBLE_HI_FREQ: u16 = 0;
const SUPER_QUAKE_RUMBLE_LOW_FREQ: u16 = 0x5000;
const SUPER_QUAKE_RUMBLE_HI_FREQ: u16 = 0;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum GamepadType {
    Unknown,
    Xbox360,
    XboxOne,
    PS3,
    PS4,
    NintendoSwitchPro,
    Virtual,
    PS5,
    AmazonLuma,
    GoogleStadia,
    NVIDIAShield,
    NintendoSwitchJoyConLeft,
    NintendoSwitchJoyConRight,
    NintendoSwitchJoyConPair,
}

impl GamepadType {
    pub fn get_name(&self) -> &str {
        match self {
            GamepadType::Unknown => "Unknown controller",
            GamepadType::Xbox360 => "Xbox 360 controller",
            GamepadType::XboxOne => "Xbox One controller",
            GamepadType::PS3 => "PlayStation 3 controller",
            GamepadType::PS4 => "PlayStation 4 controller",
            GamepadType::NintendoSwitchPro => "Nintendo Switch Pro controller",
            GamepadType::Virtual => "Virtual controller",
            GamepadType::PS5 => "PlayStation 5 controller",
            GamepadType::AmazonLuma => "Amazon Luma controller",
            GamepadType::GoogleStadia => "Google Stadia controller",
            GamepadType::NVIDIAShield => "NVIDIA Shield controller",
            GamepadType::NintendoSwitchJoyConLeft => "Nintendo Switch Joy-Con (left)",
            GamepadType::NintendoSwitchJoyConRight => "Nintendo Switch Joy-Con (right)",
            GamepadType::NintendoSwitchJoyConPair => "Nintendo Switch Joy-Con (pair)",
        }
    }
}

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[repr(u32)]
pub enum Axis {
    LeftX,
    LeftY,
    RightX,
    RightY,
    TriggerLeft,
    TriggerRight,
}

impl Axis {
    pub fn get_rect(&self, offset: usize, constants: &EngineConstants) -> Rect<u16> {
        constants.gamepad.axis_rects.get(self).unwrap()[offset]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum AxisDirection {
    None,
    Either,
    Up,
    Left,
    Right,
    Down,
}

impl AxisDirection {
    pub fn from_axis_data(axis: Axis, value: f64) -> Self {
        match axis {
            Axis::LeftX | Axis::RightX => {
                if value < 0.0 {
                    AxisDirection::Left
                } else {
                    AxisDirection::Right
                }
            }
            Axis::LeftY | Axis::RightY => {
                if value < 0.0 {
                    AxisDirection::Up
                } else {
                    AxisDirection::Down
                }
            }
            Axis::TriggerLeft | Axis::TriggerRight => AxisDirection::Either,
        }
    }

    pub fn compare(&self, value: f64, axis_sensitivity: f64) -> bool {
        match self {
            AxisDirection::None => false,
            AxisDirection::Either => value.abs() > 0.0,
            AxisDirection::Down | AxisDirection::Right => value > axis_sensitivity,
            AxisDirection::Up | AxisDirection::Left => value < -axis_sensitivity,
        }
    }
}

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[repr(u32)]
pub enum Button {
    South,
    East,
    West,
    North,

    Back,
    Guide,
    Start,
    LeftStick,
    RightStick,
    LeftShoulder,
    RightShoulder,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

impl Button {
    pub fn get_rect(&self, offset: usize, constants: &EngineConstants) -> Rect<u16> {
        match self {
            Button::Guide => Rect::new(0, 0, 0, 0),
            _ => constants.gamepad.button_rects.get(self).unwrap()[offset],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum PlayerControllerInputType {
    ButtonInput(Button),
    AxisInput(Axis, AxisDirection),
    Either(Button, Axis, AxisDirection),
}

impl PlayerControllerInputType {
    pub fn get_rect(&self, offset: usize, constants: &EngineConstants) -> Rect<u16> {
        match self {
            PlayerControllerInputType::ButtonInput(button) => button.get_rect(offset, constants),
            PlayerControllerInputType::AxisInput(axis, _) => axis.get_rect(offset, constants),
            PlayerControllerInputType::Either(button, axis, _) => button.get_rect(offset, constants),
        }
    }
}

pub struct GamepadData {
    controller: Box<dyn BackendGamepad>,
    controller_type: GamepadType,

    left_x: f64,
    left_y: f64,
    right_x: f64,
    right_y: f64,
    trigger_left: f64,
    trigger_right: f64,

    axis_sensitivity: f64,

    pressed_buttons_set: HashSet<Button>,
    axis_values: HashMap<Axis, f64>,
}

impl GamepadData {
    pub(crate) fn new(game_controller: Box<dyn BackendGamepad>, axis_sensitivity: f64) -> Self {
        GamepadData {
            controller: game_controller,
            controller_type: GamepadType::Unknown,

            left_x: 0.0,
            left_y: 0.0,
            right_x: 0.0,
            right_y: 0.0,
            trigger_left: 0.0,
            trigger_right: 0.0,

            axis_sensitivity,

            pressed_buttons_set: HashSet::with_capacity(16),
            axis_values: HashMap::with_capacity(8),
        }
    }

    pub(crate) fn set_gamepad_type(&mut self, controller_type: GamepadType) {
        self.controller_type = controller_type;
    }

    pub(crate) fn get_gamepad_sprite_offset(&self) -> usize {
        match self.controller_type {
            GamepadType::PS3 | GamepadType::PS4 | GamepadType::PS5 => 0,
            GamepadType::Xbox360 | GamepadType::XboxOne => 1,
            GamepadType::NintendoSwitchPro
            | GamepadType::NintendoSwitchJoyConLeft
            | GamepadType::NintendoSwitchJoyConRight
            | GamepadType::NintendoSwitchJoyConPair => 3,
            _ => 1,
        }
    }

    pub fn get_gamepad_name(&self) -> String {
        self.controller_type.get_name().to_owned()
    }

    pub fn set_rumble(&mut self, state: &SharedGameState, low_freq: u16, hi_freq: u16, ticks: u32) -> GameResult {
        let duration_ms = (ticks as f32 / state.settings.timing_mode.get_tps() as f32 * 1000.0) as u32;
        self.controller.set_rumble(low_freq, hi_freq, duration_ms)
    }

    pub fn instance_id(&self) -> u32 {
        self.controller.instance_id()
    }
}

pub struct GamepadContext {
    gamepads: Vec<GamepadData>,
}

impl GamepadContext {
    pub(crate) fn new() -> Self {
        Self { gamepads: Vec::new() }
    }

    fn get_gamepad(&self, gamepad_id: u32) -> Option<&GamepadData> {
        self.gamepads.iter().find(|gamepad| gamepad.controller.instance_id() == gamepad_id)
    }

    fn get_gamepad_by_index(&self, gamepad_index: usize) -> Option<&GamepadData> {
        self.gamepads.get(gamepad_index)
    }

    fn get_gamepad_mut(&mut self, gamepad_id: u32) -> Option<&mut GamepadData> {
        self.gamepads.iter_mut().find(|gamepad| gamepad.controller.instance_id() == gamepad_id)
    }

    fn get_gamepad_by_index_mut(&mut self, gamepad_index: usize) -> Option<&mut GamepadData> {
        self.gamepads.get_mut(gamepad_index)
    }

    pub(crate) fn add_gamepad(&mut self, game_controller: Box<dyn BackendGamepad>, axis_sensitivity: f64) {
        self.gamepads.push(GamepadData::new(game_controller, axis_sensitivity));
    }

    pub(crate) fn remove_gamepad(&mut self, gamepad_id: u32) {
        self.gamepads.retain(|data| data.controller.instance_id() != gamepad_id);
    }

    pub(crate) fn set_gamepad_type(&mut self, gamepad_id: u32, controller_type: GamepadType) {
        if let Some(gamepad) = self.get_gamepad_mut(gamepad_id) {
            gamepad.set_gamepad_type(controller_type);
        }
    }

    pub(crate) fn get_gamepad_sprite_offset(&self, gamepad_index: usize) -> usize {
        if let Some(gamepad) = self.get_gamepad_by_index(gamepad_index) {
            return gamepad.get_gamepad_sprite_offset();
        }

        1
    }

    pub(crate) fn set_button(&mut self, gamepad_id: u32, button: Button, pressed: bool) {
        if let Some(gamepad) = self.get_gamepad_mut(gamepad_id) {
            if pressed {
                gamepad.pressed_buttons_set.insert(button);
            } else {
                gamepad.pressed_buttons_set.remove(&button);
            }
        }
    }

    pub(crate) fn set_axis_value(&mut self, gamepad_id: u32, axis: Axis, value: f64) {
        if let Some(gamepad) = self.get_gamepad_mut(gamepad_id) {
            gamepad.axis_values.insert(axis, value);
        }
    }

    pub(crate) fn is_active(&self, gamepad_index: u32, input_type: &PlayerControllerInputType) -> bool {
        match input_type {
            PlayerControllerInputType::ButtonInput(button) => self.is_button_active(gamepad_index, *button),
            PlayerControllerInputType::AxisInput(axis, axis_direction) => {
                self.is_axis_active(gamepad_index, *axis, *axis_direction)
            }
            PlayerControllerInputType::Either(button, axis, axis_direction) => {
                self.is_button_active(gamepad_index, *button)
                    || self.is_axis_active(gamepad_index, *axis, *axis_direction)
            }
        }
    }

    pub(crate) fn is_button_active(&self, gamepad_index: u32, button: Button) -> bool {
        if let Some(gamepad) = self.get_gamepad_by_index(gamepad_index as usize) {
            return gamepad.pressed_buttons_set.contains(&button);
        }

        false
    }

    pub(crate) fn is_axis_active(&self, gamepad_index: u32, axis: Axis, direction: AxisDirection) -> bool {
        if let Some(gamepad) = self.get_gamepad_by_index(gamepad_index as usize) {
            return match axis {
                Axis::LeftX => direction.compare(gamepad.left_x, gamepad.axis_sensitivity),
                Axis::LeftY => direction.compare(gamepad.left_y, gamepad.axis_sensitivity),
                Axis::RightX => direction.compare(gamepad.right_x, gamepad.axis_sensitivity),
                Axis::RightY => direction.compare(gamepad.right_y, gamepad.axis_sensitivity),
                Axis::TriggerLeft => direction.compare(gamepad.trigger_left, 0.0),
                Axis::TriggerRight => direction.compare(gamepad.trigger_right, 0.0),
            };
        }

        false
    }

    pub(crate) fn update_axes(&mut self, gamepad_id: u32) {
        if let Some(gamepad) = self.get_gamepad_mut(gamepad_id) {
            let mut axes = [
                (&mut gamepad.left_x, Axis::LeftX),
                (&mut gamepad.left_y, Axis::LeftY),
                (&mut gamepad.right_x, Axis::RightX),
                (&mut gamepad.right_y, Axis::RightY),
                (&mut gamepad.trigger_left, Axis::TriggerLeft),
                (&mut gamepad.trigger_right, Axis::TriggerRight),
            ];

            for (axis_val, id) in axes.iter_mut() {
                if let Some(axis) = gamepad.axis_values.get(id) {
                    **axis_val = if axis.abs() < 0.12 { 0.0 } else { *axis };
                }
            }
        }
    }

    pub(crate) fn get_gamepads(&self) -> &Vec<GamepadData> {
        &self.gamepads
    }

    pub(crate) fn pressed_buttons(&self, gamepad_index: u32) -> HashSet<Button> {
        if let Some(gamepad) = self.get_gamepad_by_index(gamepad_index as usize) {
            return gamepad.pressed_buttons_set.clone();
        }

        HashSet::new()
    }

    pub(crate) fn active_axes(&self, gamepad_index: u32) -> HashMap<Axis, f64> {
        if let Some(gamepad) = self.get_gamepad_by_index(gamepad_index as usize) {
            let mut active_axes = gamepad.axis_values.clone();
            active_axes.retain(|_, v| v.abs() > gamepad.axis_sensitivity);
            return active_axes;
        }

        HashMap::new()
    }

    pub(crate) fn set_rumble(
        &mut self,
        gamepad_index: u32,
        state: &SharedGameState,
        low_freq: u16,
        hi_freq: u16,
        ticks: u32,
    ) -> GameResult {
        if let Some(gamepad) = self.get_gamepad_by_index_mut(gamepad_index as usize) {
            gamepad.set_rumble(state, low_freq, hi_freq, ticks)?;
        }

        Ok(())
    }

    pub(crate) fn set_rumble_all(
        &mut self,
        state: &SharedGameState,
        low_freq: u16,
        hi_freq: u16,
        ticks: u32,
    ) -> GameResult {
        for gamepad in self.gamepads.iter_mut() {
            gamepad.set_rumble(state, low_freq, hi_freq, ticks)?;
        }

        Ok(())
    }
}

impl Default for GamepadContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn add_gamepad(context: &mut Context, game_controller: Box<dyn BackendGamepad>, axis_sensitivity: f64) {
    context.gamepad_context.add_gamepad(game_controller, axis_sensitivity);
}

pub fn remove_gamepad(context: &mut Context, gamepad_id: u32) {
    context.gamepad_context.remove_gamepad(gamepad_id);
}

pub fn set_gamepad_type(context: &mut Context, gamepad_id: u32, controller_type: GamepadType) {
    context.gamepad_context.set_gamepad_type(gamepad_id, controller_type);
}

pub fn get_gamepad_sprite_offset(context: &Context, gamepad_index: usize) -> usize {
    context.gamepad_context.get_gamepad_sprite_offset(gamepad_index)
}

pub fn is_active(ctx: &Context, gamepad_index: u32, input_type: &PlayerControllerInputType) -> bool {
    ctx.gamepad_context.is_active(gamepad_index, input_type)
}

pub fn is_button_active(ctx: &Context, gamepad_index: u32, button: Button) -> bool {
    ctx.gamepad_context.is_button_active(gamepad_index, button)
}

pub fn is_axis_active(ctx: &Context, gamepad_index: u32, axis: Axis, direction: AxisDirection) -> bool {
    ctx.gamepad_context.is_axis_active(gamepad_index, axis, direction)
}

pub fn get_gamepads(ctx: &Context) -> &Vec<GamepadData> {
    ctx.gamepad_context.get_gamepads()
}

pub fn pressed_buttons(ctx: &Context, gamepad_index: u32) -> HashSet<Button> {
    ctx.gamepad_context.pressed_buttons(gamepad_index)
}

pub fn active_axes(ctx: &Context, gamepad_index: u32) -> HashMap<Axis, f64> {
    ctx.gamepad_context.active_axes(gamepad_index)
}

pub fn set_rumble(
    ctx: &mut Context,
    state: &SharedGameState,
    gamepad_index: u32,
    low_freq: u16,
    hi_freq: u16,
    ticks: u32,
) -> GameResult {
    ctx.gamepad_context.set_rumble(gamepad_index, state, low_freq, hi_freq, ticks)
}

pub fn set_rumble_all(
    ctx: &mut Context,
    state: &SharedGameState,
    low_freq: u16,
    hi_freq: u16,
    ticks: u32,
) -> GameResult {
    ctx.gamepad_context.set_rumble_all(state, low_freq, hi_freq, ticks)
}

pub fn set_quake_rumble(ctx: &mut Context, state: &SharedGameState, gamepad_index: u32, ticks: u32) -> GameResult {
    set_rumble(ctx, state, gamepad_index, QUAKE_RUMBLE_LOW_FREQ, QUAKE_RUMBLE_HI_FREQ, ticks)
}

pub fn set_quake_rumble_all(ctx: &mut Context, state: &SharedGameState, ticks: u32) -> GameResult {
    set_rumble_all(ctx, state, QUAKE_RUMBLE_LOW_FREQ, QUAKE_RUMBLE_LOW_FREQ, ticks)
}

pub fn set_super_quake_rumble(
    ctx: &mut Context,
    state: &SharedGameState,
    gamepad_index: u32,
    ticks: u32,
) -> GameResult {
    set_rumble(ctx, state, gamepad_index, SUPER_QUAKE_RUMBLE_LOW_FREQ, SUPER_QUAKE_RUMBLE_HI_FREQ, ticks)
}

pub fn set_super_quake_rumble_all(ctx: &mut Context, state: &SharedGameState, ticks: u32) -> GameResult {
    set_rumble_all(ctx, state, SUPER_QUAKE_RUMBLE_LOW_FREQ, SUPER_QUAKE_RUMBLE_LOW_FREQ, ticks)
}
