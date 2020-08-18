extern crate strum;
#[macro_use]
extern crate strum_macros;

use std::{env, mem};
use std::path;

use ggez::{Context, ContextBuilder, event, GameResult};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{KeyCode, KeyMods};
use ggez::event::winit_event::{ElementState, Event, KeyboardInput, WindowEvent};
use ggez::graphics;
use ggez::graphics::DrawParam;
use ggez::input::keyboard;
use ggez::mint::ColumnMatrix4;
use ggez::nalgebra::Vector2;
use log::*;
use pretty_env_logger::env_logger::Env;

use crate::engine_constants::EngineConstants;
use crate::scene::loading_scene::LoadingScene;
use crate::scene::Scene;
use crate::stage::StageData;
use crate::texture_set::TextureSet;
use crate::sound::SoundManager;

mod common;
mod engine_constants;
mod entity;
mod enemy;
mod frame;
mod map;
mod player;
mod player_hit;
mod scene;
mod stage;
mod sound;
mod text_script;
mod texture_set;

bitfield! {
  pub struct KeyState(u16);
  impl Debug;
  left, set_left: 0;
  right, set_right: 1;
  up, set_up: 2;
  down, set_down: 3;
  map, set_map: 4;
  jump, set_jump: 5;
  fire, set_fire: 6;
  weapon_next, set_weapon_next: 7;
  weapon_prev, set_weapon_prev: 8;
}

bitfield! {
  pub struct GameFlags(u32);
  impl Debug;
  pub flag_x01, set_flag_x01: 0;
  pub control_enabled, set_control_enabled: 1;
  pub flag_x04, set_flag_x04: 2;
}

struct Game {
    scene: Option<Box<dyn Scene>>,
    state: SharedGameState,
    scaled_matrix: ColumnMatrix4<f32>,
    def_matrix: ColumnMatrix4<f32>,
}

pub struct SharedGameState {
    pub flags: GameFlags,
    pub key_state: KeyState,
    pub key_trigger: KeyState,
    pub texture_set: TextureSet,
    pub base_path: String,
    pub stages: Vec<StageData>,
    pub sound_manager: SoundManager,
    pub constants: EngineConstants,
    pub scale: f32,
    pub canvas_size: (f32, f32),
    pub screen_size: (f32, f32),
    pub next_scene: Option<Box<dyn Scene>>,
    key_old: u16,
}

impl SharedGameState {
    pub fn update_key_trigger(&mut self) {
        let mut trigger = self.key_state.0 ^ self.key_old;
        trigger = self.key_state.0 & trigger;
        self.key_old = self.key_state.0;
        self.key_trigger = KeyState(trigger);
    }
}

impl Game {
    fn new(ctx: &mut Context) -> GameResult<Game> {
        let scale = 2.0;
        let screen_size = graphics::drawable_size(ctx);
        let canvas_size = (screen_size.0 / scale, screen_size.1 / scale);

        let s = Game {
            scene: None,
            scaled_matrix: DrawParam::new()
                .scale(Vector2::new(scale, scale))
                .to_matrix(),
            def_matrix: DrawParam::new().to_matrix(),
            state: SharedGameState {
                flags: GameFlags(0),
                key_state: KeyState(0),
                key_trigger: KeyState(0),
                texture_set: TextureSet::new("/"),
                base_path: "/".to_string(),
                stages: Vec::new(),
                sound_manager: SoundManager::new(),
                constants: EngineConstants::defaults(),
                scale,
                screen_size,
                canvas_size,
                next_scene: None,
                key_old: 0,
            },
        };

        Ok(s)
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if self.scene.is_some() {
            self.scene.as_mut().unwrap().tick(&mut self.state, ctx)?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());
        graphics::set_transform(ctx, self.scaled_matrix);
        graphics::apply_transformations(ctx)?;

        if self.scene.is_some() {
            self.scene.as_ref().unwrap().draw(&mut self.state, ctx)?;

            graphics::set_transform(ctx, self.def_matrix);
            graphics::apply_transformations(ctx)?;
            self.scene.as_ref().unwrap().overlay_draw(&mut self.state, ctx)?;
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, key_code: KeyCode, _key_mod: KeyMods, repeat: bool) {
        if repeat { return; }

        // todo: proper keymaps?
        let state = &mut self.state;
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


    fn key_up_event(&mut self, _ctx: &mut Context, key_code: KeyCode, _key_mod: KeyMods) {
        let state = &mut self.state;
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

pub fn main() -> GameResult {
    pretty_env_logger::env_logger::init_from_env(Env::default().default_filter_or("info"));

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("data");
        path
    } else {
        path::PathBuf::from("./data")
    };

    info!("Resource directory: {:?}", resource_dir);
    info!("Initializing engine...");

    let cb = ContextBuilder::new("doukutsu-rs", "Alula")
        .window_setup(WindowSetup::default().title("Cave Story (doukutsu-rs)"))
        .window_mode(WindowMode::default().dimensions(854.0, 480.0))
        .add_resource_path(resource_dir);

    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut Game::new(ctx)?;
    state.state.next_scene = Some(Box::new(LoadingScene::new()));

    while ctx.continuing {
        ctx.timer_context.tick();
        event_loop.poll_events(|event| {
            ctx.process_event(&event);
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => event::quit(ctx),
                    WindowEvent::KeyboardInput {
                        input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(keycode),
                            modifiers,
                            ..
                        },
                        ..
                    } => {
                        let repeat = keyboard::is_key_repeated(ctx);
                        state.key_down_event(ctx, keycode, modifiers.into(), repeat);
                    }
                    WindowEvent::KeyboardInput {
                        input:
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(keycode),
                            modifiers,
                            ..
                        },
                        ..
                    } => {
                        state.key_up_event(ctx, keycode, modifiers.into());
                    }
                    _ => {}
                },
                _ => {}
            }
        });

        state.update(ctx)?;
        state.draw(ctx)?;

        if state.state.next_scene.is_some() {
            mem::swap(&mut state.scene, &mut state.state.next_scene);
            state.state.next_scene = None;

            state.scene.as_mut().unwrap().init(&mut state.state, ctx)?;
        }
    }
    Ok(())
}
