use std::ops::Deref;

use crate::common::{Color, Rect};
use crate::entity::GameEntity;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics;
use crate::game::frame::Frame;
use crate::game::shared_game_state::SharedGameState;
use crate::game::scripting::tsc::text_script::TextScriptExecutionState;

pub struct FallingIsland {}

impl FallingIsland {
    pub fn new() -> FallingIsland {
        FallingIsland {}
    }
}

impl GameEntity<()> for FallingIsland {
    fn tick(&mut self, _state: &mut SharedGameState, _custom: ()) -> GameResult {
        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, _frame: &Frame) -> GameResult {
        let (pos_x, pos_y) =
            if let TextScriptExecutionState::FallingIsland(_, _, pos_x, pos_y, _, _) = state.textscript_vm.state {
                (pos_x, pos_y)
            } else {
                return Ok(());
            };

        let off_x = (state.canvas_size.0 - 320.0) * 0.5;
        let clip_rect: Rect = Rect::new_size(
            ((off_x + 80.0) * state.scale) as _,
            (80.0 * state.scale) as _,
            (160.0 * state.scale) as _,
            (80.0 * state.scale) as _,
        );

        graphics::clear(ctx, Color::from_rgb(0, 0, 32));
        graphics::set_clip_rect(ctx, Some(clip_rect))?;

        static RECT_BG: Rect<u16> = Rect { left: 0, top: 0, right: 160, bottom: 80 };
        static RECT_ISLAND: Rect<u16> = Rect { left: 160, top: 0, right: 200, bottom: 24 };
        static RECT_TERRAIN: Rect<u16> = Rect { left: 160, top: 48, right: 320, bottom: 80 };

        let batch = state.texture_set.get_or_load_batch(
            ctx,
            &state.constants,
            &state.npc_table.stage_textures.deref().borrow().npc1,
        )?;
        batch.add_rect(off_x + 80.0, 80.0, &RECT_BG);
        batch.add_rect(off_x + (pos_x as f32 / 512.0) - 20.0, (pos_y as f32 / 512.0) - 12.0, &RECT_ISLAND);
        batch.add_rect(off_x + 80.0, 128.0, &RECT_TERRAIN);
        batch.draw(ctx)?;

        graphics::set_clip_rect(ctx, None)?;

        Ok(())
    }
}
