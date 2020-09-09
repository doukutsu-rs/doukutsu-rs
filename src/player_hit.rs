use num_traits::clamp;

use crate::caret::CaretType;
use crate::common::{Condition, Direction, Flag, Rect};
use crate::npc::{NPC, NPCMap};
use crate::physics::PhysicalEntity;
use crate::player::Player;
use crate::SharedGameState;
use crate::stage::Stage;

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
            if let Some(npc) = npc_map.npcs.get_mut(npc_id) {
                if !npc.cond.alive() { continue; }

                let mut flags = Flag(0);

                if npc.npc_flags.solid_soft() {
                    //
                } else if npc.npc_flags.solid_hard() {
                    //
                } else {
                    flags = self.judge_hit_npc_non_solid(npc);
                }

                if npc.npc_flags.interactable() && !state.control_flags.flag_x04() && flags.0 != 0 && self.cond.interacted() {
                    state.textscript_vm.start_script(npc.event_num);
                    self.cond.set_interacted(false);
                    self.vel_x = 0;
                    self.question = false;
                }

                if npc.npc_flags.event_when_touched() && !state.control_flags.flag_x04() && flags.0 != 0 {
                    state.textscript_vm.start_script(npc.event_num);
                }
            }
        }

        if self.question {
            state.create_caret(self.x, self.y, CaretType::QuestionMark, Direction::Left);
        }
    }
}
