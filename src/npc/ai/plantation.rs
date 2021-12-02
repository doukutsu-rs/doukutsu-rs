use crate::caret::CaretType;
use crate::common::{Direction, CDEG_RAD};
use crate::framework::error::GameResult;
use crate::npc::list::NPCList;
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

    pub(crate) fn tick_n228_droll(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.y -= 0x1000;
                }
                self.vel_x = 0;
                self.action_num = 2;
                self.anim_num = 0;
            }
            2 => {
                let player = self.get_closest_player_ref(&players);
                if self.x <= player.x {
                    self.direction = Direction::Right;
                } else {
                    self.direction = Direction::Left;
                }
                self.anim_counter += 1;
                if self.anim_counter > 50 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }
                if self.anim_num > 1 {
                    self.anim_num = 0;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 2;
                    self.action_counter = 0;
                }

                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.action_num = 12;
                    self.anim_num = 3;
                    self.vel_y = -0x600;
                    if self.direction != Direction::Left {
                        self.vel_x = 0x200;
                    } else {
                        self.vel_x = -0x200;
                    }
                }
            }
            12 => {
                if self.flags.hit_bottom_wall() {
                    self.anim_num = 2;
                    self.action_num = 13;
                    self.action_counter = 0;
                }
            }
            13 => {
                self.vel_x /= 2;
                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.action_num = 1;
                }
            }
            _ => (),
        }

        self.vel_y += 0x40;
        if self.vel_y > 0x5FF {
            self.vel_y = 0x5FF;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

        self.anim_rect = state.constants.npc.n228_droll[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n231_rocket(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                }
                self.anim_num = 0;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = 0;
                }

                self.action_counter += 1;
                self.vel_y += 8;
                if self.flags.hit_bottom_wall() {
                    if self.action_counter > 9 {
                        self.action_num = 1;
                    } else {
                        self.action_num = 12;
                    }
                }
            }
            12 | 13 => {
                if self.action_num == 12 {
                    self.npc_flags.set_interactable(false);
                    self.action_num = 13;
                    self.action_counter = 0;
                    self.anim_num = 1;

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);

                    for _ in 0..10 {
                        npc.x = self.x + self.rng.range(-16..16) * 0x200;
                        npc.y = self.y + self.rng.range(-8..8) * 0x200;
                        let _ = npc_list.spawn(0x100, npc.clone());

                        state.sound_manager.play_sfx(12);
                    }
                }

                self.vel_y -= 8;
                self.action_counter += 1;

                if (self.action_counter & 1) == 0 {
                    state.create_caret(self.x - 0x1400, self.y + 0x1000, CaretType::Exhaust, Direction::Bottom);
                }
                if self.action_counter % 2 == 1 {
                    state.create_caret(self.x + 0x1400, self.y + 0x1000, CaretType::Exhaust, Direction::Bottom);
                }
                if self.action_counter % 4 == 1 {
                    state.sound_manager.play_sfx(34);
                }

                let player = self.get_closest_player_ref(&players);

                if self.flags.hit_top_wall() || player.flags.hit_top_wall() || self.action_counter > 450 {
                    if self.flags.hit_top_wall() || player.flags.hit_top_wall() {
                        self.vel_y = 0;
                    }
                    self.action_num = 15;

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);

                    for _ in 0..6 {
                        npc.x = self.x + self.rng.range(-16..16) * 0x200;
                        npc.y = self.y + self.rng.range(-8..8) * 0x200;
                        let _ = npc_list.spawn(0x100, npc.clone());

                        state.sound_manager.play_sfx(12);
                    }
                }
            }
            15 => {
                self.vel_y += 8;
                self.action_counter += 1;

                if self.vel_y < 0 {
                    if (self.action_counter & 7) == 0 {
                        state.create_caret(self.x - 0x1400, self.y + 0x1000, CaretType::Exhaust, Direction::Bottom);
                    }
                    if self.action_counter % 8 == 4 {
                        state.create_caret(self.x + 0x1400, self.y + 0x1000, CaretType::Exhaust, Direction::Bottom);
                    }

                    if self.action_counter % 16 == 1 {
                        state.sound_manager.play_sfx(34);
                    }
                }

                if self.flags.hit_bottom_wall() {
                    self.npc_flags.set_interactable(true);
                    self.action_num = 1;
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        self.vel_y = self.vel_y.clamp(-0x5ff, 0x5ff);
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n231_rocket[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n232_orangebell(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.target_x = self.x;
                    self.target_y = self.y;
                    self.vel_y = 0x200;

                    for _ in 0..8 {
                        let mut npc = NPC::create(233, &state.npc_table);
                        npc.cond.set_alive(true);

                        npc.x = self.x;
                        npc.y = self.y;
                        npc.direction = self.direction;
                        npc.parent_id = self.id;

                        let _ = npc_list.spawn(0x100, npc);
                    }
                }

                if self.vel_x < 0 && self.flags.hit_left_wall() {
                    self.direction = Direction::Right;
                }

                if self.vel_x > 0 && self.flags.hit_right_wall() {
                    self.direction = Direction::Left;
                }

                self.vel_x = self.direction.vector_x() * 0x100;

                self.vel_y += if self.y < self.target_y { 8 } else { -8 };

                self.vel_y = self.vel_y.clamp(-0x200, 0x200);

                self.animate(5, 0, 2);
            }
            _ => (),
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n232_orangebell[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n233_orangebell_bat(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.vel_x = ((self.rng.range(0..255) as f64 * CDEG_RAD).cos() * -512.0) as i32;
                    self.vel_y = ((self.rng.range(0..255) as f64 * CDEG_RAD).sin() * -512.0) as i32;

                    self.action_counter2 = 120;
                    self.vel_y2 = self.rng.range(-32..32) * 0x200;
                }

                if let Some(parent) = self.get_parent_ref_mut(npc_list) {
                    if parent.npc_type == 232 {
                        self.target_x = parent.x;
                        self.target_y = parent.y;
                        self.direction = parent.direction;
                    }
                }

                self.vel_x += (self.target_x - self.x).signum() * 8;
                self.vel_y += ((self.target_y + self.vel_y2) - self.y).signum() * 0x20;

                self.vel_x = self.vel_x.clamp(-0x400, 0x400);
                self.vel_y = self.vel_y.clamp(-0x400, 0x400);

                if self.action_counter2 >= 120 {
                    let player = self.get_closest_player_ref(&players);

                    if self.x - 0x1000 < player.x
                        && self.x + 0x1000 > player.x
                        && self.y < player.y
                        && self.y + 0x16000 > player.y
                    {
                        self.vel_x /= 4;
                        self.vel_y = 0;
                        self.action_num = 3;
                        self.npc_flags.set_ignore_solidity(false);
                    }
                } else {
                    self.action_counter2 += 1;
                }
            }
            3 => {
                self.vel_y += 0x40;
                if self.vel_y > 0x5ff {
                    self.vel_y = 0x5ff;
                }

                if self.flags.hit_bottom_wall() {
                    self.vel_y = 0;
                    self.vel_x *= 2;
                    self.action_counter2 = 0;
                    self.action_num = 1;
                    self.npc_flags.set_ignore_solidity(true);
                }
            }
            _ => (),
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        if self.action_num == 3 {
            self.anim_num = 3;
        } else {
            self.animate(1, 0, 2);
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

        self.anim_rect = state.constants.npc.n233_orangebell_bat[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n235_midorin(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.vel_x = 0;
                }

                if self.rng.range(0..30) == 1 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                if self.rng.range(0..30) == 1 {
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

                    self.direction = if self.rng.range(0..9) & 1 != 0 { Direction::Left } else { Direction::Right };
                }

                if self.direction == Direction::Left && self.flags.hit_left_wall() {
                    self.direction = Direction::Right;
                }

                if self.direction == Direction::Right && self.flags.hit_right_wall() {
                    self.direction = Direction::Left;
                }

                self.vel_x = self.direction.vector_x() * 0x400;

                self.animate(1, 2, 3);

                self.action_counter += 1;
                if self.action_counter > 64 {
                    self.action_num = 0;
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

        if self.anim_num == 2 {
            self.hit_bounds.top = 0xa00;
        } else {
            self.hit_bounds.top = 0x800;
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

        self.anim_rect = state.constants.npc.n235_midorin[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n236_gunfish(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = self.rng.range(0..50) as u16;
                    self.target_x = self.x;
                    self.target_y = self.y;
                    self.vel_y = 0;
                }

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 2;
                    self.vel_y = 0x200;
                }
            }
            2 => {
                let player = self.get_closest_player_ref(&players);
                if self.x >= player.x {
                    self.direction = Direction::Left;
                } else {
                    self.direction = Direction::Right;
                }

                if player.x - 0x10000 < self.x
                    && player.x + 0x10000 > self.x
                    && player.y - 0x4000 < self.y
                    && player.y + 0x14000 > self.y
                {
                    self.action_counter += 1;
                }

                if self.action_counter > 80 {
                    self.action_num = 10;
                    self.action_counter = 0;
                }

                self.animate(1, 0, 1);
            }
            10 => {
                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 20;
                    self.action_counter = 0;
                }

                self.animate(1, 2, 3);
            }
            20 => {
                self.action_counter += 1;
                if self.action_counter > 60 {
                    self.action_num = 2;
                    self.action_counter = 0;
                }

                if self.action_counter % 10 == 3 {
                    state.sound_manager.play_sfx(39);

                    let mut npc = NPC::create(237, &state.npc_table);
                    npc.cond.set_alive(true);

                    if self.direction == Direction::Left {
                        npc.x = self.x - 0x1000;
                        npc.y = self.y - 0x1000;
                        npc.vel_x = -0x400;
                        npc.vel_y = -0x400;
                    } else {
                        npc.x = self.x + 0x1000;
                        npc.y = self.y - 0x1000;
                        npc.vel_x = 0x400;
                        npc.vel_y = -0x400;
                    }

                    let _ = npc_list.spawn(0x100, npc);
                }

                self.animate(1, 4, 5);
            }
            _ => (),
        }

        if self.y < self.target_y {
            self.vel_y += 0x10;
        } else {
            self.vel_y -= 0x10;
        }

        self.vel_y = self.vel_y.clamp(-0x100, 0x100);

        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };

        self.anim_rect = state.constants.npc.n236_gunfish[dir_offset + self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n237_gunfish_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
        }

        if self.action_num == 1 {
            self.action_counter += 1;

            let hit = self.flags.hit_anything() || (self.action_counter > 10 && self.flags.in_water());

            if hit {
                for _ in 0..5 {
                    state.create_caret(self.x, self.y, CaretType::Bubble, Direction::Left);
                }

                state.sound_manager.play_sfx(21);
                self.cond.set_alive(false);
                return Ok(());
            }
        }

        self.vel_y += 0x20;
        if self.vel_y > 0x5FF {
            self.vel_y = 0x5FF;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n237_gunfish_projectile;

        Ok(())
    }

    pub(crate) fn tick_n240_mimiga_jailed(&mut self, state: &mut SharedGameState) -> GameResult {
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

                    self.direction = if self.rng.range(0..9) & 1 != 0 { Direction::Left } else { Direction::Right };
                }

                if self.direction == Direction::Left && self.flags.hit_left_wall() {
                    self.direction = Direction::Right;
                }

                if self.direction == Direction::Right && self.flags.hit_right_wall() {
                    self.direction = Direction::Left;
                }

                self.vel_x = self.direction.vector_x() * 0x200;

                self.animate(4, 2, 5);

                self.action_counter += 1;
                if self.action_counter > 32 {
                    self.action_num = 0;
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

        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };

        self.anim_rect = state.constants.npc.n240_mimiga_jailed[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n273_droll_projectile(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
        }

        if self.action_num == 1 {
            self.x += self.vel_x;
            self.y += self.vel_y;

            if self.flags.any_flag() {
                let mut npc = NPC::create(4, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x;
                npc.y = self.y;

                for _ in 0..3 {
                    let _ = npc_list.spawn(0x100, npc.clone());
                }

                self.vanish(state);
                return Ok(());
            }
        }

        self.action_counter += 1;
        if self.action_counter % 5 != 0 {
            state.sound_manager.play_sfx(110);
        }

        self.anim_num += 1;
        if self.anim_num > 2 {
            self.anim_num = 0;
        }

        self.anim_rect = state.constants.npc.n273_droll_projectile[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n274_droll(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 | 2 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.y -= 0x1000;
                    self.target_x = self.x;
                }

                if self.action_num == 1 {
                    self.vel_x = 0;
                    self.action_num = 2;
                    self.anim_num = 0;
                }

                let player = self.get_closest_player_ref(&players);
                self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };

                self.animate(40, 0, 1);

                if self.shock > 0 {
                    self.action_num = 10;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 2;
                    self.action_counter = 0;
                }

                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.action_num = 12;
                    self.anim_num = 3;
                    self.action_counter2 = 0;

                    self.vel_y = -0x600;
                    self.vel_x = self.direction.vector_x() * 0x200;
                }
            }
            12 => {
                if self.vel_y > 0 {
                    self.anim_num = 4;
                    if self.action_counter2 == 0 {
                        self.action_counter2 += 1;

                        let player = self.get_closest_player_ref(&players);

                        let mut npc = NPC::create(273, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.x;
                        npc.y = self.y;

                        let angle = f64::atan2((player.y - 0x1400 - self.y) as f64, (player.x - self.x) as f64);
                        npc.vel_x = (2048.0 * angle.cos()) as i32;
                        npc.vel_y = (2048.0 * angle.sin()) as i32;

                        let _ = npc_list.spawn(0x100, npc);

                        state.sound_manager.play_sfx(39);
                    }
                }

                if self.vel_y > 0x200 {
                    self.anim_num = 5;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_num = 13;
                    self.anim_num = 2;
                    self.action_counter = 0;
                    self.vel_x = 0;
                }
            }
            13 => {
                self.vel_x /= 2;

                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.action_num = 1;
                }
            }
            _ => (),
        }

        self.vel_y += 0x55;
        if self.vel_y > 0x5FF {
            self.vel_y = 0x5FF;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };

        self.anim_rect = state.constants.npc.n274_droll[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n275_puppy_plantation(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.action_counter = 0;
                }

                if self.rng.range(0..120) == 10 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 1;
                }

                let player = self.get_closest_player_ref(&players);

                if self.x - 0x8000 < player.x
                    && self.x + 0x8000 > player.x
                    && self.y - 0x4000 < player.y
                    && self.y + 0x2000 > player.y
                {
                    self.animate(3, 2, 3);
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

        self.vel_y += 0x40;
        if self.vel_y > 0x5FF {
            self.vel_y = 0x5FF;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n275_puppy_plantation[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n308_stumpy(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 | 1 => {
                self.action_num = 1;

                let player = self.get_closest_player_ref(&players);

                if player.x < self.x + 0x1e000
                    && player.x > self.x - 0x1e000
                    && player.y < self.y + 0x18000
                    && player.y > self.y - 0x18000
                {
                    self.action_num = 10;
                }
            }
            10 | 11 => {
                let player = self.get_closest_player_ref(&players);

                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = 0;
                    self.vel_x2 = 0;
                    self.vel_y2 = 0;

                    self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };
                }

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 20;
                }

                self.anim_counter += 1;
                // this is a typo in original exe
                if self.action_counter > 1 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num > 1 {
                        self.anim_num = 0;
                    }
                }

                if player.x > self.x + 0x28000
                    && player.x < self.x - 0x28000
                    && player.y > self.y + 0x1e000
                    && player.y < self.y - 0x1e000
                {
                    self.action_num = 0;
                }
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.action_counter = 0;

                    let player = self.get_closest_player_ref(&players);
                    let deg = f64::atan2((self.y - player.y) as f64, (self.x - player.x) as f64);
                    let deg = deg + self.rng.range(-3..3) as f64 * CDEG_RAD;

                    self.vel_x2 = (deg.cos() * -1024.0) as i32;
                    self.vel_y2 = (deg.sin() * -1024.0) as i32;

                    self.direction = if self.vel_x2 < 0 { Direction::Left } else { Direction::Right };
                }

                if self.vel_x2 < 0 && self.flags.hit_left_wall() {
                    self.direction = Direction::Right;
                    self.vel_x2 = -self.vel_x2;
                }

                if self.vel_x2 > 0 && self.flags.hit_right_wall() {
                    self.direction = Direction::Left;
                    self.vel_x2 = -self.vel_x2;
                }

                if self.vel_y2 < 0 && self.flags.hit_top_wall() {
                    self.vel_y2 = -self.vel_y2;
                }

                if self.vel_y2 > 0 && self.flags.hit_bottom_wall() {
                    self.vel_y2 = -self.vel_y2;
                }

                if self.flags.in_water() {
                    self.vel_y2 = -0x200;
                }

                self.x += self.vel_x2;
                self.y += self.vel_y2;

                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_num = 10;
                }

                self.anim_num += 1;
                if self.anim_num > 1 {
                    self.anim_num = 0;
                }
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n308_stumpy[self.anim_num as usize + dir_offset];

        Ok(())
    }
}
