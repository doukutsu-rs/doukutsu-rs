use crate::common::Color;
use crate::framework::context::Context;
use crate::framework::error::{GameError, GameResult};
use crate::framework::graphics;
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
            use crate::util::browser;

            if !self.flag {
                self.flag = true;
                let _ = std::fs::create_dir("/sdcard/doukutsu/");
                let _ = std::fs::write("/sdcard/doukutsu/extract game data here.txt", REL_URL);
                let _ = std::fs::write("/sdcard/doukutsu/.nomedia", b"");
            }

            let screen = Rect::new(0, 0, state.canvas_size.0 as isize, state.canvas_size.1 as isize);
            if state.touch_controls.consume_click_in(screen) {
                if let Err(err) = browser::open(REL_URL) {
                    self.err = err.to_string();
                }
            }
        }
        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::from_rgb(30, 0, 0));

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

        let mut y = 60.0;
        #[cfg(target_os = "android")]
        {
            let yellow = (255, 255, 0, 255);
            state.font.builder().center(state.canvas_size.0).y(y).color(yellow).draw(
                "It's likely that you haven't extracted the game data properly.",
                ctx,
                &state.constants,
                &mut state.texture_set,
            )?;
            y += 20.0;
            state.font.builder().center(state.canvas_size.0).y(y).color(yellow).draw(
                "Click here to open the guide.",
                ctx,
                &state.constants,
                &mut state.texture_set,
            )?;
            y += 20.0;
            state.font.builder().center(state.canvas_size.0).y(y).color(yellow).draw(
                REL_URL,
                ctx,
                &state.constants,
                &mut state.texture_set,
            )?;
            y += 20.0;
        }

        {
            // put max 80 chars per line
            let mut lines = Vec::new();
            let mut line = String::new();

            for word in self.err.split(' ') {
                let combined_word = line.clone() + " " + word;
                let line_length = state.font.compute_width(&mut combined_word.chars(), None);

                if line_length > state.canvas_size.0 as f32 {
                    lines.push(line);
                    line = String::new();
                }

                line.push_str(word);
                line.push(' ');
            }
            lines.push(line);

            for line in lines {
                state.font.builder().center(state.canvas_size.0).y(y).draw(
                    &line,
                    ctx,
                    &state.constants,
                    &mut state.texture_set,
                )?;
                y += 20.0;
            }
        }

        Ok(())
    }
}
