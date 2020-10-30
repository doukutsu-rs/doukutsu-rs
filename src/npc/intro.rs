use crate::caret::CaretType;
use crate::common::Direction;
use crate::ggez::GameResult;
use crate::npc::NPC;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n298_intro_doctor(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.y -= 8 * 0x200;
                }

                self.anim_num = 0;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.action_counter2 = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 6 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 1 {
                        self.anim_num = 0;
                        self.action_counter2 += 1;
                        if self.action_counter2 > 7 {
                            self.anim_num = 0;
                            self.action_num = 1;
                        }
                    }
                }
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.anim_num = 2;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 10 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 5 {
                        self.anim_num = 2;
                    }
                }

                self.x += 0x100;
            }
            30 => {
                self.anim_num = 6;
            }
            40 | 41 => {
                if self.action_num == 40 {
                    self.action_num = 41;
                    self.anim_num = 6;
                    self.anim_counter = 0;
                    self.action_counter2 = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 6 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 7 {
                        self.anim_num = 6;
                        self.action_counter2 += 1;
                        if self.action_counter2 > 7 {
                            self.anim_num = 6;
                            self.action_num = 30;
                        }
                    }
                }
            }
            _ => {}
        }

        self.anim_rect = state.constants.npc.n298_intro_doctor[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n299_intro_balrog_misery(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;

            match self.direction {
                Direction::Left => {
                    self.anim_num = 1;
                    self.action_counter = 25;
                    self.y -= 0x40 * 25;
                }
                Direction::Right => {
                    self.anim_num = 0;
                    self.action_counter = 0;
                }
                _ => {}
            }
        }

        self.action_counter += 1;
        self.y += if (self.action_counter / 50) % 2 != 0 {
            0x40
        } else {
            -0x40
        };

        self.anim_rect = state.constants.npc.n299_intro_balrog_misery[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n300_intro_demon_crown(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.y += 6 * 0x200;
            self.anim_rect = state.constants.npc.n300_intro_demon_crown;
        }

        self.anim_counter += 1;
        if (self.anim_counter % 8) == 1 {
            state.create_caret(self.x + state.effect_rng.range(-8..8) as isize * 0x200,
                               self.y + 8 * 0x200,
                               CaretType::LittleParticles, Direction::Up);
        }

        Ok(())
    }
}
