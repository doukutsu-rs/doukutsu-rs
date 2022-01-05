use crate::common::{FadeDirection, FadeState, Rect};
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::shared_game_state::SharedGameState;

pub struct Fade;

impl Fade {
    pub fn new() -> Self {
        Fade
    }
}

impl GameEntity<()> for Fade {
    fn tick(&mut self, state: &mut SharedGameState, _custom: ()) -> GameResult {
        match state.fade_state {
            FadeState::FadeOut(tick, direction) if tick < 15 => {
                state.fade_state = FadeState::FadeOut(tick + 1, direction);
            }
            FadeState::FadeOut(tick, _) if tick == 15 => {
                state.fade_state = FadeState::Hidden;
            }
            FadeState::FadeIn(tick, direction) if tick > -15 => {
                state.fade_state = FadeState::FadeIn(tick - 1, direction);
            }
            FadeState::FadeIn(tick, _) if tick == -15 => {
                state.fade_state = FadeState::Visible;
            }
            _ => {}
        }

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, _frame: &Frame) -> GameResult {
        match state.fade_state {
            FadeState::Visible => {
                return Ok(());
            }
            FadeState::Hidden => {
                let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Fade")?;
                let mut rect = Rect::new(0, 0, 16, 16);
                let frame = 15;
                rect.left = frame * 16;
                rect.right = rect.left + 16;

                for x in 0..(state.canvas_size.0 as i32 / 16 + 1) {
                    for y in 0..(state.canvas_size.1 as i32 / 16 + 1) {
                        batch.add_rect(x as f32 * 16.0, y as f32 * 16.0, &rect);
                    }
                }

                batch.draw(ctx)?;
            }
            FadeState::FadeIn(tick, direction) | FadeState::FadeOut(tick, direction) => {
                let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Fade")?;
                let mut rect = Rect::new(0, 0, 16, 16);

                match direction {
                    FadeDirection::Left | FadeDirection::Right => {
                        let mut frame = tick;

                        for x in 0..(state.canvas_size.0 as i32 / 16 + 1) {
                            if frame >= 15 {
                                frame = 15;
                            } else {
                                frame += 1;
                            }

                            if frame >= 0 {
                                rect.left = frame.abs() as u16 * 16;
                                rect.right = rect.left + 16;

                                for y in 0..(state.canvas_size.1 as i32 / 16 + 1) {
                                    if direction == FadeDirection::Left {
                                        batch.add_rect(
                                            state.canvas_size.0 - x as f32 * 16.0 - 16.0,
                                            y as f32 * 16.0,
                                            &rect,
                                        );
                                    } else {
                                        batch.add_rect(x as f32 * 16.0, y as f32 * 16.0, &rect);
                                    }
                                }
                            }
                        }
                    }
                    FadeDirection::Up | FadeDirection::Down => {
                        let mut frame = tick;

                        for y in 0..(state.canvas_size.1 as i32 / 16 + 1) {
                            if frame >= 15 {
                                frame = 15;
                            } else {
                                frame += 1;
                            }

                            if frame >= 0 {
                                rect.left = frame.abs() as u16 * 16;
                                rect.right = rect.left + 16;

                                for x in 0..(state.canvas_size.0 as i32 / 16 + 1) {
                                    if direction == FadeDirection::Down {
                                        batch.add_rect(x as f32 * 16.0, y as f32 * 16.0, &rect);
                                    } else {
                                        batch.add_rect(x as f32 * 16.0, state.canvas_size.1 - y as f32 * 16.0, &rect);
                                    }
                                }
                            }
                        }
                    }
                    FadeDirection::Center => {
                        let center_x = (state.canvas_size.0 / 2.0 - 8.0) as i32;
                        let center_y = (state.canvas_size.1 / 2.0 - 8.0) as i32;
                        let mut start_frame = tick;

                        for x in 0..(center_x / 16 + 2) {
                            let mut frame = start_frame;

                            for y in 0..(center_y / 16 + 2) {
                                if frame >= 15 {
                                    frame = 15;
                                } else {
                                    frame += 1;
                                }

                                if frame >= 0 {
                                    rect.left = frame.abs() as u16 * 16;
                                    rect.right = rect.left + 16;

                                    batch.add_rect((center_x - x * 16) as f32, (center_y + y * 16) as f32, &rect);
                                    batch.add_rect((center_x - x * 16) as f32, (center_y - y * 16) as f32, &rect);
                                    batch.add_rect((center_x + x * 16) as f32, (center_y + y * 16) as f32, &rect);
                                    batch.add_rect((center_x + x * 16) as f32, (center_y - y * 16) as f32, &rect);
                                }
                            }

                            start_frame += 1;
                        }
                    }
                }

                batch.draw(ctx)?;
            }
        }

        Ok(())
    }
}
