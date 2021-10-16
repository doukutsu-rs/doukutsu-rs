use crate::common::{Color, Rect};
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics;
use crate::scripting::tsc::text_script::IllustrationState;
use crate::shared_game_state::SharedGameState;

pub struct Credits {}

impl Credits {
    pub fn new() -> Credits {
        Credits {}
    }

    pub fn draw_tick(&mut self, state: &mut SharedGameState) {
        match state.textscript_vm.illustration_state {
            IllustrationState::FadeIn(mut x) => {
                x += 40.0 * state.frame_time as f32;

                state.textscript_vm.illustration_state =
                    if x >= 0.0 { IllustrationState::Shown } else { IllustrationState::FadeIn(x) };
            }
            IllustrationState::FadeOut(mut x) => {
                x -= 40.0 * state.frame_time as f32;

                state.textscript_vm.illustration_state =
                    if x <= -160.0 { IllustrationState::Hidden } else { IllustrationState::FadeOut(x) };
            }
            _ => (),
        }
    }
}

impl GameEntity<()> for Credits {
    fn tick(&mut self, _state: &mut SharedGameState, _custom: ()) -> GameResult {
        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, _frame: &Frame) -> GameResult {
        let rect = Rect::new(0, 0, (state.screen_size.0 / 2.0) as _, state.screen_size.1 as _);
        graphics::draw_rect(ctx, rect, Color::from_rgb(0, 0, 32))?;

        if state.textscript_vm.illustration_state != IllustrationState::Hidden {
            let x = match state.textscript_vm.illustration_state {
                IllustrationState::FadeIn(x) | IllustrationState::FadeOut(x) => x,
                _ => 0.0,
            };

            if let Some(tex) = &state.textscript_vm.current_illustration {
                let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, tex)?;
                batch.add(x, 0.0);
                batch.draw(ctx)?;
            }
        }

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
