use std::collections::{HashMap, HashSet};

use sdl2::controller::GameController;
use serde::{Deserialize, Serialize};

use crate::{common::Rect, engine_constants::EngineConstants, framework::context::Context};

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
        constants.gamepad.button_rects.get(self).unwrap()[offset]
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
    controller: GameController,
    controller_type: Option<sdl2_sys::SDL_GameControllerType>,

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
    pub(crate) fn new(game_controller: GameController, axis_sensitivity: f64) -> Self {
        GamepadData {
            controller: game_controller,
            controller_type: None,

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

    pub(crate) fn set_gamepad_type(&mut self, controller_type: sdl2_sys::SDL_GameControllerType) {
        self.controller_type = Some(controller_type);
    }

    pub(crate) fn get_gamepad_sprite_offset(&self) -> usize {
        if let Some(controller_type) = self.controller_type {
            return match controller_type {
                sdl2_sys::SDL_GameControllerType::SDL_CONTROLLER_TYPE_PS3
                | sdl2_sys::SDL_GameControllerType::SDL_CONTROLLER_TYPE_PS4
                | sdl2_sys::SDL_GameControllerType::SDL_CONTROLLER_TYPE_PS5 => 0,
                sdl2_sys::SDL_GameControllerType::SDL_CONTROLLER_TYPE_XBOX360
                | sdl2_sys::SDL_GameControllerType::SDL_CONTROLLER_TYPE_XBOXONE => 1,
                sdl2_sys::SDL_GameControllerType::SDL_CONTROLLER_TYPE_NINTENDO_SWITCH_PRO => 3,
                _ => 1,
            };
        }

        1
    }

    pub fn get_gamepad_name(&self) -> String {
        let name = if let Some(controller_type) = self.controller_type {
            match controller_type {
                sdl2_sys::SDL_GameControllerType::SDL_CONTROLLER_TYPE_PS3
                | sdl2_sys::SDL_GameControllerType::SDL_CONTROLLER_TYPE_PS4
                | sdl2_sys::SDL_GameControllerType::SDL_CONTROLLER_TYPE_PS5 => "PlayStation Controller".to_string(),
                sdl2_sys::SDL_GameControllerType::SDL_CONTROLLER_TYPE_XBOX360
                | sdl2_sys::SDL_GameControllerType::SDL_CONTROLLER_TYPE_XBOXONE => "Xbox Controller".to_string(),
                sdl2_sys::SDL_GameControllerType::SDL_CONTROLLER_TYPE_NINTENDO_SWITCH_PRO => {
                    "Nintendo Switch Controller".to_string()
                }
                _ => "Unknown Controller".to_string(),
            }
        } else {
            "Unknown controller".to_string()
        };

        name
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

    pub(crate) fn add_gamepad(&mut self, game_controller: GameController, axis_sensitivity: f64) {
        self.gamepads.push(GamepadData::new(game_controller, axis_sensitivity));
    }

    pub(crate) fn remove_gamepad(&mut self, gamepad_id: u32) {
        self.gamepads.retain(|data| data.controller.instance_id() != gamepad_id);
    }

    pub(crate) fn set_gamepad_type(&mut self, gamepad_id: u32, controller_type: sdl2_sys::SDL_GameControllerType) {
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
}

impl Default for GamepadContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn add_gamepad(context: &mut Context, game_controller: GameController, axis_sensitivity: f64) {
    context.gamepad_context.add_gamepad(game_controller, axis_sensitivity);
}

pub fn remove_gamepad(context: &mut Context, gamepad_id: u32) {
    context.gamepad_context.remove_gamepad(gamepad_id);
}

pub fn set_gamepad_type(context: &mut Context, gamepad_id: u32, controller_type: sdl2_sys::SDL_GameControllerType) {
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
