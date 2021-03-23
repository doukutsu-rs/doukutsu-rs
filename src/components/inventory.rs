use crate::entity::GameEntity;
use crate::inventory::Inventory;
use crate::framework::context::Context;
use crate::frame::Frame;
use crate::shared_game_state::SharedGameState;
use crate::framework::error::GameResult;

pub struct InventoryUI {
    text_y_pos: usize,
    tick: usize,
}

impl InventoryUI {
    pub fn new() -> InventoryUI {
        InventoryUI {
            text_y_pos: 24,
            tick: 0,
        }
    }
}

impl GameEntity<&mut Inventory> for InventoryUI {
    fn tick(&mut self, state: &mut SharedGameState, custom: &mut Inventory) -> GameResult<()> {
        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult<()> {
        Ok(())
    }
}
