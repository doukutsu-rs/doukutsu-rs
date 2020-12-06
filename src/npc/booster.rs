use ggez::GameResult;

use crate::npc::NPC;
use crate::shared_game_state::SharedGameState;

impl NPC {
    pub(crate) fn tick_n113_professor_booster(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
