use crate::common::Rect;
use crate::shared_game_state::SharedGameState;
use crate::framework::context::Context;
use crate::framework::error::GameResult;

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

pub fn draw_number_zeros(x: f32, y: f32, val: usize, align: Alignment, zeros: usize, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
    let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;

    let n = format!("{:01$}", val, zeros);
    let align_offset = if align == Alignment::Right { n.len() as f32 * 8.0 } else { 0.0 };

    for (offset, chr) in n.chars().enumerate() {
        let idx = chr as u16 - '0' as u16;
        batch.add_rect(x - align_offset + offset as f32 * 8.0, y, &Rect::new_size(idx * 8, 56, 8, 8));
    }

    batch.draw(ctx)?;
    Ok(())
}
