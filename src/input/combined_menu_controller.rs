use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::game::shared_game_state::SharedGameState;
use crate::input::player_controller::PlayerController;

pub struct CombinedMenuController {
    controllers: Vec<Box<dyn PlayerController>>,
}

impl CombinedMenuController {
    pub fn new() -> CombinedMenuController {
        CombinedMenuController { controllers: Vec::new() }
    }

    pub fn update(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        for cont in &mut self.controllers {
            cont.update(state, ctx)?;
        }

        Ok(())
    }

    pub fn update_trigger(&mut self) {
        for cont in &mut self.controllers {
            cont.update_trigger();
        }
    }

    pub fn add(&mut self, controller: Box<dyn PlayerController>) {
        self.controllers.push(controller);
    }

    pub fn trigger_up(&self) -> bool {
        for cont in &self.controllers {
            if cont.trigger_up() {
                return true;
            }
        }

        false
    }

    pub fn trigger_down(&self) -> bool {
        for cont in &self.controllers {
            if cont.trigger_down() {
                return true;
            }
        }

        false
    }

    pub fn trigger_left(&self) -> bool {
        for cont in &self.controllers {
            if cont.trigger_left() {
                return true;
            }
        }

        false
    }

    pub fn trigger_right(&self) -> bool {
        for cont in &self.controllers {
            if cont.trigger_right() {
                return true;
            }
        }

        false
    }

    pub fn trigger_ok(&self) -> bool {
        for cont in &self.controllers {
            if cont.trigger_menu_ok() {
                return true;
            }
        }

        false
    }

    pub fn trigger_back(&self) -> bool {
        for cont in &self.controllers {
            if cont.trigger_menu_back() {
                return true;
            }
        }

        false
    }

    pub fn trigger_shift_left(&self) -> bool {
        for cont in &self.controllers {
            if cont.trigger_prev_weapon() {
                return true;
            }
        }

        false
    }

    pub fn trigger_shift_right(&self) -> bool {
        for cont in &self.controllers {
            if cont.trigger_next_weapon() {
                return true;
            }
        }

        false
    }
}
