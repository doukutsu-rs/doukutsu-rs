use crate::framework::error::GameResult;

use crate::npc::NPC;
use crate::shared_game_state::SharedGameState;
use crate::common::Direction;

impl NPC {
    pub(crate) fn tick_n127_machine_gun_trail_l2(&mut self, state: &mut SharedGameState ) -> GameResult {
        self.anim_counter += 1;
        if self.anim_counter > 0 {
            self.anim_counter = 0;
            self.anim_num += 1;
            if self.anim_num > 2 {
                self.cond.set_alive(false);

                return Ok(());
            }
        }

        match self.direction {
            Direction::Left | Direction::Right => {
                self.anim_rect = state.constants.npc.n127_machine_gun_trail_l2[self.anim_num as usize];
            }
            Direction::Up | Direction::Bottom => {
                self.anim_rect = state.constants.npc.n127_machine_gun_trail_l2[self.anim_num as usize + 3];
            }
            _ => {}
        }

        Ok(())
    }

    pub(crate) fn tick_n128_machine_gun_trail_l3(&mut self, state: &mut SharedGameState ) -> GameResult {
        self.anim_counter += 1;
        if self.anim_counter > 0 {
            self.anim_counter = 0;
            self.anim_num += 1;
            if self.anim_num > 4 {
                self.cond.set_alive(false);

                return Ok(());
            }
        }

        if self.action_num == 0 {
            self.action_num = 1;

            match self.direction {
                Direction::Left | Direction::Right => {
                    self.display_bounds.left = 0x800;
                    self.display_bounds.top = 0x1000;
                }
                Direction::Bottom | Direction::Up => {
                    self.display_bounds.left = 0x1000;
                    self.display_bounds.top = 0x800;
                }
                _ => {}
            }
        }

        match self.direction {
            Direction::Left => {
                self.anim_rect = state.constants.npc.n128_machine_gun_trail_l3[self.anim_num as usize];
            }
            Direction::Up => {
                self.anim_rect = state.constants.npc.n128_machine_gun_trail_l3[self.anim_num as usize + 5];
            }
            Direction::Right => {
                self.anim_rect = state.constants.npc.n128_machine_gun_trail_l3[self.anim_num as usize + 10];
            }
            Direction::Bottom => {
                self.anim_rect = state.constants.npc.n128_machine_gun_trail_l3[self.anim_num as usize + 15];
            }
            _ => {}
        }

        Ok(())
    }

    pub(crate) fn tick_n129_fireball_snake_trail(&mut self, state: &mut SharedGameState) -> GameResult {
        self.anim_counter += 1;

        if self.anim_counter > 1 {
            self.anim_counter = 0;

            self.anim_num += 1;
            if self.anim_num > 2 {
                self.cond.set_alive(false);
                return Ok(());
            }
        }

        self.y += self.vel_y;

        if self.anim_counter == 1 {
            let frame = (self.action_counter2 as usize % 6) * 3 + self.anim_num as usize;
            self.anim_rect = state.constants.npc.n129_fireball_snake_trail[frame];
        }

        Ok(())
    }
}
