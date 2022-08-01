use crate::common::{interpolate_fix9_scale, Rect};
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::shared_game_state::SharedGameState;

#[derive(Debug, Copy, Clone, bincode::Encode, bincode::Decode)]
pub struct NumberPopup {
    pub value: i16,
    pub x: i32,
    pub y: i32,
    pub prev_x: i32,
    pub prev_y: i32,
    counter: u16,
}

impl NumberPopup {
    pub fn new() -> NumberPopup {
        NumberPopup { value: 0, x: 0, y: 0, prev_x: 0, prev_y: 0, counter: 0 }
    }

    pub fn set_value(&mut self, value: i16) {
        if self.counter > 32 {
            self.counter = 32;
        }

        self.value = value;
    }

    pub fn add_value(&mut self, value: i16) {
        self.set_value(self.value + value);
    }
}

impl GameEntity<()> for NumberPopup {
    fn tick(&mut self, _state: &mut SharedGameState, _custom: ()) -> GameResult<()> {
        if self.value == 0 {
            return Ok(());
        }

        self.counter += 1;
        if self.counter == 80 {
            self.counter = 0;
            self.value = 0;
        }

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult<()> {
        if self.value == 0 {
            return Ok(());
        }

        // tick 0 - 32 - move up by 0.5 pixels
        // tick 33 - 72 - stay
        // tick 73 - 80 - fade up
        let y_offset = self.counter.min(32) as f32 * 0.5;
        let clip = self.counter.max(72) - 72;

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;

        let (frame_x, frame_y) = frame.xy_interpolated(state.frame_time);
        let x = interpolate_fix9_scale(self.prev_x, self.x, state.frame_time) - frame_x;
        let y = interpolate_fix9_scale(self.prev_y, self.y, state.frame_time) - frame_y - y_offset;

        let n = format!("{:+}", self.value);

        let x = x - n.len() as f32 * 4.0;

        for (offset, chr) in n.chars().enumerate() {
            match chr {
                '+' => {
                    batch.add_rect(x + offset as f32 * 8.0, y, &Rect::new_size(32, 48 + clip, 8, 8 - clip));
                }
                '-' => {
                    batch.add_rect(x + offset as f32 * 8.0, y, &Rect::new_size(40, 48 + clip, 8, 8 - clip));
                }
                '0'..='9' => {
                    let number_set = if self.value < 0 { 64 } else { 56 };
                    let idx = chr as u16 - '0' as u16;
                    batch.add_rect(
                        x + offset as f32 * 8.0,
                        y,
                        &Rect::new_size(idx * 8, number_set + clip, 8, 8 - clip),
                    );
                }
                _ => {}
            }
        }

        batch.draw(ctx)?;

        Ok(())
    }
}
