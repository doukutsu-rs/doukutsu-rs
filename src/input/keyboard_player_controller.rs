use crate::bitfield;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::keyboard;
use crate::framework::keyboard::ScanCode;
use crate::game::shared_game_state::SharedGameState;
use crate::input::player_controller::PlayerController;
use crate::game::player::TargetPlayer;

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
  pub menu_ok, set_menu_ok: 14;
  pub menu_back, set_menu_back: 15;
}

#[derive(Clone)]
pub struct KeyboardController {
    target: TargetPlayer,
    state: KeyState,
    old_state: KeyState,
    trigger: KeyState,
}

impl KeyboardController {
    pub fn new(target: TargetPlayer) -> KeyboardController {
        KeyboardController { target, state: KeyState(0), old_state: KeyState(0), trigger: KeyState(0) }
    }
}

impl PlayerController for KeyboardController {
    fn update(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let keymap = match self.target {
            TargetPlayer::Player1 => &state.settings.player1_key_map,
            TargetPlayer::Player2 => &state.settings.player2_key_map,
        };

        self.state.set_left(keyboard::is_key_pressed(ctx, keymap.left));
        self.state.set_up(keyboard::is_key_pressed(ctx, keymap.up));
        self.state.set_right(keyboard::is_key_pressed(ctx, keymap.right));
        self.state.set_down(keyboard::is_key_pressed(ctx, keymap.down));
        self.state.set_map(keyboard::is_key_pressed(ctx, keymap.map));
        self.state.set_inventory(keyboard::is_key_pressed(ctx, keymap.inventory));
        self.state.set_jump(keyboard::is_key_pressed(ctx, keymap.jump));
        self.state.set_shoot(keyboard::is_key_pressed(ctx, keymap.shoot));
        self.state.set_skip(keyboard::is_key_pressed(ctx, keymap.skip));
        self.state.set_prev_weapon(keyboard::is_key_pressed(ctx, keymap.prev_weapon));
        self.state.set_next_weapon(keyboard::is_key_pressed(ctx, keymap.next_weapon));
        self.state.set_enter(keyboard::is_key_pressed(ctx, ScanCode::Return));
        self.state.set_escape(keyboard::is_key_pressed(ctx, ScanCode::Escape));
        self.state.set_strafe(keyboard::is_key_pressed(ctx, keymap.strafe));
        self.state.set_menu_ok(keyboard::is_key_pressed(ctx, keymap.menu_ok));
        self.state.set_menu_back(keyboard::is_key_pressed(ctx, keymap.menu_back));

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
        self.trigger.menu_ok() || self.trigger.enter()
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

    fn set_rumble(&mut self, _low_freq: u16, _hi_freq: u16, _ticks: u32) {}
}
