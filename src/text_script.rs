use crate::ggez::{Context, GameResult};

struct TextScript {

}

impl TextScript {
    pub fn load(filename: &str, ctx: &mut Context) -> GameResult<TextScript> {
        let tsc = TextScript {};
        Ok(tsc)
    }
}
