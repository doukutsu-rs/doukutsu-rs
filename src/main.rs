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

use bitvec::vec::BitVec;
use log::*;
use pretty_env_logger::env_logger::Env;
use winit::{ElementState, Event, KeyboardInput, WindowEvent};

use crate::bmfont_renderer::BMFontRenderer;
use crate::builtin_fs::BuiltinFS;
use crate::caret::{Caret, CaretType};
use crate::common::{ControlFlags, Direction, FadeState, KeyState};
use crate::engine_constants::EngineConstants;
use crate::ggez::{Context, ContextBuilder, event, filesystem, GameResult};
use crate::ggez::conf::{WindowMode, WindowSetup};
use crate::ggez::event::{KeyCode, KeyMods};
use crate::ggez::graphics;
use crate::ggez::graphics::DrawParam;
use crate::ggez::input::keyboard;
use crate::ggez::mint::ColumnMatrix4;
use crate::ggez::nalgebra::Vector2;
use crate::npc::NPCTable;
use crate::rng::RNG;
use crate::scene::loading_scene::LoadingScene;
use crate::scene::Scene;
use crate::sound::SoundManager;
use crate::stage::StageData;
use crate::text_script::TextScriptVM;
use crate::texture_set::TextureSet;
use crate::ui::UI;

mod bmfont;
mod bmfont_renderer;
mod builtin_fs;
mod bullet;
mod caret;
mod common;
mod engine_constants;
mod entity;
mod frame;
mod inventory;
mod ggez;
mod live_debugger;
mod macros;
mod map;
mod npc;
mod physics;
mod player;
mod player_hit;
mod rng;
mod scene;
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
    scaled_matrix: ColumnMatrix4<f32>,
    def_matrix: ColumnMatrix4<f32>,
}

pub struct SharedGameState {
    pub control_flags: ControlFlags,
    pub game_flags: BitVec,
    pub fade_state: FadeState,
    pub game_rng: RNG,
    pub effect_rng: RNG,
    pub quake_counter: u16,
    pub carets: Vec<Caret>,
    pub key_state: KeyState,
    pub key_trigger: KeyState,
    pub font: BMFontRenderer,
    pub texture_set: TextureSet,
    pub base_path: String,
    pub npc_table: NPCTable,
    pub stages: Vec<StageData>,
    pub sound_manager: SoundManager,
    pub constants: EngineConstants,
    pub scale: f32,
    pub god_mode: bool,
    pub speed_hack: bool,
    pub canvas_size: (f32, f32),
    pub screen_size: (f32, f32),
    pub next_scene: Option<Box<dyn Scene>>,
    pub textscript_vm: TextScriptVM,
    key_old: u16,
}

impl SharedGameState {
    pub fn update_key_trigger(&mut self) {
        let mut trigger = self.key_state.0 ^ self.key_old;
        trigger &= self.key_state.0;
        self.key_old = self.key_state.0;
        self.key_trigger = KeyState(trigger);
    }

    pub fn tick_carets(&mut self) {
        for caret in self.carets.iter_mut() {
            caret.tick(&self.effect_rng, &self.constants);
        }

        self.carets.retain(|c| !c.is_dead());
    }

    pub fn create_caret(&mut self, x: isize, y: isize, ctype: CaretType, direct: Direction) {
        self.carets.push(Caret::new(x, y, ctype, direct, &self.constants));
    }

    pub fn set_speed_hack(&mut self, toggle: bool) {
        self.speed_hack = toggle;

        if let Err(err) = self.sound_manager.set_speed(if toggle { 2.0 } else { 1.0 }) {
            log::error!("Error while sending a message to sound manager: {}", err);
        }
    }
}

impl Game {
    fn new(ctx: &mut Context) -> GameResult<Game> {
        let scale = 2.0;
        let screen_size = graphics::drawable_size(ctx);
        let canvas_size = (screen_size.0 / scale, screen_size.1 / scale);
        let mut constants = EngineConstants::defaults();
        let mut base_path = "/";

        if filesystem::exists(ctx, "/base/Nicalis.bmp") {
            info!("Cave Story+ (PC) data files detected.");
            constants.apply_csplus_patches();
            base_path = "/base/";
        } else if filesystem::exists(ctx, "/base/lighting.tbl") {
            info!("Cave Story+ (Switch) data files detected.");
            constants.apply_csplus_patches();
            constants.apply_csplus_nx_patches();
            base_path = "/base/";
        } else if filesystem::exists(ctx, "/mrmap.bin") {
            info!("CSE2E data files detected.");
        } else if filesystem::exists(ctx, "/stage.dat") {
            info!("NXEngine-evo data files detected.");
        }
        let font = BMFontRenderer::load(base_path, &constants.font_path, ctx)?;
        //.or_else(|| Some(BMFontRenderer::load("/", "builtin/builtin_font.fnt", ctx)?))
        //.ok_or_else(|| ResourceLoadError(str!("Cannot load game font.")))?;

        let s = Game {
            scene: None,
            scaled_matrix: DrawParam::new()
                .scale(Vector2::new(scale, scale))
                .to_matrix(),
            ui: UI::new(ctx)?,
            def_matrix: DrawParam::new().to_matrix(),
            state: SharedGameState {
                control_flags: ControlFlags(0),
                game_flags: bitvec::bitvec![0; 8000],
                fade_state: FadeState::Hidden,
                game_rng: RNG::new(0),
                effect_rng: RNG::new(Instant::now().elapsed().as_nanos() as i32),
                quake_counter: 0,
                carets: Vec::with_capacity(32),
                key_state: KeyState(0),
                key_trigger: KeyState(0),
                font,
                texture_set: TextureSet::new(base_path),
                base_path: str!(base_path),
                npc_table: NPCTable::new(),
                stages: Vec::with_capacity(96),
                sound_manager: SoundManager::new(ctx)?,
                constants,
                scale,
                god_mode: false,
                speed_hack: false,
                screen_size,
                canvas_size,
                next_scene: None,
                textscript_vm: TextScriptVM::new(),
                key_old: 0,
            },
        };

        Ok(s)
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if let Some(scene) = self.scene.as_mut() {
            scene.tick(&mut self.state, ctx)?;
            if self.state.speed_hack {
                scene.tick(&mut self.state, ctx)?;
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());
        graphics::set_transform(ctx, self.scaled_matrix);
        graphics::apply_transformations(ctx)?;

        if let Some(scene) = self.scene.as_mut() {
            scene.draw(&mut self.state, ctx)?;

            graphics::set_transform(ctx, self.def_matrix);
            graphics::apply_transformations(ctx)?;
            self.ui.draw(&mut self.state, ctx, scene)?;
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
        .window_mode(WindowMode::default().dimensions(854.0, 480.0))
        .add_resource_path(resource_dir);

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
                    WindowEvent::CloseRequested => event::quit(ctx),
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

        if game.state.next_scene.is_some() {
            mem::swap(&mut game.scene, &mut game.state.next_scene);
            game.state.next_scene = None;

            game.scene.as_mut().unwrap().init(&mut game.state, ctx)?;
        }
    }

    Ok(())
}
