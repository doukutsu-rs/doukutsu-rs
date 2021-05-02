use crate::common::Direction;
use crate::framework::error::GameResult;
use crate::npc::NPC;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n139_doctor(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.vel_x = 0;
                    self.vel_y = 0;
                    self.y = self.y + -0x1000;
                }

                if !self.flags.hit_bottom_wall() {
                    self.anim_num = 2;
                } else {
                    self.anim_num = 0;
                }

                self.vel_y = self.vel_y + 0x40;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 0xb;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                    self.action_counter3 = 0;
                }

                self.anim_counter = self.anim_counter + 1;
                if 6 < self.anim_counter {
                    self.anim_counter = 0;
                    self.anim_num = self.anim_num + 1;
                }

                if 1 < self.anim_num {
                    self.anim_num = 0;
                    self.action_counter3 = self.action_counter3 + 1;
                }

                if 8 < self.action_counter3 {
                    self.anim_num = 0;
                    self.action_num = 1;
                }
            }
            0x14 | 0x15 => {
                if self.action_num == 0x14 {
                    self.action_num = 0x15;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    self.target_y = self.y + -0x4000;
                }

                if self.y < self.target_y {
                    self.vel_y = self.vel_y + 0x20;
                } else {
                    self.vel_y = self.vel_y + -0x20;
                }

                self.vel_y = self.vel_y.clamp(-0x200, 0x200);
            }
            0x1e | 0x1f => {
                if self.action_num == 0x1e {
                    self.action_num = 0x1f;
                    self.vel_x = 0;
                    self.vel_y = 0;
                    self.action_counter = (self.anim_rect.bottom - self.anim_rect.top) * 2;
                    state.sound_manager.play_sfx(0x1d);
                }

                self.action_counter = self.action_counter.saturating_sub(1);
                self.anim_num = 0;

                if self.action_counter == 0 {
                    self.cond.set_alive(false);
                }
            }
            0x28 | 0x29 => {
                if self.action_num == 0x28 {
                    self.action_num = 0x29;
                    self.action_counter = 0;
                    self.vel_x = 0;
                    self.vel_y = 0;
                    state.sound_manager.play_sfx(0x1d);
                }
                self.anim_num = 2;
                self.action_counter = self.action_counter + 1;
                if 0x3f < self.action_counter {
                    self.action_num = 0x14;
                }
            }
            _ => {}
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n139_doctor[self.anim_num as usize + dir_offset];

        if self.action_num == 31 || self.action_num == 41 {
            self.anim_rect.bottom = self.action_counter / 2 + self.anim_rect.top;
            if ((self.action_counter / 2) & 1) != 0 {
                self.anim_rect.left += 1;
            }
        }

        Ok(())
    }
}
