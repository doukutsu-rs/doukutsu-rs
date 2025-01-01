use num_traits::{abs, clamp};

use crate::common::Direction;
use crate::framework::error::GameResult;
use crate::game::npc::list::NPCList;
use crate::game::npc::NPC;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::util::rng::RNG;

impl NPC {
    pub(crate) fn tick_n029_cthulhu(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_num = 0;
            self.anim_counter = 0;
        }

        let player = self.get_closest_player_mut(players);

        if abs(self.x - player.x) < 0x6000 && self.y - 0x6000 < player.y && self.y + 0x2000 > player.y {
            self.anim_num = 1;
        } else {
            self.anim_num = 0;
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n029_cthulhu[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n052_sitting_blue_robot(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_rect = state.constants.npc.n052_sitting_blue_robot;
        }

        Ok(())
    }

    pub(crate) fn tick_n055_kazuma(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 0;
                self.anim_counter = 0;
            }
            3 | 4 => {
                if self.action_num == 3 {
                    self.action_num = 4;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                }

                self.animate(4, 1, 4);
                self.x += self.direction.vector_x() * 0x200;
            }
            5 => self.anim_num = 5,
            _ => (),
        }

        self.vel_y += 0x20;

        self.clamp_fall_speed();

        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };

        self.anim_rect = state.constants.npc.n055_kazuma[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n061_king(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.vel_x = 0;
                    if self.tsc_direction == 20 {
                        self.direction = Direction::Right;
                    }
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
            5 => {
                self.anim_num = 3;
                self.vel_x = 0;
            }
            6 | 7 => {
                if self.action_num == 6 {
                    self.action_num = 7;
                    self.action_counter = 0;
                    self.anim_counter = 0;
                    self.vel_y = -0x400;
                }

                self.anim_num = 2;
                self.vel_x = self.direction.vector_x() * 0x200;

                self.action_counter += 1;

                if self.action_counter > 1 && self.flags.hit_bottom_wall() {
                    self.action_num = 5;
                }
            }
            8 | 9 => {
                if self.action_num == 8 {
                    self.action_num = 9;
                    self.anim_num = 4;
                    self.anim_counter = 0;
                }

                self.animate(4, 4, 7);
                self.vel_x = self.direction.vector_x() * 0x200;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 4;
                    self.anim_counter = 0;
                }

                self.animate(2, 4, 7);
                self.vel_x = self.direction.vector_x() * 0x400;
            }
            20 => {
                let mut npc = NPC::create(145, &state.npc_table);
                npc.cond.set_alive(true);
                npc.direction = Direction::Right;
                npc.parent_id = self.id;

                let _ = npc_list.spawn(0x100, npc);

                self.anim_num = 0;
                self.action_num = 0;
            }
            30 | 31 => {
                if self.action_num == 30 {
                    self.action_num = 31;
                    self.action_counter = 0;
                    self.anim_counter = 0;
                    self.vel_y = 0;
                }

                self.anim_num = 2;
                self.vel_x = self.direction.vector_x() * 0x600;

                if self.flags.hit_left_wall() {
                    self.action_num = 7;
                    self.action_counter = 0;
                    self.anim_counter = 0;

                    self.direction = Direction::Right;
                    self.vel_y = -0x400;
                    self.vel_x = 0x200;

                    state.sound_manager.play_sfx(71);
                    npc_list.create_death_smoke(self.x, self.y, 0x800, 4, state, &mut self.rng);
                }
            }
            40 | 42 => {
                if self.action_num == 40 {
                    self.action_num = 42;
                    self.action_counter = 0;
                    self.anim_num = 8;

                    state.sound_manager.play_sfx(29);
                }

                self.anim_num += 1;
                if self.anim_num > 9 {
                    self.anim_num = 8;
                }

                self.action_counter += 1;
                if self.action_counter > 100 {
                    self.action_num = 50;
                    self.anim_num = 10;
                    self.spritesheet_id = 20;

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.direction = Direction::Left;

                    for _ in 0..4 {
                        npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                }
            }
            60 | 61 => {
                if self.action_num == 60 {
                    self.action_num = 61;
                    self.anim_num = 6;
                    self.action_counter2 = 1;
                    self.vel_y = -0x5ff;
                    self.vel_x = 0x400;
                }

                self.vel_y += 0x40;
                if self.flags.hit_bottom_wall() {
                    self.vel_x = 0;
                    self.action_num = 0;
                    self.action_counter2 = 0;
                }
            }
            _ => (),
        }

        if self.action_num < 30 || self.action_num >= 40 {
            self.vel_y += 0x40;
            self.vel_x = self.vel_x.clamp(-0x400, 0x400);

            self.clamp_fall_speed();
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 11 };

        self.anim_rect = state.constants.npc.n061_king[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n062_kazuma_computer(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.x -= 0x800;
                    self.y += 0x2000;
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.anim_rect = state.constants.npc.n062_kazuma_computer[self.anim_num as usize];
                }

                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    self.anim_rect = state.constants.npc.n062_kazuma_computer[self.anim_num as usize];
                }

                if self.anim_num > 1 {
                    self.anim_num = 0;
                    self.anim_rect = state.constants.npc.n062_kazuma_computer[self.anim_num as usize];
                }

                if self.rng.range(0..80) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                    self.anim_rect = state.constants.npc.n062_kazuma_computer[self.anim_num as usize];
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 3;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    self.anim_rect = state.constants.npc.n062_kazuma_computer[self.anim_num as usize];
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 40 {
                    self.action_num = 3;
                    self.anim_num = 2;
                    self.action_counter = 0;
                    self.anim_rect = state.constants.npc.n062_kazuma_computer[self.anim_num as usize];
                }
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 80 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_rect = state.constants.npc.n062_kazuma_computer[self.anim_num as usize];
                }
            }
            _ => (),
        }

        Ok(())
    }

    pub(crate) fn tick_n074_jack(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.vel_x = 0;
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
            8 | 9 => {
                if self.anim_num == 8 {
                    self.action_num = 9;
                    self.anim_num = 2;
                    self.anim_counter = 0;
                }

                self.animate(4, 2, 5);

                self.vel_x = self.direction.vector_x() * 0x200;
            }
            _ => (),
        }

        self.vel_y += 0x40;
        self.vel_x = clamp(self.vel_x, -0x400, 0x400);

        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };

        self.anim_rect = state.constants.npc.n074_jack[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n145_king_sword(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        if self.action_num == 0 {
            let parent = self.get_parent_ref(npc_list);
            if let Some(parent) = parent {
                let parent = parent.borrow();

                if parent.action_counter2 != 0 {
                    if parent.direction != Direction::Left {
                        self.direction = Direction::Left;
                    } else {
                        self.direction = Direction::Right;
                    }
                } else if parent.direction != Direction::Left {
                    self.direction = Direction::Right;
                } else {
                    self.direction = Direction::Left;
                }

                self.x = parent.x + self.direction.vector_x() * 0x1400;
                self.y = parent.y;
            }
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 1 };

        self.anim_rect = state.constants.npc.n145_king_sword[dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n151_blue_robot_standing(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }

                if self.rng.range(0..100) == 0 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 16 {
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n151_blue_robot_standing[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n167_booster_falling(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 1;
            }
            10 => {
                self.anim_num = 0;
                self.vel_y += 0x40;
                self.clamp_fall_speed();
                self.y += self.vel_y;
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    state.sound_manager.play_sfx(29);
                }

                self.anim_num += 1;
                if self.anim_num > 2 {
                    self.anim_num = 1;
                }

                self.action_counter += 1;
                if self.action_counter > 100 {
                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.direction = Direction::Left;

                    for _ in 0..4 {
                        npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }

                    self.cond.set_alive(false);
                }
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n167_booster_falling[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n217_itoh(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.vel_x = 0;
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
            10 => {
                self.anim_num = 2;
                self.vel_x = 0;
            }
            20 => {
                self.action_num = 21;
                self.anim_num = 2;
                self.vel_x += 0x200;
                self.vel_y -= 0x400;
            }
            21 => {
                if self.flags.hit_bottom_wall() {
                    self.anim_num = 3;
                    self.action_num = 30;
                    self.action_counter = 0;
                    self.vel_x = 0;
                    self.target_x = self.x;
                }
            }
            30 => {
                self.anim_num = 3;
                self.action_counter += 1;
                self.x = if ((self.action_counter / 2) & 1) != 0 { self.target_x + 512 } else { self.target_x }
            }

            40 | 41 => {
                if self.action_num == 40 {
                    self.action_num = 41;
                    self.vel_y = -512;
                    self.anim_num = 2;
                }
                if self.flags.hit_bottom_wall() {
                    self.action_num = 42;
                    self.anim_num = 4;
                }
            }
            42 => {
                self.vel_x = 0;
                self.anim_num = 4;
            }

            50 | 51 => {
                if self.action_num == 50 {
                    self.action_num = 51;
                    self.action_counter = 0;
                }
                self.action_counter += 1;
                if self.action_counter > 32 {
                    self.action_num = 42;
                }
                self.vel_x = 512;

                self.animate(3, 4, 7);
            }
            _ => (),
        }

        self.vel_y += 0x40;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n217_itoh[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n278_little_family(&mut self, state: &mut SharedGameState) -> GameResult {
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
                if self.action_counter == 10 {
                    self.action_num = 11;
                    self.action_counter = self.rng.range(0..16) as u16;
                    self.anim_num = 0;
                    self.direction = if self.rng.range(0..9) % 2 != 0 { Direction::Left } else { Direction::Right };
                }

                if self.direction == Direction::Left && self.flags.hit_left_wall() {
                    self.direction = Direction::Right;
                }

                if self.direction == Direction::Right && self.flags.hit_right_wall() {
                    self.direction = Direction::Left;
                }

                self.vel_x = self.direction.vector_x() * 0x100;

                self.animate(4, 0, 1);
                self.action_counter += 1;
                if self.action_counter > 32 {
                    self.action_num = 0;
                }
            }
            _ => (),
        }

        self.vel_y += 0x20;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let offset = match self.event_num {
            200 => 0,
            210 => 2,
            _ => 4,
        };

        self.anim_rect = state.constants.npc.n278_little_family[self.anim_num as usize + offset];

        Ok(())
    }

    pub(crate) fn tick_n305_small_puppy(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.y -= 0x2000;
            self.anim_counter = self.rng.range(0..6) as u16;
        }

        if self.action_num == 1 {
            self.animate(6, 0, 1);
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n305_small_puppy[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n326_sue_itoh_human_transition(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.x += 0x2000;
                    self.y -= 0x1000;
                }

                self.action_counter += 1;
                if self.action_counter > 80 {
                    self.action_num = 10;
                    self.action_counter = 0;
                } else if self.direction != Direction::Left {
                    if self.action_counter == 50 {
                        self.anim_num = 1;
                    }
                    if self.action_counter == 60 {
                        self.anim_num = 0;
                    }
                } else {
                    if self.action_counter == 30 {
                        self.anim_num = 1;
                    }
                    if self.action_counter == 40 {
                        self.anim_num = 0;
                    }
                }
            }
            10 => {
                self.action_counter += 1;
                if self.action_counter <= 50 {
                    self.anim_num = if self.action_counter & 2 != 0 { 2 } else { 3 };
                } else {
                    self.action_num = 15;
                    self.anim_num = 4;

                    let actr: &mut i16 = unsafe { std::mem::transmute(&mut self.action_counter) };
                    if self.direction == Direction::Left {
                        *actr = -20;
                    } else {
                        *actr = 0;
                    }
                }
            }
            15 => {
                let actr: &mut i16 = unsafe { std::mem::transmute(&mut self.action_counter) };
                *actr += 1;
                if *actr > 40 {
                    *actr = 0;
                    self.action_num = 20;
                }
            }
            20 => {
                self.vel_y += 0x40;
                self.clamp_fall_speed();

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 30;
                    self.action_counter = 0;
                    self.anim_num = 6;

                    let mut npc = NPC::create(327, &state.npc_table);
                    npc.cond.set_alive(true);

                    npc.x = self.x;
                    npc.y = if self.direction == Direction::Left { self.y - 0x1000 } else { self.y - 0x2000 };
                    npc.parent_id = self.id;

                    let _ = npc_list.spawn(0x100, npc);
                }
            }
            30 => {
                self.action_counter += 1;
                if self.action_counter == 30 {
                    self.anim_num = 7;
                }
                if self.action_counter == 40 {
                    self.action_num = 40;
                }
            }
            40 | 41 => {
                if self.action_num == 40 {
                    self.action_num = 41;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }

                self.action_counter += 1;
                if self.action_counter == 30 {
                    self.anim_num = 1;
                }
                if self.action_counter == 40 {
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 8 };

        self.anim_rect = state.constants.npc.n326_sue_itoh_human_transition[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n327_sneeze(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        self.action_counter += 1;

        if self.action_num == 0 {
            if self.action_counter < 4 {
                self.y -= 0x400;
            }

            if let Some(parent) = self.get_parent_ref(npc_list) {
                let parent = parent.borrow();
                
                if parent.anim_num == 7 {
                    self.action_num = 1;
                    self.anim_num = 1;
                    self.target_x = self.x;
                    self.target_y = self.y;
                }
            }
        } else if self.action_num == 1 {
            if self.action_counter >= 48 {
                self.x = self.target_x;
                self.y = self.target_y;
            } else {
                self.x += self.target_x + self.rng.range(-1..1) * 0x200;
                self.y += self.target_y + self.rng.range(-1..1) * 0x200;
            }
        }

        if self.action_counter > 70 {
            self.cond.set_alive(false);
        }

        self.anim_rect = state.constants.npc.n327_sneeze[self.anim_num as usize];

        Ok(())
    }
}
