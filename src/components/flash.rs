use crate::common::{Color, Rect};
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics;
use crate::shared_game_state::SharedGameState;

pub enum FlashState {
    None,
    Cross(i32, i32, u16),
    Blink(u16),
}

pub struct Flash {
    state: FlashState,
}

impl Flash {
    pub fn new() -> Flash {
        Flash {
            state: FlashState::None
        }
    }

    pub fn set_cross(&mut self, x: i32, y: i32) {
        self.state = FlashState::Cross(x, y, 0);
    }

    pub fn set_blink(&mut self) {
        self.state = FlashState::Blink(0);
    }

    pub fn stop(&mut self) {
        self.state = FlashState::None;
    }
}

impl GameEntity<()> for Flash {
    fn tick(&mut self, _state: &mut SharedGameState, _custom: ()) -> GameResult<()> {
        match self.state {
            FlashState::None => {}
            FlashState::Cross(x, y, tick) => {
                self.state = if tick > 128 {
                    FlashState::None
                } else {
                    FlashState::Cross(x, y, tick + 1)
                };
            }
            FlashState::Blink(tick) => {
                self.state = if tick > 20 {
                    FlashState::None
                } else {
                    FlashState::Blink(tick + 1)
                };
            }
        }
        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult<()> {
        const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);

        match self.state {
            FlashState::None => {}
            FlashState::Cross(x, y, tick) => {
                let tick = tick as f32 + state.frame_time as f32;
                let frame_pos = frame.xy_interpolated(state.frame_time);

                let (cen_x, cen_y) = (
                    (x as f32 / 512.0) - frame_pos.0,
                    (y as f32 / 512.0) - frame_pos.1
                );

                let width = if tick > 100.0 {
                    (1.0 - (tick - 100.0).max(0.0) / 28.0).powf(2.0) * state.canvas_size.0
                } else {
                    (1.0 - (0.97f32).powf(tick)).max(0.0) * state.canvas_size.0
                };


                let mut rect = Rect {
                    left: 0,
                    top: ((cen_y - width) * state.scale) as isize,
                    right: (state.canvas_size.0 * state.scale) as isize,
                    bottom: ((cen_y + width) * state.scale) as isize
                };

                graphics::draw_rect(ctx, rect, WHITE)?;

                if tick <= 100.0 {
                    rect = Rect {
                        left: ((cen_x - width) * state.scale) as isize,
                        top: 0,
                        right: ((cen_x + width) * state.scale) as isize,
                        bottom: (state.canvas_size.1 * state.scale) as isize
                    };

                    graphics::draw_rect(ctx, rect, WHITE)?;
                }
            }
            FlashState::Blink(tick) => {
                if tick / 2 % 2 != 0 {
                    graphics::clear(ctx, WHITE);
                }
            }
        }

        Ok(())
    }
}
