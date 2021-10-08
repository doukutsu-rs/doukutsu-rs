use crate::common::Direction;
use crate::framework::error::GameResult;
use crate::npc::boss::BossNPC;
use crate::npc::NPC;
use crate::shared_game_state::SharedGameState;
use crate::rng::RNG;

impl NPC {
    pub(crate) fn tick_n196_ironhead_wall(&mut self, state: &mut SharedGameState) -> GameResult {
        self.x -= 0xC00;
        if self.x <= 0x26000 {
            self.x += 0x2C000;
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 1 };

        self.anim_rect = state.constants.npc.n196_ironhead_wall[dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n197_porcupine_fish(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 10 => {
                if self.action_num == 0 {
                    self.action_num = 10;
                    self.anim_counter = 0;
                    self.vel_y = self.rng.range(-0x200..0x200);
                    self.vel_x = 0x800;
                }

                self.animate(2, 0, 1);

                if self.vel_x < 0 {
                    self.damage = 3;
                    self.action_num = 20;
                }
            }
            20 => {
                self.damage = 3;
                self.animate(0, 2, 3);

                if self.x <= 0x5FFF {
                    // npc->destroy_voice = 0; // todo
                    self.cond.set_explode_die(true);
                }
            }
            _ => (),
        }

        if self.flags.hit_top_wall() {
            self.vel_y = 0x200;
        }
        if self.flags.hit_bottom_wall() {
            self.vel_y = -0x200;
        }

        self.vel_x -= 0xC;
        self.x += self.vel_x;
        self.y += self.vel_y;
        self.anim_rect = state.constants.npc.n197_porcupine_fish[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n198_ironhead_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_counter += 1;
            if self.action_counter > 20 {
                self.action_num = 1;
                self.vel_x = 0;
                self.vel_y = 0;
                self.action_counter3 = 0;
            }
        } else if self.action_num == 1 {
            self.vel_x += 0x20;
        }

        self.animate(0, 0, 2);
        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n198_ironhead_projectile[self.anim_num as usize];

        self.action_counter3 += 1;
        if self.action_counter3 > 100 {
            self.cond.set_alive(false);
        }

        if self.action_counter3 % 4 == 1 {
            state.sound_manager.play_sfx(46);
        }

        Ok(())
    }
}

impl BossNPC {
    pub(crate) fn tick_b05_ironhead(&mut self) {}
}
