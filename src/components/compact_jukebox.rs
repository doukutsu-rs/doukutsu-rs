use crate::entity::GameEntity;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::game::frame::Frame;
use crate::game::shared_game_state::SharedGameState;
use crate::graphics::font::Font;

#[derive(Clone, Copy)]
pub struct CompactJukebox {
    song_id: usize,
    shown: bool,
}

impl CompactJukebox {
    pub fn new() -> CompactJukebox {
        CompactJukebox { song_id: 0, shown: false }
    }

    pub fn change_song(&mut self, song_id: usize, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.song_id = song_id;

        if self.song_id == state.sound_manager.current_song() {
            return Ok(());
        }

        return state.sound_manager.play_song(song_id, &state.constants, &state.settings, ctx, false);
    }

    pub fn next_song(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let mut new_song_id = if self.song_id == state.constants.music_table.len() - 1 { 1 } else { self.song_id + 1 };

        // skip ika if soundtrack is not set to new
        if self.is_ika_unavailable(new_song_id, state) {
            new_song_id = 1;
        }

        self.change_song(new_song_id, state, ctx)
    }

    pub fn prev_song(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let mut new_song_id = if self.song_id == 1 { state.constants.music_table.len() - 1 } else { self.song_id - 1 };

        // skip ika if soundtrack is not set to new
        if self.is_ika_unavailable(new_song_id, state) {
            new_song_id = 42;
        }

        self.change_song(new_song_id, state, ctx)
    }

    pub fn show(&mut self) {
        self.shown = true;
    }

    pub fn is_shown(&self) -> bool {
        self.shown
    }

    fn is_ika_unavailable(&self, song_id: usize, state: &SharedGameState) -> bool {
        song_id == 43 && state.settings.soundtrack != "New"
    }
}

impl GameEntity<&mut Context> for CompactJukebox {
    fn tick(&mut self, _state: &mut SharedGameState, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, _frame: &Frame) -> GameResult {
        if !self.shown {
            return Ok(());
        }

        let text = format!("< {:02} >", self.song_id);

        let font_builder = state.font.builder();
        let text_width = font_builder.compute_width(&text);

        let x = state.canvas_size.0 as f32 - text_width - 15.0;
        let y = state.canvas_size.1 as f32 - 15.0;

        font_builder.x(x).y(y).shadow(true).draw(&text, ctx, &state.constants, &mut state.texture_set)?;

        Ok(())
    }
}
