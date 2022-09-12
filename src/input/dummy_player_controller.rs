use crate::bitfield;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::input::player_controller::{KeyState, PlayerController};
use crate::shared_game_state::SharedGameState;

/// A no-op implementation of player controller.
#[derive(Clone)]
pub struct DummyPlayerController {
    state: KeyState,
    old_state: KeyState,
    trigger: KeyState,
}

impl DummyPlayerController {
    pub fn new() -> DummyPlayerController {
        DummyPlayerController { state: KeyState(0), old_state: KeyState(0), trigger: KeyState(0) }
    }
}

impl PlayerController for DummyPlayerController {
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

    fn dump_state(&self) -> (u16, u16, u16) {
        (self.state.0, self.old_state.0, self.trigger.0)
    }

    fn set_state(&mut self, state: (u16, u16, u16)) {
        self.state = KeyState(state.0);
        //self.old_state = KeyState(state.1);
        //self.trigger = KeyState(state.2);
    }
}
