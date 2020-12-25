use ggez::{Context, GameResult};

use crate::common::Rect;
use crate::shared_game_state::SharedGameState;

#[derive(Debug, EnumIter, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Alignment {
    Left,
    Right,
}

pub fn draw_number(x: f32, y: f32, val: usize, align: Alignment, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
    let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;

    let n = val.to_string();
    let align_offset = if align == Alignment::Right { n.len() as f32 * 8.0 } else { 0.0 };

    for (offset, chr) in n.chars().enumerate() {
        let idx = chr as u16 - '0' as u16;
        batch.add_rect(x - align_offset + offset as f32 * 8.0, y, &Rect::new_size(idx * 8, 56, 8, 8));
    }

    batch.draw(ctx)?;
    Ok(())
}
