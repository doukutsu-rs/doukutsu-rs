use crate::ggez::{Context, filesystem, GameResult};
use crate::npc::NPCTable;
use crate::scene::Scene;
use crate::scene::title_scene::TitleScene;
use crate::shared_game_state::SharedGameState;
use crate::stage::StageData;
use crate::text_script::TextScript;

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
            let npc_table = NPCTable::load_from(filesystem::open(ctx, [&state.base_path, "/npc.tbl"].join(""))?)?;
            state.npc_table = npc_table;
            let head_script = TextScript::load_from(filesystem::open(ctx, [&state.base_path, "/Head.tsc"].join(""))?)?;
            state.textscript_vm.set_global_script(head_script);

            state.next_scene = Some(Box::new(TitleScene::new()));
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
