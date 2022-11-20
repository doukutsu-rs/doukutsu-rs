use crate::framework::context::Context;
use crate::framework::error::{GameError, GameResult};
use crate::game::shared_game_state::SharedGameState;
use crate::graphics::font::Font;
use crate::scene::Scene;

pub struct NoDataScene {
    #[cfg(target_os = "android")]
    flag: bool,
    err: String,
}

impl NoDataScene {
    pub fn new(err: GameError) -> Self {
        Self {
            #[cfg(target_os = "android")]
            flag: false,
            err: err.to_string(),
        }
    }
}

#[cfg(target_os = "android")]
static REL_URL: &str = "https://github.com/doukutsu-rs/doukutsu-rs#data-files";

impl Scene for NoDataScene {
    #[allow(unused)]
    fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        #[cfg(target_os = "android")]
        {
            use crate::common::Rect;

            if !self.flag {
                self.flag = true;
                let _ = std::fs::create_dir("/sdcard/doukutsu/");
                let _ = std::fs::write("/sdcard/doukutsu/extract game data here.txt", REL_URL);
                let _ = std::fs::write("/sdcard/doukutsu/.nomedia", b"");
            }

            let screen = Rect::new(0, 0, state.canvas_size.0 as isize, state.canvas_size.1 as isize);
            if state.touch_controls.consume_click_in(screen) {
                if let Err(err) = webbrowser::open(REL_URL) {
                    self.err = err.to_string();
                }
            }
        }
        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        state.font.builder().center(state.canvas_size.0).y(10.0).color((255, 100, 100, 255)).draw(
            "doukutsu-rs internal error",
            ctx,
            &state.constants,
            &mut state.texture_set,
        )?;

        state.font.builder().center(state.canvas_size.0).y(30.0).color((255, 100, 100, 255)).draw(
            "Failed to load game data.",
            ctx,
            &state.constants,
            &mut state.texture_set,
        )?;

        #[cfg(target_os = "android")]
        {
            let yellow = (255, 255, 0, 255);
            state.font.builder().center(state.canvas_size.0).y(60.0).color(yellow).draw(
                "It's likely that you haven't extracted the game data properly.",
                ctx,
                &state.constants,
                &mut state.texture_set,
            )?;
            state.font.builder().center(state.canvas_size.0).y(80.0).color(yellow).draw(
                "Click here to open the guide.",
                ctx,
                &state.constants,
                &mut state.texture_set,
            )?;
            state.font.builder().center(state.canvas_size.0).y(100.0).color(yellow).draw(
                REL_URL,
                ctx,
                &state.constants,
                &mut state.texture_set,
            )?;
        }

        {
            state.font.builder().center(state.canvas_size.0).y(140.0).draw(
                &self.err,
                ctx,
                &state.constants,
                &mut state.texture_set,
            )?;
        }

        Ok(())
    }
}
