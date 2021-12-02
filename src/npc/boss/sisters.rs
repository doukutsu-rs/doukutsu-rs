use crate::common::{Direction, Rect, SliceExt, CDEG_RAD};
use crate::components::flash::Flash;
use crate::npc::boss::BossNPC;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::rng::RNG;
use crate::SharedGameState;

impl BossNPC {
    pub(crate) fn tick_b06_sisters(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        flash: &mut Flash,
    ) {
        match self.parts[0].action_num {
            0 => {
                self.parts[0].cond.set_alive(true);
                self.parts[0].direction = Direction::Left;
                self.parts[0].action_num = 10;
                self.parts[0].exp = 0;
                self.parts[0].x = 0x14000;
                self.parts[0].y = 0x10000;
                self.parts[0].display_bounds = Rect::new(0x1000, 0x1000, 0x10000, 0x1000);
                self.parts[0].hit_bounds = Rect::new(0x1000, 0x1000, 0x1000, 0x1000);
                self.hurt_sound[0] = 54;
                self.parts[0].npc_flags.set_ignore_solidity(true);
                self.parts[0].npc_flags.set_event_when_killed(true);
                self.parts[0].size = 3;
                self.parts[0].damage = 0;
                self.parts[0].event_num = 1000;
                self.parts[0].life = 500;
                self.parts[0].action_counter3 = self.parts[0].rng.range(700..1200) as u16;
                self.parts[0].target_x = 0xB4;
                self.parts[0].target_y = 0x3D;

                self.parts[2].display_bounds = Rect::new(0x2800, 0x2000, 0x2800, 0x2000);
                self.parts[2].hit_bounds = Rect::new(0x1800, 0x1400, 0x1800, 0x1400);
                self.parts[2].npc_flags.set_ignore_solidity(true);
                self.parts[2].npc_flags.set_invulnerable(true);
                self.parts[2].parent_id = 3;
                self.parts[2].cond.set_alive(true);
                self.parts[2].cond.set_damage_boss(true);
                self.parts[2].damage = 10;

                self.parts[3].cond.set_alive(true);
                self.parts[3].display_bounds = Rect::new(0x2800, 0x2800, 0x2800, 0x2800);
                self.parts[3].hit_bounds = Rect::new(0x1800, 0x400, 0x1800, 0x2000);
                self.parts[3].npc_flags.set_ignore_solidity(true);
                self.parts[3].parent_id = 0;
                self.parts[3].damage = 10;

                self.parts[4] = self.parts[2].clone();
                self.parts[4].id = 4;
                self.parts[4].parent_id = 5;

                self.parts[5] = self.parts[3].clone();
                self.parts[5].id = 5;
                self.parts[5].action_counter2 = 128;
            }
            20 => {
                self.parts[0].target_x -= 1;
                if self.parts[0].target_x <= 112 {
                    self.parts[0].action_num = 100;
                    self.parts[0].action_counter = 0;
                    self.parts[2].action_num = 100;
                    self.parts[4].action_num = 100;
                    self.parts[3].action_num = 100;
                    self.parts[5].action_num = 100;
                }
            }
            100 => {
                let actr2: &mut i16 = unsafe { std::mem::transmute(&mut self.parts[0].action_counter2) };
                let mut b = true;

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter < 100 {
                    *actr2 += 1;
                } else if self.parts[0].action_counter < 120 {
                    *actr2 += 2;
                } else if self.parts[0].action_counter < self.parts[0].action_counter3 {
                    *actr2 += 4;
                } else if self.parts[0].action_counter < self.parts[0].action_counter3 + 40 {
                    *actr2 += 2;
                } else if self.parts[0].action_counter < self.parts[0].action_counter3 + 60 {
                    *actr2 += 1;
                } else {
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_num = 110;
                    self.parts[0].action_counter3 = self.parts[0].rng.range(400..700) as u16;
                    b = false;
                }

                if b && *actr2 >= 0x400 {
                    *actr2 -= 0x400;
                }
            }
            110 => {
                let actr2: &mut i16 = unsafe { std::mem::transmute(&mut self.parts[0].action_counter2) };
                let mut b = true;

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter < 20 {
                    *actr2 -= 1;
                } else if self.parts[0].action_counter < 60 {
                    *actr2 -= 2;
                } else if self.parts[0].action_counter < self.parts[0].action_counter2 {
                    *actr2 -= 4;
                } else if self.parts[0].action_counter < self.parts[0].action_counter2 + 40 {
                    *actr2 -= 2;
                } else if self.parts[0].action_counter < self.parts[0].action_counter2 + 60 {
                    *actr2 -= 1;
                } else {
                    if self.parts[0].life >= 300 {
                        self.parts[0].action_counter = 0;
                        self.parts[0].action_num = 100;
                        *actr2 = self.parts[0].rng.range(400..700) as i16;
                    } else {
                        self.parts[0].action_counter = 0;
                        self.parts[0].action_num = 400;
                        self.parts[2].action_num = 400;
                        self.parts[4].action_num = 400;
                        b = false;
                    }
                }

                if b && *actr2 <= 0 {
                    *actr2 += 0x400;
                }
            }
            400 => {
                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter > 100 {
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_num = 401;
                }
            }
            401 => {
                let actr2: &mut i16 = unsafe { std::mem::transmute(&mut self.parts[0].action_counter2) };
                let mut b = true;

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter < 100 {
                    *actr2 += 1;
                } else if self.parts[0].action_counter < 120 {
                    *actr2 += 2;
                } else if self.parts[0].action_counter < 500 {
                    *actr2 += 2;
                } else if self.parts[0].action_counter < 540 {
                    *actr2 += 4;
                } else if self.parts[0].action_counter < 560 {
                    *actr2 += 2;
                } else {
                    self.parts[0].action_num = 100;
                    self.parts[0].action_counter = 0;
                    self.parts[2].action_num = 100;
                    self.parts[4].action_num = 100;
                    b = false;
                }

                if b && *actr2 >= 0x400 {
                    *actr2 -= 0x400;
                }
            }
            1000 | 1001 => {
                if self.parts[0].action_num == 1000 {
                    self.parts[0].action_num = 1001;
                    self.parts[0].action_counter = 0;
                    self.parts[2].action_num = 1000;
                    self.parts[3].action_num = 1000;
                    self.parts[4].action_num = 1000;
                    self.parts[5].action_num = 1000;

                    npc_list.create_death_smoke(
                        self.parts[0].x,
                        self.parts[0].y,
                        self.parts[0].display_bounds.right as usize,
                        40,
                        state,
                        &self.parts[0].rng,
                    );
                }

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter > 100 {
                    self.parts[0].action_num = 1010;
                }

                let mut npc = NPC::create(4, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.parts[0].x + self.parts[0].rng.range(-128..128) * 0x200;
                npc.y = self.parts[0].y + self.parts[0].rng.range(-70..70) * 0x200;

                let _ = npc_list.spawn(0x100, npc);
            }
            1010 => {
                let actr2: &mut i16 = unsafe { std::mem::transmute(&mut self.parts[0].action_counter2) };
                *actr2 += 4;

                if *actr2 >= 0x400 {
                    *actr2 -= 0x400;
                }

                if self.parts[0].target_x > 8 {
                    self.parts[0].target_x -= 1;
                }
                if self.parts[0].target_y > 0 {
                    self.parts[0].target_y -= 1;
                }
                if self.parts[0].target_x < -8 {
                    self.parts[0].target_x += 1;
                }
                if self.parts[0].target_y < 0 {
                    self.parts[0].target_y += 1;
                }

                if self.parts[0].target_y == 0 {
                    self.parts[0].action_num = 1020;
                    self.parts[0].action_counter = 0;

                    flash.set_cross(self.parts[0].x, self.parts[0].y);
                    state.sound_manager.play_sfx(35);
                }
            }
            1020 => {
                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter > 50 {
                    npc_list.kill_npcs_by_type(211, true, state);

                    self.parts[0].cond.set_alive(false);
                    self.parts[1].cond.set_alive(false);
                    self.parts[2].cond.set_alive(false);
                    self.parts[3].cond.set_alive(false);
                    self.parts[4].cond.set_alive(false);
                    self.parts[5].cond.set_alive(false);

                    self.parts[0].action_num = 0;
                }
            }
            _ => (),
        }

        self.tick_b06_sisters_dragon_head(2, state, &players, npc_list);
        self.tick_b06_sisters_dragon_body(3, state, &players);
        self.tick_b06_sisters_dragon_head(4, state, &players, npc_list);
        self.tick_b06_sisters_dragon_body(5, state, &players);

        self.parts[0].anim_rect = Rect::new(0, 0, 0, 0);
    }

    fn tick_b06_sisters_dragon_head(
        &mut self,
        i: usize,
        state: &mut SharedGameState,
        players: &[&mut Player; 2],
        npc_list: &NPCList,
    ) {
        let parent = self.parts[i].parent_id as usize;
        let (base, part) = if let Some(x) = self.parts.get_two_mut(parent, i) {
            x
        } else {
            return;
        };

        match part.action_num {
            0 => {
                part.action_num = 1;
            }
            100 | 200 | 201 => {
                if part.action_num == 100 {
                    part.action_num = 200;
                }

                if part.action_num == 200 {
                    part.action_num = 201;
                    part.anim_num = 0;
                    part.hit_bounds.left = 0x2000;
                    part.npc_flags.set_shootable(false);
                    part.action_counter2 = part.rng.range(100..200) as u16;
                }

                if part.action_counter2 > 0 {
                    part.action_counter2 -= 1;
                } else {
                    part.action_num = 210;
                    part.action_counter = 0;
                    part.action_counter3 = 0;
                }
            }
            210 => {
                part.action_counter += 1;
                if part.action_counter == 3 {
                    part.anim_num = 1;
                }

                if part.action_counter == 6 {
                    part.anim_num = 2;
                    part.hit_bounds.left = 0x1000;
                    part.npc_flags.set_shootable(true);
                    part.action_counter3 = 0;
                }

                if part.action_counter > 150 {
                    part.action_num = 220;
                    part.action_counter = 0;
                }

                if part.shock != 0 {
                    part.action_counter3 += 1;
                }

                if part.action_counter3 > 10 {
                    part.action_num = 300;
                    part.action_counter = 0;
                    part.anim_num = 3;
                    part.hit_bounds.left = 0x2000;

                    state.sound_manager.play_sfx(51);
                    npc_list.remove_by_type(211, state);
                }
            }
            220 => {
                part.action_counter += 1;
                if part.action_counter % 8 == 1 {
                    let mut npc = NPC::create(202, &state.npc_table);
                    npc.cond.set_alive(true);

                    npc.x = part.x + 0x1000 * part.direction.vector_x();
                    npc.y = part.y;

                    let player = part.get_closest_player_ref(players);
                    let angle = f64::atan2((player.y - npc.y) as f64, (player.x - npc.x) as f64)
                        + (part.rng.range(-6..6) as f64 * CDEG_RAD);

                    npc.vel_x = (angle.cos() * 512.0) as i32;
                    npc.vel_y = (angle.sin() * 512.0) as i32;

                    let _ = npc_list.spawn(0x100, npc);

                    state.sound_manager.play_sfx(33);
                }

                if part.action_counter > 50 {
                    part.action_num = 200;
                }
            }
            300 => {
                part.action_counter += 1;
                if part.action_counter > 100 {
                    part.action_num = 200;
                }
            }
            400 | 401 => {
                if part.action_num == 400 {
                    part.action_num = 401;
                    part.action_counter = 0;
                    part.anim_num = 0;
                    part.hit_bounds.left = 0x2000;
                    part.npc_flags.set_shootable(false);
                }

                part.action_counter += 1;
                if part.action_counter == 3 {
                    part.anim_num = 1;
                }

                if part.action_counter == 6 {
                    part.anim_num = 2;
                    part.hit_bounds.left = 0x1000;
                    part.npc_flags.set_shootable(true);
                    part.action_counter3 = 0;
                }

                if part.action_counter > 20 && part.action_counter % 32 == 1 {
                    let mut npc = NPC::create(202, &state.npc_table);
                    npc.cond.set_alive(true);

                    let player = part.get_closest_player_ref(players);
                    let angle = f64::atan2((player.y - npc.y) as f64, (player.x - npc.x) as f64)
                        + (part.rng.range(-6..6) as f64 * CDEG_RAD);

                    npc.x = part.x + 0x1000 * part.direction.vector_x();
                    npc.y = part.y;

                    npc.vel_x = (angle.cos() * 512.0) as i32;
                    npc.vel_y = (angle.sin() * 512.0) as i32;

                    let _ = npc_list.spawn(0x100, npc);

                    state.sound_manager.play_sfx(33);
                }
            }
            _ => (),
        }

        part.direction = base.direction;
        part.x = base.x + 0x800 * part.direction.vector_x();
        part.y = base.y - 0x1000;

        let dir_offset = if part.direction == Direction::Left { 0 } else { 4 };

        part.anim_rect = state.constants.npc.b06_sisters[part.anim_num as usize + dir_offset];
    }

    fn tick_b06_sisters_dragon_body(&mut self, i: usize, state: &mut SharedGameState, players: &[&mut Player; 2]) {
        let parent = self.parts[i].parent_id as usize;
        let (base, part) = if let Some(x) = self.parts.get_two_mut(parent, i) {
            x
        } else {
            return;
        };

        match part.action_num {
            0 | 10 => {
                if part.action_num == 0 {
                    part.action_num = 10;
                    let angle =
                        ((part.action_counter2) as u8).wrapping_add((base.action_counter2 / 4) as u8) as f64 * CDEG_RAD;
                    part.x += base.x + base.target_x as i32 * (angle.cos() * -512.0) as i32;
                    part.y += base.y + base.target_y as i32 * (angle.sin() * -512.0) as i32;
                }

                let player = part.get_closest_player_ref(players);
                part.direction = if part.x > player.x { Direction::Left } else { Direction::Right };
            }
            100 => {
                let angle =
                    ((part.action_counter2) as u8).wrapping_add((base.action_counter2 / 4) as u8) as f64 * CDEG_RAD;
                part.target_x = base.x + base.target_x as i32 * (angle.cos() * -512.0) as i32;
                part.target_y = base.y + base.target_y as i32 * (angle.sin() * -512.0) as i32;

                part.x += (part.target_x - part.x) / 8;
                part.y += (part.target_y - part.y) / 8;

                let player = part.get_closest_player_ref(players);
                part.direction = if part.x > player.x { Direction::Left } else { Direction::Right };
            }
            1000 | 1001 => {
                if part.action_num == 1000 {
                    part.action_num = 1001;
                    part.npc_flags.set_shootable(false);
                }

                let angle =
                    ((part.action_counter2) as u8).wrapping_add((base.action_counter2 / 4) as u8) as f64 * CDEG_RAD;
                part.target_x = base.x + base.target_x as i32 * (angle.cos() * -512.0) as i32;
                part.target_y = base.y + base.target_y as i32 * (angle.sin() * -512.0) as i32;

                part.x += (part.target_x - part.x) / 8;
                part.y += (part.target_y - part.y) / 8;

                part.direction = if part.x > base.x { Direction::Left } else { Direction::Right };
            }
            _ => (),
        }

        part.animate(2, 0, 2);

        let dir_offset = if part.direction == Direction::Left { 0 } else { 3 };
        part.anim_rect = state.constants.npc.b06_sisters[part.anim_num as usize + dir_offset + 8];
    }
}
