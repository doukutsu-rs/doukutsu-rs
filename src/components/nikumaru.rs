use byteorder::{LE, ReadBytesExt, WriteBytesExt};

use crate::common::Rect;
use crate::components::draw_common::{Alignment, draw_number, draw_number_zeros};
use crate::entity::GameEntity;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::framework::vfs::OpenOptions;
use crate::game::frame::Frame;
use crate::game::shared_game_state::{SharedGameState, TimingMode};
use crate::game::player::Player;
use crate::game::scripting::tsc::text_script::TextScriptExecutionState;
use crate::util::rng::RNG;

#[derive(Clone, Copy)]
pub struct NikumaruCounter {
    pub tick: usize,
    pub shown: bool,
}

impl NikumaruCounter {
    pub fn new() -> NikumaruCounter {
        NikumaruCounter { tick: 0, shown: false }
    }

    fn load_time(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult<u32> {
        if let Ok(mut data) = filesystem::user_open(ctx, [state.get_rec_filename(), ".rec".to_string()].join("")) {
            let mut ticks: [u32; 4] = [0; 4];

            for iter in 0..=3 {
                ticks[iter] = data.read_u32::<LE>()?;
            }

            let random = data.read_u32::<LE>()?;
            let random_list: [u8; 4] = random.to_le_bytes();

            for iter in 0..=3 {
                ticks[iter] = u32::from_le_bytes([
                    ticks[iter].to_le_bytes()[0].wrapping_sub(random_list[iter]),
                    ticks[iter].to_le_bytes()[1].wrapping_sub(random_list[iter]),
                    ticks[iter].to_le_bytes()[2].wrapping_sub(random_list[iter]),
                    ticks[iter].to_le_bytes()[3].wrapping_sub(random_list[iter] / 2),
                ]);
            }

            if ticks[0] == ticks[1] && ticks[0] == ticks[2] {
                return Ok(ticks[0]);
            }
        } else {
            log::warn!("Failed to open 290 record.");
        }
        Ok(0)
    }

    fn save_time(&mut self, new_time: u32, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if let Ok(mut data) = filesystem::open_options(
            ctx,
            [state.get_rec_filename(), ".rec".to_string()].join(""),
            OpenOptions::new().write(true).create(true),
        ) {
            let mut ticks: [u32; 4] = [new_time; 4];
            let mut random_list: [u8; 4] = [0; 4];

            for iter in 0..=3 {
                random_list[iter] = state.effect_rng.range(0..250) as u8 + iter as u8;

                ticks[iter] = u32::from_le_bytes([
                    ticks[iter].to_le_bytes()[0].wrapping_add(random_list[iter]),
                    ticks[iter].to_le_bytes()[1].wrapping_add(random_list[iter]),
                    ticks[iter].to_le_bytes()[2].wrapping_add(random_list[iter]),
                    ticks[iter].to_le_bytes()[3].wrapping_add(random_list[iter] / 2),
                ]);

                data.write_u32::<LE>(ticks[iter])?;
            }

            data.write_u32::<LE>(u32::from_le_bytes(random_list))?;
        } else {
            log::warn!("Failed to write 290 record.");
        }
        Ok(())
    }

    pub fn load_counter(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.tick = self.load_time(state, ctx)? as usize;
        if self.tick > 0 {
            self.shown = true;
        } else {
            self.shown = false;
        }
        Ok(())
    }

    pub fn save_counter(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult<bool> {
        let old_record = self.load_time(state, ctx)? as usize;
        if self.tick < old_record || old_record == 0 {
            self.save_time(self.tick as u32, state, ctx)?;
            return Ok(true);
        }
        Ok(false)
    }
}

impl GameEntity<&Player> for NikumaruCounter {
    fn tick(&mut self, state: &mut SharedGameState, player: &Player) -> GameResult {
        if !player.equip.has_nikumaru() {
            self.tick = 0;
            self.shown = false;
            return Ok(());
        }

        self.shown = true;

        if state.control_flags.control_enabled() {
            self.tick += 1;
        }

        if self.tick >= 300000 {
            self.tick = 300000;
        }

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, _frame: &Frame) -> GameResult {
        if !self.shown {
            return Ok(());
        }

        if state.textscript_vm.state == TextScriptExecutionState::MapSystem {
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
