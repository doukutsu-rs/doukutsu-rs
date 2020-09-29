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
            let npc_tbl = filesystem::open(ctx, [&state.base_path, "/npc.tbl"].join(""))?;
            let npc_table = NPCTable::load_from(npc_tbl)?;
            state.npc_table = npc_table;
            let head_tsc = filesystem::open(ctx, [&state.base_path, "/Head.tsc"].join(""))?;
            let head_script = TextScript::load_from(head_tsc, &state.constants)?;
            state.textscript_vm.set_global_script(head_script);

            let arms_item_tsc = filesystem::open(ctx, [&state.base_path, "/ArmsItem.tsc"].join(""))?;
            let arms_item_script = TextScript::load_from(arms_item_tsc, &state.constants)?;
            state.textscript_vm.set_inventory_script(arms_item_script);

            let stage_select_tsc = filesystem::open(ctx, [&state.base_path, "/StageSelect.tsc"].join(""))?;
            let stage_select_script = TextScript::load_from(stage_select_tsc, &state.constants)?;
            state.textscript_vm.set_stage_select_script(stage_select_script);

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
