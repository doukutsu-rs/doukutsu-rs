use std::cell::RefCell;
use std::collections::BTreeMap;

use ggez::GameResult;

use crate::bullet::BulletManager;
use crate::caret::CaretType;
use crate::common::{Direction, Rect};
use crate::npc::{NPC, NPCMap};
use crate::npc::boss::BossNPC;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n048_omega_projectiles(&mut self, state: &mut SharedGameState) -> GameResult {
        if (self.flags.hit_left_wall() && self.vel_x < 0)
            || (self.flags.hit_right_wall() && self.vel_x > 0) {
            self.vel_x = -self.vel_x;
        } else if self.flags.hit_bottom_wall() {
            self.action_counter2 += 1;
            if self.action_counter2 <= 2 && self.direction != Direction::Right {
                self.vel_y = -0x100;
            } else {
                self.cond.set_drs_destroyed(true);
                state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
            }
        }

        if self.direction == Direction::Right {
            self.npc_flags.set_shootable(false);
            self.npc_flags.set_invulnerable(true);
        }

        self.vel_y += 0x5;
        self.x += self.vel_x;
        self.y += self.vel_y;

        self.action_counter += 1;
        if self.action_counter > 750 {
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
            self.cond.set_alive(false);
        }

        self.animate(2, 0, 1);

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };

        self.anim_rect = state.constants.npc.n048_omega_projectiles[self.anim_num as usize + dir_offset];

        Ok(())
    }
}

