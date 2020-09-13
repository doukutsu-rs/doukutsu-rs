use nalgebra::clamp;

use crate::common::Direction;
use crate::ggez::GameResult;
use crate::npc::NPC;
use crate::player::Player;
use crate::SharedGameState;

impl NPC {
    pub(crate) fn tick_n002_behemoth(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.flags.hit_left_wall() {
            self.direction = Direction::Right;
        } else if self.flags.hit_right_wall() {
            self.direction = Direction::Left;
        }

        match self.action_num {
            0 => {
                self.vel_x = match self.direction {
                    Direction::Left => { -0x100 }
                    Direction::Right => { 0x100 }
                    _ => { 0 }
                };

                self.anim_counter += 1;
                if self.anim_counter > 8 {
                    self.anim_counter = 0;
                    self.anim_num = (self.anim_num + 1) % 3;
                    self.anim_rect = state.constants.npc.n002_behemoth[self.anim_num as usize + if self.direction == Direction::Right { 7 } else { 0 }];
                }

                if self.shock > 0 {
                    self.action_counter = 0;
                    self.action_num = 1;
                    self.anim_num = 4;
                    self.anim_rect = state.constants.npc.n002_behemoth[self.anim_num as usize + if self.direction == Direction::Right { 7 } else { 0 }];
                }
            }
            1 => {
                self.vel_x = (self.vel_x * 7) / 8;

                self.action_counter += 1;
                if self.action_counter > 40 {
                    if self.shock > 0 {
                        self.action_counter = 0;
                        self.action_num = 2;
                        self.anim_num = 6;
                        self.anim_counter = 0;
                        self.damage = 5;
                        self.anim_rect = state.constants.npc.n002_behemoth[self.anim_num as usize + if self.direction == Direction::Right { 7 } else { 0 }];
                    } else {
                        self.action_num = 0;
                        self.anim_counter = 0;
                    }
                }
            }
            2 => {
                self.vel_x = match self.direction {
                    Direction::Left => { -0x400 }
                    Direction::Right => { 0x400 }
                    _ => { 0 }
                };

                self.action_counter += 1;
                if self.action_counter > 200 {
                    self.action_num = 0;
                    self.damage = 1;
                }

                self.anim_counter += 1;
                if self.anim_counter > 5 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 6 {
                        self.anim_num = 5;
                        // todo play sound 26
                        state.quake_counter = 8;
                    }

                    self.anim_rect = state.constants.npc.n002_behemoth[self.anim_num as usize + if self.direction == Direction::Right { 7 } else { 0 }];
                }
            }
            _ => {}
        }

        self.vel_y += 0x40;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        Ok(())
    }

    pub(crate) fn tick_n005_green_critter(&mut self, state: &mut SharedGameState, player: &Player) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.y += 3 * 0x200;
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_rect = state.constants.npc.n005_green_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                }

                if self.x > player.x {
                    self.direction = Direction::Left;
                } else {
                    self.direction = Direction::Right;
                }

                if self.target_x < 100 {
                    self.target_x += 1;
                }

                if self.action_counter >= 8
                    && self.x - (112 * 0x200) < player.x
                    && self.x + (112 * 0x200) > player.x
                    && self.y - (80 * 0x200) < player.y
                    && self.y + (80 * 0x200) > player.y {
                    if self.anim_num != 1 {
                        self.anim_num = 1;
                        self.anim_rect = state.constants.npc.n005_green_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }
                } else {
                    if self.action_counter < 8 {
                        self.action_counter += 1;
                    }

                    if self.anim_num != 0 {
                        self.anim_num = 0;
                        self.anim_rect = state.constants.npc.n005_green_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }
                }

                if self.shock > 0 {
                    self.action_num = 2;
                    self.action_counter = 0;

                    if self.anim_num != 0 {
                        self.anim_num = 0;
                        self.anim_rect = state.constants.npc.n005_green_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }
                }

                if self.action_counter >= 8
                    && self.target_x >= 100
                    && self.x - (64 * 0x200) < player.x
                    && self.x + (64 * 0x200) > player.x
                    && self.y - (80 * 0x200) < player.y
                    && self.y + (80 * 0x200) > player.y {
                    self.action_num = 2;
                    self.action_counter = 0;

                    if self.anim_num != 0 {
                        self.anim_num = 0;
                        self.anim_rect = state.constants.npc.n005_green_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 3;

                    if self.anim_num != 2 {
                        self.anim_num = 2;
                        self.anim_rect = state.constants.npc.n005_green_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }

                    self.vel_y = -0x5ff;
                    // todo play sound 30

                    if self.direction == Direction::Left {
                        self.vel_x = -0x100;
                    } else {
                        self.vel_x = 0x100;
                    }
                }
            }
            3 => {
                if self.flags.hit_bottom_wall() {
                    self.vel_x = 0;
                    self.action_counter = 0;
                    self.action_num = 1;

                    // tood play sound 23

                    if self.anim_num != 0 {
                        self.anim_num = 0;
                        self.anim_rect = state.constants.npc.n005_green_critter[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
                    }
                }
            }
            _ => {}
        }

        self.vel_y += 0x40;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        Ok(())
    }

    pub(crate) fn tick_n006_green_beetle(&mut self, state: &mut SharedGameState, player: &Player) -> GameResult {

        Ok(())
    }

    pub(crate) fn tick_n007_basil(&mut self, state: &mut SharedGameState, player: &Player) -> GameResult {
        match self.action_num {
            0 => {
                self.x = player.x;

                if self.direction == Direction::Left {
                    self.action_num = 1;
                } else {
                    self.action_num = 2;
                }
            }
            1 => {
                self.vel_x -= 0x40;

                if self.x < (player.x - 192 * 0x200) {
                    self.action_num = 2;
                }

                if self.flags.hit_left_wall() {
                    self.vel_x = 0;
                    self.action_num = 2;
                }
            }
            2 => {
                self.vel_x += 0x40;

                if self.x > (player.x + 192 * 0x200) {
                    self.action_num = 1;
                }

                if self.flags.hit_right_wall() {
                    self.vel_x = 0;
                    self.action_num = 1;
                }
            }
            _ => {}
        }

        if self.vel_x < 0 {
            self.direction = Direction::Left;
        } else {
            self.direction = Direction::Right;
        }

        self.vel_x = clamp(self.vel_x, -0x5ff, 0x5ff);

        self.x += self.vel_x;

        self.anim_counter += 1;
        if self.anim_counter > 1 {
            self.anim_counter = 0;
            self.anim_num = (self.anim_num + 1) % 2;
            self.anim_rect = state.constants.npc.n007_basil[self.anim_num as usize + if self.direction == Direction::Right { 3 } else { 0 }];
        }

        Ok(())
    }
}
