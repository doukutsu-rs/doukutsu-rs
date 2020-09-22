#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate gfx;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate smart_default;
extern crate strum;
#[macro_use]
extern crate strum_macros;

use std::{env, mem};
use std::path;
use std::time::Instant;

use log::*;
use pretty_env_logger::env_logger::Env;
use winit::{ElementState, Event, KeyboardInput, WindowEvent};

use crate::builtin_fs::BuiltinFS;
use crate::ggez::{Context, ContextBuilder, filesystem, GameResult};
use crate::ggez::conf::{WindowMode, WindowSetup};
use crate::ggez::event::{KeyCode, KeyMods};
use crate::ggez::graphics;
use crate::ggez::graphics::DrawParam;
use crate::ggez::input::keyboard;
use crate::ggez::mint::ColumnMatrix4;
use crate::ggez::nalgebra::Vector2;
use crate::scene::loading_scene::LoadingScene;
use crate::scene::Scene;
use crate::shared_game_state::{SharedGameState, TimingMode};
use crate::ui::UI;

mod bmfont;
mod bmfont_renderer;
mod builtin_fs;
mod bullet;
mod caret;
mod common;
mod encoding;
mod engine_constants;
mod entity;
mod frame;
mod inventory;
mod ggez;
mod live_debugger;
mod macros;
mod map;
mod menu;
mod npc;
mod physics;
mod player;
mod player_hit;
mod profile;
mod rng;
mod scene;
mod shared_game_state;
mod stage;
mod sound;
mod text_script;
mod texture_set;
mod ui;
mod weapon;

struct Game {
    scene: Option<Box<dyn Scene>>,
    state: SharedGameState,
    ui: UI,
    def_matrix: ColumnMatrix4<f32>,
    start_time: Instant,
    next_tick: u64,
    loops: u64,
}

impl Game {
    fn new(ctx: &mut Context) -> GameResult<Game> {
        let s = Game {
            scene: None,
            ui: UI::new(ctx)?,
            def_matrix: DrawParam::new().to_matrix(),
            state: SharedGameState::new(ctx)?,
            start_time: Instant::now(),
            next_tick: 0,
            loops: 0,
        };

        Ok(s)
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if let Some(scene) = self.scene.as_mut() {
            match self.state.timing_mode {
                TimingMode::_50Hz | TimingMode::_60Hz => {
                    while self.start_time.elapsed().as_millis() as u64 > self.next_tick && self.loops < 3 {
                        self.next_tick += self.state.timing_mode.get_delta() as u64;
                        self.loops += 1;
                    }

                    for _ in 0..self.loops {
                        scene.tick(&mut self.state, ctx)?;
                        if self.state.speed_hack {
                            scene.tick(&mut self.state, ctx)?;
                        }
                    }
                }
                TimingMode::FrameSynchronized => {
                    scene.tick(&mut self.state, ctx)?;
                    if self.state.speed_hack {
                        scene.tick(&mut self.state, ctx)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());
        graphics::set_transform(ctx, DrawParam::new()
            .scale(Vector2::new(self.state.scale, self.state.scale))
            .to_matrix());
        graphics::apply_transformations(ctx)?;

        if let Some(scene) = self.scene.as_mut() {
            scene.draw(&mut self.state, ctx)?;

            graphics::set_transform(ctx, self.def_matrix);
            graphics::apply_transformations(ctx)?;
            self.ui.draw(&mut self.state, ctx, scene)?;
        }

        graphics::present(ctx)?;
        self.loops = 0;
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
            KeyCode::F11 => { state.god_mode = !state.god_mode }
            KeyCode::F12 => { state.set_speed_hack(!state.speed_hack) }
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
        path::PathBuf::from(&env::var("CAVESTORY_DATA_DIR").unwrap_or(str!("data")))
    };

    info!("Resource directory: {:?}", resource_dir);
    info!("Initializing engine...");

    let cb = ContextBuilder::new("doukutsu-rs")
        .window_setup(WindowSetup::default().title("Cave Story (doukutsu-rs)"))
        .window_mode(WindowMode::default()
            .resizable(true)
            .min_dimensions(320.0, 240.0)
            .dimensions(854.0, 480.0))
        .add_resource_path(resource_dir)
        .add_resource_path(path::PathBuf::from(str!("./")));

    let (ctx, event_loop) = &mut cb.build()?;
    ctx.filesystem.mount_vfs(Box::new(BuiltinFS::new()));

    let game = &mut Game::new(ctx)?;
    game.state.next_scene = Some(Box::new(LoadingScene::new()));

    while ctx.continuing {
        ctx.timer_context.tick();
        event_loop.poll_events(|event| {
            ctx.process_event(&event);
            game.ui.handle_events(ctx, &event);

            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => { game.state.shutdown(); }
                    WindowEvent::Resized(_) => {
                        game.state.handle_resize(ctx).unwrap();
                        gfx_window_glutin::update_views(graphics::window(ctx), &mut game.ui.main_color, &mut game.ui.main_depth);
                    }
                    WindowEvent::KeyboardInput {
                        input:
                        KeyboardInput {
                            state: el_state,
                            virtual_keycode: Some(keycode),
                            modifiers,
                            ..
                        },
                        ..
                    } => {
                        match el_state {
                            ElementState::Pressed => {
                                let repeat = keyboard::is_key_repeated(ctx);
                                game.key_down_event(ctx, keycode, modifiers.into(), repeat);
                            }
                            ElementState::Released => {
                                game.key_up_event(ctx, keycode, modifiers.into());
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        game.update(ctx)?;
        game.draw(ctx)?;

        if game.state.shutdown {
            log::info!("Shutting down...");
            break;
        }

        if game.state.next_scene.is_some() {
            mem::swap(&mut game.scene, &mut game.state.next_scene);
            game.state.next_scene = None;

            game.scene.as_mut().unwrap().init(&mut game.state, ctx)?;
        }
    }

    Ok(())
}
