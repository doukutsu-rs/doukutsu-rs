use ggez::{Context, GameResult};
use log::info;

use crate::bitfield;
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::GameContext;
use crate::player::Player;
use crate::scene::game_scene::GameScene;
use crate::stage::Stage;

bitfield! {
  pub struct KeyState(u16);
  impl Debug;
  pub left, set_left: 0;
  pub right, set_right: 1;
  pub up, set_up: 2;
  pub down, set_down: 3;
  pub map, set_map: 4;
  pub jump, set_jump: 5;
  pub fire, set_fire: 6;
  pub weapon_next, set_weapon_next: 7;
  pub weapon_prev, set_weapon_prev: 8;
}

bitfield! {
  pub struct GameFlags(u32);
  impl Debug;
  pub flag_x01, set_flag_x01: 0;
  pub control_enabled, set_control_enabled: 1;
  pub flag_x04, set_flag_x04: 2;
}

pub struct GameState {
    pub tick: usize,
    pub stage: Stage,
    pub frame: Frame,
    pub flags: GameFlags,
    pub key_state: KeyState,
    pub key_trigger: KeyState,
    player: Player,
    key_old: u16,
}

impl GameState {
    pub fn new(game_ctx: &mut GameContext, ctx: &mut Context) -> GameResult<Self> {
        Ok(Self {
            tick: 0,
            stage: Stage::empty(),
            player: Player::new(&game_ctx.constants, ctx)?,
            frame: Frame {
                x: 0,
                y: 0,
                wait: 16,
            },
            flags: GameFlags(0),
            key_state: KeyState(0),
            key_trigger: KeyState(0),
            key_old: 0,
        })
    }

    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }

    pub fn update_key_trigger(&mut self) {
        let trigger = self.key_state.0 & (self.key_state.0 ^ self.key_old);
        self.key_old = self.key_state.0;
        self.key_trigger = KeyState(trigger);
    }

    pub fn switch_to_stage(&mut self, id: usize, game_ctx: &mut GameContext, ctx: &mut Context) -> GameResult {
        let stage = Stage::load(ctx, &game_ctx.base_path, &game_ctx.stages[id])?;
        info!("Loaded stage: {}", stage.data.name);
        info!("Map size: {}x{}", stage.map.width, stage.map.height);

        game_ctx.next_scene = Some(Box::new(GameScene::new(self, game_ctx, ctx)?));
        self.stage = stage;

        Ok(())
    }

    pub fn init(&mut self, game_ctx: &mut GameContext, ctx: &mut Context) -> GameResult {
        self.tick = 0;
        self.player = Player::new(&game_ctx.constants, ctx)?;

        self.player.x = 700 * 0x200;
        self.player.y = 1000 * 0x200;

        self.flags.set_flag_x01(true);
        self.flags.set_control_enabled(true);

        //game_ctx.sound_manager.play_song(ctx)?;
        Ok(())
    }
}
