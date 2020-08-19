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
    fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        state.texture_set.ensure_texture_loaded(ctx, &state.constants, "Loading")?;
        Ok(())
    }

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
        let loading = state.texture_set.tex_map.get_mut("Loading");

        if loading.is_some() {
            let img = loading.unwrap();
            img.add(((state.canvas_size.0 - img.width() as f32) / 2.0).floor(),
                    ((state.canvas_size.1 - img.height() as f32) / 2.0).floor());
            img.draw(ctx)?;
        }

        Ok(())
    }
}
