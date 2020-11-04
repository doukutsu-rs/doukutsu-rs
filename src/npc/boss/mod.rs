use std::cell::RefCell;
use std::collections::HashMap;

use crate::common::{Direction, interpolate_fix9_scale};
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::ggez::{Context, GameResult};
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
    pub parts: [NPC; 16],
    pub hurt_sound: [u8; 16],
    pub death_sound: [u8; 16],
}

impl BossNPC {
    pub fn new() -> BossNPC {
        let mut parts = [{
            let mut part = NPC::empty();
            part.cond.set_drs_boss(true);
            part
        }; 16];
        parts[0].cond.set_alive(true);

        BossNPC {
            boss_type: 0,
            parts,
            hurt_sound: [0; 16],
            death_sound: [0; 16],
        }
    }
}

impl GameEntity<(&mut Player, &HashMap<u16, RefCell<NPC>>, &mut Stage)> for BossNPC {
    fn tick(&mut self, state: &mut SharedGameState, (player, map, stage): (&mut Player, &HashMap<u16, RefCell<NPC>>, &mut Stage)) -> GameResult {
        if !self.parts[0].cond.alive() {
            return Ok(());
        }

        match self.boss_type {
            1 => self.tick_b01_omega(),
            2 => self.tick_b02_balfrog(state, player),
            3 => self.tick_b03_monster_x(),
            4 => self.tick_b04_core(),
            5 => self.tick_b05_ironhead(),
            6 => self.tick_b06_twins(),
            7 => self.tick_b07_undead_core(),
            8 => self.tick_b09_ballos(),
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

        for npc in self.parts.iter() {
            if !npc.cond.alive() || npc.cond.hidden() {
                continue;
            }

            let off_x = if npc.direction == Direction::Left { npc.display_bounds.left } else { npc.display_bounds.right } as isize;
            let shock = if npc.shock > 0 {
                (2 * ((npc.shock as isize / 2) % 2) - 1) as f32
            } else { 0.0 };

            batch.add_rect(
                interpolate_fix9_scale(npc.prev_x - off_x - frame.prev_x,
                                       npc.x - off_x - frame.x,
                                       state.frame_time, state.scale) + shock,
                interpolate_fix9_scale(npc.prev_y - npc.display_bounds.top as isize - frame.prev_y,
                                       npc.y - npc.display_bounds.top as isize - frame.y,
                                       state.frame_time, state.scale),
                &npc.anim_rect,
            );
        }

        batch.draw(ctx)?;

        Ok(())
    }
}
