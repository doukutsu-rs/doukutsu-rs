use crate::ggez::{Context, filesystem, GameResult};
use crate::scene::game_scene::GameScene;
use crate::scene::Scene;
use crate::SharedGameState;
use crate::stage::StageData;
use crate::text_script::{TextScript, TextScriptExecutionState};
use crate::common::FadeState;
use crate::npc::NPCTable;

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
            
            let mut next_scene = GameScene::new(state, ctx, 13)?;
            next_scene.player.x = 10 * 16 * 0x200;
            next_scene.player.y = 8 * 16 * 0x200;
            state.fade_state = FadeState::Hidden;
            state.textscript_vm.state = TextScriptExecutionState::Running(200, 0);

            state.next_scene = Some(Box::new(next_scene));
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
