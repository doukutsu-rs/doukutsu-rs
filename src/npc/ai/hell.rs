use crate::{GameResult, SharedGameState};
use crate::caret::CaretType;
use crate::common::Direction;
use crate::npc::NPC;
use crate::npc::list::NPCList;
use crate::rng::RNG;
use crate::player::Player;

impl NPC {
    pub(crate) fn tick_n337_numahachi(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.y -= 0x1000;
        }

        if self.action_num == 1 {
            self.action_num = 2;
            self.anim_num = 0;
            self.vel_x = 0;
        }

        if self.action_num == 2{
            self.animate(50, 0, 1);
        }

        self.vel_y += 0x40;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n337_numahachi[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n357_puppy_ghost(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 => {
                self.anim_rect = state.constants.npc.n357_puppy_ghost;
                self.action_counter += 1;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    state.sound_manager.play_sfx(29);
                }

                self.action_counter += 1;
                self.anim_rect = state.constants.npc.n357_puppy_ghost;

                if self.action_counter & 2 != 0 {
                    self.anim_rect.right = self.anim_rect.left;
                }

                if self.action_counter > 50 {
                    self.cond.set_alive(false);
                }
            }
            _ => (),
        }

        if self.action_counter & 8 == 1 {
            state.create_caret(
                self.x + self.rng.range(-8..8) * 0x200,
                self.y + 0x1000,
                CaretType::LittleParticles,
                Direction::Up,
            );
        }

