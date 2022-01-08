use crate::common::Rect;
use crate::components::draw_common::{Alignment, draw_number, draw_number_zeros};
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::player::Player;
use crate::shared_game_state::{SharedGameState, TimingMode};

pub struct NikumaruCounter {
    pub tick: usize,
    pub shown: bool,
}

impl NikumaruCounter {
    pub fn new() -> NikumaruCounter {
        NikumaruCounter { tick: 0, shown: false }
    }
}

impl GameEntity<&Player> for NikumaruCounter {
    fn tick(&mut self, _state: &mut SharedGameState, player: &Player) -> GameResult {
        if !player.equip.has_nikumaru() {
            self.tick = 0;
            self.shown = false;
            return Ok(());
        }

        self.shown = true;
        self.tick += 1;

        if self.tick >= 300000 {
            self.tick = 300000;
        }

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, _frame: &Frame) -> GameResult {
        if !self.shown {
            return Ok(());
        }

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;

        let x = 16.0;
        let y = 8.0;

        const CLOCK_RECTS: [Rect<u16>; 2] = [
            Rect { left: 112, top: 104, right: 120, bottom: 112 },
            Rect { left: 120, top: 104, right: 128, bottom: 112 },
        ];
        const PRIME: Rect<u16> = Rect { left: 128, top: 104, right: 160, bottom: 112 };

        let (one_tenth, second, minute) = match state.settings.timing_mode {
            TimingMode::_60Hz => (6, 60, 3600),
            _ => (5, 50, 3000),
        };

        if self.tick % 30 <= 10 {
            batch.add_rect(x, y, &CLOCK_RECTS[1]);
        } else {
            batch.add_rect(x, y, &CLOCK_RECTS[0]);
        }
        batch.add_rect(x + 30.0, y, &PRIME);

        batch.draw(ctx)?;

        draw_number(x + 32.0, y, self.tick / minute, Alignment::Right, state, ctx)?;
        draw_number_zeros(x + 52.0, y, (self.tick / second) % 60, Alignment::Right, 2, state, ctx)?;
        draw_number(x + 64.0, y, (self.tick / one_tenth) % 10, Alignment::Right, state, ctx)?;

        Ok(())
    }
}
