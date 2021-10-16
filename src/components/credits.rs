use crate::common::Rect;
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::shared_game_state::SharedGameState;

pub struct Credits {}

impl Credits {
    pub fn new() -> Credits {
        Credits {}
    }
}

impl GameEntity<()> for Credits {
    fn tick(&mut self, state: &mut SharedGameState, custom: ()) -> GameResult {
        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, _frame: &Frame) -> GameResult {
        if state.creditscript_vm.lines.is_empty() {
            return Ok(());
        }

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Casts")?;
        for line in state.creditscript_vm.lines.iter() {
            let x = (line.cast_id % 13) * 24;
            let y = ((line.cast_id / 13) & 0xff) * 24;
            let rect = Rect::new_size(x, y, 24, 24);

            batch.add_rect(line.pos_x - 24.0, line.pos_y - 8.0, &rect);
        }
        batch.draw(ctx)?;

        for line in state.creditscript_vm.lines.iter() {
            state.font.draw_text_with_shadow(
                line.text.chars(),
                line.pos_x,
                line.pos_y,
                &state.constants,
                &mut state.texture_set,
                ctx,
            )?;
        }

        Ok(())
    }
}
