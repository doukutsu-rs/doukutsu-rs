use crate::common::Direction;
use crate::framework::error::GameResult;
use crate::game::caret::CaretType;
use crate::game::npc::list::NPCList;
use crate::game::npc::NPC;
use crate::game::player::Player;
use crate::game::shared_game_state::SharedGameState;
use crate::game::stage::Stage;
use crate::util::rng::RNG;

impl NPC {
    pub(crate) fn tick_n241_critter_red(
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
                if self.x > player.x {
                    self.direction = Direction::Left;
                } else {
                    self.direction = Direction::Right;
                }

                if self.action_counter >= 8
                    && self.x - 0x12000 < player.x
                    && self.x + 0x12000 > player.x
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
                    && self.x - 0xc000 < player.x
                    && self.x + 0xc000 > player.x
                    && self.y - 0xa000 < player.y
                    && self.y + 0xc000 > player.y
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
                        self.vel_x = -0x200;
                    } else {
                        self.vel_x = 0x200;
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

        self.vel_y += 0x55;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n241_critter_red[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n242_bat_last_cave(&mut self, state: &mut SharedGameState, stage: &mut Stage) -> GameResult {
        if self.x < 0 || self.x > stage.map.width as i32 * state.tile_size.as_int() * 0x200 {
            self.vanish(state);
            return Ok(());
        }
        loop {
            match self.action_num {
                0 => {
                    self.action_num = 1;
                    self.target_x = self.x;
                    self.target_y = self.y;
                    self.action_counter = self.rng.range(0..50) as u16;
                    continue;
                }
                1 if self.action_counter > 0 => {
                    self.action_counter -= 1;
                }
                1 => {
                    self.action_num = 2;
                    self.vel_y = 0x400;
                    continue;
                }
                2 => {
                    self.vel_x = self.direction.vector_x() * 0x100;
                    self.vel_y += (self.target_y - self.y).signum() * 0x10;
                    self.vel_y = self.vel_y.clamp(-0x300, 0x300);
                }
                _ => (),
            }
            break;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.animate(1, 0, 2);

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

        self.anim_rect = state.constants.npc.n242_bat_last_cave[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n243_bat_generator(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.action_counter = self.rng.range(0..500) as u16;
                }

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 0;

                    let mut npc = NPC::create(242, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y + self.rng.range(-32..32) * 0x200;
                    npc.direction = self.direction;

                    let _ = npc_list.spawn(0x100, npc);
                }
            }
            _ => return Ok(()),
        }

        Ok(())
    }

    pub(crate) fn tick_n244_lava_drop(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        self.vel_y += 0x40;

        // idfk why was that there in original code but I'll leave it there in case
        let mut hit = self.flags.hit_anything();

        if self.action_counter > 10 && self.flags.in_water() {
            hit = true;
        }

        if hit {
            for _ in 0..3 {
                state.create_caret(self.x, self.y + 0x800, CaretType::Bubble, Direction::Right);
            }

            let player = self.get_closest_player_ref(&players);
            if self.x > player.x - 0x20000
                && self.x < player.x + 0x20000
                && self.y > player.y - 0x14000
                && self.y < player.y + 0x14000
            {
                state.sound_manager.play_sfx(21);
            }

            self.cond.set_alive(false);
            return Ok(());
        }

        self.clamp_fall_speed();

        self.y += self.vel_y;
        self.anim_rect = state.constants.npc.n244_lava_drop;

        Ok(())
    }

    pub(crate) fn tick_n245_lava_drop_generator(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.target_x = self.x;
                    self.action_counter = self.event_num;
                }

                self.anim_num = 0;
                if self.action_counter > 0 {
                    self.action_counter -= 1;
                    return Ok(());
                }

                self.action_num = 10;
                self.anim_counter = 0;
            }
            10 => {
                self.anim_counter += 1;
                if self.anim_counter > 10 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num > 3 {
                        self.anim_num = 0;
                        self.action_num = 1;
                        self.action_counter = self.flag_num;

                        let mut npc = NPC::create(244, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.x;
                        npc.y = self.y;

                        let _ = npc_list.spawn(0x100, npc);
                    }
                }
            }
            _ => (),
        }

        self.x = if (self.anim_counter & 2) != 0 { self.target_x } else { self.target_x + 0x200 };
        self.anim_rect = state.constants.npc.n245_lava_drop_generator[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n276_red_demon(
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
                }

                if self.action_num == 1 {
                    self.vel_x = 0;
                    self.action_num = 2;
                    self.anim_num = 0;
                }

                let player = self.get_closest_player_ref(&players);
                self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };

