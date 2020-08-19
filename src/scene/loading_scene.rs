use ggez::{Context, GameResult};

use crate::game_state::GameState;
use crate::GameContext;
use crate::scene::Scene;
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
    fn init(&mut self, state: &mut GameState, game_ctx: &mut GameContext, ctx: &mut Context) -> GameResult {
        game_ctx.texture_set.ensure_texture_loaded(ctx, &game_ctx.constants, "Loading")?;
        Ok(())
    }

    fn tick(&mut self, state: &mut GameState, game_ctx: &mut GameContext, ctx: &mut Context) -> GameResult {
        // deferred to let the loading image draw
        if self.tick == 1 {
            let stages = StageData::load_stage_table(ctx, &game_ctx.base_path)?;
            game_ctx.stages = stages;
        } else if self.tick == 2 {
            state.init(game_ctx, ctx)?;
            state.switch_to_stage(53, game_ctx, ctx)?;
        }

        self.tick += 1;
        Ok(())
    }

    fn draw(&self, state: &GameState, game_ctx: &mut GameContext, ctx: &mut Context) -> GameResult {
        let loading = game_ctx.texture_set.tex_map.get_mut("Loading");

        if loading.is_some() {
            let img = loading.unwrap();
            img.add(((game_ctx.canvas_size.0 - img.width() as f32) / 2.0).floor(),
                    ((game_ctx.canvas_size.1 - img.height() as f32) / 2.0).floor());
            img.draw(ctx)?;
        }

        Ok(())
    }
}
