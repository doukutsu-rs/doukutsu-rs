extern crate strum;
#[macro_use]
extern crate strum_macros;

use std::{env, mem};
use std::path;

use ggez::{Context, ContextBuilder, event, filesystem, GameResult};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::KeyCode;
use ggez::event::winit_event::{ElementState, Event, KeyboardInput, WindowEvent};
use ggez::graphics;
use ggez::graphics::DrawParam;
use ggez::input::keyboard;
use ggez::mint::ColumnMatrix4;
use ggez::nalgebra::Vector2;
use log::*;
use pretty_env_logger::env_logger::Env;

use crate::engine_constants::EngineConstants;
use crate::entity::GameEntity;
use crate::game_state::GameState;
use crate::game_state::KeyState;
use crate::live_debugger::LiveDebugger;
use crate::scene::game_scene::GameScene;
use crate::scene::loading_scene::LoadingScene;
use crate::scene::Scene;
use crate::sound::SoundManager;
use crate::stage::{Stage, StageData};
use crate::texture_set::TextureSet;
use crate::ui::UI;

mod common;
mod engine_constants;
mod entity;
mod enemy;
mod frame;
mod game_state;
mod live_debugger;
mod map;
mod player;
mod player_hit;
mod scene;
mod stage;
mod sound;
mod text_script;
mod texture_set;
mod ui;

struct Game {
    scene: Option<Box<dyn Scene>>,
    state: Option<GameState>,
    ctx: GameContext,
    ui: UI,
    scaled_matrix: ColumnMatrix4<f32>,
    def_matrix: ColumnMatrix4<f32>,
}

pub struct GameContext {
    pub texture_set: TextureSet,
    pub base_path: String,
    pub stages: Vec<StageData>,
    pub sound_manager: SoundManager,
    pub constants: EngineConstants,
    pub scale: f32,
    pub canvas_size: (f32, f32),
    pub screen_size: (f32, f32),
    pub next_scene: Option<Box<dyn Scene>>,
}

impl Game {
    fn new(ctx: &mut Context) -> GameResult<Game> {
        let scale = 2.0;
        let screen_size = graphics::drawable_size(ctx);
        let canvas_size = (screen_size.0 / scale, screen_size.1 / scale);
        let mut constants = EngineConstants::defaults();
        let mut base_path = "/";

        if filesystem::exists(ctx, "/base/Nicalis.bmp") {
            info!("Cave Story+ data files detected.");
            constants.apply_csplus_patches();
            base_path = "/base/";
        } else if filesystem::exists(ctx, "/mrmap.bin") || filesystem::exists(ctx, "/Font/font") {
            info!("CSE2E data files detected.");
        } else if filesystem::exists(ctx, "/stage.dat") || filesystem::exists(ctx, "/sprites.sif") {
            info!("NXEngine-evo data files detected.");
        }

        let s = Game {
            scene: None,
            state: None,
            ctx: GameContext {
                texture_set: TextureSet::new(base_path),
                base_path: str!(base_path),
                stages: Vec::new(),
                sound_manager: SoundManager::new(),
                constants,
                scale,
                screen_size,
                canvas_size,
                next_scene: None,
            },
            ui: UI::new(ctx)?,
            scaled_matrix: DrawParam::new()
                .scale(Vector2::new(scale, scale))
                .to_matrix(),
            def_matrix: DrawParam::new().to_matrix(),
        };

        Ok(s)
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if self.state.is_none() {
            self.state = Some(GameState::new(&mut self.ctx, ctx)?);
        }

        if self.scene.is_some() && self.state.is_some() {
            self.scene.as_mut().unwrap().tick(self.state.as_mut().unwrap(), &mut self.ctx, ctx)?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());
        graphics::set_transform(ctx, self.scaled_matrix);
        graphics::apply_transformations(ctx)?;

        if let (Some(scene), Some(state)) = (self.scene.as_mut(), self.state.as_mut()) {
            scene.draw(state, &mut self.ctx, ctx)?;

            graphics::set_transform(ctx, self.def_matrix);
            graphics::apply_transformations(ctx)?;

            self.ui.draw(state, &mut self.ctx, ctx, scene)?;
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, key_code: KeyCode, repeat: bool) {
        if repeat { return; }

        // todo: proper keymaps?
        if let Some(state) = self.state.as_mut() {
            match key_code {
                KeyCode::Left => { state.key_state.set_left(true) }
                KeyCode::Right => { state.key_state.set_right(true) }
                KeyCode::Up => { state.key_state.set_up(true) }
                KeyCode::Down => { state.key_state.set_down(true) }
                KeyCode::Z => { state.key_state.set_jump(true) }
                KeyCode::X => { state.key_state.set_fire(true) }
                KeyCode::A => { state.key_state.set_weapon_prev(true) }
                KeyCode::S => { state.key_state.set_weapon_next(true) }
                _ => {}
            }
        }
    }


    fn key_up_event(&mut self, _ctx: &mut Context, key_code: KeyCode) {
        if let Some(state) = self.state.as_mut() {
            match key_code {
                KeyCode::Left => { state.key_state.set_left(false) }
                KeyCode::Right => { state.key_state.set_right(false) }
                KeyCode::Up => { state.key_state.set_up(false) }
                KeyCode::Down => { state.key_state.set_down(false) }
                KeyCode::Z => { state.key_state.set_jump(false) }
                KeyCode::X => { state.key_state.set_fire(false) }
                KeyCode::A => { state.key_state.set_weapon_prev(false) }
                KeyCode::S => { state.key_state.set_weapon_next(false) }
                _ => {}
            }
        }
    }
}

pub fn main() -> GameResult {
    pretty_env_logger::env_logger::init_from_env(Env::default().default_filter_or("info"));

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("data");
        path
    } else {
        path::PathBuf::from(&env::var("CAVESTORY_DATA_DIR").unwrap_or(str!("data")))
    };

    info!("Resource directory: {:?}", resource_dir);
    info!("Initializing engine...");

    let cb = ContextBuilder::new("doukutsu-rs", "Alula")
        .window_setup(WindowSetup::default().title("Cave Story (doukutsu-rs)"))
        .window_mode(WindowMode::default().dimensions(854.0, 480.0))
        .add_resource_path(resource_dir);

    let (ctx, event_loop) = &mut cb.build()?;
    let game = &mut Game::new(ctx)?;
    game.ctx.next_scene = Some(Box::new(LoadingScene::new()));

    while ctx.continuing {
        ctx.timer_context.tick();
        event_loop.poll_events(|event| {
            ctx.process_event(&event);
            game.ui.handle_events(ctx, &event);

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => event::quit(ctx),
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            virtual_keycode: Some(keycode),
                            state: input_state,
                            ..
                        },
                        ..
                    } => {
                        match input_state {
                            ElementState::Pressed => {
                                let repeat = keyboard::is_key_repeated(ctx);
                                game.key_down_event(ctx, keycode, repeat);
                            }
                            ElementState::Released => {
                                game.key_up_event(ctx, keycode);
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        });

        game.update(ctx)?;
        game.draw(ctx)?;

        if game.ctx.next_scene.is_some() && game.state.is_some() {
            mem::swap(&mut game.scene, &mut game.ctx.next_scene);
            game.ctx.next_scene = None;
            game.scene.as_mut().unwrap().init(game.state.as_mut().unwrap(), &mut game.ctx, ctx)?;
        }
    }
    Ok(())
}
