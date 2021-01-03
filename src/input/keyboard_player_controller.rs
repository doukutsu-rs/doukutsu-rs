use ggez::{Context, GameResult};
use ggez::input::keyboard;
use winit::event::VirtualKeyCode;

use crate::bitfield;
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
  pub enter, set_enter: 11;
  pub skip, set_skip: 12;
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
        KeyboardController {
            target,
            state: KeyState(0),
            old_state: KeyState(0),
            trigger: KeyState(0),
        }
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
        self.state.set_enter(keyboard::is_key_pressed(ctx, VirtualKeyCode::Return));
        self.state.set_escape(keyboard::is_key_pressed(ctx, VirtualKeyCode::Escape));

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

    fn jump(&self) -> bool {
        self.state.jump()
    }

    fn shoot(&self) -> bool {
        self.state.shoot()
    }

    fn skip(&self) -> bool {
        self.state.skip()
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

    fn trigger_jump(&self) -> bool {
        self.trigger.jump()
    }

    fn trigger_shoot(&self) -> bool {
        self.trigger.shoot()
    }

    fn trigger_skip(&self) -> bool {
        self.trigger.skip()
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
