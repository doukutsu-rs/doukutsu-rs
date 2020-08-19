use crate::ggez::{Context, GameResult};

pub mod pixtone;

pub struct SoundManager {
    intro: Vec<u8>,
    sloop: Vec<u8>,
}

//unsafe impl Send for SoundManager {}

impl SoundManager {
    pub fn new(ctx: &mut Context) -> SoundManager {
        SoundManager {
            intro: Vec::new(),
            sloop: Vec::new(),
        }
    }

    pub fn play_song(&mut self, ctx: &mut Context) -> GameResult {
        /*self.intro.clear();
        self.sloop.clear();
        ggez::filesystem::open(ctx, "/base/Ogg11/curly_intro.ogg")?.read_to_end(&mut self.intro)?;
        ggez::filesystem::open(ctx, "/base/Ogg11/curly_loop.ogg")?.read_to_end(&mut self.sloop)?;

        let sink = Sink::new(ctx.audio_context.device());
        sink.append(rodio::Decoder::new(Cursor::new(self.intro.clone()))?);
        sink.append(rodio::Decoder::new(Cursor::new(self.sloop.clone()))?);
        sink.detach();*/

        Ok(())
    }
}
