use crate::common::Direction;
use crate::framework::error::GameResult;
use crate::npc::NPC;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n241_critter_red(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.y += 3 * 0x200;
                    self.action_num = 1;
                    self.anim_num = 0;
                }

                let player = self.get_closest_player_mut(players);
                if self.x > player.x {
                    self.direction = Direction::Left;
                } else {
                    self.direction = Direction::Right;
                }

                if self.target_x < 100 {
                    self.target_x += 1;
                }

                if self.action_counter >= 8
                    && self.x - (144 * 0x200) < player.x
                    && self.x + (144 * 0x200) > player.x
                    && self.y - (96 * 0x200) < player.y
                    && self.y + (96 * 0x200) > player.y
                {
                    self.anim_num = 1;
                } else {
                    if self.action_counter < 8 {
                        self.action_counter += 1;
                    }

                    self.anim_num = 0;
                }

                if self.shock > 0 {
                    self.action_num = 2;
                    self.action_counter = 0;

                    self.anim_num = 0;
                }

                if self.action_counter >= 8
                    && self.target_x >= 100
                    && self.x - (96 * 0x200) < player.x
                    && self.x + (96 * 0x200) > player.x
                    && self.y - (80 * 0x200) < player.y
                    && self.y + (80 * 0x200) > player.y
                {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 3;
                    self.anim_num = 2;

                    self.vel_y = -0x5ff;
                    state.sound_manager.play_sfx(30);

                    if self.direction == Direction::Left {
                        self.vel_x = -0x200;
                    } else {
                        self.vel_x = 0x200;
                    }
                }
            }
            3 => {
                if self.flags.hit_bottom_wall() {
                    self.vel_x = 0;
                    self.action_counter = 0;
                    self.action_num = 1;
                    self.anim_num = 0;

                    state.sound_manager.play_sfx(23);
                }
            }
            _ => {}
        }

        self.vel_y += 0x55;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n241_critter_red[self.anim_num as usize + dir_offset];

        Ok(())
    }
}
