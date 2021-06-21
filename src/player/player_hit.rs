use std::borrow::Borrow;

use num_traits::abs;

use crate::caret::CaretType;
use crate::common::{Condition, Direction, Flag, Rect};
use crate::inventory::Inventory;
use crate::npc::boss::BossNPC;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::physics::PhysicalEntity;
use crate::player::{ControlMode, Player, TargetPlayer};
use crate::shared_game_state::SharedGameState;
use crate::weapon::WeaponType;

impl PhysicalEntity for Player {
    #[inline(always)]
    fn x(&self) -> i32 {
        self.x
    }

    #[inline(always)]
    fn y(&self) -> i32 {
        self.y
    }

    #[inline(always)]
    fn vel_x(&self) -> i32 {
        self.vel_x
    }

    #[inline(always)]
    fn vel_y(&self) -> i32 {
        self.vel_y
    }

    fn hit_rect_size(&self) -> usize {
        if self.hit_bounds.top > 0x1000 || self.hit_bounds.bottom > 0x1000 || self.hit_bounds.right > 0x1000 {
            4
        } else {
            2
        }
    }

    #[inline(always)]
    fn hit_bounds(&self) -> &Rect<u32> {
        &self.hit_bounds
    }

    #[inline(always)]
    fn set_x(&mut self, x: i32) {
        self.x = x;
    }

    #[inline(always)]
    fn set_y(&mut self, y: i32) {
        self.y = y;
    }

    #[inline(always)]
    fn set_vel_x(&mut self, vel_x: i32) {
        self.vel_x = vel_x;
    }

    #[inline(always)]
    fn set_vel_y(&mut self, vel_y: i32) {
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
    fn direction(&self) -> Direction {
        self.direction
    }

    #[inline(always)]
    fn is_player(&self) -> bool {
        true
    }

    fn player_left_pressed(&self) -> bool {
        self.controller.move_left()
    }

    fn player_right_pressed(&self) -> bool {
        self.controller.move_right()
    }
}

impl Player {
    fn test_hit_npc_solid_soft(&mut self, npc: &NPC) -> Flag {
        let mut flags = Flag(0);

        if ((self.y - self.hit_bounds.top as i32) < (npc.y + npc.hit_bounds.bottom as i32 - 0x600))
            && ((self.y + self.hit_bounds.top as i32) > (npc.y - npc.hit_bounds.bottom as i32 + 0x600))
            && ((self.x - self.hit_bounds.right as i32) < (npc.x + npc.hit_bounds.right as i32))
            && ((self.x - self.hit_bounds.right as i32) > npc.x) {
            if self.vel_x < 0x200 {
                self.vel_x += 0x200;
            }

            flags.set_hit_left_wall(true);
        }

        if ((self.y - self.hit_bounds.top as i32) < (npc.y + npc.hit_bounds.bottom as i32 - 0x600))
            && ((self.y + self.hit_bounds.top as i32) > (npc.y - npc.hit_bounds.bottom as i32 + 0x600))
            && ((self.x + self.hit_bounds.right as i32 - 0x200) > (npc.x - npc.hit_bounds.right as i32))
            && ((self.x + self.hit_bounds.right as i32 - 0x200) < npc.x) {
            if self.vel_x > -0x200 {
                self.vel_x -= 0x200;
            }

            flags.set_hit_right_wall(true);
        }


        if ((self.x - self.hit_bounds.right as i32) < (npc.x + npc.hit_bounds.right as i32 - 0x600))
            && ((self.x + self.hit_bounds.right as i32) > (npc.x - npc.hit_bounds.right as i32 + 0x600))
            && ((self.y - self.hit_bounds.top as i32) < (npc.y + npc.hit_bounds.bottom as i32))
            && ((self.y - self.hit_bounds.top as i32) > npc.y) {
            if self.vel_y < 0 {
                self.vel_y = 0;
            }

            flags.set_hit_top_wall(true);
        }

        if ((self.x - self.hit_bounds.right as i32) < (npc.x + npc.hit_bounds.right as i32 - 0x600))
            && ((self.x + self.hit_bounds.right as i32) > (npc.x - npc.hit_bounds.right as i32 + 0x600))
            && ((self.y + self.hit_bounds.bottom as i32) > (npc.y - npc.hit_bounds.top as i32))
            && ((self.y + self.hit_bounds.bottom as i32) < (npc.y + 0x600)) {
            if npc.npc_flags.bouncy() {
                self.vel_y = npc.vel_y - 0x200;
                flags.set_hit_bottom_wall(true);
            } else if !self.flags.hit_bottom_wall() && self.vel_y > npc.vel_y {
                self.y = npc.y - npc.hit_bounds.top as i32 - self.hit_bounds.bottom as i32 + 0x200;
                self.vel_y = npc.vel_y;
                self.x += npc.vel_x;
                flags.set_hit_bottom_wall(true);
            }
        }

        flags
    }

