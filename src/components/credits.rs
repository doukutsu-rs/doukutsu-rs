use crate::common::{Colorf, Rect};
use crate::entity::GameEntity;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics;
use crate::game::frame::Frame;
use crate::game::npc::NPC;
use crate::game::scripting::tsc::text_script::IllustrationState;
use crate::game::shared_game_state::SharedGameState;
use crate::graphics::font::Font;

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
        graphics::draw_rect(ctx, rect, Colorf::from_srgb(0, 0, 32))?;

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
        for line in &state.creditscript_vm.lines {
            let x = (line.cast_id % 13) * 24;
            let y = ((line.cast_id / 13) & 0xff) * 24;
            let rect = Rect::new_size(x, y, 24, 24);

            if state.more_rust && line.cast_id == 1 {
                // sue with more rust
                batch.add_rect_tinted(line.pos_x - 24.0, line.pos_y - 8.0, (200, 200, 255, 255), &rect);
            } else {
                batch.add_rect(line.pos_x - 24.0, line.pos_y - 8.0, &rect);
            }
        }
        batch.draw(ctx)?;

        if state.more_rust {
            // draw sue's headband separately because rust doesn't let me mutate the texture set multiple times at once

            let headband_spritesheet = NPC::get_headband_spritesheet(state, "Casts");

            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, headband_spritesheet.as_str())?;

            for line in &state.creditscript_vm.lines {
                if line.cast_id != 1 {
                    continue;
                }

                let x = (line.cast_id % 13) * 24;
                let y = ((line.cast_id / 13) & 0xff) * 24;
                let rect = Rect::new_size(x, y, 24, 24);

                batch.add_rect(line.pos_x - 24.0, line.pos_y - 8.0, &rect);

                break;
            }

            batch.draw(ctx)?;
        }

        for line in &state.creditscript_vm.lines {
            let mut text_ovr = None;

            if state.more_rust {
                text_ovr = Some(line.text.replace("Sue Sakamoto", "Crabby Sue"));
            }

            let mut text = line.text.as_str();
            if let Some(ovr) = text_ovr.as_ref() {
                text = ovr.as_str();
            }

            state.font.builder().position(line.pos_x, line.pos_y).shadow(true).draw(
                text,
                ctx,
                &state.constants,
                &mut state.texture_set,
            )?;
        }

        Ok(())
    }
}
