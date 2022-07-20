use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::gamepad::{self, AxisDirection};
use crate::input::player_controller::PlayerController;
use crate::player::TargetPlayer;
use crate::shared_game_state::SharedGameState;
use crate::{bitfield, settings::PlayerControllerInputType};

use gilrs::{Button, GamepadId};

bitfield! {
    #[derive(Clone, Copy)]
    pub struct KeyState(u16);
    impl Debug;

    pub left, set_left: 0;
    pub right, set_right: 1;
    pub up, set_up: 2;
    pub down, set_down: 3;
    pub map, set_map: 4;
    pub inventory, set_inventory: 5;
    pub jump, set_jump: 6;
    pub shoot, set_shoot: 7;
    pub next_weapon, set_next_weapon: 8;
    pub prev_weapon, set_prev_weapon: 9;
    pub escape, set_escape: 10;
    pub enter, set_enter: 11;
    pub skip, set_skip: 12;
    pub strafe, set_strafe: 13;
}

#[derive(Clone)]
pub struct GamepadController {
    gamepad_id: GamepadId,
    target: TargetPlayer,
    state: KeyState,
    old_state: KeyState,
    trigger: KeyState,
}

impl GamepadController {
    pub fn new(gamepad_id: GamepadId, target: TargetPlayer) -> GamepadController {
        GamepadController { gamepad_id, target, state: KeyState(0), old_state: KeyState(0), trigger: KeyState(0) }
    }
}

impl PlayerController for GamepadController {
    fn update(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let button_map = match self.target {
            TargetPlayer::Player1 => &state.settings.player1_controller_button_map,
            TargetPlayer::Player2 => &state.settings.player2_controller_button_map,
        };

        if let Some(gilrs) = &state.gilrs {
            if let Some(gamepad) = gilrs.connected_gamepad(self.gamepad_id) {
                gamepad::update_axes(ctx, &gamepad);

                self.state.set_up(gamepad::is_active(ctx, &gamepad, &button_map.up, AxisDirection::Up));
                self.state.set_down(gamepad::is_active(ctx, &gamepad, &button_map.down, AxisDirection::Down));
                self.state.set_left(gamepad::is_active(ctx, &gamepad, &button_map.left, AxisDirection::Left));
                self.state.set_right(gamepad::is_active(ctx, &gamepad, &button_map.right, AxisDirection::Right));
                self.state.set_map(gamepad::is_active(ctx, &gamepad, &button_map.map, AxisDirection::None));
                self.state.set_inventory(gamepad::is_active(ctx, &gamepad, &button_map.inventory, AxisDirection::None));
                self.state.set_jump(gamepad::is_active(ctx, &gamepad, &button_map.jump, AxisDirection::None));
                self.state.set_shoot(gamepad::is_active(ctx, &gamepad, &button_map.shoot, AxisDirection::None));
                self.state.set_next_weapon(gamepad::is_active(
                    ctx,
                    &gamepad,
                    &button_map.next_weapon,
                    AxisDirection::None,
                ));
                self.state.set_prev_weapon(gamepad::is_active(
                    ctx,
                    &gamepad,
                    &button_map.prev_weapon,
                    AxisDirection::None,
                ));
                self.state.set_escape(gamepad::is_active(
                    ctx,
                    &gamepad,
                    &PlayerControllerInputType::ButtonInput(Button::Start),
                    AxisDirection::None,
                ));
                self.state.set_enter(gamepad::is_active(ctx, &gamepad, &button_map.jump, AxisDirection::None));
                self.state.set_skip(gamepad::is_active(ctx, &gamepad, &button_map.skip, AxisDirection::None));
                self.state.set_strafe(gamepad::is_active(ctx, &gamepad, &button_map.strafe, AxisDirection::None));
            }
        }

        Ok(())
    }

    fn update_trigger(&mut self) {
        let mut trigger = self.state.0 ^ self.old_state.0;
        trigger &= self.state.0;
        self.old_state = self.state;
        self.trigger = KeyState(trigger);
    }

    fn move_up(&self) -> bool {
        self.state.up()
    }

    fn move_left(&self) -> bool {
        self.state.left()
    }

    fn move_down(&self) -> bool {
        self.state.down()
    }

    fn move_right(&self) -> bool {
        self.state.right()
    }

    fn prev_weapon(&self) -> bool {
        self.state.prev_weapon()
    }

    fn next_weapon(&self) -> bool {
        self.state.next_weapon()
    }

    fn map(&self) -> bool {
        self.state.map()
    }

    fn inventory(&self) -> bool {
        self.state.inventory()
    }

    fn jump(&self) -> bool {
        self.state.jump()
    }

    fn shoot(&self) -> bool {
        self.state.shoot()
    }

    fn skip(&self) -> bool {
        self.state.skip()
    }

    fn strafe(&self) -> bool {
        self.state.strafe()
    }

    fn trigger_up(&self) -> bool {
        self.trigger.up()
    }

    fn trigger_left(&self) -> bool {
        self.trigger.left()
    }

    fn trigger_down(&self) -> bool {
        self.trigger.down()
    }

    fn trigger_right(&self) -> bool {
        self.trigger.right()
    }

    fn trigger_prev_weapon(&self) -> bool {
        self.trigger.prev_weapon()
    }

    fn trigger_next_weapon(&self) -> bool {
        self.trigger.next_weapon()
    }

    fn trigger_map(&self) -> bool {
        self.trigger.map()
    }

    fn trigger_inventory(&self) -> bool {
        self.trigger.inventory()
    }

    fn trigger_jump(&self) -> bool {
        self.trigger.jump()
    }

    fn trigger_shoot(&self) -> bool {
        self.trigger.shoot()
    }

    fn trigger_skip(&self) -> bool {
        self.trigger.skip()
    }

    fn trigger_strafe(&self) -> bool {
        self.trigger.strafe()
    }

    fn trigger_menu_ok(&self) -> bool {
        self.trigger.jump() || self.trigger.enter()
    }

    fn trigger_menu_back(&self) -> bool {
        self.trigger.shoot() || self.trigger.escape()
    }

    fn trigger_menu_pause(&self) -> bool {
        self.trigger.escape()
    }

    fn look_up(&self) -> bool {
        self.state.up()
    }

    fn look_left(&self) -> bool {
        self.state.left()
    }

    fn look_down(&self) -> bool {
        self.state.down()
    }

    fn look_right(&self) -> bool {
        self.state.right()
    }

    fn move_analog_x(&self) -> f64 {
        if self.state.left() && self.state.right() {
            0.0
        } else if self.state.left() {
            -1.0
        } else if self.state.right() {
            1.0
        } else {
            0.0
        }
    }

    fn move_analog_y(&self) -> f64 {
        if self.state.up() && self.state.down() {
            0.0
        } else if self.state.up() {
            -1.0
        } else if self.state.down() {
            1.0
        } else {
            0.0
        }
    }
}
