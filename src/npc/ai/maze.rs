use crate::framework::context::Context;
use crate::framework::error::GameResult;
use num_traits::clamp;

use crate::caret::CaretType;
use crate::common::Direction;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n154_gaudi_dead(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 => {
                self.npc_flags.set_shootable(false);
                self.npc_flags.set_ignore_solidity(false);
                self.damage = 0;
                self.action_num = 1;
                self.anim_num = 0;
                self.vel_y = -0x200;

                match self.direction {
                    Direction::Left => self.vel_x = 0x100,
                    Direction::Right => self.vel_x = -0x100,
                    _ => {}
                };
                state.sound_manager.play_sfx(53);
            }
            1 if self.flags.hit_bottom_wall() => {
                self.action_num = 2;
                self.action_counter = 0;
                self.anim_num = 1;
                self.anim_counter = 0;
            }
            2 => {
                self.vel_x = 8 * self.vel_x / 9;
                self.animate(3, 1, 2);

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.cond.set_explode_die(true);
                }
            }
            _ => {}
        }

        self.vel_y += 0x20;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n154_gaudi_dead[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n156_gaudi_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_counter > 300 || (self.flags.0 & 0xff) != 0 {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_num = (self.anim_num + 1) % 3;

        self.anim_rect = state.constants.npc.n156_gaudi_projectile[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n166_chaba(&mut self, state: &mut SharedGameState) -> GameResult {
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
            _ => {}
        }


        self.anim_rect = state.constants.npc.n166_chaba[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n173_gaudi_armored(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        let player = self.get_closest_player_mut(players);

        Ok(())
    }
}
