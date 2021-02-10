use std::mem::MaybeUninit;

use ggez::{Context, GameResult};

use crate::bullet::BulletManager;
use crate::common::{Direction, interpolate_fix9_scale};
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;
use crate::stage::Stage;

pub mod balfrog;
pub mod ballos;
pub mod core;
pub mod ironhead;
pub mod monster_x;
pub mod omega;
pub mod press;
pub mod twins;
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
            part.rng.load_state(((i as u32)
                .wrapping_add(3271284409)
                .rotate_left(5)
                .wrapping_mul(3815776271)
                .rotate_right(9)
                .wrapping_sub(2626817629) & 0xffffffff) as u32);
        }

        BossNPC {
            boss_type: 0,
            parts,
            hurt_sound: [0; 20],
            death_sound: [0; 20],
        }
    }
}

impl GameEntity<([&mut Player; 2], &NPCList, &mut Stage, &BulletManager)> for BossNPC {
    fn tick(&mut self, state: &mut SharedGameState, (players, npc_list, _stage, bullet_manager): ([&mut Player; 2], &NPCList, &mut Stage, &BulletManager)) -> GameResult {
        if !self.parts[0].cond.alive() {
            return Ok(());
        }

        match self.boss_type {
            1 => self.tick_b01_omega(state, players, npc_list, bullet_manager),
            2 => self.tick_b02_balfrog(state, players, npc_list),
            3 => self.tick_b03_monster_x(state, players, npc_list),
            4 => self.tick_b04_core(),
            5 => self.tick_b05_ironhead(),
            6 => self.tick_b06_twins(),
            7 => self.tick_b07_undead_core(),
            8 => self.tick_b08_press(),
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
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, state.npc_table.tex_npc2_name.as_str())?;

        for npc in self.parts.iter().rev() {
            if !npc.cond.alive() || npc.cond.hidden() {
                continue;
            }

            let off_x = if npc.direction == Direction::Left { npc.display_bounds.left } else { npc.display_bounds.right } as i32;
            let shock = if npc.shock > 0 {
                (2 * ((npc.shock as i32 / 2) % 2) - 1) as f32
            } else { 0.0 };

            batch.add_rect(
                interpolate_fix9_scale(npc.prev_x - off_x - frame.prev_x,
                                       npc.x - off_x - frame.x,
                                       state.frame_time) + shock,
                interpolate_fix9_scale(npc.prev_y - npc.display_bounds.top as i32 - frame.prev_y,
                                       npc.y - npc.display_bounds.top as i32 - frame.y,
                                       state.frame_time),
                &npc.anim_rect,
            );
        }

        batch.draw(ctx)?;

        Ok(())
    }
}
