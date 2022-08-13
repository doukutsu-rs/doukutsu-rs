use crate::bitfield;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::gamepad::{self, Button, PlayerControllerInputType};
use crate::input::player_controller::PlayerController;
use crate::player::TargetPlayer;
use crate::shared_game_state::SharedGameState;

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
    pub skip, set_skip: 11;
    pub strafe, set_strafe: 12;
    pub menu_ok, set_menu_ok: 13;
    pub menu_back, set_menu_back: 14;
}

#[derive(Clone)]
pub struct GamepadController {
    gamepad_id: u32,
    target: TargetPlayer,
    state: KeyState,
    old_state: KeyState,
    trigger: KeyState,
    rumble_state: Option<RumbleState>,
    rumble_enabled: bool,
}

#[derive(Clone)]
pub struct RumbleState {
    pub low_freq: u16,
    pub hi_freq: u16,
    pub ticks: u32,
}

impl GamepadController {
    pub fn new(gamepad_id: u32, target: TargetPlayer) -> GamepadController {
        GamepadController {
            gamepad_id,
            target,
            state: KeyState(0),
            old_state: KeyState(0),
            trigger: KeyState(0),
            rumble_state: None,
            rumble_enabled: false,
        }
    }

    pub fn set_rumble_enabled(&mut self, enabled: bool) {
        self.rumble_enabled = enabled;
    }
}

impl PlayerController for GamepadController {
    fn update(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let button_map = match self.target {
            TargetPlayer::Player1 => &state.settings.player1_controller_button_map,
            TargetPlayer::Player2 => &state.settings.player2_controller_button_map,
        };

        self.state.set_up(gamepad::is_active(ctx, self.gamepad_id, &button_map.up));
        self.state.set_down(gamepad::is_active(ctx, self.gamepad_id, &button_map.down));
        self.state.set_left(gamepad::is_active(ctx, self.gamepad_id, &button_map.left));
        self.state.set_right(gamepad::is_active(ctx, self.gamepad_id, &button_map.right));
        self.state.set_map(gamepad::is_active(ctx, self.gamepad_id, &button_map.map));
        self.state.set_inventory(gamepad::is_active(ctx, self.gamepad_id, &button_map.inventory));
        self.state.set_jump(gamepad::is_active(ctx, self.gamepad_id, &button_map.jump));
        self.state.set_shoot(gamepad::is_active(ctx, self.gamepad_id, &button_map.shoot));
        self.state.set_next_weapon(gamepad::is_active(ctx, self.gamepad_id, &button_map.next_weapon));
        self.state.set_prev_weapon(gamepad::is_active(ctx, self.gamepad_id, &button_map.prev_weapon));
        self.state.set_escape(gamepad::is_active(
            ctx,
            self.gamepad_id,
            &PlayerControllerInputType::ButtonInput(Button::Start),
        ));
        self.state.set_skip(gamepad::is_active(ctx, self.gamepad_id, &button_map.skip));
        self.state.set_strafe(gamepad::is_active(ctx, self.gamepad_id, &button_map.strafe));
        self.state.set_menu_ok(gamepad::is_active(ctx, self.gamepad_id, &button_map.menu_ok));
        self.state.set_menu_back(gamepad::is_active(ctx, self.gamepad_id, &button_map.menu_back));

        if let Some(rumble_data) = &self.rumble_state {
            gamepad::set_rumble(
                ctx,
                state,
                self.gamepad_id,
                rumble_data.low_freq,
                rumble_data.hi_freq,
                rumble_data.ticks,
            )?;
            self.rumble_state = None;
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
        self.trigger.menu_ok()
    }

    fn trigger_menu_back(&self) -> bool {
        self.trigger.menu_back() || self.trigger.escape()
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

    fn set_rumble(&mut self, low_freq: u16, hi_freq: u16, ticks: u32) {
        if !self.rumble_enabled {
            return;
        }

        if !self.rumble_state.is_none() {
            return;
        }

        self.rumble_state = Some(RumbleState { low_freq, hi_freq, ticks });
    }
}
