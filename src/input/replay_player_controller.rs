use bitfield::bitfield;

use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::game::shared_game_state::SharedGameState;
use crate::input::player_controller::PlayerController;

bitfield! {
  #[allow(unused)]
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

#[derive(Copy, Clone)]
pub struct ReplayController {
    //target: TargetPlayer,
    pub state: KeyState,
    pub old_state: KeyState,
    trigger: KeyState,
}

impl ReplayController {
    pub fn new() -> ReplayController {
        ReplayController {
            //target,
            state: KeyState(0),
            old_state: KeyState(0),
            trigger: KeyState(0),
        }
    }
}

impl PlayerController for ReplayController {
    fn update(&mut self, _state: &mut SharedGameState, _ctx: &mut Context) -> GameResult {
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
