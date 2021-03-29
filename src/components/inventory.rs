use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::inventory::Inventory;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;
use crate::text_script::ScriptMode;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
enum InventoryFocus {
    None,
    Weapons,
    Items,
}

pub struct InventoryUI {
    text_y_pos: usize,
    tick: usize,
    current_script: usize,
    focus: InventoryFocus,
}

impl InventoryUI {
    pub fn new() -> InventoryUI {
        InventoryUI { text_y_pos: 24, tick: 0, current_script: 0, focus: InventoryFocus::None }
    }
}

impl GameEntity<(&mut Player, &mut Inventory)> for InventoryUI {
    fn tick(
        &mut self,
        state: &mut SharedGameState,
        (player, inventory): (&mut Player, &mut Inventory),
    ) -> GameResult<()> {
        if state.control_flags.control_enabled()
            && (player.controller.trigger_inventory() || player.controller.trigger_menu_back())
        {
            self.focus = InventoryFocus::None;
            state.textscript_vm.reset();
            state.textscript_vm.set_mode(ScriptMode::Map);
            return Ok(());
        }

        match self.focus {
            InventoryFocus::None => {
                self.focus = InventoryFocus::Weapons;
                state.textscript_vm.start_script(1004);
            }
            InventoryFocus::Weapons => {}
            InventoryFocus::Items => {}
        }

        self.tick = self.tick.wrapping_add(1);

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, _frame: &Frame) -> GameResult<()> {
        let mut y = 8.0;

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;
        for i in 0..=18 {
            let rect = match i {
                0 => &state.constants.textscript.inventory_rect_top,
                18 => &state.constants.textscript.inventory_rect_bottom,
                _ => &state.constants.textscript.inventory_rect_middle,
            };

            batch.add_rect(((state.canvas_size.0 - 244.0) / 2.0).floor(), y, rect);
            y += 8.0;
        }

        batch.draw(ctx)?;

        Ok(())
    }
}
