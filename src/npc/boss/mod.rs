use std::mem::MaybeUninit;
use std::ops::Deref;

use crate::common::{interpolate_fix9_scale, Direction};
use crate::components::flash::Flash;
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;
use crate::stage::Stage;
use crate::weapon::bullet::BulletManager;

pub mod balfrog;
pub mod ballos;
pub mod core;
pub mod ironhead;
pub mod monster_x;
pub mod omega;
pub mod heavy_press;
pub mod sisters;
pub mod undead_core;

pub struct BossNPC {
    pub boss_type: u16,
    pub parts: [NPC; 20],
    pub hurt_sound: [u8; 20],
    pub death_sound: [u8; 20],
}

impl BossNPC {
    pub fn new() -> BossNPC {
        let mut parts = unsafe {
            let mut parts_uninit: [NPC; 20] = MaybeUninit::uninit().assume_init();

            for part in parts_uninit.iter_mut() {
                *part = NPC::empty();
                part.cond.set_drs_boss(true);
            }

            parts_uninit
        };

        parts[0].cond.set_alive(true);

        for (i, part) in parts.iter_mut().enumerate() {
            part.rng.load_state(
                ((i as u32)
                    .wrapping_add(3271284409)
                    .rotate_left(5)
                    .wrapping_mul(3815776271)
                    .rotate_right(9)
                    .wrapping_sub(2626817629)
                    & 0xffffffff) as u32,
            );
        }

        BossNPC { boss_type: 0, parts, hurt_sound: [0; 20], death_sound: [0; 20] }
    }
}

impl GameEntity<([&mut Player; 2], &NPCList, &mut Stage, &BulletManager, &mut Flash)> for BossNPC {
    fn tick(
        &mut self,
        state: &mut SharedGameState,
        (players, npc_list, stage, bullet_manager, flash): (
            [&mut Player; 2],
            &NPCList,
            &mut Stage,
            &BulletManager,
            &mut Flash,
        ),
    ) -> GameResult {
        if !self.parts[0].cond.alive() {
            return Ok(());
        }

        match self.boss_type {
            1 => self.tick_b01_omega(state, players, npc_list, bullet_manager, flash),
            2 => self.tick_b02_balfrog(state, players, npc_list),
            3 => self.tick_b03_monster_x(state, players, npc_list, flash),
            4 => self.tick_b04_core(state, players, npc_list, stage),
            5 => self.tick_b05_ironhead(state, players, npc_list),
            6 => self.tick_b06_sisters(state, players, npc_list, flash),
            7 => self.tick_b07_undead_core(state, npc_list, stage, flash),
            8 => self.tick_b08_heavy_press(state, npc_list, stage),
            9 => self.tick_b09_ballos(),
            _ => {}
        }

        for part in self.parts.iter_mut() {
            if part.shock > 0 {
                part.shock -= 1;
            }
        }
        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(
            ctx,
            &state.constants,
            &state.npc_table.stage_textures.deref().borrow().npc2,
        )?;

        for npc in self.parts.iter().rev() {
            if !npc.cond.alive() || npc.cond.hidden() {
                continue;
            }

            let off_x =
                if npc.direction == Direction::Left { npc.display_bounds.left } else { npc.display_bounds.right }
                    as i32;
            let shock = if npc.shock > 0 { (2 * ((npc.shock as i32 / 2) & 1) - 1) as f32 } else { 0.0 };
            let (frame_x, frame_y) = frame.xy_interpolated(state.frame_time);

            batch.add_rect(
                interpolate_fix9_scale(npc.prev_x - off_x, npc.x - off_x, state.frame_time) + shock - frame_x,
                interpolate_fix9_scale(
                    npc.prev_y - npc.display_bounds.top as i32,
                    npc.y - npc.display_bounds.top as i32,
                    state.frame_time,
                ) - frame_y,
                &npc.anim_rect,
            );
        }

        batch.draw(ctx)?;

        Ok(())
    }
}
