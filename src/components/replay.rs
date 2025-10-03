use std::io::{Cursor, Read};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};

use crate::entity::GameEntity;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::framework::keyboard::ScanCode;
use crate::framework::vfs::OpenOptions;
use crate::game::frame::Frame;
use crate::game::shared_game_state::{ReplayKind, ReplayState, SharedGameState};
use crate::input::replay_player_controller::{KeyState, ReplayController};
use crate::game::player::Player;
use crate::graphics::font::Font;

#[derive(Clone)]
pub struct Replay {
    replay_version: u16,
    keylist: Vec<u16>,
    last_input: KeyState,
    rng_seed: u64,
    pub controller: ReplayController,
    tick: usize,
    resume_tick: usize,
    is_active: bool,
}

impl Replay {
    pub fn new() -> Replay {
        Replay {
            replay_version: 0,
            keylist: Vec::new(),
            last_input: KeyState(0),
            rng_seed: 0,
            controller: ReplayController::new(),
            tick: 0,
            resume_tick: 0,
            is_active: false,
        }
    }

    pub fn initialize_recording(&mut self, state: &mut SharedGameState) {
        if !self.is_active {
            self.rng_seed = state.game_rng.dump_state();
            self.is_active = true;
        }
    }

    pub fn stop_recording(
        &mut self,
        state: &mut SharedGameState,
        ctx: &mut Context,
        is_new_record: bool,
    ) -> GameResult {
        state.replay_state = ReplayState::None;

        self.write_replay(state, ctx, ReplayKind::Last)?;

        if is_new_record {
            self.write_replay(state, ctx, ReplayKind::Best)?;
        }

        Ok(())
    }

    pub fn initialize_playback(
        &mut self,
        state: &mut SharedGameState,
        ctx: &mut Context,
        replay_kind: ReplayKind,
    ) -> GameResult {
        if !self.is_active {
            state.replay_state = ReplayState::Playback(replay_kind);
            self.read_replay(state, ctx, replay_kind)?;
            state.game_rng.load_state(self.rng_seed);
            self.is_active = true;
        }
        Ok(())
    }

    fn write_replay(&mut self, state: &mut SharedGameState, ctx: &mut Context, replay_kind: ReplayKind) -> GameResult {
        if let Ok(mut file) = filesystem::open_options(
            ctx,
            state.get_rec_filename(replay_kind.get_suffix()),
            OpenOptions::new().write(true).create(true),
        ) {
            file.write_u16::<LE>(0)?; // Space for versioning replay files
            file.write_u64::<LE>(self.rng_seed)?;
            for input in &self.keylist {
                file.write_u16::<LE>(*input)?;
            }
        }
        Ok(())
    }

    fn read_replay(&mut self, state: &mut SharedGameState, ctx: &mut Context, replay_kind: ReplayKind) -> GameResult {
        if let Ok(mut file) = filesystem::user_open(ctx, state.get_rec_filename(replay_kind.get_suffix()))
        {
            self.replay_version = file.read_u16::<LE>()?;
            self.rng_seed = file.read_u64::<LE>()?;

            let mut data = Vec::new();
            file.read_to_end(&mut data)?;

            let count = data.len() / 2;
            let mut inputs = Vec::new();
            let mut f = Cursor::new(data);

            for _ in 0..count {
                inputs.push(f.read_u16::<LE>()?);
            }

            self.keylist = inputs;
        }
        Ok(())
    }
}

impl GameEntity<(&mut Context, &mut Player)> for Replay {
    fn tick(&mut self, state: &mut SharedGameState, (ctx, player): (&mut Context, &mut Player)) -> GameResult {
        match state.replay_state {
            ReplayState::Recording => {
                // This mimics the KeyState bitfield
                let inputs = player.controller.move_left() as u16
                    + ((player.controller.move_right() as u16) << 1)
                    + ((player.controller.move_up() as u16) << 2)
                    + ((player.controller.move_down() as u16) << 3)
                    + ((player.controller.trigger_map() as u16) << 4)
                    + ((player.controller.trigger_inventory() as u16) << 5)
                    + (((player.controller.jump() || player.controller.trigger_menu_ok()) as u16) << 6)
                    + (((player.controller.shoot() || player.controller.trigger_menu_back()) as u16) << 7)
                    + ((player.controller.next_weapon() as u16) << 8)
                    + ((player.controller.prev_weapon() as u16) << 9)
                    + ((player.controller.trigger_menu_ok() as u16) << 11)
                    + ((player.controller.skip() as u16) << 12)
                    + ((player.controller.strafe() as u16) << 13);

                self.keylist.push(inputs);
            }
            ReplayState::Playback(_) => {
                let pause = ctx.keyboard_context.is_key_pressed(ScanCode::Escape) && (self.tick - self.resume_tick > 3);

                let next_input = if pause { 1 << 10 } else { *self.keylist.get(self.tick).unwrap_or(&0) };

                self.controller.state = KeyState(next_input);
                self.controller.old_state = self.last_input;
                player.controller = Box::new(self.controller);

                if !pause {
                    self.last_input = KeyState(next_input);
                    self.tick += 1;
                } else {
                    self.resume_tick = self.tick;
                };

                if self.tick >= self.keylist.len() {
                    state.replay_state = ReplayState::None;
                    player.controller = state.settings.create_player1_controller();
                }
            }
            ReplayState::None => {}
        }

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, _frame: &Frame) -> GameResult {
        let x = state.canvas_size.0 - 32.0;
        let y = 8.0 + if state.settings.fps_counter { 12.0 } else { 0.0 };

        match state.replay_state {
            ReplayState::None => {}
            ReplayState::Playback(_) => {
                state.font.builder()
                    .position(x, y)
                    .draw("PLAY", ctx, &state.constants, &mut state.texture_set)?;
            }
            ReplayState::Recording => {
                state.font.builder()
                    .position(x, y)
                    .draw("REC", ctx, &state.constants, &mut state.texture_set)?;
            }
        }

        Ok(())
    }
}
