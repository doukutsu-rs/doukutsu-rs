use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::graphics;
use crate::scene::no_data_scene::NoDataScene;
use crate::scene::Scene;
use crate::shared_game_state::SharedGameState;

pub struct LoadingScene {
    tick: usize,
}

impl LoadingScene {
    pub fn new() -> Self {
        Self { tick: 0 }
    }

    fn load_stuff(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        state.reload_resources(ctx)?;

        if ctx.headless {
            log::info!("Headless mode detected, skipping intro and loading last saved game.");
            state.load_or_start_game(ctx)?;
        } else {
            state.start_intro(ctx)?;
        }

        Ok(())
    }
}

impl Scene for LoadingScene {
    fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        // deferred to let the loading image draw
        if self.tick == 1 {
            if let Err(err) = self.load_stuff(state, ctx) {
                log::error!("Failed to load game data: {}", err);

                state.next_scene = Some(Box::new(NoDataScene::new(err)));
            }
        }

        self.tick += 1;
        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        graphics::set_vsync_mode(ctx, state.settings.vsync_mode)?;

        match state.texture_set.get_or_load_batch(ctx, &state.constants, "Loading") {
            Ok(batch) => {
                batch.add(
                    ((state.canvas_size.0 - batch.width() as f32) / 2.0).floor(),
                    ((state.canvas_size.1 - batch.height() as f32) / 2.0).floor(),
                );
                batch.draw(ctx)?;
            }
            Err(err) => {
                log::error!("Failed to load game data: {}", err);

                state.next_scene = Some(Box::new(NoDataScene::new(err)));
            }
        }

        Ok(())
    }
}
