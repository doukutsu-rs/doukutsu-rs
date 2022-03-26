use crate::common::{Direction, CDEG_RAD};
use crate::framework::error::GameResult;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n083_igor_cutscene(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.vel_x = 0;
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.action_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 5 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 1 {
                        self.anim_num = 0;
                    }
                }
            }
            2 | 3 => {
                if self.action_num == 2 {
                    self.action_num = 3;
                    self.anim_num = 2;
                    self.anim_counter = 0;
                }

                self.anim_counter += 1;
                if self.anim_counter > 3 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 5 {
                        self.anim_num = 2;
                    }
                }

                self.vel_x = self.direction.vector_x() * 0x200;
            }
            4 | 5 => {
                if self.action_num == 4 {
                    self.vel_x = 0;
                    self.action_num = 5;
                    self.action_counter = 0;
                    self.anim_num = 6;
                }

                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.action_counter = 0;
                    self.action_num = 6;
                    self.anim_num = 7;

                    state.sound_manager.play_sfx(70);
                }
            }
            6 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 0;
                    self.anim_num = 0;
                }
            }
            7 => {
                self.action_num = 1;
            }
            _ => (),
        }

        self.vel_y += 0x40;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 8 };
        self.anim_rect = state.constants.npc.n083_igor_cutscene[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n088_igor_boss(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_counter = 0;
                    self.vel_x = 0;
                }

                self.animate(5, 0, 1);

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 2;
                }
            }
            2 | 3 => {
                let player = self.get_closest_player_mut(players);
                if self.action_num == 2 {
                    self.action_num = 3;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    self.anim_counter = 0;

                    self.vel_x2 += 1;
                    if self.vel_x2 < 3 || self.life > 150 {
                        self.action_counter2 = 0;
                    } else {
                        self.action_counter2 = 1;
                    }

                    self.face_player(player);
                }

                self.action_counter += 1;
                self.animate(3, 2, 5);

                self.vel_x = self.direction.vector_x() * 0x200;

                if self.action_counter2 != 0 {
                    if self.action_counter > 16 {
                        self.vel_x = 0;
                        self.action_num = 9;
                        self.anim_num = 10;
                    }
                } else if self.action_counter > 50 {
                    self.action_num = 7;
                    self.action_counter = 0;
                    self.anim_num = 8;
                    self.damage = 2;
                    self.vel_x = (self.vel_x * 3) / 2;
                    self.vel_y = -0x400;
                } else if (self.direction == Direction::Left && self.x - 0x3000 < player.x)
                    || (self.direction == Direction::Right && self.x + 0x3000 > player.x)
                {
                    self.action_num = 4;
                }
            }
            4 | 5 => {
                if self.action_num == 4 {
                    self.action_num = 5;
                    self.action_counter = 0;
                    self.anim_num = 6;
                    self.vel_x = 0;
                }

                self.action_counter += 1;
                if self.action_counter > 12 {
                    self.action_num = 6;
                    self.action_counter = 0;
                    self.anim_num = 7;
                    self.damage = 5;
                    self.hit_bounds.left = 0x3000;
                    self.hit_bounds.top = 1;

                    state.sound_manager.play_sfx(70);
                }
            }
            6 => {
                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.action_num = 0;
                    self.anim_num = 0;
                    self.damage = 0;
                    self.hit_bounds.left = 0x1000;
                    self.hit_bounds.top = 0x2000;
                }
            }
            7 => {
                if self.flags.hit_bottom_wall() {
                    self.action_num = 8;
                    self.anim_num = 9;
                    self.damage = 0;

                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 30;

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);

                    for _ in 0..4 {
                        npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                }
            }
            8 => {
                self.vel_x = 0;
                self.action_counter += 1;

                if self.action_counter > 10 {
                    self.action_num = 0;
                    self.anim_num = 0;
                    self.damage = 0;
                }
            }
            9 | 10 => {
                if self.action_num == 9 {
                    self.action_num = 10;
                    self.action_counter = 0;

                    let player = self.get_closest_player_ref(&players);
                    self.face_player(player);
                }

                self.action_counter += 1;
                if self.action_counter > 100 && self.action_counter % 6 == 1 {
                    let deg = (if self.direction == Direction::Left { 0x88 } else { 0xf8 } + self.rng.range(-16..16))
                        as f64
                        * CDEG_RAD;
                    let vel_x = (deg.cos() * 1536.0) as i32;
                    let vel_y = (deg.sin() * 1536.0) as i32;

                    let mut npc = NPC::create(11, &state.npc_table);

                    npc.cond.set_alive(true);
                    npc.direction = Direction::Left;
                    npc.x = self.x;
                    npc.y = self.y + 0x800;
                    npc.vel_x = vel_x;
                    npc.vel_y = vel_y;

                    let _ = npc_list.spawn(0x100, npc);
                    state.sound_manager.play_sfx(12);
                }

                self.anim_num = if self.action_counter > 50 && (self.action_counter & 0x02) != 0 { 11 } else { 10 };

                if self.action_counter > 132 {
                    self.action_num = 0;
                    self.anim_num = 0;
                    self.vel_x2 = 0;
                }
            }
            _ => (),
        }

        self.vel_y += 0x40;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 12 };

        self.anim_rect = state.constants.npc.n088_igor_boss[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n089_igor_dead(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;

                    let player = self.get_closest_player_mut(players);
                    self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);

                    for _ in 0..8 {
                        npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                }

                self.action_counter += 1;
                if self.action_counter > 100 {
                    self.action_num = 2;
                    self.action_counter = 0;
                }

                if self.action_counter % 5 == 0 {
                    let mut npc = NPC::create(4, &state.npc_table);

                    npc.cond.set_alive(true);
                    npc.direction = Direction::Left;
                    npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                    npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                    npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                    npc.vel_y = self.rng.range(-0x600..0) as i32;

                    let _ = npc_list.spawn(0x100, npc);
                }

                let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };
                self.anim_rect = state.constants.npc.n089_igor_dead[dir_offset];

                if (self.action_counter & 0x02) != 0 {
                    self.anim_rect.left -= 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if (self.action_counter & 0x02) != 0 && self.action_counter < 100 {
                    self.anim_num = 0;
                    self.display_bounds.left = 0x2800;
                    self.display_bounds.right = 0x2800;
                    self.display_bounds.top = 0x2800;
                } else {
                    self.anim_num = 1;
                    self.display_bounds.left = 0x1800;
                    self.display_bounds.right = 0x1800;
                    self.display_bounds.top = 0x1000;
                }

                if self.action_counter > 150 {
                    self.action_num = 3;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                if self.action_counter % 9 == 0 {
                    let mut npc = NPC::create(4, &state.npc_table);

                    npc.cond.set_alive(true);
                    npc.direction = Direction::Left;
                    npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                    npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                    npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                    npc.vel_y = self.rng.range(-0x600..0) as i32;

                    let _ = npc_list.spawn(0x100, npc);
                }

                let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };
                self.anim_rect = state.constants.npc.n089_igor_dead[self.anim_num as usize + dir_offset];
            }
            3 => {
                self.anim_counter += 1;

                if self.anim_counter > 50 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num == 3 {
                        self.action_num = 4;
                    }

                    let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };
                    self.anim_rect = state.constants.npc.n089_igor_dead[self.anim_num as usize + dir_offset];
                }
            }
            _ => (),
        }

        Ok(())
    }

    pub(crate) fn tick_n268_igor_enemy(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        let player = self.get_closest_player_ref(&players);

        if self.x < player.x - 0x28000
            || self.x > player.x + 0x28000
            || self.y < player.y - 0x1E000
            || self.y > player.y + 0x1E000
        {
            self.action_num = 1;
        }

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.y += 0x1000;
                }

                self.animate(20, 0, 1);

                if self.x < player.x + 0xE000
                    && self.x > player.x - 0xE000
                    && self.x < player.x + 0x6000
                    && self.x > player.x - 0xE000
                {
                    self.action_num = 10;
                }

                if self.shock != 0 {
                    self.action_num = 10;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.action_counter = 0;
                    self.direction = if self.x < player.x { Direction::Right } else { Direction::Left };
                }

                self.vel_x = self.direction.vector_x() * 0x200;
                if self.x < player.x + 0x8000 && self.x > player.x - 0x8000 {
                    self.action_num = 20;
                    self.action_counter = 0;
                }

                if self.vel_x < 0 && self.flags.hit_left_wall() {
                    self.action_num = 20;
                    self.action_counter = 0;
                }

                if self.vel_x > 0 && self.flags.hit_right_wall() {
                    self.action_num = 20;
                    self.action_counter = 0;
                }

                self.animate(4, 2, 5);
            }
            20 => {
                self.vel_x = 0;
                self.anim_num = 6;

                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.action_num = 30;

                    self.vel_x = self.direction.vector_x() * 0x200;
                    self.vel_y = -0x5FF;

                    state.sound_manager.play_sfx(108);
                }
            }
            30 => {
                self.anim_num = 7;
                if self.flags.hit_bottom_wall() {
                    self.action_num = 40;
                    self.action_counter = 0;
                    state.quake_counter = 20;
                    state.sound_manager.play_sfx(26);
                }
            }
            40 => {
                self.vel_x = 0;
                self.anim_num = 6;
                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.action_num = 50;
                }
            }
            50 | 51 => {
                if self.action_num == 50 {
                    self.action_num = 51;
                    self.action_counter = 0;

                    let player = self.get_closest_player_ref(&players);
                    self.face_player(player);
                }

                self.action_counter += 1;
                if self.action_counter > 30 && self.action_counter % 4 == 1 {
                    let deg = (if self.direction == Direction::Left { 0x88 } else { 0xf8 } + self.rng.range(-16..16))
                        as f64
                        * CDEG_RAD;
                    let vel_x = (deg.cos() * 1536.0) as i32;
                    let vel_y = (deg.sin() * 1536.0) as i32;

                    let mut npc = NPC::create(11, &state.npc_table);

                    npc.cond.set_alive(true);
                    npc.direction = Direction::Left;
                    npc.x = self.x;
                    npc.y = self.y + 0x800;
                    npc.vel_x = vel_x;
                    npc.vel_y = vel_y;

                    let _ = npc_list.spawn(0x100, npc);
                    state.sound_manager.play_sfx(12);
                }

                if self.action_counter < 50 && self.action_counter & 2 != 0 {
                    self.anim_num = 9;
                } else {
                    self.anim_num = 8;
                }

                if self.action_counter > 82 {
                    self.action_num = 10;
                    self.face_player(player);
                }
            }
            _ => (),
        }

        self.vel_y += 0x33;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 10 };

        self.anim_rect = state.constants.npc.n268_igor_enemy[self.anim_num as usize + dir_offset];

        Ok(())
    }
}
