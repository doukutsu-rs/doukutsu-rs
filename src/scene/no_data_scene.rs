use crate::framework::context::Context;
use crate::framework::error::{GameResult, GameError};

use crate::common::Rect;
use crate::scene::Scene;
use crate::shared_game_state::SharedGameState;

pub struct NoDataScene {
    flag: bool,
    err: String,
}

impl NoDataScene {
    pub fn new(err: GameError) -> Self {
        Self {
            flag: false,
            err: err.to_string(),
        }
    }
}

#[cfg(target_os = "android")]
static REL_URL: &str = "https://github.com/doukutsu-rs/game-data/releases";

impl Scene for NoDataScene {
    fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {

        #[cfg(target_os = "android")]
            {
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
        {
            let die = "doukutsu-rs internal error";
            let die_width = state.font.text_width(die.chars().clone(), &state.constants);
            state.font.draw_colored_text(die.chars(), (state.canvas_size.0 - die_width) / 2.0, 10.0,
                                         (255, 100, 100, 255), &state.constants, &mut state.texture_set, ctx)?;
        }

        {
            let ftl = "Failed to load game data.";
            let ftl_width = state.font.text_width(ftl.chars().clone(), &state.constants);
            state.font.draw_colored_text(ftl.chars(), (state.canvas_size.0 - ftl_width) / 2.0, 30.0,
                                         (255, 100, 100, 255), &state.constants, &mut state.texture_set, ctx)?;
        }

        #[cfg(target_os = "android")]
            {
                let ftl = "It's likely that you haven't extracted the game data properly.";
                let ftl2 = "Click here to open the guide.";
                let ftl_width = state.font.text_width(ftl.chars().clone(), &state.constants);
                let ftl2_width = state.font.text_width(ftl2.chars().clone(), &state.constants);
                let ftl3_width = state.font.text_width(REL_URL.chars().clone(), &state.constants);

                state.font.draw_colored_text(ftl.chars(), (state.canvas_size.0 - ftl_width) / 2.0, 60.0,
                                             (255, 255, 0, 255), &state.constants, &mut state.texture_set, ctx)?;


                state.font.draw_colored_text(ftl2.chars(), (state.canvas_size.0 - ftl2_width) / 2.0, 80.0,
                                             (255, 255, 0, 255), &state.constants, &mut state.texture_set, ctx)?;

                state.font.draw_colored_text(REL_URL.chars(), (state.canvas_size.0 - ftl3_width) / 2.0, 100.0,
                                             (255, 255, 0, 255), &state.constants, &mut state.texture_set, ctx)?;
            }

        {
            let err_width = state.font.text_width(self.err.chars().clone(), &state.constants);
            state.font.draw_text(self.err.chars(), (state.canvas_size.0 - err_width) / 2.0, 140.0,
                                 &state.constants, &mut state.texture_set, ctx)?;
        }


        Ok(())
    }
}
