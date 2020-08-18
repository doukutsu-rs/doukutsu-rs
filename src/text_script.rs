use ggez::{Context, GameResult};

struct TextScript {

}

impl TextScript {
    pub fn load(ctx: &mut Context, filename: &str) -> GameResult<TextScript> {
        let tsc = TextScript {};
        Ok(tsc)
    }
}
