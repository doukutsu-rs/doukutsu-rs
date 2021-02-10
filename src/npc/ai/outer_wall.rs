use num_traits::abs;

use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::npc::NPC;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n215_sandcroc_outer_wall(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.target_y = self.y;
                    self.npc_flags.set_shootable(false);
                    self.npc_flags.set_ignore_solidity(false);
                    self.npc_flags.set_invulnerable(false);
                    self.npc_flags.set_solid_soft(false);
                }

                let player = self.get_closest_player_mut(players);
                if abs(self.x - player.x) < 12 * 0x200 && player.y > self.y && player.y < self.y + 8 * 0x200 {
                    self.action_num = 15;
                    self.action_counter = 0;
                }
            }
            15 => {
                self.action_counter += 1;
                if self.action_counter > 10 {
                    state.sound_manager.play_sfx(102);
                    self.action_num = 20;
                }
            }
            20 => {
                self.anim_counter += 1;
                if self.anim_counter > 3 {
                    self.anim_num += 1;
                    self.anim_counter = 0;
                }

                match self.anim_num {
                    3 => self.damage = 15,
                    4 => {
                        self.action_num = 30;
                        self.action_counter = 0;
                        self.npc_flags.set_shootable(true);
                    }
                    _ => {}
                }
            }
            30 => {
                self.damage = 0;
                self.npc_flags.set_solid_soft(true);

                self.action_counter += 1;
                if self.shock > 0 {
                    self.action_num = 40;
                    self.action_counter = 0;
                }
            }
            40 => {
                self.npc_flags.set_ignore_solidity(true);
                self.y += 0x200;
                self.action_counter += 1;
                if self.action_counter == 32 {
                    self.action_num = 50;
                    self.action_counter = 0;
                    self.npc_flags.set_solid_soft(false);
                    self.npc_flags.set_shootable(false);
                }
            }
            50 => {
                if self.action_counter > 99 {
                    self.y = self.target_y;
                    self.action_num = 0;
                    self.anim_num = 0;
                } else {
                    self.action_counter += 1;
                }
            }
            _ => {}
        }

        self.anim_rect = state.constants.npc.n215_sandcroc_outer_wall[self.anim_num as usize];

        Ok(())
    }
}
