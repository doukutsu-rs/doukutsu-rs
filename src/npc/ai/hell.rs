use crate::{GameResult, SharedGameState};
use crate::caret::CaretType;
use crate::common::Direction;
use crate::npc::NPC;
use crate::rng::RNG;

impl NPC {
    pub(crate) fn tick_n337_numahachi(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.y -= 0x1000;
        }

        if self.action_num == 1 {
            self.action_num = 2;
            self.anim_num = 0;
            self.vel_x = 0;
        }

        if self.action_num == 2{
            self.animate(50, 0, 1);
        }

        self.vel_y += 0x40;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n337_numahachi[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n357_puppy_ghost(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 => {
                self.anim_rect = state.constants.npc.n357_puppy_ghost;
                self.action_counter += 1;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    state.sound_manager.play_sfx(29);
                }

                self.action_counter += 1;
                self.anim_rect = state.constants.npc.n357_puppy_ghost;

                if self.action_counter & 2 != 0 {
                    self.anim_rect.right = self.anim_rect.left;
                }

                if self.action_counter > 50 {
                    self.cond.set_alive(false);
                }
            }
            _ => (),
        }

        if self.action_counter & 8 == 1 {
            state.create_caret(
                self.x + self.rng.range(-8..8) * 0x200,
                self.y + 0x1000,
                CaretType::LittleParticles,
                Direction::Up,
            );
        }

        Ok(())
    }
}
