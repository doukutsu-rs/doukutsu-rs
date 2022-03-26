use std::hint::unreachable_unchecked;

use crate::common::{Direction, Rect, SliceExt, CDEG_RAD};
use crate::components::flash::Flash;
use crate::npc::boss::BossNPC;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::stage::Stage;
use crate::{GameResult, SharedGameState};

impl NPC {
    pub(crate) fn tick_n282_mini_undead_core_active(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 20;
            self.target_y = self.y;
            self.vel_y = if self.rng.range(0..100) & 1 == 0 { -0x100 } else { 0x100 };
        }

        if self.action_num == 20 {
            self.vel_x = -0x200;
            if self.x < -0x8000 {
                self.cond.set_alive(false);
            }

            if self.target_y < self.y {
                self.vel_y -= 0x10;
            }
            if self.target_y > self.y {
                self.vel_y += 0x10;
            }

            self.vel_y = self.vel_y.clamp(-0x100, 0x100);

            let player = self.get_closest_player_ref(&players);
            if player.flags.hit_bottom_wall()
                && player.y < self.y - 0x800
                && player.x > self.x - 0x3000
                && player.x < self.x + 0x3000
            {
                self.target_y = 0x12000;
                self.anim_num = 2;
            } else if self.anim_num != 1 {
                self.anim_num = 0;
            }

            if player.flags.hit_left_wall()
                && player.x < self.x - self.hit_bounds.right as i32
                && player.x > self.x - self.hit_bounds.right as i32 - 0x1000
                && player.hit_bounds.bottom as i32 + player.y > self.y - self.hit_bounds.top as i32
                && (player.y - player.hit_bounds.top as i32) < self.hit_bounds.bottom as i32 + self.y
            {
                self.npc_flags.set_solid_hard(false);
                self.anim_num = 1;
            } else if player.flags.hit_right_wall()
                && player.x > self.hit_bounds.right as i32 + self.x
                && player.x < self.x + self.hit_bounds.right as i32 + 0x1000
                && player.hit_bounds.bottom as i32 + player.y > self.y - self.hit_bounds.top as i32
                && (player.y - player.hit_bounds.top as i32) < self.hit_bounds.bottom as i32 + self.y
            {
                self.npc_flags.set_solid_hard(false);
                self.anim_num = 1;
            } else if player.flags.hit_top_wall()
                && player.y < self.y - self.hit_bounds.top as i32
                && player.y > self.y - self.hit_bounds.top as i32 - 0x1000
                && player.hit_bounds.left as i32 + player.x > self.x - self.hit_bounds.right as i32
                && (player.x - player.hit_bounds.right as i32) < self.hit_bounds.left as i32 + self.x
            {
                self.npc_flags.set_solid_hard(false);
                self.anim_num = 1;
            } else if player.flags.hit_bottom_wall()
                && player.y > self.hit_bounds.bottom as i32 + self.y - 0x800
                && player.y < self.y + self.hit_bounds.bottom as i32 + 0x1800
                && player.hit_bounds.left as i32 + player.x > self.x - self.hit_bounds.right as i32 - 0x800
                && (player.x - player.hit_bounds.right as i32) < self.x + self.hit_bounds.left as i32 + 0x800
            {
                self.npc_flags.set_solid_hard(false);
                self.anim_num = 1;
            }
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n282_mini_undead_core_active[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n291_mini_undead_core_inactive(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 20;
            if self.direction == Direction::Right {
                self.npc_flags.set_solid_hard(false);
                self.anim_num = 1;
            }
        }

        self.anim_rect = state.constants.npc.n291_mini_undead_core_inactive[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n293_undead_core_energy_shot(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
        }

        if self.action_num == 1 {
            self.anim_num += 1;
            if self.anim_num > 1 {
                self.anim_num = 0;
            }

            let mut npc = NPC::create(4, &state.npc_table);
            npc.cond.set_alive(true);
            npc.x = self.x + self.rng.range(0..16) * 0x200;
            npc.y = self.y + self.rng.range(-16..16) * 0x200;

            let _ = npc_list.spawn(0x100, npc);

            self.x -= 0x1000;
            if self.x < -0x4000 {
                self.cond.set_alive(false);
            }
        }

        self.anim_rect = state.constants.npc.n293_undead_core_energy_shot[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n285_undead_core_spiral_projectile(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
        stage: &mut Stage,
    ) -> GameResult {
        if self.x < 0 || self.x > stage.map.width as i32 * state.tile_size.as_int() * 0x200 {
            self.vanish(state);
            return Ok(());
        }

        if self.action_num == 0 {
            self.action_num = 1;
            self.target_x = self.x;
            self.target_y = self.y;
            self.action_counter2 = self.tsc_direction / 8;
            self.tsc_direction &= 7;
        }

        if self.action_num == 1 {
            self.action_counter2 += 24;
            self.action_counter2 &= 0xFF;

            if self.action_counter < 128 {
                self.action_counter += 1;
            }

            self.vel_x += self.direction.vector_x() * 0x15;
            self.target_x += self.vel_x;

            self.x = self.target_x + 4 * ((self.action_counter2 as f64).cos() * -512.0) as i32;
            self.y = self.target_y + 6 * ((self.action_counter2 as f64).sin() * -512.0) as i32;

            let mut npc = NPC::create(286, &state.npc_table);
            npc.cond.set_alive(true);
            npc.x = self.x;
            npc.y = self.y;

            let _ = npc_list.spawn(0x100, npc);
        }

        self.anim_rect = state.constants.npc.n285_undead_core_spiral_projectile;

        Ok(())
    }

    pub(crate) fn tick_n286_undead_core_spiral_projectile_trail(&mut self, state: &mut SharedGameState) -> GameResult {
        self.anim_counter += 1;
        if self.anim_counter > 0 {
            self.anim_counter = 0;
            self.anim_num += 1;
        }

        if self.anim_num < 3 {
            self.anim_rect = state.constants.npc.n286_undead_core_spiral_projectile_trail[self.anim_num as usize];
        } else {
            self.cond.set_alive(false);
        }

        Ok(())
    }

    pub(crate) fn tick_n287_orange_smoke(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.vel_x = self.rng.range(-4..4) * 0x200;
            self.action_num = 1;
        } else if self.action_num == 1 {
            self.vel_x = 20 * self.vel_x / 21;
            self.vel_y = 20 * self.vel_y / 21;
            self.x += self.vel_x;
            self.y += self.vel_y;
        }

        self.anim_counter += 1;
        if self.anim_counter > 1 {
            self.anim_counter = 0;
            self.anim_num += 1;
        }

        if self.anim_num < 7 {
            self.anim_rect = state.constants.npc.n287_orange_smoke[self.anim_num as usize];
        } else {
            self.cond.set_alive(false);
        }

        Ok(())
    }

    pub(crate) fn tick_n288_undead_core_exploding_rock(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        stage: &mut Stage,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.vel_x = -0x200;
                }

                match self.direction {
                    Direction::Up => {
                        self.vel_y -= 0x20;
                        if self.vel_y < -0x5FF {
                            self.vel_y = -0x5FF;
                        }

                        if self.flags.hit_top_wall() {
                            self.action_num = 2;
                        }
                    }
                    Direction::Bottom => {
                        self.vel_y += 0x20;
                        self.clamp_fall_speed();

                        if self.flags.hit_bottom_wall() {
                            self.action_num = 2;
                        }
                    }
                    _ => (),
                }

                self.animate(3, 0, 1);
            }
            2 | 3 => {
                if self.action_num == 2 {
                    let player = self.get_closest_player_ref(&players);

                    self.action_num = 3;
                    self.action_counter = 0;
                    self.npc_flags.set_ignore_solidity(true);
                    self.vel_x = if self.x > player.x { -0x400 } else { 0x400 };
                    self.vel_y = 0;

                    self.display_bounds = Rect::new(0x1800, 0x1800, 0x1800, 0x1800);
                    state.sound_manager.play_sfx(44);
                }

                self.anim_num += 1;
                if self.anim_num > 4 {
                    self.anim_num = 2;
                }

                self.action_counter += 1;
                if self.action_counter & 3 == 1 {
                    let mut npc = NPC::create(287, &state.npc_table);
                    npc.cond.set_alive(true);

                    npc.x = self.x;
                    npc.y = self.y;
                    npc.vel_y = self.direction.vector_y() * 0x400;

                    let _ = npc_list.spawn(0x100, npc);
                }

                if self.x < 0x2000 || self.x > (stage.map.width as i32 + 1) * state.tile_size.as_int() * 0x200 {
                    self.cond.set_alive(false);
                }
            }
            _ => (),
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n288_undead_core_exploding_rock[self.anim_num as usize];

        Ok(())
    }
}

impl BossNPC {
    pub(crate) fn tick_b07_undead_core(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
        stage: &mut Stage,
        flash: &mut Flash,
    ) {
        let mut v19 = false;

        match self.parts[0].action_num {
            1 => {
                self.parts[0].action_num = 10;
                self.parts[0].exp = 1;
                self.parts[0].cond.set_alive(true);
                self.parts[0].npc_flags.set_ignore_solidity(true);
                self.parts[0].npc_flags.set_invulnerable(true);
                self.parts[0].npc_flags.set_show_damage(true);
                self.parts[0].life = 700;
                self.hurt_sound[0] = 114;
                self.parts[0].x = 0x4a000;
                self.parts[0].y = 0xf000;
                self.parts[0].vel_x = 0;
                self.parts[0].vel_y = 0;
                self.parts[0].event_num = 1000;
                self.parts[0].npc_flags.set_event_when_killed(true);

                self.parts[3].cond.set_alive(true);
                self.parts[3].action_num = 0;

                self.parts[4].cond.set_alive(true);
                self.parts[4].action_num = 10;

                self.parts[5].cond.set_alive(true);
                self.parts[5].action_num = 10;

                self.parts[8].cond.set_alive(true);
                self.parts[8].npc_flags.set_ignore_solidity(true);
                self.parts[8].display_bounds.left = 0;
                self.parts[8].display_bounds.top = 0;
                self.parts[8].hit_bounds.right = 0x5000;
                self.parts[8].hit_bounds.top = 0x2000;
                self.parts[8].hit_bounds.bottom = 0x2000;
                self.parts[8].action_counter2 = 0;

                self.parts[9] = self.parts[8].clone();
                self.parts[9].hit_bounds.right = 0x4800;
                self.parts[9].hit_bounds.top = 0x3000;
                self.parts[9].hit_bounds.bottom = 0x3000;
                self.parts[9].action_counter2 = 1;

                self.parts[10] = self.parts[8].clone();
                self.parts[10].hit_bounds.right = 0x5800;
                self.parts[10].hit_bounds.top = 0x1000;
                self.parts[10].hit_bounds.bottom = 0x1000;
                self.parts[10].action_counter2 = 2;

                self.parts[11] = self.parts[8].clone();
                self.parts[11].cond.set_damage_boss(true);
                self.parts[11].hit_bounds.right = 0x2800;
                self.parts[11].hit_bounds.top = 0x2800;
                self.parts[11].hit_bounds.bottom = 0x2800;
                self.parts[11].action_counter2 = 3;

                self.parts[1].cond.set_alive(true);
                self.parts[1].action_num = 0;
                self.parts[1].npc_flags.set_shootable(true);
                self.parts[1].npc_flags.set_ignore_solidity(true);
                self.parts[1].life = 1000;
                self.hurt_sound[1] = 54;
                self.parts[1].hit_bounds.right = 0x3000;
                self.parts[1].hit_bounds.top = 0x2000;
                self.parts[1].hit_bounds.bottom = 0x2000;
                self.parts[1].display_bounds.left = 0x4000;
                self.parts[1].display_bounds.top = 0x2800;
                self.parts[1].parent_id = 0;

                self.parts[2] = self.parts[1].clone();
                self.parts[2].action_counter3 = 128;

                self.parts[6] = self.parts[1].clone();
                self.parts[6].action_counter2 = 1;

                self.parts[7] = self.parts[1].clone();
                self.parts[7].action_counter2 = 1;
                self.parts[7].action_counter3 = 128;

                self.parts[19].action_counter = self.parts[0].life;

                for i in 0u16..20 {
                    self.parts[i as usize].id = i;
                }
            }
            15 => {
                self.parts[0].action_num = 16;
                self.parts[0].direction = Direction::Left;
                self.parts[3].action_num = 10;
                self.parts[4].anim_num = 0;
                v19 = true;
            }
            20 => {
                self.parts[0].action_num = 210;
                self.parts[0].direction = Direction::Left;
                self.parts[1].action_num = 5;
                self.parts[2].action_num = 5;
                self.parts[6].action_num = 5;
                self.parts[7].action_num = 5;
                v19 = true;
            }
            200 | 201 => {
                if self.parts[0].action_num == 200 {
                    self.parts[0].action_num = 201;
                    self.parts[0].action_counter = 0;
                    self.parts[3].action_num = 0;
                    self.parts[4].anim_num = 2;
                    self.parts[5].anim_num = 0;
                    self.parts[8].npc_flags.set_invulnerable(false);
                    self.parts[9].npc_flags.set_invulnerable(false);
                    self.parts[10].npc_flags.set_invulnerable(false);
                    self.parts[11].npc_flags.set_shootable(false);

                    state.npc_super_pos.1 = 0;
                    state.sound_manager.stop_sfx(40);
                    state.sound_manager.stop_sfx(41);
                    state.sound_manager.stop_sfx(58);

                    v19 = true;
                }

                self.parts[0].action_counter += 1;
                if (self.parts[0].direction == Direction::Right
                    || self.parts[0].anim_num > 0
                    || self.parts[0].life < 200)
                    && self.parts[0].action_counter > 200
                {
                    self.parts[0].action_counter2 += 1;
                    state.sound_manager.play_sfx(115);

                    if self.parts[0].life >= 200 {
                        self.parts[0].action_num = if self.parts[0].action_counter2 <= 2 { 210 } else { 220 };
                    } else {
                        self.parts[0].action_num = 230;
                    }
                }
            }
            210 | 211 => {
                if self.parts[0].action_num == 210 {
                    self.parts[0].action_num = 211;
                    self.parts[0].action_counter = 0;
                    self.parts[3].action_num = 10;
                    self.parts[8].npc_flags.set_invulnerable(true);
                    self.parts[9].npc_flags.set_invulnerable(true);
                    self.parts[10].npc_flags.set_invulnerable(true);
                    self.parts[11].npc_flags.set_shootable(true);
                    self.parts[19].action_counter = self.parts[0].life;
                    v19 = true;
                }

                self.parts[19].action_counter2 += 1;
                if self.parts[0].shock != 0 && (self.parts[19].action_counter2 & 2) != 0 {
                    self.parts[4].anim_num = 1;
                    self.parts[5].anim_num = 1;
                } else {
                    self.parts[4].anim_num = 0;
                    self.parts[5].anim_num = 0;
                }

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter % 100 == 1 {
                    state.npc_curly_counter = self.parts[0].rng.range(80..100) as u16;
                    state.npc_curly_target = (self.parts[11].x, self.parts[11].y);
                }

                if self.parts[0].action_counter < 300 {
                    if self.parts[0].action_counter % 120 == 1 {
                        let mut npc = NPC::create(288, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.parts[0].x - 0x4000;
                        npc.y = self.parts[0].y - 0x2000;
                        npc.direction = Direction::Up;

                        let _ = npc_list.spawn(0x20, npc);
                    }

                    if self.parts[0].action_counter % 120 == 61 {
                        let mut npc = NPC::create(288, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.parts[0].x - 0x4000;
                        npc.y = self.parts[0].y + 0x2000;
                        npc.direction = Direction::Bottom;

                        let _ = npc_list.spawn(0x20, npc);
                    }
                }

                if self.parts[0].life + 50 < self.parts[19].action_counter || self.parts[0].action_counter > 400 {
                    self.parts[0].action_num = 200;
                }
            }
            220 | 221 => {
                if self.parts[0].action_num == 220 {
                    state.npc_super_pos.1 = 1;

                    self.parts[0].action_num = 221;
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_counter2 = 0;
                    self.parts[3].action_num = 20;
                    self.parts[8].npc_flags.set_invulnerable(true);
                    self.parts[9].npc_flags.set_invulnerable(true);
                    self.parts[10].npc_flags.set_invulnerable(true);
                    self.parts[11].npc_flags.set_shootable(true);
                    self.parts[19].action_counter = self.parts[0].life;

                    state.quake_counter = 100;
                }

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter % 40 == 1 {
                    let (x, y) = match self.parts[0].rng.range(0..3) {
                        0 => (self.parts[1].x, self.parts[1].y),
                        1 => (self.parts[2].x, self.parts[2].y),
                        2 => (self.parts[6].x, self.parts[6].y),
                        3 => (self.parts[7].x, self.parts[7].y),
                        _ => unsafe {
                            unreachable_unchecked();
                        },
                    };

                    state.sound_manager.play_sfx(25);

                    let mut npc = NPC::create(285, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = x - 0x2000;
                    npc.y = y;
                    let _ = npc_list.spawn(0x100, npc.clone());

                    npc.tsc_direction = 1024;
                    let _ = npc_list.spawn(0x100, npc);
                }

                self.parts[19].action_counter2 += 1;

                if self.parts[0].shock != 0 && (self.parts[19].action_counter2 & 2) != 0 {
                    self.parts[4].anim_num = 1;
                    self.parts[5].anim_num = 1;
                } else {
                    self.parts[4].anim_num = 0;
                    self.parts[5].anim_num = 0;
                }

                if self.parts[0].life + 150 < self.parts[0].action_counter
                    || self.parts[0].action_counter > 400
                    || self.parts[0].life < 200
                {
                    self.parts[0].action_num = 200;
                }
            }
            230 | 231 => {
                if self.parts[0].action_num == 230 {
                    self.parts[0].action_num = 231;
                    self.parts[0].action_counter = 0;
                    self.parts[3].action_num = 30;
                    self.parts[8].npc_flags.set_invulnerable(true);
                    self.parts[9].npc_flags.set_invulnerable(true);
                    self.parts[10].npc_flags.set_invulnerable(true);
                    self.parts[11].npc_flags.set_shootable(true);

                    state.sound_manager.play_sfx(25);

                    let mut npc = NPC::create(285, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[3].x - 0x2000;
                    npc.y = self.parts[3].y;

                    let _ = npc_list.spawn(0x100, npc.clone());

                    npc.tsc_direction = 1024;
                    let _ = npc_list.spawn(0x100, npc.clone());

                    npc.tsc_direction = 0;
                    npc.x = self.parts[3].x;
                    npc.y = self.parts[3].y - 0x2000;
                    let _ = npc_list.spawn(0x100, npc.clone());

                    npc.tsc_direction = 1024;
                    let _ = npc_list.spawn(0x100, npc.clone());

                    npc.tsc_direction = 0;
                    npc.x = self.parts[3].x;
                    npc.y = self.parts[3].y + 0x2000;
                    let _ = npc_list.spawn(0x100, npc.clone());

                    npc.tsc_direction = 1024;
                    let _ = npc_list.spawn(0x100, npc);

                    self.parts[19].action_counter = self.parts[0].life;
                    v19 = true;
                }

                self.parts[19].action_counter2 += 1;
                if self.parts[0].shock != 0 && (self.parts[19].action_counter2 & 2) != 0 {
                    self.parts[4].anim_num = 1;
                    self.parts[5].anim_num = 1;
                } else {
                    self.parts[4].anim_num = 0;
                    self.parts[5].anim_num = 0;
                }

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter % 100 == 1 {
                    state.npc_curly_counter = self.parts[0].rng.range(80..100) as u16;
                    state.npc_curly_target = (self.parts[11].x, self.parts[11].y);
                }

                if self.parts[0].action_counter % 120 == 1 {
                    let mut npc = NPC::create(288, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x - 0x4000;
                    npc.y = self.parts[0].y - 0x2000;
                    npc.direction = Direction::Up;

                    let _ = npc_list.spawn(0x20, npc.clone());
                }

                if self.parts[0].action_counter % 120 == 61 {
                    let mut npc = NPC::create(288, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x - 0x4000;
                    npc.y = self.parts[0].y + 0x2000;
                    npc.direction = Direction::Bottom;

                    let _ = npc_list.spawn(0x20, npc.clone());
                }
            }
            500 | 501 => {
                if self.parts[0].action_num == 500 {
                    state.sound_manager.stop_sfx(40);
                    state.sound_manager.stop_sfx(41);
                    state.sound_manager.stop_sfx(58);

                    self.parts[0].action_num = 501;
                    self.parts[0].action_counter = 0;
                    self.parts[0].vel_x = 0;
                    self.parts[0].vel_y = 0;
                    self.parts[3].action_num = 0;
                    self.parts[4].anim_num = 2;
                    self.parts[5].anim_num = 0;
                    self.parts[1].action_num = 5;
                    self.parts[2].action_num = 5;
                    self.parts[6].action_num = 5;
                    self.parts[7].action_num = 5;

                    state.quake_counter = 20;

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);

                    for _ in 0..100 {
                        npc.x = self.parts[0].x + self.parts[0].rng.range(-128..128) * 0x200;
                        npc.y = self.parts[0].y + self.parts[0].rng.range(-64..64) * 0x200;
                        npc.vel_x = self.parts[0].rng.range(-128..128) * 0x200;
                        npc.vel_y = self.parts[0].rng.range(-128..128) * 0x200;

                        let _ = npc_list.spawn(0, npc.clone());
                    }

                    npc_list.kill_npcs_by_type(282, true, state);

                    self.parts[11].npc_flags.set_shootable(false);

                    for i in 0..12 {
                        self.parts[i].npc_flags.set_invulnerable(false);
                    }
                }

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter % 16 != 0 {
                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x + self.parts[0].rng.range(-64..64) * 0x200;
                    npc.y = self.parts[0].y + self.parts[0].rng.range(-32..32) * 0x200;
                    npc.vel_x = self.parts[0].rng.range(-128..128) * 0x200;
                    npc.vel_y = self.parts[0].rng.range(-128..128) * 0x200;

                    let _ = npc_list.spawn(0x100, npc);
                }

                self.parts[0].x += 0x40;
                self.parts[0].y += 0x80;

                if self.parts[0].action_counter > 200 {
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_num = 1000;
                }
            }
            1000 => {
                state.quake_counter = 100;

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter % 8 == 0 {
                    state.sound_manager.play_sfx(44);
                }

                npc_list.create_death_smoke(
                    self.parts[0].x + self.parts[0].rng.range(-72..72) * 0x200,
                    self.parts[0].y + self.parts[0].rng.range(-64..64) * 0x200,
                    1,
                    1,
                    state,
                    &self.parts[0].rng,
                );

                if self.parts[0].action_counter > 100 {
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_num = 1001;

                    state.sound_manager.play_sfx(35);
                    flash.set_cross(self.parts[0].x, self.parts[0].y);
                }
            }
            1001 => {
                state.quake_counter = 40;
                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter > 50 {
                    for i in 0..20 {
                        self.parts[i].cond.set_alive(false);
                    }

                    npc_list.kill_npcs_by_type(158, true, state);
                    npc_list.kill_npcs_by_type(301, true, state);
                }
            }
            _ => (),
        }

        if v19 {
            state.quake_counter = 20;
            state.sound_manager.play_sfx(26);

            if self.parts[0].action_num == 201 {
                self.parts[7].action_num = 10;
                self.parts[6].action_num = 10;
                self.parts[2].action_num = 10;
                self.parts[1].action_num = 10;
            }

            if self.parts[0].action_num == 221 {
                self.parts[7].action_num = 20;
                self.parts[6].action_num = 20;
                self.parts[2].action_num = 20;
                self.parts[1].action_num = 20;
            }

            if self.parts[0].action_num == 231 {
                self.parts[7].action_num = 30;
                self.parts[6].action_num = 30;
                self.parts[2].action_num = 30;
                self.parts[1].action_num = 30;
            }

            let mut npc = NPC::create(4, &state.npc_table);
            npc.cond.set_alive(true);

            for _ in 0..8 {
                npc.x = self.parts[4].x + self.parts[0].rng.range(-32..16) * 0x200;
                npc.y = self.parts[4].y;
                npc.vel_x = self.parts[0].rng.range(-0x200..0x200);
                npc.vel_y = self.parts[0].rng.range(-0x100..0x100);

                let _ = npc_list.spawn(0x100, npc.clone());
            }
        }

        if self.parts[0].action_num >= 200 && self.parts[0].action_num < 300 {
            if self.parts[0].x < 0x18000 {
                self.parts[0].direction = Direction::Right;
            }
            if self.parts[0].x > (stage.map.width as i32 - 4) * state.tile_size.as_int() * 0x200 {
                self.parts[0].direction = Direction::Left;
            }

            self.parts[0].vel_x += self.parts[0].direction.vector_x() * 4;
        }

        if [201, 211, 221, 231].contains(&self.parts[0].action_num) {
            self.parts[0].action_counter3 += 1;
            if self.parts[0].action_counter3 == 150 {
                self.parts[0].action_counter3 = 0;

                let mut npc = NPC::create(282, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = stage.map.width as i32 * state.tile_size.as_int() * 0x200 + 0x40;
                npc.y = (self.parts[0].rng.range(-1..3) + 10) * state.tile_size.as_int() * 0x200;

                let _ = npc_list.spawn(0x30, npc);
            } else if self.parts[0].action_counter3 == 75 {
                let mut npc = NPC::create(282, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = stage.map.width as i32 * state.tile_size.as_int() * 0x200 + 0x40;
                npc.y = (self.parts[0].rng.range(-3..0) + 3) * state.tile_size.as_int() * 0x200;

                let _ = npc_list.spawn(0x30, npc);
            }
        }

        self.parts[0].vel_x = self.parts[0].vel_x.clamp(-0x80, 0x80);
        self.parts[0].vel_y = self.parts[0].vel_y.clamp(-0x80, 0x80);

        self.parts[0].x += self.parts[0].vel_x;
        self.parts[0].y += self.parts[0].vel_y;

        self.tick_b07_undead_core_face(3, state, npc_list);
        self.tick_b07_undead_core_head(4, state);
        self.tick_b07_undead_core_tail(5, state);
        self.tick_b07_undead_core_small_head(1, state, stage);
        self.tick_b07_undead_core_small_head(2, state, stage);
        self.tick_b07_undead_core_small_head(6, state, stage);
        self.tick_b07_undead_core_small_head(7, state, stage);
        self.tick_b07_undead_core_hitbox(8);
        self.tick_b07_undead_core_hitbox(9);
        self.tick_b07_undead_core_hitbox(10);
        self.tick_b07_undead_core_hitbox(11);
    }

    fn tick_b07_undead_core_face(&mut self, i: usize, state: &mut SharedGameState, npc_list: &NPCList) {
        let (head, tail) = self.parts.split_at_mut(i);
        let base = &mut head[0];
        let part = &mut tail[0];

        match part.action_num {
            0 => {
                part.anim_num = 0;
            }
            10 => {
                part.anim_num = 1;
            }
            20 => {
                part.anim_num = 2;
            }
            30 | 31 => {
                if part.action_num == 30 {
                    part.action_num = 31;
                    part.anim_num = 3;
                    part.action_counter = 100;
                }

                part.action_counter += 1;
                if part.action_counter > 300 {
                    part.action_counter = 0;
                }

                if part.action_counter > 250 && part.action_counter % 16 == 1 {
                    state.sound_manager.play_sfx(26);
                }

                if part.action_counter > 250 && part.action_counter % 16 == 7 {
                    state.sound_manager.play_sfx(101);

                    let mut npc = NPC::create(293, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = part.x;
                    npc.y = part.y;

                    let _ = npc_list.spawn(0x80, npc);
                }

                if part.action_counter == 200 {
                    state.sound_manager.play_sfx(116);
                }

                part.anim_num = if part.action_counter > 200 && part.action_counter & 1 != 0 { 4 } else { 3 };
            }
            _ => (),
        }

        part.display_bounds.right = 0x4800;
        part.display_bounds.left = 0x4800;
        part.display_bounds.top = 0x2800;
        part.x = base.x - 0x4800;
        part.y = base.y + 0x800;
        part.npc_flags.set_ignore_solidity(true);

        part.anim_rect = state.constants.npc.b07_undead_core[part.anim_num as usize];
    }

    fn tick_b07_undead_core_head(&mut self, i: usize, state: &mut SharedGameState) {
        let (head, tail) = self.parts.split_at_mut(i);
        let base = &mut head[0];
        let part = &mut tail[0];

        match part.action_num {
            10 | 11 => {
                if part.action_num == 10 {
                    part.action_num = 11;
                    part.anim_num = 2;
                    part.npc_flags.set_ignore_solidity(true);
                    part.display_bounds.left = 0x4800;
                    part.display_bounds.top = 0x7000;
                }

                part.x = base.x - 0x4800;
                part.y = base.y;
            }
            50 | 51 => {
                if part.action_num == 50 {
                    part.action_num = 51;
                    part.action_counter = 112;
                }

                part.action_counter = part.action_counter.saturating_sub(1);
                if part.action_counter == 0 {
                    part.action_num = 100;
                    part.anim_num = 3;
                }
            }
            100 => {
                part.anim_num = 3;
            }
            _ => (),
        }

        part.anim_rect = state.constants.npc.b07_undead_core[part.anim_num as usize + 5];

        if part.action_num == 51 {
            part.anim_rect.bottom = part.action_counter + part.anim_rect.top;
        }
    }

    fn tick_b07_undead_core_tail(&mut self, i: usize, state: &mut SharedGameState) {
        let (head, tail) = self.parts.split_at_mut(i);
        let base = &mut head[0];
        let part = &mut tail[0];

        match part.action_num {
            10 | 11 => {
                if part.action_num == 10 {
                    part.action_num = 11;
                    part.anim_num = 0;
                    part.npc_flags.set_ignore_solidity(true);
                    part.display_bounds.left = 0x5800;
                    part.display_bounds.top = 0x7000;
                }

                part.x = base.x + 0x5800;
                part.y = base.y;
            }
            50 | 51 => {
                if part.action_num == 50 {
                    part.action_num = 51;
                    part.action_counter = 112;
                }

                part.action_counter = part.action_counter.saturating_sub(1);
                if part.action_counter == 0 {
                    part.action_num = 100;
                    part.anim_num = 2;
                }
            }
            100 => {
                part.anim_num = 2;
            }
            _ => (),
        }

        part.anim_rect = state.constants.npc.b07_undead_core[part.anim_num as usize + 9];

        if part.action_num == 51 {
            part.anim_rect.bottom = part.action_counter + part.anim_rect.top;
        }
    }

    fn tick_b07_undead_core_small_head(&mut self, i: usize, state: &mut SharedGameState, stage: &mut Stage) {
        let parent = self.parts[i].parent_id as usize;
        let (base, part) = if let Some(x) = self.parts.get_two_mut(parent, i) {
            x
        } else {
            return;
        };

        if !part.cond.alive() {
            return;
        }

        part.life = 1000;
        match part.action_num {
            0 => {
                part.npc_flags.set_shootable(false);
            }
            5 => {
                part.anim_num = 0;
                part.npc_flags.set_shootable(false);
                part.action_counter3 += 1;
                part.action_counter3 &= 0xff;
            }
            10 => {
                part.anim_num = 0;
                part.npc_flags.set_shootable(false);
                part.action_counter3 += 2;
                part.action_counter3 &= 0xff;
            }
            20 => {
                part.anim_num = 1;
                part.npc_flags.set_shootable(false);
                part.action_counter3 += 2;
                part.action_counter3 &= 0xff;
            }
            30 => {
                part.anim_num = 0;
                part.npc_flags.set_shootable(false);
                part.action_counter3 += 4;
                part.action_counter3 &= 0xff;
            }
            200 | 201 => {
                if part.action_num == 200 {
                    part.action_num = 201;
                    part.anim_num = 2;
                    part.vel_x = 0;
                    part.vel_y = 0;
                }

                part.vel_x += 0x20;
                part.x += part.vel_x;
                if part.x > (stage.map.width as i32) * state.tile_size.as_int() * 0x200 + 0x4000 {
                    part.cond.set_alive(false);
                }
            }
            _ => (),
        }

        if part.action_num < 50 {
            let angle =
                if part.action_counter2 != 0 { part.action_counter3 + 0x80 } else { part.action_counter3 + 0x180 };

            let angle = (angle / 2) as f64 * CDEG_RAD;

            part.x = base.x + 0x30 * (angle.cos() * -512.0) as i32 - 0x1000;
            part.y = base.y + 0x50 * (angle.sin() * -512.0) as i32;
        }

        part.anim_rect = state.constants.npc.b07_undead_core[part.anim_num as usize + 12];
    }

    fn tick_b07_undead_core_hitbox(&mut self, i: usize) {
        let (head, tail) = self.parts.split_at_mut(i);
        let base = &mut head[0];
        let part = &mut tail[0];

        match part.action_counter2 {
            0 => {
                part.x = base.x;
                part.y = base.y - 0x4000;
            }
            1 => {
                part.x = base.x + 0x3800;
                part.y = base.y;
            }
            2 => {
                part.x = base.x + 0x800;
                part.y = base.y + 0x4000;
            }
            3 => {
                part.x = base.x - 0x3800;
                part.y = base.y + 0x800;
            }
            _ => (),
        }
    }
}