    fn test_hit_npc_solid_hard(&mut self, npc: &NPC, state: &mut SharedGameState) -> Flag {
        let mut flags = Flag(0);

        let fx1 = abs(self.x - npc.x) as f32;
        let fy1 = abs(self.y - npc.y) as f32;

        let fx2 = npc.hit_bounds.right as f32;
        let fy2 = npc.hit_bounds.top as f32;

        let fx1 = if fx1 == 0.0 { 1.0 } else { fx1 };
        let fx2 = if fx2 == 0.0 { 1.0 } else { fx2 };

        if fy1 / fx1 <= fy2 / fx2 {
            if (self.y - self.hit_bounds.top as i32) < (npc.y + npc.hit_bounds.bottom as i32)
                && (self.y + self.hit_bounds.bottom as i32) > (npc.y - npc.hit_bounds.top as i32) {
                if (self.x - self.hit_bounds.right as i32) < (npc.x + npc.hit_bounds.right as i32)
                    && (self.x - self.hit_bounds.right as i32) > npc.x {
                    if self.vel_x < npc.vel_x {
                        self.vel_x = npc.vel_x;
                    }

                    self.x = npc.x + npc.hit_bounds.right as i32 + self.hit_bounds.right as i32;
                    flags.set_hit_left_wall(true);
                }

                if (self.x + self.hit_bounds.right as i32) > (npc.x - npc.hit_bounds.right as i32)
                    && (self.x + self.hit_bounds.right as i32) < npc.x {
                    if self.vel_x > npc.vel_x {
                        self.vel_x = npc.vel_x;
                    }

                    self.x = npc.x - npc.hit_bounds.right as i32 - self.hit_bounds.right as i32;
                    flags.set_hit_right_wall(true);
                }
            }
        } else if (self.x - self.hit_bounds.right as i32) < (npc.x + npc.hit_bounds.right as i32)
            && (self.x + self.hit_bounds.right as i32) > (npc.x - npc.hit_bounds.right as i32) {
            if (self.y - self.hit_bounds.top as i32) < (npc.y + npc.hit_bounds.bottom as i32)
                && (self.y - self.hit_bounds.top as i32) > npc.y {
                if self.vel_y >= npc.vel_y {
                    if self.vel_y < 0 {
                        self.vel_y = 0;
                    }
                } else {
                    self.y = npc.y + npc.hit_bounds.bottom as i32 + self.hit_bounds.top as i32 + 0x200;
                    self.vel_y = npc.vel_y;
                }

                flags.set_hit_top_wall(true);
            }

            if (self.y + self.hit_bounds.bottom as i32) > (npc.y - npc.hit_bounds.top as i32)
                && (self.y + self.hit_bounds.bottom as i32) < (npc.y + 0x600) {
                if self.vel_y - npc.vel_y > 0x400 {
                    state.sound_manager.play_sfx(23);
                }

                if self.control_mode == ControlMode::IronHead {
                    self.y = npc.y - npc.hit_bounds.top as i32 - self.hit_bounds.bottom as i32 + 0x200;
                    flags.set_hit_bottom_wall(true);
                } else if npc.npc_flags.bouncy() {
                    self.vel_y = npc.vel_y - 0x200;
                    flags.set_hit_bottom_wall(true);
                } else if !self.flags.hit_bottom_wall() && self.vel_y > npc.vel_y {
                    self.x += npc.vel_x;
                    self.y = npc.y - npc.hit_bounds.top as i32 - self.hit_bounds.bottom as i32 + 0x200;
                    self.vel_y = npc.vel_y;

                    flags.set_hit_bottom_wall(true);
                }
            }
        }

        flags
    }

    fn test_hit_npc_non_solid(&mut self, npc: &NPC) -> Flag {
        let mut flags = Flag(0);
        let hit_left = if npc.direction == Direction::Left { npc.hit_bounds.left } else { npc.hit_bounds.right } as i32;
        let hit_right = if npc.direction == Direction::Left { npc.hit_bounds.right } else { npc.hit_bounds.left } as i32;

        if self.x + 0x400 > npc.x - hit_left
            && self.x - 0x400 < npc.x + hit_right
            && self.y + 0x400 > npc.y - npc.hit_bounds.top as i32
            && self.y - 0x400 < npc.y + npc.hit_bounds.bottom as i32 {
            flags.set_hit_left_wall(true);
        }

        flags
    }

