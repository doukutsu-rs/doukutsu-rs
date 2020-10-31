use crate::ggez::GameResult;
use crate::npc::NPC;
use crate::shared_game_state::SharedGameState;
use crate::common::Direction;
use crate::player::Player;

impl NPC {
    pub fn tick_n111_quote_teleport_out(&mut self, state: &mut SharedGameState, player: &Player) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 0;
                self.y -= 16 * 0x200;
            }
            1 => {
                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.vel_y = -0x2ff;
                }
            }
            2 => {
                if self.vel_y > 0 {
                    self.hit_bounds.bottom = 16 * 0x200;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_counter = 0;
                    self.action_num = 3;
                    self.anim_num = 0;
                }
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 40 {
                    self.action_counter = 64;
                    self.action_num = 4;

                    state.sound_manager.play_sfx(29);
                }
            }
            4 => {
                self.anim_num = 0;
                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.cond.set_alive(false);
                }
            }
            _ => {}
        }

        self.vel_y += 0x40;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };
        self.anim_rect = state.constants.npc.n111_quote_teleport_out[self.anim_num as usize + dir_offset];

        if player.equip.has_mimiga_mask() {
            self.anim_rect.top += 32;
            self.anim_rect.bottom += 32;
        }

        if self.action_num == 4 {
            self.anim_rect.bottom = self.anim_rect.top + self.action_counter as usize / 4;

            if self.action_counter / 2 % 2 != 0 {
                self.anim_rect.left += 1;
            }
        }

        Ok(())
    }

    pub fn tick_n112_quote_teleport_in(&mut self, state: &mut SharedGameState, player: &Player) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 0;
                self.anim_counter = 0;
                self.x += 16 * 0x200;
                self.y += 8 * 0x200;

                state.sound_manager.play_sfx(29);
            }
            1 => {
                self.action_counter += 1;
                if self.action_counter >= 64 {
                    self.action_num = 2;
                    self.action_counter = 0;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 3;
                    self.anim_num = 1;
                    self.hit_bounds.bottom = 8 * 0x200;
                }
            }
            3 => {
                if self.flags.hit_bottom_wall() {
                    self.action_counter = 0;
                    self.action_num = 4;
                    self.anim_num = 0;
                }
            }
            _ => {}
        }

        self.vel_y += 0x40;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };
        self.anim_rect = state.constants.npc.n111_quote_teleport_out[self.anim_num as usize + dir_offset];

        if player.equip.has_mimiga_mask() {
            self.anim_rect.top += 32;
            self.anim_rect.bottom += 32;
        }

        if self.action_num == 1 {
            self.anim_rect.bottom = self.anim_rect.top + self.action_counter as usize / 4;

            if self.action_counter / 2 % 2 != 0 {
                self.anim_rect.left += 1;
            }
        }

        Ok(())
    }
}
