use crate::common::Direction;
use crate::framework::error::GameResult;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n220_shovel_brigade(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }

            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n220_shovel_brigade[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n221_shovel_brigade_walking(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.vel_x = 0;
                }

                if self.rng.range(0..60) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
                if self.rng.range(0..60) == 1 {
                    self.action_num = 10;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = self.rng.range(0..16) as u16;
                    self.anim_num = 2;
                    self.anim_counter = 0;

                    if (self.rng.range(0..9) & 1) != 0 {
                        self.direction = Direction::Left;
                    } else {
                        self.direction = Direction::Right;
                    }
                }

                if self.direction != Direction::Left || !self.flags.hit_left_wall() {
                    if self.direction == Direction::Right && self.flags.hit_right_wall() {
                        self.direction = Direction::Left;
                    }
                } else {
                    self.direction = Direction::Right;
                }

                if self.direction != Direction::Left {
                    self.vel_x = 0x200;
                } else {
                    self.vel_x = -0x200;
                }

                self.animate(4, 2, 5);

                self.action_counter += 1;
                if self.action_counter > 32 {
                    self.action_num = 0;
                }
            }
            _ => (),
        }
        self.vel_y += 0x20;
        if self.vel_y > 0x5FF {
            self.vel_y = 0x5FF;
        }
        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };

        self.anim_rect = state.constants.npc.n221_shovel_brigade_walking[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n223_momorin(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                if self.rng.range(0..160) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 12 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            3 => {
                self.anim_num = 2;
            }
            _ => (),
        }

        let player = self.get_closest_player_ref(&players);

        if self.action_num <= 1 && player.y < self.y + 0x2000 && player.y > self.y - 0x2000 {
            if player.x >= self.x {
                self.direction = Direction::Right;
            } else {
                self.direction = Direction::Left;
            }
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n223_momorin[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n224_chie(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                if self.rng.range(0..160) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 12 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }
        let player = self.get_closest_player_ref(&players);

        if self.action_num <= 1 && player.y < self.y + 0x2000 && player.y > self.y - 0x2000 {
            if player.x >= self.x {
                self.direction = Direction::Right;
            } else {
                self.direction = Direction::Left;
            }
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n224_chie[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n225_megane(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_num = 0;
            self.anim_counter = 0;
        }

        if self.action_num == 1 {
            if self.rng.range(0..160) == 1 {
                self.action_num = 2;
                self.action_counter = 0;
                self.anim_num = 1;
            }
        } else if self.action_num == 2 {
            self.action_counter += 1;

            if self.action_counter > 12 {
                self.action_num = 1;
                self.anim_num = 0;
            }
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n225_megane[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n226_kanpachi_plantation(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.vel_x = 0;
                }
                if self.rng.range(0..60) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 2;
                    self.anim_counter = 0;
                }

                self.vel_x = 0x200;

                self.animate(4, 2, 5);
                self.action_counter += 1;
            }
            20 => {
                self.vel_x = 0;
                self.anim_num = 6;
            }
            _ => (),
        }

        self.vel_y += 0x20;

        if self.vel_y > 0x5FF {
            self.vel_y = 0x5FF;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n226_kanpachi_plantation[self.anim_num as usize];
        Ok(())
    }
}