    fn tick_npc_collision(&mut self, id: TargetPlayer, state: &mut SharedGameState, npc: &mut NPC, npc_list: &NPCList, inventory: &mut Inventory) {
        let flags: Flag;

        if npc.npc_flags.solid_soft() {
            flags = self.test_hit_npc_solid_soft(npc.borrow());
            self.flags.0 |= flags.0;
        } else if npc.npc_flags.solid_hard() {
            flags = self.test_hit_npc_solid_hard(npc.borrow(), state);
            self.flags.0 |= flags.0;
        } else {
            flags = self.test_hit_npc_non_solid(npc.borrow());
        }

        if !npc.cond.drs_boss() && flags.0 != 0 {
            match npc.npc_type {
                // experience pickup
                1 => {
                    state.sound_manager.play_sfx(14);
                    inventory.add_xp(npc.exp, self, state);

                    if self.popup.value > 0 {
                        self.popup.add_value(npc.exp as i16);
                    } else {
                        self.popup.set_value(npc.exp as i16);
                    }

                    npc.cond.set_alive(false);
                }
                // missile pickup
                86 => {
                    // todo add bullets
                    if let Some(weapon) = inventory.get_weapon_by_type_mut(WeaponType::MissileLauncher) {
                        weapon.refill_ammo(npc.exp);
                    } else if let Some(weapon) = inventory.get_weapon_by_type_mut(WeaponType::SuperMissileLauncher) {
                        weapon.refill_ammo(npc.exp);
                    }

                    npc.cond.set_alive(false);

                    state.sound_manager.play_sfx(42);
                }
                // heart pickup
                87 => {
                    self.life = self.max_life.min(self.life.saturating_add(npc.exp));
                    npc.cond.set_alive(false);

                    state.sound_manager.play_sfx(20);
                }
                _ => {}
            }
        }

        if state.settings.touch_controls && npc.npc_flags.interactable() && flags.0 != 0 {
            // todo make it less hacky
            state.touch_controls.interact_icon = true;
        }

        if npc.npc_flags.event_when_touched() && !state.control_flags.interactions_disabled() && flags.0 != 0 {
            state.control_flags.set_tick_world(true);
            state.control_flags.set_interactions_disabled(true);
            state.textscript_vm.executor_player = id;
            state.textscript_vm.start_script(npc.event_num);
        }

        if state.control_flags.control_enabled() && !npc.npc_flags.interactable() {
            if npc.npc_flags.rear_and_top_not_hurt() {
                if flags.hit_left_wall() && npc.vel_x > 0
                    || flags.hit_right_wall() && npc.vel_x < 0
                    || flags.hit_top_wall() && npc.vel_y > 0
                    || flags.hit_bottom_wall() && npc.vel_y < 0 {
                    self.damage(npc.damage as i32, state, npc_list);
                }
            } else if flags.0 != 0 && npc.damage != 0 && !state.control_flags.interactions_disabled() {
                self.damage(npc.damage as i32, state, npc_list);
            }
        }

        if npc.npc_flags.interactable() && !state.control_flags.interactions_disabled() && flags.0 != 0 && self.cond.interacted() {
            state.control_flags.set_tick_world(true);
            state.control_flags.set_interactions_disabled(true);
            state.textscript_vm.executor_player = id;
            state.textscript_vm.start_script(npc.event_num);
            self.cond.set_interacted(false);
            self.vel_x = 0;
            self.question = false;
        }
    }

    pub fn tick_npc_collisions(&mut self, id: TargetPlayer, state: &mut SharedGameState, npc_list: &NPCList, boss: &mut BossNPC, inventory: &mut Inventory) {
        if !self.cond.alive() {
            return;
        }

        for npc in npc_list.iter_alive() {
            self.tick_npc_collision(id, state, npc, npc_list, inventory);
        }

        for boss_npc in boss.parts.iter_mut() {
            if boss_npc.cond.alive() {
                self.tick_npc_collision(id, state, boss_npc, npc_list, inventory);
            }
        }

        if self.question {
            state.create_caret(self.x, self.y, CaretType::QuestionMark, Direction::Left);
        }
    }
}