        Ok(())
    }

    pub(crate) fn tick_n309_bute(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {

        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                self.action_num = 1;

                if (self.direction == Direction::Left && player.x > self.x - 0x24000 && player.x < self.x - 0x22000) || 
                   (player.x < self.x + 0x24000 && player.x > self.x + 0x22000) {
                    self.action_num = 10;
                    return Ok(())
                }

            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.npc_flags.set_shootable(true);
                    self.damage = 5;
                }

                if self.x > player.x {
                    self.direction = Direction::Left;
                    self.vel_x2 -= 0x10;
                }
                else {
                    self.direction = Direction::Right;
                    self.vel_x2 += 0x10;
                }

                if self.y > player.y {
                    self.vel_y2 -= 0x10;
                }
                else {
                    self.vel_y2 += 0x10;
                }

                if self.vel_x2 < 0 && self.flags.hit_left_wall() { self.vel_x2 *= -1 };
                if self.vel_x2 > 0 && self.flags.hit_right_wall() { self.vel_x2 *= -1 };
                if self.vel_y2 < 0 && self.flags.hit_top_wall() { self.vel_y2 *= -1 };
                if self.vel_y2 > 0 && self.flags.hit_bottom_wall() { self.vel_y2 *= -1 };

                self.vel_x2 = self.vel_x2.clamp(-0x5FF, 0x5FF);
                self.vel_y2 = self.vel_y2.clamp(-0x5FF, 0x5FF);

                self.x += self.vel_x2;
                self.y += self.vel_y2;

                self.animate(1, 0, 1);
                let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };
                self.anim_rect = state.constants.npc.n309_bute[self.anim_num as usize + dir_offset];

            }
            _ => (),
        }

        if self.life <= 996 {
            self.npc_type = 316;
            self.action_num = 0;
        }

        Ok(())
    }

    pub(crate) fn tick_n310_bute_sword(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {

        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.damage = 0;
                    self.npc_flags.set_shootable(false);
                    self.npc_flags.set_invulnerable(true);
                }

                self.direction = if player.x < self.x {Direction::Left} else {Direction::Right};

                self.anim_counter = 0;

                if player.x > self.x - 0x10000 && player.x < self.x + 0x10000 && player.y > self.y - 0x10000 && player.y < self.y + 0x2000 {
                    self.action_num = 10;
                    return Ok(())
                }

            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.vel_x = 0;
                    self.action_counter = 0;
                    self.damage = 0;
                    self.anim_counter = 0;
                    self.npc_flags.set_shootable(false);
                    self.npc_flags.set_invulnerable(true);
                }

                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 20;
                    return Ok(())
                }
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.action_counter = 0;
                    self.damage = 0;
                    self.npc_flags.set_shootable(true);
                    self.npc_flags.set_invulnerable(false);

                    self.direction = if player.x < self.x {Direction::Left} else {Direction::Right};
                }

                self.vel_x = 0x400 * if self.direction == Direction::Left {-1} else {1};

                self.animate(3, 0, 1);

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 10;
                }

                if self.x < player.x + 0x5000 && self.x > player.x - 0x5000 {
                    self.vel_y = -0x300;
                    self.vel_x /= 2;
                    self.anim_num = 2;
                    self.action_num = 30;
                    state.sound_manager.play_sfx(30);
                }
            }
            30 => {
                if self.vel_y > -0x80 {
                    self.action_num = 31;
                    self.anim_counter = 0;
                    self.anim_num = 0;
                    self.damage = 9;
                }
            }
            31 => {
                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num = 4;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_num = 32;
                    self.action_counter = 0;
                    self.vel_x = 0;
                    self.damage = 3;
                }
            }
            32 => {
                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 10;
                    self.damage = 0;
                }
            }
            _ => (),
        }

        self.vel_y += 0x20;

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 5 };
        self.anim_rect = state.constants.npc.n310_bute_sword[self.anim_num as usize + dir_offset];

        if self.life <= 996 {
            self.npc_type = 316;
            self.action_num = 0;
        }

        Ok(())
    }

    pub(crate) fn tick_n311_bute_archer(&mut self, state: &mut SharedGameState, players: [&mut Player; 2], npc_list: &NPCList) -> GameResult {

        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                self.action_num = 1;

                if (player.y > self.y - 0x14000 && player.y < self.y + 0x14000) && 
                    ((self.direction == Direction::Left && player.x > self.x - 0x28000 && player.x < self.x) ||
                     (player.x > self.x && player.x < self.x + 0x28000)) {
                        self.action_num = 10;
                    }
            }
            10 | 11 => {
                self.action_num = 11;

                self.direction = if player.x < self.x {Direction::Left} else {Direction::Right};

                if player.x > self.x - 0x1C000 && player.x < self.x + 0x1C000 && player.y > self.y - 0x1000 {
                    self.anim_num = 1;
                    self.action_counter2 = 0;
                }
                else {
                    self.anim_num = 4;
                    self.action_counter2 = 1;
                }

                self.action_counter += 1;
                if self.action_counter > 10 { self.action_num = 20; }

            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.action_counter = 0;
                }

                if self.action_counter2 == 0  { self.animate(1, 1, 2); }
                else { self.animate(1, 4, 5); }

                self.action_counter += 1;
                if self.action_counter > 30 { self.action_num = 30; }
            }
            30 | 31 => {
                if self.action_num == 30 {
                    self.action_num = 31;
                    self.action_counter = 0;

                    let mut npc = NPC::create(312, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    npc.vel_x = if self.direction == Direction::Left {-0x600} else {0x600};
                    npc.vel_y = if self.action_counter2 == 0 {0} else {-0x600};
                    npc.direction = self.direction;

                    let _ = npc_list.spawn(0x100, npc);

                    self.anim_num = if self.action_counter2 == 0 {3} else {6};
                }

                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 40;
                    self.action_counter = self.rng.range(0..100) as u16;
                }
            }
            40 => {
                self.anim_num = 0;

                self.action_counter += 1;
                if self.action_counter > 150 {self.action_num = 10;}

                if player.x < self.x - 0x2C000 ||
                   player.x > self.x + 0x2C000 ||
                   player.y < self.y - 0x1E000 || 
                   player.y > self.y + 0x1E000 {
                       self.action_num = 40;
                       self.action_counter = 0;
                   }
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 7 };
        self.anim_rect = state.constants.npc.n311_bute_archer[self.anim_num as usize + dir_offset];

        if self.life <= 992 {
            self.npc_type = 316;
            self.action_num = 0;
        }

        Ok(())
    }

    pub(crate) fn tick_n312_bute_arrow_projectile(&mut self, state: &mut SharedGameState) -> GameResult { 

        if self.flags.0 != 0 && self.action_num > 0 && self.action_num < 20 {
            self.action_num = 20;
        }

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;

                    self.direction = if self.vel_x < 0 {Direction::Left} else {Direction::Right};
                    self.anim_num = if self.vel_y < 0 {0} else {2};
                }

                self.action_counter += 1;

                if self.action_counter == 3 {self.npc_flags.set_ignore_solidity(false)};
                if self.action_counter > 10 {self.action_num = 10}
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_counter = 0;
                    self.vel_x = 3 * self.vel_x / 4;
                    self.vel_y = 3 * self.vel_y / 4;
                }

                self.vel_y += 32;

                self.animate(10, 4, 4);
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.action_counter = 0;
                    self.vel_x = 0;
                    self.vel_y = 0;
                    self.damage = 0;
                }

                self.action_counter += 1;
                if self.action_counter > 30 {self.action_num = 30}
            }
            30 | 31 => {
                if self.action_num == 30 {
                    self.action_num = 31;
                    self.action_counter = 0;
                }

                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.cond.set_alive(false);
                    return Ok(());
                }
            }
            _ => (),
        }

        if self.vel_y > 0x5FF {self.vel_y = 0x5FF};

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 5 };
        self.anim_rect = state.constants.npc.n312_bute_arrow_projectile[self.anim_num as usize + dir_offset];

        if self.action_num == 31 && self.action_counter / 2 % 2 != 0 {
            self.anim_rect.left = 0;
            self.anim_rect.right = 0;
        }

        Ok(()) 
    }

    pub(crate) fn tick_n316_bute_dead(&mut self, state: &mut SharedGameState) -> GameResult {
        // (nearly) same as Gaudi death
        match self.action_num {
            0 => {
                self.npc_flags.set_shootable(false);
                self.npc_flags.set_ignore_solidity(false);
                self.damage = 0;
                self.action_num = 1;
                self.anim_num = 0;
                self.display_bounds.top = 0x1800;
                self.display_bounds.right = 0x1800;
                self.display_bounds.left = 0x1800;
                self.vel_y = -0x200;

                match self.direction {
                    Direction::Left => self.vel_x = 0x100,
                    Direction::Right => self.vel_x = -0x100,
                    _ => (),
                };
                state.sound_manager.play_sfx(50);
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
            _ => (),
        }

        self.vel_y += 0x20;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n316_bute_dead[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n323_bute_spinning(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {

        let player = self.get_closest_player_mut(players);
    
        self.animate(3, 0, 3);

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;

                    match self.direction {
                        Direction::Left => {
                            self.vel_x = -0x600;
                        }
                        Direction::Up => {
                            self.vel_y = -0x600;
                        }
                        Direction::Right => {
                            self.vel_x = 0x600;
                        }
                        Direction::Bottom => {
                            self.vel_y = 0x600;
                        }
                        Direction::FacingPlayer => unreachable!(),
                    }
                }

                self.action_counter += 1;
                if self.action_counter == 16 {self.npc_flags.set_ignore_solidity(false)};

                self.x += self.vel_x;
                self.y += self.vel_y;

                if self.flags.0 != 0 {self.action_num = 10};

                if self.action_counter > 20 {
                    if (self.direction == Direction::Left && self.x <= player.x + 0x4000) 
                    || (self.direction == Direction::Up && self.y <= player.y + 0x4000)
                    || (self.direction == Direction::Right && self.x <= player.x - 0x4000) 
                    || (self.direction == Direction::Bottom && self.y <= player.y - 0x4000) {
                        self.action_num = 10
                        };
                }
            }
            10 => {
                self.npc_type = 309;
                self.anim_num = 0;
                self.action_num = 11;
                self.npc_flags.set_shootable(true);
                self.npc_flags.set_ignore_solidity(false);
                self.damage = 5;
                self.display_bounds.top = 0x1000;
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n323_bute_spinning[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n324_bute_generator(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {

        if self.action_num == 10 {
            self.action_num = 11;
            self.action_counter = 0;
        }

        if self.action_num == 11 {
            self.action_counter += 1;
            if self.action_counter % 50 == 1 {
                let mut npc = NPC::create(323, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x;
                npc.y = self.y;
                npc.direction = self.direction;

                let _ = npc_list.spawn(0x100, npc);
            }

            if self.action_counter > 351 { self.action_num = 0 };
        }
    
        Ok(())
    }

}
