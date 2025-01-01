use crate::common::Rect;
use crate::entity::GameEntity;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::game::frame::Frame;
use crate::game::shared_game_state::SharedGameState;
use crate::game::npc::boss::BossNPC;
use crate::game::npc::list::{NPCAccessToken, NPCList};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum BossLifeTarget {
    None,
    NPC(u16),
    Boss,
}

pub struct BossLifeBar {
    target: BossLifeTarget,
    life: u16,
    max_life: u16,
    prev_life: u16,
    counter: u16,
}

impl BossLifeBar {
    pub fn new() -> BossLifeBar {
        BossLifeBar { target: BossLifeTarget::None, life: 0, max_life: 0, prev_life: 0, counter: 0 }
    }

    pub fn set_npc_target(&mut self, npc_id: u16, npc_list: &NPCList, token: &NPCAccessToken) {
        if let Some(npc) = npc_list.get_npc(npc_id as usize, token) {
            let npc = npc.borrow();

            self.target = BossLifeTarget::NPC(npc.id);
            self.life = npc.life;
            self.max_life = self.life;
            self.prev_life = self.life;
        }
    }

    pub fn set_boss_target(&mut self, boss: &BossNPC) {
        self.target = BossLifeTarget::Boss;
        self.life = boss.parts[0].life;
        self.max_life = self.life;
        self.prev_life = self.life;
    }

    fn draw_regular(&self, state: &mut SharedGameState, ctx: &mut Context, _frame: &Frame) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;

        let box_length = 256;
        let bar_length = box_length - 58;

        let text_rect = Rect::new_size(0, 48, 32, 8);
        let box_rect1 = Rect::new_size(0, 0, 244, 8);
        let box_rect2 = Rect::new_size(0, 16, 244, 8);
        let mut rect_prev_bar = Rect::new_size(0, 32, 232, 8);
        let mut rect_life_bar = Rect::new_size(0, 24, 232, 8);

        rect_prev_bar.right = ((self.prev_life as u32 * bar_length) / self.max_life as u32).min(bar_length) as u16;
        rect_life_bar.right = ((self.life as u32 * bar_length) / self.max_life as u32).min(bar_length) as u16;

        batch.add_rect(
            ((state.canvas_size.0 - box_length as f32) / 2.0).floor(),
            state.canvas_size.1 - 20.0,
            &box_rect1,
        );
        batch.add_rect(
            ((state.canvas_size.0 - box_length as f32) / 2.0).floor(),
            state.canvas_size.1 - 12.0,
            &box_rect2,
        );
        batch.add_rect(
            ((state.canvas_size.0 - box_length as f32) / 2.0).floor(),
            state.canvas_size.1 - 20.0,
            &box_rect1,
        );
        batch.add_rect(
            ((state.canvas_size.0 - box_length as f32) / 2.0 + 40.0).floor(),
            state.canvas_size.1 - 16.0,
            &rect_prev_bar,
        );
        batch.add_rect(
            ((state.canvas_size.0 - box_length as f32) / 2.0 + 40.0).floor(),
            state.canvas_size.1 - 16.0,
            &rect_life_bar,
        );
        batch.add_rect(
            ((state.canvas_size.0 - box_length as f32) / 2.0 + 8.0).floor(),
            state.canvas_size.1 - 16.0,
            &text_rect,
        );

        batch.draw(ctx)?;

        Ok(())
    }

    fn draw_nx(&self, state: &mut SharedGameState, ctx: &mut Context, _frame: &Frame) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;

        let box_length = 148;
        let bar_length = box_length - 52;

        let text_rect = Rect::new_size(0, 48, 32, 8);
        let box_rect1 = Rect::new_size(0, 0, 136, 8);
        let box_rect2 = Rect::new_size(0, 16, 136, 8);
        let box_rect3 = Rect::new_size(238, 0, 6, 8);
        let box_rect4 = Rect::new_size(238, 16, 6, 8);
        let mut rect_prev_bar = Rect::new_size(0, 32, 124, 8);
        let mut rect_life_bar = Rect::new_size(0, 24, 124, 8);

        rect_prev_bar.right = ((self.prev_life as u32 * bar_length) / self.max_life as u32).min(bar_length) as u16;
        rect_life_bar.right = ((self.life as u32 * bar_length) / self.max_life as u32).min(bar_length) as u16;

        let base_x = state.canvas_size.0 - box_length as f32;

        batch.add_rect((base_x - 6.0).floor(), state.canvas_size.1 - 20.0, &box_rect1);
        batch.add_rect((base_x - 6.0).floor(), state.canvas_size.1 - 12.0, &box_rect2);
        batch.add_rect((base_x - 6.0).floor(), state.canvas_size.1 - 20.0, &box_rect1);
        batch.add_rect((state.canvas_size.0 - 18.0).floor(), state.canvas_size.1 - 20.0, &box_rect3);
        batch.add_rect((state.canvas_size.0 - 18.0).floor(), state.canvas_size.1 - 12.0, &box_rect4);
        batch.add_rect((state.canvas_size.0 - 18.0).floor(), state.canvas_size.1 - 20.0, &box_rect3);
        batch.add_rect((base_x + 34.0).floor(), state.canvas_size.1 - 16.0, &rect_prev_bar);
        batch.add_rect((base_x + 34.0).floor(), state.canvas_size.1 - 16.0, &rect_life_bar);
        batch.add_rect((base_x + 2.0).floor(), state.canvas_size.1 - 16.0, &text_rect);

        batch.draw(ctx)?;

        Ok(())
    }
}

impl GameEntity<(&NPCList, &NPCAccessToken, &BossNPC)> for BossLifeBar {
    fn tick(&mut self, _state: &mut SharedGameState, (npc_list, token, boss): (&NPCList, &NPCAccessToken, &BossNPC)) -> GameResult<()> {
        match self.target {
            BossLifeTarget::NPC(npc_id) => {
                if let Some(npc) = npc_list.get_npc(npc_id as usize, token) {
                    let npc = npc.borrow();
                    
                    self.life = npc.life;
                }
            }
            BossLifeTarget::Boss => {
                self.life = boss.parts[0].life;
            }
            _ => {
                return Ok(());
            }
        }

        if self.life == 0 {
            self.target = BossLifeTarget::None;
        } else if self.prev_life > self.life {
            self.counter += 1;
            if self.counter > 30 {
                self.prev_life = self.prev_life.saturating_sub(1);
            }
        } else {
            self.counter = 0;
        }

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, _frame: &Frame) -> GameResult<()> {
        if self.max_life == 0 || self.target == BossLifeTarget::None {
            return Ok(());
        }

        match state.constants.is_switch {
            true => self.draw_nx(state, ctx, _frame),
            false => self.draw_regular(state, ctx, _frame),
        }
    }
}
