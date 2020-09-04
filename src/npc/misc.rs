use crate::common::Direction;
use crate::ggez::GameResult;
use crate::npc::NPC;
use crate::SharedGameState;

impl NPC {
    pub(crate) fn tick_n000_null(&mut self, state: &mut SharedGameState) -> GameResult {
        Ok(())
    }


    pub(crate) fn tick_n016_save_point(&mut self, state: &mut SharedGameState) -> GameResult {
        Ok(())
    }

    pub(crate) fn tick_n017_health_refill(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
        }

        match self.action_num {
            1 => {
                let rand = state.game_rng.range(0..30);

                if rand < 10 {
                    self.action_num = 2;
                } else if rand < 25 {
                    self.action_num = 3;
                } else {
                    self.action_num = 4;
                }

                self.action_counter = state.game_rng.range(0x10..0x40) as u16;
                self.anim_counter = 0;
            }
            2 => {
                self.anim_rect = state.constants.npc.n017_health_refill[0];

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 1;
                }
            }
            3 => {
                self.anim_counter += 1;

                if self.anim_counter % 2 == 0 {
                    self.anim_rect = state.constants.npc.n017_health_refill[1];
                } else {
                    self.anim_rect = state.constants.npc.n017_health_refill[0];
                }

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 1;
                }
            }
            4 => {
                self.anim_rect = state.constants.npc.n017_health_refill[1];

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 1;
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub(crate) fn tick_n018_door(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 => {
                match self.direction {
                    Direction::Left => { self.anim_rect = state.constants.npc.n018_door_rects[0] }
                    Direction::Right => { self.anim_rect = state.constants.npc.n018_door_rects[1] }
                    _ => {}
                }
            }
            1 => {
                // todo smoke
                self.action_num = 0;
            }
            _ => {}
        }

        Ok(())
    }
}