                self.animate(20, 0, 1);

                if self.shock > 0 {
                    self.action_num = 10;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.anim_num = 3;
                    self.action_counter = 0;
                    self.npc_flags.set_shootable(true);
                }

                self.action_counter += 1;
                match self.action_counter {
                    30 | 40 | 50 => {
                        self.anim_num = 4;
                        let player = self.get_closest_player_ref(&players);

                        let mut npc = NPC::create(277, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.x;
                        npc.y = self.y;

                        let angle = f64::atan2((self.y - player.y) as f64, (self.x - player.x) as f64);
                        npc.vel_x = (-2048.0 * angle.cos()) as i32;
                        npc.vel_y = (-2048.0 * angle.sin()) as i32;

                        let _ = npc_list.spawn(0x100, npc);

                        state.sound_manager.play_sfx(39);
                    }
                    34 | 44 | 54 => {
                        self.anim_num = 3;
                    }
                    _ => (),
                }

                if self.action_counter > 60 {
                    self.action_num = 20;
                    self.action_counter = 0;
                    self.anim_num = 2;
                }
            }
            20 => {
                self.action_counter += 1;
                if self.action_counter > 20 {
                    self.action_num = 21;
                    self.action_counter = 0;
                    self.anim_num = 5;

                    self.vel_y = -0x5ff;

                    let player = self.get_closest_player_ref(&players);
                    self.vel_x = if self.x < player.x { 0x100 } else { -0x100 };
                }
            }
            21 => {
                self.action_counter += 1;
                match self.action_counter {
                    30 | 40 | 50 => {
                        self.anim_num = 6;
                        let player = self.get_closest_player_ref(&players);

                        let mut npc = NPC::create(277, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.x;
                        npc.y = self.y - 0x1400;

                        let angle = f64::atan2((self.y - 0x1400 - player.y) as f64, (self.x - player.x) as f64);
                        npc.vel_x = (-2048.0 * angle.cos()) as i32;
                        npc.vel_y = (-2048.0 * angle.sin()) as i32;

                        let _ = npc_list.spawn(0x100, npc);

                        state.sound_manager.play_sfx(39);
                    }
                    34 | 44 => {
                        self.anim_num = 5;
                    }
                    _ => (),
                }

                if self.action_counter > 53 {
                    self.anim_num = 7;
                }

                if self.flags.hit_bottom_wall() {
                    self.action_num = 22;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    state.quake_counter = 10;
                    state.quake_rumble_counter = 10;
                    state.sound_manager.play_sfx(26);
                }
            }
            22 => {
                self.vel_x /= 2;

                self.action_counter += 1;
                if self.action_counter > 22 {
                    self.action_num = 10;
                }
            }
            50 => {
                self.npc_flags.set_shootable(false);
                self.damage = 0;

                if self.flags.hit_bottom_wall() {
                    self.action_num = 51;
                    self.action_counter = 0;
                    self.anim_num = 2;
                    state.quake_counter = 10;
                    state.quake_rumble_counter = 10;
                    state.sound_manager.play_sfx(72);
                    self.create_xp_drop_custom(self.x, self.y, 19, state, npc_list);

                    npc_list.create_death_smoke(
                        self.x,
                        self.y,
                        self.display_bounds.right as usize,
                        8,
                        state,
                        &self.rng,
                    );
                }
            }
            51 => {
                self.vel_x = 7 * self.vel_x / 8;
                self.anim_num = 8;
            }
            _ => (),
        }

        self.vel_y += 0x20;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        if self.action_num < 50 {
            let player = self.get_closest_player_ref(&players);
            self.direction = if self.x < player.x { Direction::Right } else { Direction::Left };
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 9 };

        self.anim_rect = state.constants.npc.n276_red_demon[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n277_red_demon_projectile(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
        }

        if self.action_num == 1 {
            self.x += self.vel_x;
            self.y += self.vel_y;

            if self.flags.hit_anything() {
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
        if self.action_counter % 5 == 0 {
            state.sound_manager.play_sfx(110);
        }

        self.anim_num += 1;
        if self.anim_num > 2 {
            self.anim_num = 0;
        }

        self.anim_rect = state.constants.npc.n277_red_demon_projectile[self.anim_num as usize];

        Ok(())
    }
}
