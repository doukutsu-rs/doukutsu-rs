use crate::common::{Direction, CDEG_RAD};
use crate::framework::error::GameResult;
use crate::game::caret::CaretType;
use crate::game::npc::list::BorrowedNPC;
use crate::game::npc::{NPCContext, NPC};
use crate::game::shared_game_state::SharedGameState;
use crate::util::rng::RNG;

impl BorrowedNPC<'_> {
    // Gaudi from room 2
    pub(crate) fn tick_n361_flying_gaudi(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                self.action_num = 1;

                let player = self.get_closest_player_ref(&players);
                let spawn_distance = state.tile_size.as_int() * 0x200 * 13;
                if (self.direction == Direction::Right && self.x - player.x <= -spawn_distance)
                    || (self.direction == Direction::Left && self.x - player.x >= spawn_distance)
                {
                    self.action_num = 10;
                } else {
                    return Ok(());
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.damage = 5;
                    self.npc_flags.set_shootable(true);
                }

                let player = self.get_closest_player_ref(&players);

                self.vel_x2 += if self.x > player.x { -0x10 } else { 0x10 };
                self.vel_y2 += if self.y > player.y { -0x10 } else { 0x10 };
                self.direction = if self.x > player.x { Direction::Right } else { Direction::Left };

                if self.flags.hit_left_wall() {
                    self.vel_x2 /= -2;
                }

                if self.flags.hit_right_wall() {
                    self.vel_x2 /= -2;
                }

                if self.flags.hit_top_wall() {
                    self.vel_y2 *= -1;
                }

                if self.flags.hit_bottom_wall() {
                    self.vel_y2 /= -2;
                }

                self.vel_x2 = self.vel_x2.clamp(-0x5FF, 0x5FF);
                self.vel_y2 = self.vel_y2.clamp(-0x5FF, 0x5FF);
            }
            _ => (),
        }

        self.x += self.vel_x2;
        self.y += self.vel_y2;

        self.animate(1, 0, 1);
        let dir_offset = if self.direction == Direction::Right { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n361_flying_gaudi[self.anim_num as usize + dir_offset];

        if self.life <= 985 {
            self.npc_type = 154;
            self.action_num = 0;
        }
        Ok(())
    }

    // Curly clone from room 3, shoots machine gun-like bullets
    pub(crate) fn tick_n362_curly_clone(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, npc_list, .. }: NPCContext,
    ) -> GameResult {
        let player = self.get_closest_player_ref(&players);

        if self.x > player.x + 0x28000
            || self.x < player.x - 0x28000
            || self.y > player.y + 0x1E000
            || self.y < player.y - 0x1E000
        {
            return Ok(());
        }

        match self.action_num {
            0 => {
                self.vel_x = 0;
                self.anim_num = 0;
                self.action_num = 1;
            }
            1 => {
                if self.rng.range(0..60) == 1 {
                    self.direction = self.direction.opposite();
                    self.action_num = 10;
                }

                if self.y > player.y - 0x2000 && self.y < player.y + 0x2000 && self.rng.range(0..50) == 1 {
                    self.action_num = 5
                }
            }
            5 => {
                self.face_player(player);
                self.anim_num = if self.action_counter % 2 != 0 { 5 } else { 0 };
                self.action_counter += 1;

                if self.action_counter >= 30 {
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.action_num = 6;
                }
            }
            6 => {
                self.action_counter += 1;

                if self.action_counter % 4 == 1 {
                    self.anim_counter = 0;

                    let mut npc = NPC::create(364, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x + 0x1000 * self.direction.vector_x();
                    npc.y = self.y + 0x800;
                    npc.direction = self.direction;

                    let _ = npc_list.spawn(0x100, npc);

                    self.x += 0x200 * self.direction.opposite().vector_x();
                }

                if self.action_counter >= 9 {
                    self.action_num = 1;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.action_counter = self.rng.range(25..100) as u16;
                    self.anim_num = 1;
                }

                self.animate(2, 0, 1);

                self.vel_x = 0x200 * self.direction.vector_x();

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.vel_x = 0;
                }

                if (self.direction == Direction::Left && self.flags.hit_left_wall())
                    || (self.direction == Direction::Right && self.flags.hit_right_wall())
                {
                    self.anim_num = 1;
                    self.vel_y = -0x5FF;
                    self.action_num = 20;
                    state.sound_manager.play_sfx(30);

                    if player.cond.hidden() {
                        state.sound_manager.play_sfx(30); // ???
                    }
                }
            }
            20 => {
                if (self.direction == Direction::Left && self.flags.hit_left_wall())
                    || (self.direction == Direction::Right && self.flags.hit_right_wall())
                {
                    self.action_counter2 += 1;

                    if self.action_counter2 > 12 {
                        self.direction = self.direction.opposite();
                    }
                } else {
                    self.action_counter2 = 0;
                }

                self.vel_x = 0x200 * self.direction.vector_x();

                if self.flags.hit_bottom_wall() {
                    self.action_num = 21;
                    self.action_counter = 0;
                    self.anim_num = 0;
                    self.vel_x = 0;

                    if player.cond.hidden() {
                        state.sound_manager.play_sfx(23); // ???
                    }
                }
            }
            21 => {
                self.action_counter += 1;

                if self.action_counter >= 10 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.action_counter = 0;
                }
            }
            _ => (),
        }

        self.vel_y += 0x40;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 7 };
        self.anim_rect = state.constants.npc.n362_curly_clone[self.anim_num as usize + dir_offset];

        if self.life <= 970 {
            self.npc_type = 363; // Dead Curly Clone
            self.action_num = 0;
        }

        Ok(())
    }

    // Dead Curly Clone
    pub(crate) fn tick_n363_dead_curly_clone(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { npc_list, .. }: NPCContext,
    ) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = 2;
                self.damage = 0;
                self.npc_flags.set_ignore_solidity(false);
                self.npc_flags.set_shootable(false);

                self.vel_x = 0x100 * self.direction.opposite().vector_x();
                self.vel_y = -0x200;

                state.sound_manager.play_sfx(53);
            }
            1 => {
                if self.flags.hit_bottom_wall() {
                    self.action_num = 2;
                    self.anim_num = 3;
                }
            }
            2 => {
                self.vel_x = self.vel_x * 8 / 9;

                self.action_counter += 1;

                if self.action_counter2 == 77 && self.action_counter >= 250 {
                    self.action_num = 3;

                    let mut npc = NPC::create(369, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.y = 0x1000;

                    npc.x = self.rng.range(62..78) * 0x2000;
                    let _ = npc_list.spawn(0x100, npc.clone());

                    npc.x = self.rng.range(62..78) * 0x2000;
                    let _ = npc_list.spawn(0x100, npc);
                } else if self.action_counter2 != 77 && self.action_counter >= 50 {
                    self.cond.set_explode_die(true);
                }
            }
            _ => (),
        }

        self.vel_y += 0x20;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 7 };
        self.anim_rect = state.constants.npc.n362_curly_clone[self.anim_num as usize + dir_offset];
        Ok(())
    }

    // Fast, machine gun-like bullets shot by Curly clone (NPC 362)
    pub(crate) fn tick_n364_fast_bullet(&mut self, state: &mut SharedGameState, _: NPCContext) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.action_counter2 = 0;

                state.create_caret(self.x, self.y, CaretType::Shoot, Direction::Left);

                match self.direction {
                    Direction::Left | Direction::Right => {
                        self.vel_x = 0xA00 * self.direction.vector_x();
                        self.vel_y = self.rng.range(-128..128);
                    }
                    Direction::Up | Direction::Bottom => {
                        self.vel_y = 0xA00 * self.direction.vector_y();
                        self.vel_x = self.rng.range(-128..128);
                    }
                    _ => (),
                }
            }
            1 => {
                self.x += self.vel_x;
                self.y += self.vel_y;

                if self.flags.hit_anything() {
                    state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Right);
                    state.sound_manager.play_sfx(28);
                    self.cond.set_alive(false);
                }
            }
            _ => (),
        }

        self.action_counter2 += 1;

        if self.action_counter2 >= 300 {
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Right);
            state.sound_manager.play_sfx(28);
            self.cond.set_alive(false);
        }

        self.anim_rect = state.constants.npc.n364_fast_bullet[self.direction as usize];
        Ok(())
    }

    // Curly clone from room 3, doesn't move, shoots a pair of spherical bullets
    pub(crate) fn tick_n365_still_curly_clone(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, npc_list, .. }: NPCContext,
    ) -> GameResult {
        let player = self.get_closest_player_ref(&players);

        if self.x > player.x + 0x28000
            || self.x < player.x - 0x28000
            || self.y > player.y + 0x1E000
            || self.y < player.y - 0x1E000
        {
            return Ok(());
        }

        match self.action_num {
            0 => {
                self.vel_x = 0;
                self.anim_num = 4;
                self.action_num = 1;
            }
            1 => {
                self.direction = if self.x > player.x { Direction::Left } else { Direction::Right };
                if self.rng.range(0..70) == 1 {
                    self.action_num = 5;
                }
            }
            5 => {
                self.vel_x = self.vel_x * 8 / 9;

                self.anim_num = if self.action_counter % 2 == 0 { 6 } else { 4 };
                self.action_counter += 1;

                if self.action_counter > 30 {
                    self.action_num = 6;
                    self.action_counter = 0;
                    self.anim_num = 4;
                }
            }
            6 => {
                self.action_counter += 1;

                if self.action_counter % 4 == 1 {
                    let angle = f64::atan2((self.y - player.y) as f64, (self.x - player.x) as f64)
                        + self.rng.range(-6..6) as f64 * CDEG_RAD;
                    let mut npc = NPC::create(148, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    npc.vel_x = (angle.cos() * -1536.0) as i32;
                    npc.vel_y = (angle.sin() * -1536.0) as i32;
                    let _ = npc_list.spawn(0x100, npc);
                }

                if self.action_counter >= 7 {
                    self.action_num = 1;
                    self.action_counter = 0;
                }
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 7 };
        self.anim_rect = state.constants.npc.n362_curly_clone[self.anim_num as usize + dir_offset];

        if self.life <= 970 {
            self.npc_type = 363; // Dead Curly Clone
            self.action_num = 0;
        }

        Ok(())
    }

    // Curly clone, moves very slowly
    pub(crate) fn tick_n366_zombie_curly_clone(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, .. }: NPCContext,
    ) -> GameResult {
        let player = self.get_closest_player_ref(&players);
        if self.x > player.x + 0x28000
            || self.x < player.x - 0x28000
            || self.y > player.y + 0x1E000
            || self.y < player.y - 0x1E000
        {
            return Ok(());
        }

        match self.action_num {
            0 => {
                self.vel_x = 0;
                self.anim_num = 0;

                if (self.x - 0x10000 < player.x && player.x < self.x + 0x10000)
                    && (self.y - 0x8000 < player.y && player.y < self.y + 0x8000)
                {
                    self.direction = if self.x < player.x { Direction::Right } else { Direction::Left };
                    self.anim_num = 1;
                    self.action_num = 1;
                    self.action_counter = 1;
                }
            }
            1 => {
                self.action_counter += 1;

                if self.action_counter >= 70 {
                    self.direction = if self.x < player.x { Direction::Right } else { Direction::Left };
                    self.action_num = 10;
                }
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    self.vel_x = 0x40 * self.direction.vector_x();
                    self.action_counter = self.rng.range(25..100) as u16;
                    self.anim_num = 2;
                }

                self.animate(16, 1, 2);

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 1;
                    self.action_counter = 70;
                    self.vel_x = 0;
                }

                if (self.direction == Direction::Right && self.flags.hit_right_wall())
                    || (self.direction == Direction::Left && self.flags.hit_left_wall())
                {
                    self.vel_x = 0x40 * self.direction.vector_x();
                    self.direction = self.direction.opposite();
                }
            }
            _ => (),
        }

        self.vel_y += 0x40;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n366_zombie_curly_clone[self.anim_num as usize + dir_offset];

        if self.life <= 970 {
            self.npc_type = 363; // Dead Curly Clone
            self.action_num = 0;
            self.action_counter = 0;
        }
        Ok(())
    }

    // Incubator, contains a Curly clone
    pub(crate) fn tick_n367_curly_clone_incubator(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, npc_list, .. }: NPCContext,
    ) -> GameResult {
        let player = self.get_closest_player_ref(&players);

        let within_range = match self.direction {
            Direction::Left => self.x - 0x6000 < player.x && player.x < self.x - 0x2000,
            Direction::Right => self.x + 0x2000 < player.x && player.x < self.x + 0x6000,
            _ => false,
        } && (self.y - 0x1400 < player.y && player.y < self.y + 0x1400);

        if within_range {
            let mut npc = NPC::create(366, &state.npc_table);
            npc.cond.set_alive(true);
            npc.x = self.x;
            npc.y = self.y;

            let _ = npc_list.spawn(0x100, npc.clone());

            state.sound_manager.play_sfx(72);
            npc_list.create_death_smoke(self.x, self.y, 0, 1, state, &self.rng);
            self.cond.set_alive(false);
        }

        self.anim_rect = state.constants.npc.n367_curly_clone_incubator;
        Ok(())
    }

    // G-CLONE boss
    pub(crate) fn tick_n368_gclone(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, npc_list, .. }: NPCContext,
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    state.sound_manager.play_sfx(29);
                    self.x -= 0x800;
                    self.y += 0x1400;
                    self.action_num = 1;
                    self.action_counter2 = 0;
                }

                self.action_counter += 1;
                self.anim_counter += 1;

                if self.action_counter >= 50 {
                    self.action_num = 100;
                    self.anim_num = 0;
                }
            }
            100 => {
                self.npc_flags.set_invulnerable(true);
                self.action_num = 101;
                self.action_counter = 1;
            }
            101 => {
                self.action_counter += 1;

                if self.action_counter >= 150 {
                    self.action_counter = 0;
                    self.action_num = 110;
                }
            }
            110 => {
                self.action_counter += 1;

                if self.action_counter > 50 && self.action_counter % 6 == 1 {
                    let mut angle =
                        f64::atan2((self.y - 0x3400 - player.y) as f64, (self.x - 0x2000 - player.x) as f64);

                    let mut npc = NPC::create(11, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x - 0x3400;
                    npc.y = self.y - 0x2000;
                    angle += self.rng.range(-16..16) as f64 * CDEG_RAD;
                    npc.vel_x = (angle.cos() * -1536.0) as i32;
                    npc.vel_y = (angle.sin() * -1536.0) as i32;
                    let _ = npc_list.spawn(0x100, npc);
                    state.sound_manager.play_sfx(33);
                }

                if self.action_counter > 100 {
                    self.action_counter = 0;
                    self.action_num = 101;
                }
            }
            500 | 501 => {
                if self.action_num == 500 {
                    self.anim_num = 2;
                    self.npc_flags.set_invulnerable(true);
                    self.action_num = 501;
                }

                state.quake_counter = 2;
                state.quake_rumble_counter = 2;

                self.action_counter += 1;
                if self.action_counter % 20 == 0 {
                    state.sound_manager.play_sfx(52);
                }

                npc_list.kill_npcs_by_type(369, true, state, self);

                npc_list.create_death_smoke(
                    self.x + (self.rng.range(-32..32) << 9) as i32,
                    self.y + (self.rng.range(-48..24) << 9) as i32,
                    self.display_bounds.right as usize,
                    8,
                    state,
                    &self.rng,
                );

                if self.action_counter > 100 {
                    self.action_num = 510;
                }
            }
            510 => {}
            _ => {}
        }

        if self.action_num < 500 {
            match self.action_counter2 {
                0 => {
                    if self.life < 300 {
                        self.action_counter2 = 1;
                        self.npc_flags.set_shootable(false);
                        let mut npc = NPC::create(369, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.rng.range(62..78) * 0x2000;
                        npc.y = 0x1000;
                        npc.direction = Direction::Right;

                        let _ = npc_list.spawn(0x100, npc);
                        state.sound_manager.play_sfx(4);
                    }
                }
                1 => {
                    if self.anim_num <= 2 {
                        self.anim_counter += 1;
                        if self.anim_counter >= 2 {
                            self.anim_num += 1;
                            self.anim_counter = 0;
                        }
                    } else {
                        self.anim_num = 2;
                        self.action_counter2 = 2;
                    }
                }
                2 => {
                    self.action_counter3 = self.action_counter3.wrapping_add(1);
                    if self.action_counter3 >= 500 {
                        self.action_counter3 = 0;
                        self.action_counter2 = 3;

                        self.anim_counter += 1;
                        if self.anim_counter > 2 {
                            self.anim_counter = 0;
                            self.anim_num = self.anim_num.saturating_sub(1);
                        }
                    }
                }
                3 => {
                    if self.anim_num > 0 {
                        self.anim_counter += 1;
                        if self.anim_counter > 2 {
                            self.anim_counter = 0;
                            self.anim_num = self.anim_num.saturating_sub(1);
                        }
                    } else {
                        self.action_counter2 = 4;
                        self.npc_flags.set_shootable(true);

                        state.sound_manager.play_sfx(4);
                    }
                }
                4 => {
                    if self.life < 200 {
                        self.action_counter2 = 5;
                        let mut npc = NPC::create(369, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.rng.range(62..78) * 0x2000;
                        npc.y = 0x1000;
                        npc.direction = Direction::Right;

                        let _ = npc_list.spawn(0x100, npc);
                        state.sound_manager.play_sfx(4);
                    }
                }
                5 => {
                    if self.life >= 100 {
                        self.action_counter2 = 6;
                        self.npc_flags.set_shootable(false);
                        let mut npc = NPC::create(369, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.rng.range(62..78) * 0x2000;
                        npc.y = 0x1000;
                        npc.direction = Direction::Right;

                        let _ = npc_list.spawn(0x100, npc);
                        state.sound_manager.play_sfx(4);
                    }
                }
                6 => {
                    if self.anim_num == 2 {
                        self.action_counter2 = 7;
                    } else {
                        self.animate(2, 0, 2);
                    }
                }
                7 => {
                    self.action_counter3 = self.action_counter3.wrapping_add(1);
                    if self.action_counter3 >= 500 {
                        self.action_counter2 = 8;

                        if self.anim_num == 0 {
                            self.action_counter2 = 9;
                            self.npc_flags.set_shootable(true);
                        } else {
                            self.anim_counter += 1;
                            if self.anim_counter > 2 {
                                self.anim_counter = 0;
                                self.anim_num = self.anim_num.saturating_sub(1);
                            }
                        }
                    }
                }
                8 => {
                    if self.anim_num == 0 {
                        self.action_counter2 = 9;
                        self.npc_flags.set_shootable(true);
                        self.anim_num = 0;
                    } else {
                        self.anim_counter += 1;
                        if self.anim_counter > 2 {
                            self.anim_counter = 0;
                            self.anim_num = self.anim_num.saturating_sub(1);
                        }
                    }
                }
                _ => (),
            }
        }

        self.anim_rect = state.constants.npc.n368_gclone[self.anim_num as usize];

        Ok(())
    }

    // Curly clones that spawn during the G-CLONE boss battle
    pub(crate) fn tick_n369_gclone_curly_clone(
        &mut self,
        state: &mut SharedGameState,
        NPCContext { players, npc_list, .. }: NPCContext,
    ) -> GameResult {
        // action_counter3 is used to keep track of grabbed player
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 1 | 10 => {
                if self.action_num == 0 {
                    self.anim_num = 0;
                    self.action_num = 1;
                    self.event_num = 450;
                }

                if self.action_num == 10 || self.flags.hit_bottom_wall() {
                    if self.action_num != 10 {
                        state.sound_manager.play_sfx(23);
                        self.action_counter = 0;
                        self.anim_num = 3;
                        self.action_num = 10;
                    }

                    if self.x - 0x1000 >= player.x || self.x + 0x1000 <= player.x {
                        self.action_counter += 1;

                        if self.action_counter > 32 {
                            self.action_num = 11;
                            self.action_counter = 2;
                            self.anim_num = 0;
                        }
                    }
                }
            }
            11 | 20 => {
                if self.action_num == 11 {
                    self.action_counter += 1;

                    if self.action_counter >= 32 {
                        self.action_num = 20;
                        self.action_counter = self.rng.range(100..150) as u16;
                        self.anim_counter = 0;
                    }
                }

                if self.action_num == 20 || self.action_counter >= 32 {
                    if self.x >= player.x {
                        self.direction = Direction::Left;
                        self.vel_x -= 64;
                    } else {
                        self.direction = Direction::Right;
                        self.vel_x += 64;
                    }

                    self.vel_x = i32::clamp(self.vel_x, -1024, 1024);

                    self.animate(3, 1, 2);

                    if (self.direction == Direction::Left && self.vel_x > 0)
                        || (self.direction == Direction::Right && self.vel_x < 0)
                    {
                        self.anim_counter = 0;
                    }

                    self.action_counter = self.action_counter.saturating_sub(1);

                    if self.action_counter <= 0 {
                        self.anim_num = 3;
                        self.vel_x = 3 * self.vel_x / 4;
                        self.action_num = 30;
                        self.action_counter = 1;
                    }
                }
            }
            14 => {
                self.action_counter += 1;

                if self.action_counter >= 40 {
                    self.action_num = 11;
                    self.action_counter = 2;
                    self.anim_num = 0;
                }
            }
            30 => {
                self.action_counter += 1;

                if self.action_counter >= 12 {
                    if self.rng.range(0..10) % 2 != 0 {
                        self.vel_y = -0x800;
                    } else {
                        self.vel_y = -0x400;
                        self.vel_x = 0x5FF * self.direction.vector_x();
                    }

                    self.anim_num = 4;
                    self.damage = 0;
                    self.action_num = 31;
                    state.sound_manager.play_sfx(30);
                }
            }
            31 => {
                if self.action_counter3 == 0 {
                    if self.flags.hit_left_wall() && self.vel_x < 0 {
                        self.vel_x *= -1;
                        self.direction = Direction::Right;
                    }

                    if self.flags.hit_right_wall() && self.vel_x > 0 {
                        self.vel_x *= -1;
                        self.direction = Direction::Left;
                    }

                    if self.flags.hit_bottom_wall() {
                        self.vel_x = 0;
                        self.damage = 3;
                        self.anim_num = 3;
                        self.action_num = 10;
                    }
                }
            }
            40 | 41 => {
                if self.flags.hit_bottom_wall() {
                    if self.action_num == 40 {
                        self.action_num = 41;
                    }

                    self.animate(3, 7, 8);

                    self.action_counter += 1;
                    if self.action_counter > 100 {
                        state.sound_manager.play_sfx(25);

                        player.x = self.x;
                        player.y = self.y;
                        player.cond.set_hidden(false);
                        player.shock_counter = 0;

                        if self.direction == Direction::Right {
                            player.x -= 0x800;
                            player.vel_x = -0x5FF;
                            player.vel_y = -0x400;
                            player.direction = Direction::Left;
                        } else {
                            player.x += 0x800;
                            player.vel_x = 0x5FF;
                            player.vel_y = -0x400;
                            player.direction = Direction::Right;
                        }

                        self.direction = self.direction.opposite();
                        self.anim_num = 9;
                        self.action_num = 51;
                        self.action_counter = 1;
                        self.action_counter3 = 0;
                    }
                }
            }
            50 => {
                state.sound_manager.play_sfx(25);

                player.x = self.x;
                player.y = self.y;
                player.cond.set_hidden(false);
                player.shock_counter = 0;

                if self.direction == Direction::Right {
                    player.x -= 0x800;
                    player.vel_x = -0x5FF;
                    player.vel_y = -0x400;
                    player.direction = Direction::Left;
                } else {
                    player.x += 0x800;
                    player.vel_x = 0x5FF;
                    player.vel_y = -0x400;
                    player.direction = Direction::Right;
                }

                self.direction = self.direction.opposite();
                self.anim_num = 9;
                self.action_num = 51;
                self.action_counter = 1;
                self.action_counter3 = 0;
            }
            51 => {
                self.action_counter += 1;

                if self.action_counter > 30 {
                    self.action_num = 10;
                }
            }
            200 => {
                self.action_counter2 = 77;
                self.npc_type = 363;
                self.action_num = 0;
                self.damage = 0;
                self.event_num = 451;
            }
            _ => {}
        }

        if self.action_num == 31 && self.action_counter3 == 0 && player.shock_counter == 0 {
            self.action_counter += 1;
            if self.action_counter > 2 {
                if self.x - 0x1000 < player.x
                    && self.x + 0x1000 > player.x
                    && self.y - 0x1000 < player.y
                    && self.y + 0x1000 > player.y
                {
                    self.action_counter3 = 1;
                    self.action_num = 40;
                    self.action_counter = 0;
                    self.anim_num = 8;
                    self.vel_x = 0;
                    player.cond.set_hidden(true);
                    player.shock_counter = 180;
                    // Switch spawns a quote NPC behind this NPC
                    if state.constants.is_switch {
                        let mut npc = NPC::create(150, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.x;
                        npc.y = self.y;
                        npc.direction = player.direction;
                        npc.parent_id = self.id;
                        npc.action_num = 200;

                        let _ = npc_list.spawn(0xAA, npc);
                    }
                }
            }
        }

        self.vel_y += 0x40;
        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 10 };
        self.anim_rect = state.constants.npc.n369_gclone_curly_clone[self.anim_num as usize + dir_offset];

        if self.life <= 900 && !player.cond.hidden() {
            self.action_counter2 = 77;
            self.npc_type = 363;
            self.action_num = 0;
            self.damage = 0;
            self.event_num = 451;
            self.action_counter3 = 0;
        }

        Ok(())
    }
}
