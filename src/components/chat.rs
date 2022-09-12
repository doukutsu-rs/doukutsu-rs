use std::collections::LinkedList;

use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::shared_game_state::SharedGameState;

struct MessageData {
    content: String,
    fade: u16,
}

pub struct Chat {
    messages: LinkedList<MessageData>,
    max_messages: usize,
}

impl Chat {
    pub fn new() -> Chat {
        Chat { messages: LinkedList::new(), max_messages: 50 }
    }

    pub fn push_message(&mut self, content: String) {
        self.messages.push_front(MessageData { content, fade: 300 });

        while self.messages.len() > self.max_messages {
            self.messages.pop_back();
        }
    }
}

impl GameEntity<()> for Chat {
    fn tick(&mut self, state: &mut SharedGameState, custom: ()) -> GameResult {
        for message in self.messages.iter_mut() {
            if message.fade > 0 {
                message.fade -= 1;
            }
        }

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, _frame: &Frame) -> GameResult {
        let mut ctr = 8;
        let font_height = state.font.line_height(&state.constants);

        for message in self.messages.iter() {
            if message.fade > 0 {
                let fade = message.fade.min(50) as u8;

                state.font.draw_colored_text_with_shadow_scaled(
                    message.content.chars(),
                    2.0,
                    2.0 + ctr as f32 * font_height,
                    1.0,
                    (255, 255, 255, 5 + fade * 5),
                    &state.constants,
                    &mut state.texture_set,
                    ctx,
                )?;
            }

            if ctr > 0 {
                ctr -= 1;
            } else {
                break;
            }
        }

        Ok(())
    }
}
