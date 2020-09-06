use crate::ggez::GameResult;
use crate::npc::NPC;
use crate::player::Player;
use crate::SharedGameState;

impl NPC {
    pub(crate) fn tick_n059_eye_door(&mut self, state: &mut SharedGameState, player: &Player) -> GameResult {
        self.npc_flags.set_event_when_touched(true);

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                }

                self.anim_rect = state.constants.npc.n059_eye_door[self.anim_num as usize];
            }
            2 => {}
            3 => {}
            _ => {}
        }

        Ok(())
    }
}
