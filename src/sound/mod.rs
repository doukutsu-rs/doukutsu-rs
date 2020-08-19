use std::io::{Cursor, Read};

use ggez::{Context, GameResult};

pub mod pixtone;

pub struct SoundManager {
    intro: Cursor<Vec<u8>>,
    sloop: Cursor<Vec<u8>>,
}

impl SoundManager {
    pub fn new() -> SoundManager {
        SoundManager {
            intro: Cursor::new(Vec::new()),
            sloop: Cursor::new(Vec::new()),
        }
    }

    pub fn play_song(&mut self, ctx: &mut Context) -> GameResult {
        /*self.intro.get_mut().clear();
        ggez::filesystem::open(ctx, "/Soundtracks/Arranged/oside_intro.ogg")?.read_to_end(self.intro.get_mut())?;

        let sink = rodio::play_once(ctx.audio_context.device(), self.intro.clone())?;
        sink.detach();*/

        Ok(())
    }
}
