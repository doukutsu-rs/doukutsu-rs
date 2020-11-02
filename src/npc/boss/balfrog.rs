use crate::common::{Direction, Rect};
use crate::npc::boss::BossNPC;
use crate::npc::NPCMap;
use crate::shared_game_state::SharedGameState;

impl BossNPC {
    pub(crate) fn tick_b02_balfrog(&mut self, state: &mut SharedGameState) {
        match self.parts[0].action_num {
            0 => {
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
