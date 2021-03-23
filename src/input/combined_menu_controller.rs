use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::input::player_controller::PlayerController;
use crate::shared_game_state::SharedGameState;

pub struct CombinedMenuController {
    controllers: Vec<Box<dyn PlayerController>>,
}

impl CombinedMenuController {
    pub fn new() -> CombinedMenuController {
        CombinedMenuController {
            controllers: Vec::new(),
        }
    }

    pub fn update(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        for cont in self.controllers.iter_mut() {
            cont.update(state, ctx)?;
        }

        Ok(())
    }

    pub fn update_trigger(&mut self) {
        for cont in self.controllers.iter_mut() {
            cont.update_trigger();
        }
    }

    pub fn add(&mut self, controller: Box<dyn PlayerController>) {
        self.controllers.push(controller);
    }

    pub fn trigger_up(&self) -> bool {
        for cont in self.controllers.iter() {
            if cont.trigger_up() {
                return true;
            }
        }

        false
    }

    pub fn trigger_down(&self) -> bool {
        for cont in self.controllers.iter() {
            if cont.trigger_down() {
                return true;
            }
        }

        false
    }

    pub fn trigger_left(&self) -> bool {
        for cont in self.controllers.iter() {
            if cont.trigger_left() {
                return true;
            }
        }

        false
    }

    pub fn trigger_right(&self) -> bool {
        for cont in self.controllers.iter() {
            if cont.trigger_right() {
                return true;
            }
        }

        false
    }

    pub fn trigger_ok(&self) -> bool {
        for cont in self.controllers.iter() {
            if cont.trigger_menu_ok() {
                return true;
            }
        }

        false
    }

    pub fn trigger_back(&self) -> bool {
        for cont in self.controllers.iter() {
            if cont.trigger_menu_back() {
                return true;
            }
        }

        false
    }
}
