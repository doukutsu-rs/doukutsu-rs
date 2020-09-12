use num_traits::clamp;

use crate::caret::CaretType;
use crate::common::{Condition, Direction, Flag, Rect};
use crate::npc::{NPC, NPCMap};
use crate::physics::PhysicalEntity;
use crate::player::Player;
use crate::SharedGameState;
use crate::stage::Stage;
use std::borrow::Borrow;

impl PhysicalEntity for Player {
    #[inline(always)]
    fn x(&self) -> isize {
        self.x
    }

    #[inline(always)]
    fn y(&self) -> isize {
        self.y
    }

    #[inline(always)]
    fn vel_x(&self) -> isize {
        self.vel_x
    }

    #[inline(always)]
    fn vel_y(&self) -> isize {
        self.vel_y
    }

    #[inline(always)]
    fn size(&self) -> u8 {
        1
    }

    #[inline(always)]
    fn hit_bounds(&self) -> &Rect<usize> {
        &self.hit_bounds
    }

    #[inline(always)]
    fn set_x(&mut self, x: isize) {
        self.x = x;
    }

    #[inline(always)]
    fn set_y(&mut self, y: isize) {
        self.y = y;
    }

    #[inline(always)]
    fn set_vel_x(&mut self, vel_x: isize) {
        self.vel_x = vel_x;
    }

    #[inline(always)]
    fn set_vel_y(&mut self, vel_y: isize) {
        self.vel_y = vel_y;
    }

    #[inline(always)]
    fn cond(&mut self) -> &mut Condition {
        &mut self.cond
    }

    #[inline(always)]
    fn flags(&mut self) -> &mut Flag {
        &mut self.flags
    }

    #[inline(always)]
    fn is_player(&self) -> bool {
        true
    }
}

impl Player {
    fn judge_hit_npc_solid_soft(&mut self, npc: &NPC) -> Flag {
        let mut flags = Flag(0);

        if ((self.y - self.hit_bounds.top as isize) < (npc.y + npc.hit_bounds.bottom as isize - 3 * 0x200))
            && ((self.y + self.hit_bounds.top as isize) > (npc.y - npc.hit_bounds.bottom as isize + 3 * 0x200))
            && ((self.x - self.hit_bounds.right as isize) < (npc.x + npc.hit_bounds.right as isize))
            && ((self.x - self.hit_bounds.right as isize) > npc.x) {
            if self.vel_x < 0x200 {
                self.vel_x += 0x200;
            }

            flags.set_hit_left_wall(true);
        }

        if ((self.y - self.hit_bounds.top as isize) < (npc.y + npc.hit_bounds.bottom as isize - 3 * 0x200))
            && ((self.y + self.hit_bounds.top as isize) > (npc.y - npc.hit_bounds.bottom as isize + 3 * 0x200))
            && ((self.x + self.hit_bounds.right as isize - 0x200) > (npc.x - npc.hit_bounds.right as isize))
            && ((self.x + self.hit_bounds.right as isize - 0x200) < npc.x) {
            if self.vel_x > -0x200 {
                self.vel_x -= 0x200;
            }

            flags.set_hit_right_wall(true);
        }


        if ((self.x - self.hit_bounds.right as isize) < (npc.x + npc.hit_bounds.right as isize - 3 * 0x200))
            && ((self.x + self.hit_bounds.right as isize) > (npc.x - npc.hit_bounds.right as isize + 3 * 0x200))
            && ((self.y - self.hit_bounds.top as isize) < (npc.y + npc.hit_bounds.bottom as isize))
            && ((self.y - self.hit_bounds.top as isize) > npc.y) {
            if self.vel_y < 0 {
                self.vel_y = 0;
            }

            flags.set_hit_top_wall(true);
        }

        if ((self.x - self.hit_bounds.right as isize) < (npc.x + npc.hit_bounds.right as isize - 3 * 0x200))
            && ((self.x + self.hit_bounds.right as isize) > (npc.x - npc.hit_bounds.right as isize + 3 * 0x200))
            && ((self.y + self.hit_bounds.bottom as isize - 0x200) > (npc.y - npc.hit_bounds.top as isize))
            && ((self.y + self.hit_bounds.bottom as isize - 0x200) < (npc.y + 3 * 0x200)) {
            if npc.npc_flags.bouncy() {
                self.vel_y = npc.vel_y - 0x200;
                flags.set_hit_bottom_wall(true);
            } else if !self.flags.hit_bottom_wall() && self.vel_y > npc.vel_y {
                self.y = npc.y - npc.hit_bounds.top as isize - self.hit_bounds.bottom as isize + 0x200;
                self.vel_y = npc.vel_y;
                self.x += npc.vel_x;
                flags.set_hit_bottom_wall(true);
            }
        }

        flags
    }

    fn judge_hit_npc_non_solid(&mut self, npc: &NPC) -> Flag {
        let mut flags = Flag(0);
        let hit_left = if npc.direction == Direction::Left { npc.hit_bounds.left } else { npc.hit_bounds.right } as isize;
        let hit_right = if npc.direction == Direction::Left { npc.hit_bounds.right } else { npc.hit_bounds.left } as isize;

        if self.x + (2 * 0x200) > npc.x - hit_left
            && self.x - (2 * 0x200) < npc.x + hit_right
            && self.y + (2 * 0x200) > npc.y - npc.hit_bounds.top as isize
            && self.y - (2 * 0x200) < npc.y + npc.hit_bounds.bottom as isize {
            flags.set_hit_left_wall(true);
        }

        flags
    }

    pub fn tick_npc_collisions(&mut self, state: &mut SharedGameState, npc_map: &mut NPCMap) {
        for npc_id in npc_map.npc_ids.iter() {
            if let Some(npc_cell) = npc_map.npcs.get(npc_id) {
                let npc = npc_cell.borrow_mut();
                if !npc.cond.alive() { continue; }

                let mut flags = Flag(0);

                if npc.npc_flags.solid_soft() {
                    flags = self.judge_hit_npc_solid_soft(npc.borrow());
                    self.flags.0 |= flags.0;
                } else if npc.npc_flags.solid_hard() {
                    //
                } else {
                    flags = self.judge_hit_npc_non_solid(npc.borrow());
                }

                if npc.npc_flags.interactable() && !state.control_flags.interactions_disabled() && flags.0 != 0 && self.cond.interacted() {
                    state.textscript_vm.start_script(npc.event_num);
                    self.cond.set_interacted(false);
                    self.vel_x = 0;
                    self.question = false;
                }

                if npc.npc_flags.event_when_touched() && !state.control_flags.interactions_disabled() && flags.0 != 0 {
                    state.textscript_vm.start_script(npc.event_num);
                }

                if state.control_flags.control_enabled() && !npc.npc_flags.interactable() {
                    if flags.0 != 0 && npc.damage != 0 && !state.control_flags.interactions_disabled() {
                        self.damage(npc.damage as isize, state);
                    }
                }
            }
        }

        if self.question {
            state.create_caret(self.x, self.y, CaretType::QuestionMark, Direction::Left);
        }
    }
}
