use crate::common::Direction;
use ggez::GameResult;
use crate::npc::NPC;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n083_igor_cutscene(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.vel_x = 0;
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.action_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 5 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 1 {
                        self.anim_num = 0;
                    }
                }
            }
            2 | 3 => {
                if self.action_num == 2 {
                    self.action_num = 3;
                    self.anim_num = 2;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 3 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 5 {
                        self.anim_num = 2;
                    }
                }

                self.vel_x = self.direction.vector_x() * 0x200;
            }
            4 | 5 => {
                if self.action_num == 4 {
                    self.vel_x = 0;
                    self.action_num = 5;
                    self.action_counter = 0;
                    self.anim_num = 6;
                }

                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.action_counter = 0;
                    self.action_num = 6;
                    self.anim_num = 7;

                    state.sound_manager.play_sfx(70);
                }
            }
            6 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 0;
                    self.anim_num = 0;
                }
            }
            7 => {
                self.action_num = 1;
            }
            _ => {}
        }

        self.vel_y += 0x40;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 8 };
        self.anim_rect = state.constants.npc.n083_igor_cutscene[self.anim_num as usize + dir_offset];

        Ok(())
    }
}
