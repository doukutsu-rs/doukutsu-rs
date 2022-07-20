use std::collections::HashMap;

use gilrs::{Axis, Button, Gamepad, GamepadId};
use serde::{Deserialize, Serialize};

use crate::{framework::context::Context, settings::PlayerControllerInputType};

#[derive(Clone, Debug)]
pub enum AxisDirection {
    None,
    Up,
    Left,
    Right,
    Down,
}

impl AxisDirection {
    pub fn compare(&self, value: f64, axis_sensitivity: f64) -> bool {
        match self {
            AxisDirection::None => false,
            AxisDirection::Up => value > axis_sensitivity,
            AxisDirection::Left => value < -axis_sensitivity,
            AxisDirection::Right => value > axis_sensitivity,
            AxisDirection::Down => value < -axis_sensitivity,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GamepadData {
    left_x: f64,
    left_y: f64,
    right_x: f64,
    right_y: f64,
    axis_sensitivity: f64,
}

impl GamepadData {
    pub(crate) fn new(axis_sensitivity: f64) -> Self {
        GamepadData { left_x: 0.0, left_y: 0.0, right_x: 0.0, right_y: 0.0, axis_sensitivity }
    }
}

#[derive(Clone, Debug)]
pub struct GamepadContext {
    gamepads: HashMap<GamepadId, GamepadData>,
}

impl GamepadContext {
    pub(crate) fn new() -> Self {
        Self { gamepads: HashMap::new() }
    }

    fn gamepad_exists(&self, gamepad: &Gamepad) -> bool {
        self.gamepads.contains_key(&gamepad.id())
    }

    pub(crate) fn add_gamepad(&mut self, gamepad: &Gamepad, axis_sensitivity: f64) {
        self.gamepads.insert(gamepad.id(), GamepadData::new(axis_sensitivity));
    }

    pub(crate) fn remove_gamepad(&mut self, gamepad: &Gamepad) {
        self.gamepads.remove(&gamepad.id());
    }

    pub(crate) fn is_active(
        &self,
        gamepad: &Gamepad,
        input_type: &PlayerControllerInputType,
        axis_direction: AxisDirection,
    ) -> bool {
        match input_type {
            PlayerControllerInputType::ButtonInput(button) => self.is_button_active(gamepad, *button),
            PlayerControllerInputType::AxisInput(axis) => self.is_axis_active(gamepad, *axis, axis_direction),
            PlayerControllerInputType::Either(button, axis) => {
                self.is_button_active(gamepad, *button) || self.is_axis_active(gamepad, *axis, axis_direction)
            }
        }
    }

    pub(crate) fn is_button_active(&self, gamepad: &Gamepad, button: Button) -> bool {
        if !self.gamepad_exists(gamepad) {
            return false;
        }

        gamepad.is_pressed(button)
    }

    pub(crate) fn is_axis_active(&self, gamepad: &Gamepad, axis: Axis, direction: AxisDirection) -> bool {
        if !self.gamepad_exists(gamepad) {
            return false;
        }

        let data = self.gamepads.get(&gamepad.id()).unwrap();

        match axis {
            Axis::LeftStickX => direction.compare(data.left_x, data.axis_sensitivity),
            Axis::LeftStickY => direction.compare(data.left_y, data.axis_sensitivity),
            Axis::RightStickX => direction.compare(data.right_x, data.axis_sensitivity),
            Axis::RightStickY => direction.compare(data.right_y, data.axis_sensitivity),
            _ => false,
        }
    }

    pub(crate) fn update_axes(&mut self, gamepad: &Gamepad) {
        if !self.gamepad_exists(gamepad) {
            return;
        }

        let data = self.gamepads.get_mut(&gamepad.id()).unwrap();

        let mut axes = [
            (&mut data.left_x, Axis::LeftStickX),
            (&mut data.left_y, Axis::LeftStickY),
            (&mut data.right_x, Axis::RightStickX),
            (&mut data.right_y, Axis::RightStickY),
        ];

        for (axis_val, id) in axes.iter_mut() {
            if let Some(axis) = gamepad.axis_data(*id) {
                **axis_val = if axis.value().abs() < 0.12 { 0.0 } else { axis.value() } as f64;
            }
        }
    }
}

impl Default for GamepadContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn add_gamepad(context: &mut Context, gamepad: &Gamepad, axis_sensitivity: f64) {
    context.gamepad_context.add_gamepad(gamepad, axis_sensitivity);
}

pub fn remove_gamepad(context: &mut Context, gamepad: &Gamepad) {
    context.gamepad_context.remove_gamepad(gamepad);
}

pub fn is_active(
    ctx: &Context,
    gamepad: &Gamepad,
    input_type: &PlayerControllerInputType,
    axis_direction: AxisDirection,
) -> bool {
    ctx.gamepad_context.is_active(gamepad, input_type, axis_direction)
}

pub fn is_button_active(ctx: &Context, gamepad: &Gamepad, button: Button) -> bool {
    ctx.gamepad_context.is_button_active(gamepad, button)
}

pub fn is_axis_active(ctx: &Context, gamepad: &Gamepad, axis: Axis, direction: AxisDirection) -> bool {
    ctx.gamepad_context.is_axis_active(gamepad, axis, direction)
}

pub fn update_axes(ctx: &mut Context, gamepad: &Gamepad) {
    ctx.gamepad_context.update_axes(gamepad);
}