impl BossNPC {
    pub(crate) fn tick_b01_omega(&mut self, state: &mut SharedGameState, players: [&mut Player; 2], npc_map: &BTreeMap<u16, RefCell<NPC>>, bullet_manager: &BulletManager) {
        match self.parts[0].action_num {
            0 => {
                self.parts[0].cond.set_alive(true);
                self.parts[0].size = 3;
                self.parts[0].exp = 1;
                self.parts[0].event_num = 210;
                self.parts[0].life = 400;
                self.parts[0].npc_flags.set_show_damage(true);
                self.parts[0].npc_flags.set_event_when_killed(true);
                self.parts[0].npc_flags.set_ignore_solidity(true);
                self.parts[0].x = 219 * 16 * 0x200;
                self.parts[0].y = 16 * 16 * 0x200;
                self.parts[0].target_x = self.parts[0].x;
                self.parts[0].target_y = self.parts[0].y;
                self.parts[0].display_bounds = Rect {
                    left: 40 * 0x200,
                    top: 40 * 0x200,
                    right: 40 * 0x200,
                    bottom: 16 * 0x200,
                };
                self.parts[0].hit_bounds = Rect {
                    left: 8 * 0x200,
                    top: 24 * 0x200,
                    right: 8 * 0x200,
                    bottom: 16 * 0x200,
                };

                self.parts[1].cond.set_alive(true);
                self.parts[1].display_bounds = Rect {
                    left: 12 * 0x200,
                    top: 8 * 0x200,
                    right: 12 * 0x200,
                    bottom: 8 * 0x200,
                };
                self.parts[1].npc_flags.set_ignore_solidity(true);
                self.parts[1].direction = Direction::Left;

                self.parts[2].cond.set_alive(true);
                self.parts[2].display_bounds = self.parts[1].display_bounds;
                self.parts[2].npc_flags = self.parts[1].npc_flags;
                self.parts[2].direction = Direction::Right;

                self.parts[3].cond.set_alive(true);
                self.parts[3].npc_flags.set_ignore_solidity(true);
                self.parts[3].direction = Direction::Left;
                self.parts[3].x = self.parts[0].x + 16 * 0x200;
                self.parts[3].y = self.parts[0].y;
                self.parts[3].display_bounds = Rect {
                    left: 24 * 0x200,
                    top: 16 * 0x200,
                    right: 16 * 0x200,
                    bottom: 16 * 0x200,
                };
                self.parts[3].hit_bounds = Rect {
                    left: 8 * 0x200,
                    top: 8 * 0x200,
                    right: 8 * 0x200,
                    bottom: 8 * 0x200,
                };
                self.hurt_sound[3] = 52;


                self.parts[4].cond.set_alive(true);
                self.parts[4].display_bounds = self.parts[3].display_bounds;
                self.parts[4].hit_bounds = self.parts[3].hit_bounds;
                self.parts[4].npc_flags = self.parts[3].npc_flags;
                self.parts[4].y = self.parts[3].y;
                self.parts[4].direction = Direction::Right;
                self.hurt_sound[4] = 52;

                self.parts[5].cond.set_alive(true);
            }
            20 | 30 => {
                if self.parts[0].action_num == 20 {
                    self.parts[0].action_num = 30;
                    self.parts[0].action_counter = 0;
                    self.parts[0].anim_num = 0;
                }

                self.parts[0].y -= 0x200;
                self.parts[0].action_counter += 1;
                state.quake_counter = 2;

                if self.parts[0].action_counter % 4 == 0 {
                    state.sound_manager.play_sfx(26);
                }

                if self.parts[0].action_counter == 48 {
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_num = 40;

                    if self.parts[0].life <= 280 {
                        self.parts[0].action_num = 110;
                        self.parts[0].npc_flags.set_shootable(true);
                        self.parts[0].npc_flags.set_ignore_solidity(false);
                        self.parts[3].npc_flags.set_ignore_solidity(false);
                        self.parts[4].npc_flags.set_ignore_solidity(false);
                        self.parts[3].action_num = 3;
                        self.parts[4].action_num = 3;
                        self.parts[5].hit_bounds.top = 16 * 0x200;
                    }
                }
            }
            40 => {
                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter == 48 {
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_num = 50;
                    self.parts[0].anim_counter = 0;
                    self.parts[5].hit_bounds.top = 16 * 0x200;

                    state.sound_manager.play_sfx(102);
                }
            }
            50 => {
                self.parts[0].anim_counter += 1;
                if self.parts[0].anim_counter > 2 {
                    self.parts[0].anim_counter = 0;
                    self.parts[0].anim_num += 1;

                    if self.parts[0].anim_num == 3 {
                        self.parts[0].action_num = 60;
                        self.parts[0].action_counter = 0;
                        self.parts[0].npc_flags.set_shootable(true);
                        self.parts[0].hit_bounds.left = 16 * 0x200;
                        self.parts[0].hit_bounds.right = 16 * 0x200;
                    }
                }
            }
            60 => {
                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter % 3 == 0 && (20..80).contains(&self.parts[0].action_counter) {
                    let mut npc = NPCMap::create_npc(48, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x;
                    npc.y = self.parts[0].y - 16 * 0x200;
                    npc.vel_x = self.parts[0].rng.range(-0x100..0x100) as isize;
                    npc.vel_y = -0x333;
                    npc.direction = if self.parts[0].rng.range(0..9) <= 7 {
                        Direction::Left
                    } else {
                        Direction::Right
                    };

                    state.new_npcs.push(npc);
                    state.sound_manager.play_sfx(39);
                }

                if self.parts[0].action_counter == 200 || bullet_manager.count_bullets_type_idx_all(6) > 0 {
                    self.parts[0].action_num = 70;
                    self.parts[0].anim_counter = 0;

                    state.sound_manager.play_sfx(102);
                }
            }
            70 => {
                self.parts[0].anim_counter += 1;
                if self.parts[0].anim_counter > 2 {
                    self.parts[0].anim_counter = 0;
                    self.parts[0].anim_num -= 1;

                    match self.parts[0].anim_num {
                        1 => self.parts[0].damage = 20,
                        0 => {
                            self.parts[0].action_num = 80;
                            self.parts[0].action_counter = 0;
                            self.parts[0].npc_flags.set_shootable(false);
                            self.parts[0].hit_bounds.left = 24 * 0x200;
                            self.parts[0].hit_bounds.right = 24 * 0x200;
                            self.parts[0].damage = 0;
                            self.parts[5].hit_bounds.top = 36 * 0x200;
                        }
                        _ => {}
                    }
                }
            }
            80 => {
                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter == 48 {
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_num = 90;
                }
            }
            90 => {
                state.quake_counter = 2;
                self.parts[0].y += 0x200;
                self.parts[0].action_counter += 1;

                if self.parts[0].action_counter % 4 == 0 {
                    state.sound_manager.play_sfx(26);
                }

                if self.parts[0].action_counter == 48 {
                    self.parts[0].action_num = 100;
                    self.parts[0].action_counter = 0;
                }
            }
            100 => {
                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter == 120 {
                    self.parts[0].action_num = 30;
                    self.parts[0].action_counter = 0;
                    self.parts[0].x = self.parts[0].target_x + self.parts[0].rng.range(-0x40..0x40) as isize * 0x200;
                    self.parts[0].y = self.parts[0].target_y;
                }
            }
            110 => {
                self.parts[0].anim_counter += 1;
                if self.parts[0].anim_counter > 2 {
                    self.parts[0].anim_counter = 0;
                    self.parts[0].anim_num += 1;
                    if self.parts[0].anim_num == 3 {
                        self.parts[0].action_num = 120;
                        self.parts[0].action_counter = 0;
                        self.parts[0].hit_bounds.left = 16 * 0x200;
                        self.parts[0].hit_bounds.right = 16 * 0x200;
                    }
                }
            }
            120 => {
                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter == 50 || bullet_manager.count_bullets_type_idx_all(6) > 0 {
                    self.parts[0].action_num = 130;
                    self.parts[0].action_counter = 0;
                    self.parts[0].anim_counter = 0;

                    state.sound_manager.play_sfx(102);
                }

                if self.parts[0].action_counter <= 29 && self.parts[0].action_counter % 5 == 0 {
                    let mut npc = NPCMap::create_npc(48, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.parts[0].x;
                    npc.y = self.parts[0].y - 16 * 0x200;
                    npc.vel_x = self.parts[0].rng.range(-0x155..0x155) as isize;
                    npc.vel_y = -0x333;
                    npc.direction = Direction::Left;

                    state.new_npcs.push(npc);
                    state.sound_manager.play_sfx(39);
                }
            }
            130 => {
                self.parts[0].anim_counter += 1;
                if self.parts[0].anim_counter > 2 {
                    self.parts[0].anim_counter = 0;
                    self.parts[0].anim_num -= 1;

                    match self.parts[0].anim_num {
                        1 => self.parts[0].damage = 20,
                        0 => {
                            self.parts[0].action_num = 140;
                            self.parts[0].npc_flags.set_shootable(true);
                            self.parts[0].hit_bounds.left = 16 * 0x200;
                            self.parts[0].hit_bounds.right = 16 * 0x200;
                            let player = self.parts[0].get_closest_player_mut(players);
                            self.parts[0].vel_x = (player.x - self.parts[0].x).signum() * 0x100;
                            self.parts[0].damage = 0;
                            self.parts[0].hit_bounds.top = 36 * 0x200;
                        }
                        _ => {}
                    }
                }
            }
            140 => {
                let player = self.parts[5].get_closest_player_mut(players);
                self.parts[5].damage = if player.flags.hit_bottom_wall() && self.parts[0].vel_y > 0 {
                    20
                } else {
                    0
                };
                self.parts[0].vel_y += 0x24;
                if self.parts[0].vel_y > 0x5ff {
                    self.parts[0].vel_y = 0x5ff;
                }

                self.parts[0].y += self.parts[0].vel_y;
                self.parts[0].x += self.parts[0].vel_x;

                if self.parts[0].flags.hit_bottom_wall() {
                    self.parts[0].action_num = 110;
                    self.parts[0].action_counter = 0;
                    self.parts[0].anim_counter = 0;
                    self.parts[5].hit_bounds.top = 16 * 0x200;
                    self.parts[5].damage = 0;

                    state.sound_manager.play_sfx(26);
                    state.sound_manager.play_sfx(12);
                    state.quake_counter = 30;
                }
            }
            150 => {
                state.quake_counter = 2;

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter % 12 == 0 {
                    state.sound_manager.play_sfx(52);
                }

                let dest_x = self.parts[0].x + self.parts[0].rng.range(-0x30..0x30) as isize * 0x200;
                let dest_y = self.parts[0].y + self.parts[0].rng.range(-0x30..0x18) as isize * 0x200;

                let mut npc = NPCMap::create_npc(4, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = dest_x;
                npc.y = dest_y;
                state.new_npcs.push(npc);
                state.create_caret(dest_x, dest_y, CaretType::Explosion, Direction::Left);

                if self.parts[0].action_counter > 100 {
                    self.parts[0].action_num = 160;
                    self.parts[0].action_counter = 0;
                    // todo flash
                    state.sound_manager.play_sfx(35);
                }
            }
            160 => {
                state.quake_counter = 40;

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter > 50 {
                    for i in 0..6 {
                        self.parts[i].cond.set_alive(false);
                    }
                }
            }
            _ => {}
        }

        self.parts[0].anim_rect = state.constants.npc.b01_omega[self.parts[0].anim_num as usize];
        for i in 1..5 {
            self.parts[i].shock = self.parts[0].shock;
        }

        for &i in [3, 4].iter() {
            match self.parts[i].action_num {
                0 | 1 => {
                    if self.parts[i].action_num == 0 {
                        self.parts[i].action_num = 1;
                    }

                    self.parts[i].y = self.parts[0].y;

                    match i {
                        3 => self.parts[i].x = self.parts[0].x - 16 * 0x200,
                        4 => self.parts[i].x = self.parts[0].x + 16 * 0x200,
                        _ => {}
                    }
                }
                3 => {
                    self.parts[i].target_y = self.parts[0].y + 24 * 0x200;

                    match i {
                        3 => self.parts[i].x = self.parts[0].x - 16 * 0x200,
                        4 => self.parts[i].x = self.parts[0].x + 16 * 0x200,
                        _ => {}
                    }

                    self.parts[i].y += (self.parts[0].target_y - self.parts[0].y) / 2;
                }
                _ => {}
            }

            if self.parts[i].flags.hit_bottom_wall() || self.parts[i].y <= self.parts[i].target_y {
                self.parts[i].anim_num = 0;
            } else {
                self.parts[i].anim_num = 1;
            }

            let dir_offset = if self.parts[i].direction == Direction::Left { 0 } else { 2 };

            self.parts[i].anim_rect = state.constants.npc.b01_omega[6 + dir_offset + self.parts[i].anim_num as usize];
        }

        for &i in [1, 2].iter() {
            self.parts[i].x = self.parts[0].x + self.parts[i].direction.vector_x() * 16 * 0x200;
            self.parts[i].y = (self.parts[0].y + self.parts[i + 2].y - 8 * 0x200) / 2;

            let dir_offset = if self.parts[i].direction == Direction::Left { 0 } else { 1 };

            self.parts[i].anim_rect = state.constants.npc.b01_omega[4 + dir_offset];
        }

        if self.parts[5].action_num == 0 {
            self.parts[5].action_num = 1;
            self.parts[5].npc_flags.set_solid_soft(true);
            self.parts[5].npc_flags.set_ignore_solidity(true);
            self.parts[5].hit_bounds = Rect {
                left: 20 * 0x200,
                top: 36 * 0x200,
                right: 20 * 0x200,
                bottom: 16 * 0x200,
            };
        }

        self.parts[5].x = self.parts[0].x;
        self.parts[5].y = self.parts[0].y;

        if self.parts[0].life == 0 && self.parts[0].action_num < 150 {
            self.parts[0].action_num = 150;
            self.parts[0].action_counter = 0;
            self.parts[0].damage = 0;
            self.parts[5].damage = 5;

            for npc_cell in npc_map.values() {
                let mut npc = npc_cell.borrow_mut();
                if npc.cond.alive() && npc.npc_type == 48 {
                    npc.cond.set_alive(false);
                }
            }
        }
    }
}
