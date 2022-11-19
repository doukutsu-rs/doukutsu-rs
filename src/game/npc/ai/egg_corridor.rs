use num_traits::{abs, clamp};

use crate::common::{CDEG_RAD, Direction, Rect};
use crate::framework::error::GameResult;
use crate::game::caret::CaretType;
use crate::game::npc::list::NPCList;
use crate::game::npc::NPC;
use crate::game::player::{Player, TargetPlayer};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::bullet::BulletManager;
use crate::util::rng::RNG;

impl NPC {
    pub(crate) fn tick_n002_behemoth(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        if self.flags.hit_left_wall() {
            self.direction = Direction::Right;
        } else if self.flags.hit_right_wall() {
            self.direction = Direction::Left;
        }

        match self.action_num {
            0 => {
                self.vel_x = self.direction.vector_x() * 0x100;

                self.animate(8, 0, 3);

                if self.shock > 0 {
                    self.action_counter = 0;
                    self.action_num = 1;
                    self.anim_num = 4;
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
                    } else {
                        self.action_num = 0;
                        self.anim_counter = 0;
                    }
                }
            }
            2 => {
                self.vel_x = self.direction.vector_x() * 0x400;

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
                        state.quake_counter = 8;
                        state.quake_rumble_counter = 8;

                        state.sound_manager.play_sfx(26);

                        let mut npc = NPC::create(4, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.x;
                        npc.y = self.y + 0x600;

                        let _ = npc_list.spawn(0x100, npc);
                    }
                }
            }
            _ => (),
        }

        self.vel_y += 0x40;

        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 7 };

        self.anim_rect = state.constants.npc.n002_behemoth[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n005_green_critter(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.y += 0x600;
                    self.action_num = 1;
                    self.anim_num = 0;
                }

                let player = self.get_closest_player_ref(&players);
                self.face_player(player);

                if self.target_x < 100 {
                    self.target_x += 1;
                }

                if self.action_counter >= 8
                    && self.x - 0xe000 < player.x
                    && self.x + 0xe000 > player.x
                    && self.y - 0xa000 < player.y
                    && self.y + 0xa000 > player.y
                {
                    self.anim_num = 1;
                } else {
                    if self.action_counter < 8 {
                        self.action_counter += 1;
                    }

                    self.anim_num = 0;
                }

                if self.shock > 0 {
                    self.action_num = 2;
                    self.action_counter = 0;

                    self.anim_num = 0;
                }

                if self.action_counter >= 8
                    && self.target_x >= 100
                    && self.x - 0x6000 < player.x
                    && self.x + 0x6000 > player.x
                    && self.y - 0xa000 < player.y
                    && self.y + 0xa000 > player.y
                {
                    self.action_num = 2;
                    self.action_counter = 0;

                    self.anim_num = 0;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 3;
                    self.anim_num = 2;

                    self.vel_x = self.direction.vector_x() * 0x100;
                    self.vel_y = -0x5ff;
                    state.sound_manager.play_sfx(30);
                }
            }
            3 => {
                if self.flags.hit_bottom_wall() {
                    self.vel_x = 0;
                    self.action_counter = 0;
                    self.action_num = 1;
                    self.anim_num = 0;

                    state.sound_manager.play_sfx(23);
                }
            }
            _ => (),
        }

        self.vel_y += 0x40;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n005_green_critter[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n006_green_beetle(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;

                match self.direction {
                    Direction::Left => {
                        self.action_num = 1;
                    }
                    Direction::Right => {
                        self.action_num = 3;
                    }
                    _ => (),
                }
            }
            1 => {
                self.vel_x -= 0x10;

                if self.vel_x < -0x400 {
                    self.vel_x = -0x400;
                }

                if self.shock > 0 {
                    self.x += self.vel_x / 2;
                } else {
                    self.x += self.vel_x;
                }

                self.animate(1, 1, 2);

                if self.flags.hit_left_wall() {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.vel_x = 0;
                    self.direction = Direction::Right;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 60 {
                    self.action_num = 3;
                    self.anim_counter = 0;
                    self.anim_num = 1;
                }
            }
            3 => {
                self.vel_x += 0x10;

                if self.vel_x > 0x400 {
                    self.vel_x = 0x400;
                }

                if self.shock > 0 {
                    self.x += self.vel_x / 2;
                } else {
                    self.x += self.vel_x;
                }

                self.animate(1, 1, 2);

                if self.flags.hit_right_wall() {
                    self.action_num = 4;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.vel_x = 0;
                    self.direction = Direction::Left;
                }
            }
            4 => {
                self.action_counter += 1;
                if self.action_counter > 60 {
                    self.action_num = 1;
                    self.anim_counter = 0;
                    self.anim_num = 1;
                }
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 5 };

        self.anim_rect = state.constants.npc.n006_green_beetle[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n007_basil(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 => {
                let player = self.get_closest_player_mut(players);
                self.x = player.x;

                if self.direction == Direction::Left {
                    self.action_num = 1;
                } else {
                    self.action_num = 2;
                }
            }
            1 => {
                self.vel_x -= 0x40;

                let player = self.get_closest_player_mut(players);
                if self.x < (player.x - 0x18000) {
                    self.action_num = 2;
                }

                if self.flags.hit_left_wall() {
                    self.vel_x = 0;
                    self.action_num = 2;
                }
            }
            2 => {
                self.vel_x += 0x40;

                let player = self.get_closest_player_mut(players);
                if self.x > (player.x + 0x18000) {
                    self.action_num = 1;
                }

                if self.flags.hit_right_wall() {
                    self.vel_x = 0;
                    self.action_num = 1;
                }
            }
            _ => (),
        }

        if self.vel_x < 0 {
            self.direction = Direction::Left;
        } else {
            self.direction = Direction::Right;
        }

        self.vel_x = clamp(self.vel_x, -0x5ff, 0x5ff);
        self.x += self.vel_x;

        self.animate(1, 0, 1);

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n007_basil[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n008_blue_beetle(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        match self.action_num {
            0 => {
                let player = self.get_closest_player_mut(players);

                if player.x < self.x + 0x2000 && player.x > self.x - 0x2000 {
                    self.npc_flags.set_shootable(true);
                    self.vel_y = -0x100;
                    self.target_y = self.y;
                    self.action_num = 1;
                    self.damage = 2;

                    self.x = player.x + self.direction.opposite().vector_x() * 0x20000;
                    self.vel_x = self.direction.vector_x() * 0x2ff;
                } else {
                    self.npc_flags.set_shootable(false);
                    self.anim_rect.left = 0;
                    self.anim_rect.right = 0;
                    self.damage = 0;
                    self.vel_x = 0;
                    self.vel_y = 0;

                    return Ok(());
                }
            }
            1 => {
                let player = self.get_closest_player_mut(players);

                self.face_player(player);

                self.vel_x += self.direction.vector_x() * 0x10;
                self.vel_y += if self.y < self.target_y { 8 } else { -8 };

                self.vel_x = clamp(self.vel_x, -0x2ff, 0x2ff);
                self.vel_y = clamp(self.vel_y, -0x100, 0x100);

                if self.shock > 0 {
                    self.x += self.vel_x / 2;
                    self.y += self.vel_y / 2;
                } else {
                    self.x += self.vel_x;
                    self.y += self.vel_y;
                }
            }
            _ => (),
        }

        self.animate(1, 0, 1);

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n008_blue_beetle[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n025_lift(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.x += 0x1000;
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 1;
                }

                self.action_counter += 1;
                if self.action_counter > 150 {
                    self.action_num = 2;
                    self.action_counter = 0;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 64 {
                    self.action_num = 3;
                    self.action_counter = 0;
                } else {
                    self.y -= 0x200;
                }
                self.animate(1, 0, 1);
            }
            3 => {
                self.action_counter += 1;
                if self.action_counter > 150 {
                    self.action_num = 4;
                    self.action_counter = 0;
                }
            }
            4 => {
                self.action_counter += 1;
                if self.action_counter > 64 {
                    self.action_num = 5;
                    self.action_counter = 0;
                } else {
                    self.y -= 0x200;
                }
                self.animate(1, 0, 1);
            }
            5 => {
                self.action_counter += 1;
                if self.action_counter > 150 {
                    self.action_num = 6;
                    self.action_counter = 0;
                }
            }
            6 => {
                self.action_counter += 1;
                if self.action_counter > 64 {
                    self.action_num = 7;
                    self.action_counter = 0;
                } else {
                    self.y += 0x200;
                }
                self.animate(1, 0, 1);
            }
            7 => {
                self.action_counter += 1;
                if self.action_counter > 150 {
                    self.action_num = 8;
                    self.action_counter = 0;
                }
            }
            8 => {
                self.action_counter += 1;
                if self.action_counter > 64 {
                    self.action_num = 1;
                    self.action_counter = 0;
                } else {
                    self.y += 0x200;
                }
                self.animate(1, 0, 1);
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n025_lift[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n058_basu(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 => {
                if player.x < self.x + 0x2000 && player.x > self.x - 0x2000 {
                    self.target_x = self.x;
                    self.target_y = self.y;
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.action_counter2 = 0;
                    self.damage = 6;
                    self.vel_y = -0x100;
                    self.tsc_direction = self.direction as u16;
                    self.npc_flags.set_shootable(true);

                    self.x = player.x + self.direction.opposite().vector_x() * 0x20000;
                    self.vel_x = self.direction.vector_x() * 0x2ff;
                } else {
                    self.anim_rect = Rect::new(0, 0, 0, 0);
                    self.damage = 0;
                    self.vel_x = 0;
                    self.vel_y = 0;
                    self.npc_flags.set_shootable(false);
                }

                return Ok(());
            }
            1 => {
                if self.x > player.x {
                    self.direction = Direction::Left;
                    self.vel_x -= 0x10;
                } else {
                    self.direction = Direction::Right;
                    self.vel_x += 0x10;
                }

                if self.flags.hit_left_wall() {
                    self.vel_x = 0x200;
                }

                if self.flags.hit_right_wall() {
                    self.vel_x = -0x200;
                }

                self.vel_y += ((self.target_y - self.y).signum() | 1) * 0x08;

                self.vel_x = clamp(self.vel_x, -0x2ff, 0x2ff);
                self.vel_y = clamp(self.vel_y, -0x100, 0x100);

                if self.shock > 0 {
                    self.x += self.vel_x / 2;
                    self.y += self.vel_y / 2;
                } else {
                    self.x += self.vel_x;
                    self.y += self.vel_y;
                }

                if player.x > self.x + 0x32000 || player.x < self.x - 0x32000 {
                    self.action_num = 0;
                    self.vel_x = 0;
                    self.x = self.target_x;
                    self.damage = 0;
                    self.direction = Direction::from_int_facing(self.tsc_direction as usize).unwrap_or(Direction::Left);
                    self.anim_rect = Rect::new(0, 0, 0, 0);
                    return Ok(());
                }
            }
            _ => (),
        }

        if self.action_counter < 150 {
            self.action_counter += 1;
        } else {
            self.action_counter2 += 1;
            if (self.action_counter2 % 8) == 0 && abs(self.x - player.x) < 0x14000 {
                let angle = f64::atan2((self.y - player.y) as f64, (self.x - player.x) as f64)
                    + self.rng.range(-6..6) as f64 * CDEG_RAD;

                let mut npc = NPC::create(84, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x;
                npc.y = self.y;
                npc.vel_x = (angle.cos() * -1024.0) as i32;
                npc.vel_y = (angle.sin() * -1024.0) as i32;

                let _ = npc_list.spawn(0x100, npc);
                state.sound_manager.play_sfx(39);
            }

            if self.action_counter2 > 8 {
                self.action_counter = 0;
                self.action_counter2 = 0;
            }
        }

        self.anim_counter += 1;
        if self.anim_counter > 1 {
            self.anim_counter = 0;
            self.anim_num += 1;
            if self.anim_num > 1 {
                self.anim_num = 0;
            }
        }

        if self.action_counter > 120 && self.action_counter & 0x02 == 1 && self.anim_num == 1 {
            self.anim_num = 2;
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n058_basu[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n084_basu_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_counter += 1;
        if self.anim_counter > 2 {
            self.anim_counter = 0;
            self.anim_num += 1;

            if self.anim_num > 3 {
                self.anim_num = 0;
            }
        }

        self.anim_rect = state.constants.npc.n084_basu_projectile[self.anim_num as usize];

        self.action_counter2 += 1;
        if self.flags.hit_anything() || self.action_counter2 > 300 {
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
            self.cond.set_alive(false);
        }

        Ok(())
    }

    pub(crate) fn tick_n200_zombie_dragon(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        if self.action_num < 100 && self.life < 950 {
            self.action_num = 100;
            self.npc_flags.set_shootable(false);
            self.damage = 0;

            state.sound_manager.play_sfx(72);
            npc_list.create_death_smoke(self.x, self.y, self.display_bounds.right as usize, 8, state, &self.rng);
            self.create_xp_drop(state, npc_list);
        }

        match self.action_num {
            0 | 10 => {
                if self.action_num == 0 {
                    self.action_num = 10;
                    self.action_counter3 = 0;
                }

                let player = self.get_closest_player_mut(players);

                self.animate(30, 0, 1);

                if self.action_counter3 != 0 {
                    self.action_counter3 -= 1;
                }

                if self.action_counter3 == 0 && player.x > self.x - 0xE000 && player.x < self.x + 0xE000 {
                    self.action_num = 20;
                }
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.action_counter = 0;
                }

                self.action_counter += 1;
                if (self.action_counter & 1) != 0 {
                    self.anim_num = 2;
                } else {
                    self.anim_num = 3;
                }
                if self.action_counter > 30 {
                    self.action_num = 30;
                }

                let player = self.get_closest_player_mut(players);
                self.direction = if player.x >= self.x { Direction::Right } else { Direction::Left };
            }
            30 | 31 => {
                let player = self.get_closest_player_mut(players);
                if self.action_num == 30 {
                    self.action_num = 31;
                    self.action_counter = 0;
                    self.anim_num = 4;
                    self.target_x = player.x;
                    self.target_y = player.y;
                }

                self.action_counter += 1;
                if self.action_counter <= 39 && self.action_counter % 8 == 1 {
                    let px = self.x + (self.direction.vector_x() * 0x1c00) - self.target_x;
                    let py = self.y - self.target_y;

                    let deg = f64::atan2(py as f64, px as f64) + self.rng.range(-6..6) as f64 * CDEG_RAD;

                    let mut npc = NPC::create(202, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x + self.direction.vector_x() * 0x1c00;
                    npc.y = self.y;
                    npc.vel_x = (deg.cos() * -1536.0) as i32;
                    npc.vel_y = (deg.sin() * -1536.0) as i32;

                    let _ = npc_list.spawn(0x100, npc);

                    if !player.cond.hidden() {
                        state.sound_manager.play_sfx(33);
                    }
                }
                if self.action_counter > 60 {
                    self.action_num = 10;
                    self.action_counter3 = self.rng.range(100..200) as u16;
                    self.anim_counter = 0;
                }
            }
            100 => {
                self.anim_num = 5;
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 6 };

        self.anim_rect = state.constants.npc.n200_zombie_dragon[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n201_zombie_dragon_dead(&mut self, state: &mut SharedGameState) -> GameResult {
        let dir_offset = if self.direction == Direction::Left { 0 } else { 1 };

        self.anim_rect = state.constants.npc.n201_zombie_dragon_dead[dir_offset];
        Ok(())
    }

    pub(crate) fn tick_n202_zombie_dragon_projectile(&mut self, state: &mut SharedGameState) -> GameResult {
        self.y += self.vel_y;
        self.x += self.vel_x;

        self.animate(1, 0, 2);

        self.anim_rect = state.constants.npc.n202_zombie_dragon_projectile[self.anim_num as usize];
        self.action_counter3 += 1;
        if self.flags.hit_anything() || self.action_counter3 > 300 {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
        }

        Ok(())
    }

    pub(crate) fn tick_n203_critter_destroyed_egg_corridor(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.y += 0x600;
                    self.action_num = 1;
                    self.anim_num = 0;
                }

                let player = self.get_closest_player_mut(players);

                if self.x > player.x {
                    self.direction = Direction::Left;
                } else {
                    self.direction = Direction::Right;
                }

                if self.target_x < 100 {
                    self.target_x += 1;
                }

                if self.action_counter >= 8
                    && self.x - 0xe000 < player.x
                    && self.x + 0xe000 > player.x
                    && self.y - 0xa000 < player.y
                    && self.y + 0xa000 > player.y
                {
                    self.anim_num = 1;
                } else if self.action_counter < 8 {
                    self.action_counter += 1;
                }

                if self.shock > 0 {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }

                if self.action_counter >= 8
                    && self.target_x >= 100
                    && self.x - 0x6000 < player.x
                    && self.x + 0x6000 > player.x
                    && self.y - 0xa000 < player.y
                    && self.y + 0xa000 > player.y
                {
                    self.action_num = 2;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 3;
                    self.anim_num = 2;

                    self.vel_y = -0x5ff;
                    state.sound_manager.play_sfx(30);

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
                    self.anim_num = 0;

                    state.sound_manager.play_sfx(23);
                }
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Right { 3 } else { 0 };
        self.anim_rect = state.constants.npc.n203_critter_destroyed_egg_corridor[self.anim_num as usize + dir_offset];

        self.vel_y += 0x40;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        Ok(())
    }

    pub(crate) fn tick_n204_small_falling_spike(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.target_x = self.x;
                }

                let player = self.get_closest_player_ref(&players);

                if player.x > self.x - 0x1800 && player.x < self.x + 0x1800 && player.y > self.y {
                    self.action_num = 2;
                }
            }
            2 => {
                self.action_counter += 1;
                self.x = if ((self.action_counter / 6) & 1) != 0 { self.target_x - 0x200 } else { self.target_x };

                if self.action_counter > 30 {
                    self.action_num = 3;
                    self.anim_num = 1;
                }
            }
            3 => {
                self.vel_y += 0x20;

                if self.flags.hit_anything() {
                    let player = self.get_closest_player_ref(&players);
                    if !player.cond.hidden() {
                        state.sound_manager.play_sfx(12);
                    }

                    npc_list.create_death_smoke(
                        self.x,
                        self.y,
                        self.display_bounds.right as usize,
                        4,
                        state,
                        &self.rng,
                    );
                    self.cond.set_alive(false);
                }
            }
            _ => (),
        }

        if self.vel_y > 3072 {
            self.vel_y = 3072;
        }

        self.y += self.vel_y;
        self.anim_rect = state.constants.npc.n204_small_falling_spike[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n205_large_falling_spike(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        bullet_manager: &mut BulletManager,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.target_x = self.x;
                    self.y += 0x800;
                }

                let player = self.get_closest_player_ref(&players);
                if player.x > self.x - 0x1800 && player.x < self.x + 0x1800 && player.y > self.y {
                    self.action_num = 2;
                }
            }
            2 => {
                self.action_counter += 1;

                if ((self.action_counter / 6) & 1) != 0 {
                    self.x = self.target_x - 0x200;
                } else {
                    self.x = self.target_x;
                }

                if self.action_counter > 30 {
                    self.action_num = 3;
                    self.anim_num = 1;
                    self.action_counter = 0;
                }
            }
            3 => {
                let player = self.get_closest_player_ref(&players);

                if player.y <= self.y {
                    self.npc_flags.set_solid_hard(true);
                    self.damage = 0;
                } else {
                    self.npc_flags.set_solid_hard(false);
                    self.damage = 127;
                }

                self.vel_y += 0x20;
                self.action_counter += 1;

                if self.action_counter > 8 && self.flags.any_flag() {
                    self.action_num = 4;
                    self.action_counter = 0;
                    self.damage = 0;
                    self.vel_y = 0;
                    self.npc_flags.set_solid_hard(true);

                    state.sound_manager.play_sfx(12);
                    npc_list.create_death_smoke(
                        self.x,
                        self.y,
                        self.display_bounds.right as usize,
                        4,
                        state,
                        &self.rng,
                    );
                    bullet_manager.create_bullet(
                        self.x,
                        self.y,
                        24,
                        TargetPlayer::Player1,
                        Direction::Left,
                        &state.constants,
                    );
                }
            }
            4 => {
                self.action_counter += 1;
                if self.action_counter > 4 {
                    self.action_num = 5;
                    self.npc_flags.set_shootable(true);
                }
            }
            _ => (),
        }

        if self.vel_y > 0xC00 {
            self.vel_y = 0xC00;
        }

        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n205_large_falling_spike[self.anim_num as usize];
        Ok(())
    }

    pub(crate) fn tick_n206_counter_bomb(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.target_x = self.x;
                    self.target_y = self.y;
                    self.action_counter3 = 120;
                    self.action_counter = self.rng.range(0..50) as u16;
                }
                self.action_counter += 1;
                if self.action_counter >= 50 {
                    self.action_counter = 0;
                    self.action_num = 2;
                    self.vel_y = 0x300;
                }
            }
            2 => {
                let player = self.get_closest_player_ref(&players);
                if player.x > self.x - 0xA000 && player.x < self.x + 0xA000 {
                    self.action_counter = 0;
                    self.action_num = 3;
                }

                if self.shock > 0 {
                    self.action_counter = 0;
                    self.action_num = 3;
                }
            }
            3 => {
                let mut npc = NPC::create(207, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x + 0x2000;
                npc.y = self.y + 0x800;

                match self.action_counter {
                    0 => {
                        npc.tsc_direction = 0;
                        let _ = npc_list.spawn(0x100, npc);
                    }
                    60 => {
                        npc.tsc_direction = 1;
                        let _ = npc_list.spawn(0x100, npc);
                    }
                    120 => {
                        npc.tsc_direction = 2;
                        let _ = npc_list.spawn(0x100, npc);
                    }
                    180 => {
                        npc.tsc_direction = 3;
                        let _ = npc_list.spawn(0x100, npc);
                    }
                    240 => {
                        npc.tsc_direction = 4;
                        let _ = npc_list.spawn(0x100, npc);
                    }
                    300 => {
                        self.hit_bounds.right = 0x10000;
                        self.hit_bounds.left = 0x10000;
                        self.hit_bounds.top = 0xC800;
                        self.hit_bounds.bottom = 0xC800;
                        self.damage = 30;
                        self.cond.set_explode_die(true);

                        state.quake_counter = 20;
                        state.quake_rumble_counter = 20;
                        state.sound_manager.play_sfx(35);
                        npc_list.create_death_smoke(self.x, self.y, 0x10000, 100 as usize, state, &self.rng);
                    }
                    _ => (),
                }

                self.action_counter += 1;
            }
            _ => (),
        }

        if self.action_num > 1 {
            if self.target_y < self.y {
                self.vel_y -= 0x10;
            }

            if self.target_y > self.y {
                self.vel_y += 0x10;
            }

            self.vel_y = self.vel_y.clamp(-0x100, 0x100);
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.animate(4, 0, 2);

        self.anim_rect = state.constants.npc.n206_counter_bomb[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n207_counter_bomb_countdown(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = self.tsc_direction;
                    state.sound_manager.play_sfx(43);
                }

                self.x += 0x200;

                self.action_counter += 1;
                if self.action_counter > 8 {
                    self.action_num = 2;
                    self.action_counter = 0;
                }
            }
            2 => {
                self.action_counter += 1;
                if self.action_counter > 30 {
                    self.cond.set_alive(false);
                }
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n207_counter_bomb_countdown[self.anim_num as usize % 5];

        Ok(())
    }

    pub(crate) fn tick_n208_basu_destroyed_egg_corridor(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 => {
                if player.x < self.x + 0x2000 && player.x > self.x - 0x2000 {
                    self.target_x = self.x;
                    self.target_y = self.y;
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.action_counter2 = 0;
                    self.damage = 6;
                    self.vel_y = -0x100;
                    self.tsc_direction = self.direction as u16;
                    self.npc_flags.set_shootable(true);

                    self.x = player.x + self.direction.opposite().vector_x() * 0x20000;
                    self.vel_x = self.direction.vector_x() * 0x2ff;
                } else {
                    self.anim_rect = Rect::new(0, 0, 0, 0);
                    self.damage = 0;
                    self.vel_x = 0;
                    self.vel_y = 0;
                    self.npc_flags.set_shootable(false);
                }

                return Ok(());
            }
            1 => {
                if self.x > player.x {
                    self.direction = Direction::Left;
                    self.vel_x -= 0x10;
                } else {
                    self.direction = Direction::Right;
                    self.vel_x += 0x10;
                }

                if self.flags.hit_left_wall() {
                    self.vel_x = 0x200;
                }

                if self.flags.hit_right_wall() {
                    self.vel_x = -0x200;
                }

                self.vel_y += ((self.target_y - self.y).signum() | 1) * 0x08;

                self.vel_x = clamp(self.vel_x, -0x2ff, 0x2ff);
                self.vel_y = clamp(self.vel_y, -0x100, 0x100);

                if self.shock > 0 {
                    self.x += self.vel_x / 2;
                    self.y += self.vel_y / 2;
                } else {
                    self.x += self.vel_x;
                    self.y += self.vel_y;
                }

                if player.x > self.x + 0x32000 || player.x < self.x - 0x32000 {
                    self.action_num = 0;
                    self.vel_x = 0;
                    self.x = self.target_x;
                    self.damage = 0;
                    self.direction = Direction::from_int_facing(self.tsc_direction as usize).unwrap_or(Direction::Left);
                    self.anim_rect = Rect::new(0, 0, 0, 0);
                    return Ok(());
                }
            }
            _ => (),
        }

        if self.action_counter < 150 {
            self.action_counter += 1;
        } else {
            self.action_counter2 += 1;
            if (self.action_counter2 % 8) == 0 && abs(self.x - player.x) < 0x14000 {
                let angle = f64::atan2((self.y - player.y) as f64, (self.x - player.x) as f64)
                    + self.rng.range(-6..6) as f64 * CDEG_RAD;

                let mut npc = NPC::create(209, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x;
                npc.y = self.y;
                npc.vel_x = (angle.cos() * -1536.0) as i32;
                npc.vel_y = (angle.sin() * -1536.0) as i32;

                let _ = npc_list.spawn(0x100, npc);
                state.sound_manager.play_sfx(39);
            }

            if self.action_counter2 > 16 {
                self.action_counter = 0;
                self.action_counter2 = 0;
            }
        }

        self.anim_counter += 1;
        if self.anim_counter > 1 {
            self.anim_counter = 0;
            self.anim_num += 1;
            if self.anim_num > 1 {
                self.anim_num = 0;
            }
        }

        if self.action_counter > 120 && self.action_counter & 0x02 == 1 && self.anim_num == 1 {
            self.anim_num = 2;
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n208_basu_destroyed_egg_corridor[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n209_basu_projectile_destroyed_egg_corridor(
        &mut self,
        state: &mut SharedGameState,
    ) -> GameResult {
        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_counter += 1;
        if self.anim_counter > 2 {
            self.anim_counter = 0;
            self.anim_num += 1;

            if self.anim_num > 3 {
                self.anim_num = 0;
            }
        }

        self.anim_rect = state.constants.npc.n209_basu_projectile_destroyed_egg_corridor[self.anim_num as usize];

        self.action_counter2 += 1;
        if self.flags.hit_anything() || self.action_counter2 > 300 {
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
            self.cond.set_alive(false);
        }

        Ok(())
    }

    pub(crate) fn tick_n210_beetle_destroyed_egg_corridor(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        match self.action_num {
            0 => {
                let player = self.get_closest_player_mut(players);

                if player.x < self.x + 0x2000 && player.x > self.x - 0x2000 {
                    self.npc_flags.set_shootable(true);
                    self.vel_y = -0x200;
                    self.target_y = self.y;
                    self.action_num = 1;
                    self.damage = 2;

                    match self.direction {
                        Direction::Left => {
                            self.x = player.x + 0x20000;
                            self.vel_x = -0x2ff;
                        }
                        Direction::Right => {
                            self.x = player.x - 0x20000;
                            self.vel_x = 0x2ff;
                        }
                        _ => (),
                    }
                } else {
                    self.npc_flags.set_shootable(false);
                    self.anim_rect.left = 0;
                    self.anim_rect.right = 0;
                    self.damage = 0;
                    self.vel_x = 0;
                    self.vel_y = 0;

                    return Ok(());
                }
            }
            1 => {
                let player = self.get_closest_player_mut(players);

                if self.x > player.x {
                    self.direction = Direction::Left;
                    self.vel_x -= 0x10;
                } else {
                    self.direction = Direction::Right;
                    self.vel_x += 0x10;
                }

                self.vel_y += if self.y < self.target_y { 8 } else { -8 };

                self.vel_x = clamp(self.vel_x, -0x2ff, 0x2ff);
                self.vel_y = clamp(self.vel_y, -0x200, 0x200);

                if self.shock > 0 {
                    self.x += self.vel_x / 2;
                    self.y += self.vel_y / 2;
                } else {
                    self.x += self.vel_x;
                    self.y += self.vel_y;
                }
            }
            _ => (),
        }

        self.anim_counter += 1;
        if self.anim_counter > 1 {
            self.anim_counter = 0;
            self.anim_num += 1;

            if self.anim_num > 1 {
                self.anim_num = 0;
            }
        }

        if self.anim_counter == 1 {
            self.anim_rect = state.constants.npc.n210_beetle_destroyed_egg_corridor
                [self.anim_num as usize + if self.direction == Direction::Right { 2 } else { 0 }];
        }

        Ok(())
    }
}
