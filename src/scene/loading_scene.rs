use ggez::{Context, GameResult};

use crate::scene::game_scene::GameScene;
use crate::scene::Scene;
use crate::SharedGameState;
use crate::stage::StageData;

pub struct LoadingScene {
    tick: usize,
}

impl LoadingScene {
    pub fn new() -> Self {
        Self {
            tick: 0,
        }
    }
}

impl Scene for LoadingScene {
    fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        // deferred to let the loading image draw
        if self.tick == 1 {
            let stages = StageData::load_stage_table(ctx, &state.base_path)?;
            state.stages = stages;
            state.next_scene = Some(Box::new(GameScene::new(state, ctx, 53)?));
        }

        self.tick += 1;
        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "Loading")?;

        batch.add(((state.canvas_size.0 - batch.width() as f32) / 2.0).floor(),
                  ((state.canvas_size.1 - batch.height() as f32) / 2.0).floor());
        batch.draw(ctx)?;
        Ok(())
    }
}
