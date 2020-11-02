use crate::common::{Direction, Rect};
use crate::npc::boss::BossNPC;
use crate::npc::NPCMap;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;

impl BossNPC {
    pub(crate) fn tick_b02_balfrog(&mut self, state: &mut SharedGameState, player: &Player) {
        match self.parts[0].action_num {
            0 => {
                self.hurt_sound[0] = 52;
                self.parts[0].x = 6 * 16 * 0x200;
                self.parts[0].y = 12 * 16 * 0x200;
                self.parts[0].direction = Direction::Right;
                self.parts[0].display_bounds = Rect {
                    left: 48 * 0x200,
                    top: 48 * 0x200,
                    right: 32 * 0x200,
                    bottom: 16 * 0x200,
                };
                self.parts[0].hit_bounds = Rect {
                    left: 24 * 0x200,
                    top: 16 * 0x200,
                    right: 24 * 0x200,
                    bottom: 16 * 0x200,
                };
                self.parts[0].size = 3;
                self.parts[0].exp = 1;
                self.parts[0].event_num = 1000;
                self.parts[0].npc_flags.set_event_when_killed(true);
                self.parts[0].npc_flags.set_show_damage(true);
                self.parts[0].life = 300;
            }
            10 => {
                self.parts[0].action_num = 11;
                self.parts[0].anim_num = 3;
                self.parts[0].cond.set_alive(true);
                self.parts[0].anim_rect = state.constants.npc.b02_balfrog[9];

                self.parts[1].cond.set_alive(true);
                self.parts[1].cond.set_damage_boss(true);
                self.parts[1].damage = 5;

                self.parts[2].cond.set_alive(true);
                self.parts[2].damage = 5;

                let mut npc = NPCMap::create_npc(4, &state.npc_table);

                for _ in 0..8 {
                    npc.cond.set_alive(true);
                    npc.direction = Direction::Left;
                    npc.x = self.parts[0].x + state.game_rng.range(-12..12) as isize * 0x200;
                    npc.y = self.parts[0].y + state.game_rng.range(-12..12) as isize * 0x200;
                    npc.vel_x = state.game_rng.range(-0x155..0x155) as isize;
                    npc.vel_y = state.game_rng.range(-0x600..0) as isize;

                    state.new_npcs.push(npc);
                }
            }
            20 | 21 => {
                if self.parts[0].action_num == 20 {
                    self.parts[0].action_num = 0;
                    self.parts[0].action_counter = 0
                }

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter / 2 % 2 != 0 {
                    self.parts[0].anim_num = 3;
                } else {
                    self.parts[0].anim_num = 0;
                }
            }
            100 | 101 => {
                if self.parts[0].action_num == 100 {
                    self.parts[0].action_num = 101;
                    self.parts[0].action_counter = 0;
                    self.parts[0].anim_num = 1;
                    self.parts[0].vel_x = 0;
                }

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter > 50 {
                    self.parts[0].action_num = 102;
                    self.parts[0].anim_counter = 0;
                    self.parts[0].anim_num = 2;
                }
            }
            102 => {
                self.parts[0].anim_counter += 1;

                if self.parts[0].anim_counter > 10 {
                    self.parts[0].action_num = 103;
                    self.parts[0].anim_counter = 0;
                    self.parts[0].anim_num = 1;
                }
            }
            103 => {
                self.parts[0].anim_counter += 1;
                if self.parts[0].anim_counter > 4 {
                    self.parts[0].action_num = 104;
                    self.parts[0].anim_num = 5;
                    self.parts[0].vel_x = self.parts[0].direction.vector_x() * 0x200;
                    self.parts[0].vel_y = -2 * 0x200;
                    self.parts[0].display_bounds.top = 64 * 0x200;
                    self.parts[0].display_bounds.bottom = 24 * 0x200;

                    state.sound_manager.play_sfx(25);
                }
            }
            104 => {
                if self.parts[0].direction == Direction::Left && self.parts[0].flags.hit_left_wall() {
                    self.parts[0].direction = Direction::Right;
                    self.parts[0].vel_x = 0x200;
                }

                if self.parts[0].direction == Direction::Right && self.parts[0].flags.hit_right_wall() {
                    self.parts[0].direction = Direction::Left;
                    self.parts[0].vel_x = -0x200;
                }

                if self.parts[0].flags.hit_bottom_wall() {
                    self.parts[0].action_num = 100;
                    self.parts[0].anim_num = 1;
                    self.parts[0].display_bounds.top = 48 * 0x200;
                    self.parts[0].display_bounds.bottom = 16 * 0x200;

                    if self.parts[0].direction == Direction::Left && self.parts[0].x < player.x {
                        self.parts[0].direction = Direction::Right;
                        self.parts[0].action_num = 110;
                    }

                    if self.parts[0].direction == Direction::Right && self.parts[0].x > player.x {
                        self.parts[0].direction = Direction::Left;
                        self.parts[0].action_num = 110;
                    }

                    let mut npc = NPCMap::create_npc(110, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = state.game_rng.range(4..16) as isize * 16 * 0x200;
                    npc.y = state.game_rng.range(0..4) as isize * 16 * 0x200;
                    npc.direction = if npc.x < player.x { Direction::Left } else { Direction::Right };

                    state.new_npcs.push(npc);

                    let mut npc = NPCMap::create_npc(4, &state.npc_table);

                    for _ in 0..4 {
                        npc.cond.set_alive(true);
                        npc.direction = Direction::Left;
                        npc.x = self.parts[0].x + state.game_rng.range(-12..12) as isize * 0x200;
                        npc.y = self.parts[0].y + state.game_rng.range(-12..12) as isize * 0x200;
                        npc.vel_x = state.game_rng.range(-0x155..0x155) as isize;
                        npc.vel_y = state.game_rng.range(-0x600..0) as isize;

                        state.new_npcs.push(npc);
                    }

                    state.quake_counter = 30;
                    state.sound_manager.play_sfx(26);
                }
            }
            110 | 111 => {
                if self.parts[0].action_num == 110 {
                    self.parts[0].anim_num = 1;
                    self.parts[0].action_num = 111;
                    self.parts[0].action_counter = 0;
                }

                self.parts[0].action_counter += 1;

                self.parts[0].vel_x = self.parts[0].vel_x * 8 / 9;

                if self.parts[0].action_counter > 50 {
                    self.parts[0].anim_num = 2;
                    self.parts[0].anim_counter = 0;
                    self.parts[0].action_num = 112;
                }
            }
            112 => {
                self.parts[0].anim_counter += 1;

                if self.parts[0].anim_counter > 4 {
                    self.parts[0].action_num = 113;
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_counter2 = 16;
                    self.parts[0].anim_num = 3;
                    self.parts[0].target_x = self.parts[0].life as isize;
                    self.parts[1].npc_flags.set_shootable(true);
                }
            }
            113 => {
                if self.parts[0].shock != 0 {
                    if self.parts[0].action_counter2 / 2 % 2 != 0 {
                        self.parts[0].anim_num = 4;
                    } else {
                        self.parts[0].anim_num = 3;
                    }
                } else {
                    self.parts[0].action_counter2 = 0;
                    self.parts[0].anim_num = 3;
                }

                self.parts[0].vel_x = self.parts[0].vel_x * 10 / 11;

                self.parts[0].action_counter += 1;
                if self.parts[0].action_counter > 16 {
                    self.parts[0].action_counter = 0;
                    self.parts[0].action_counter2 = self.parts[0].action_counter2.saturating_sub(1);

                    let px = self.parts[0].x + self.parts[0].direction.vector_x() * 2 * 16 * 0x200;
                    let py = self.parts[0].y - 8 * 0x200 - player.y;

                    let deg = f64::atan2(py as f64, px as f64);
                    // todo rand

                    let mut npc = NPCMap::create_npc(108, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = px;
                    npc.y = py;
                    npc.vel_x = (deg.cos() * 512.0) as isize;
                    npc.vel_y = (deg.sin() * 512.0) as isize;

                    state.sound_manager.play_sfx(39);

                    if self.parts[0].action_counter2 == 0 || (self.parts[0].life as isize) < self.parts[0].target_x - 90 {
                        self.parts[0].action_num = 114;
                        self.parts[0].action_counter = 0;
                        self.parts[0].anim_num = 2;
                        self.parts[0].anim_counter = 0;
                        self.parts[1].npc_flags.set_shootable(false);
                    }
                }
            }
            114 => {
                self.parts[0].anim_counter += 1;
                if self.parts[0].anim_counter > 10 {
                    self.parts[0].anim_num = 1;
                    self.parts[0].anim_counter = 0;

                    self.parts[1].action_counter2 += 1;
                    if self.parts[1].action_counter2 > 2 {
                        self.parts[1].action_counter2 = 0;
                        self.parts[0].action_num = 120;
                    } else {
                        self.parts[0].action_num = 100;
                    }
                }
            }
            _ => {}
        }

        self.parts[0].vel_y += 0x40;
        if self.parts[0].vel_y > 0x5ff {
            self.parts[0].vel_y = 0x5ff;
        }

        self.parts[0].x += self.parts[0].vel_x;
        self.parts[0].y += self.parts[0].vel_y;

        let dir_offset = if self.parts[0].direction == Direction::Left { 0 } else { 9 };

        self.parts[0].anim_rect = state.constants.npc.b02_balfrog[self.parts[0].anim_num as usize + dir_offset];
    }
}
