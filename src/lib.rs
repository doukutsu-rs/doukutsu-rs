#[macro_use]
extern crate log;
#[cfg_attr(feature = "scripting", macro_use)]
#[cfg(feature = "scripting")]
extern crate lua_ffi;
extern crate strum;
#[macro_use]
extern crate strum_macros;

use std::cell::UnsafeCell;
use std::env;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use directories::ProjectDirs;
use lazy_static::lazy_static;

use crate::builtin_fs::BuiltinFS;
use crate::framework::context::Context;
use crate::framework::error::{GameError, GameResult};
use crate::framework::filesystem::{mount_user_vfs, mount_vfs};
use crate::framework::graphics;
use crate::framework::ui::UI;
use crate::framework::vfs::PhysicalFS;
use crate::scene::loading_scene::LoadingScene;
use crate::scene::Scene;
use crate::shared_game_state::{SharedGameState, TimingMode};
use crate::texture_set::{G_MAG, I_MAG};

mod bmfont;
mod bmfont_renderer;
mod builtin_fs;
mod caret;
mod common;
mod components;
mod encoding;
mod engine_constants;
mod entity;
mod frame;
mod framework;
mod input;
mod inventory;
#[cfg(feature = "hooks")]
mod hooks;
mod live_debugger;
mod macros;
mod map;
mod menu;
#[cfg(feature = "netplay")]
mod netplay;
mod npc;
mod physics;
mod player;
mod profile;
mod rng;
mod scene;
#[cfg(feature = "scripting")]
mod scripting;
mod settings;
#[cfg(feature = "backend-gfx")]
mod shaders;
mod shared_game_state;
mod sound;
mod stage;
mod text_script;
mod texture_set;
mod weapon;

lazy_static! {
    pub static ref GAME_SUSPENDED: Mutex<bool> = Mutex::new(false);
}

pub struct Game {
    scene: Option<Box<dyn Scene>>,
    state: UnsafeCell<SharedGameState>,
    ui: UI,
    start_time: Instant,
    last_tick: u128,
    next_tick: u128,
    loops: u64,
}

impl Game {
    fn new(ctx: &mut Context) -> GameResult<Game> {
        let s = Game {
            scene: None,
            ui: UI::new(ctx)?,
            state: UnsafeCell::new(SharedGameState::new(ctx)?),
            start_time: Instant::now(),
            last_tick: 0,
            next_tick: 0,
            loops: 0,
        };

        Ok(s)
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if let Some(scene) = self.scene.as_mut() {
            let state_ref = unsafe { &mut *self.state.get() };

            match state_ref.timing_mode {
                TimingMode::_50Hz | TimingMode::_60Hz => {
                    let last_tick = self.next_tick;

                    while self.start_time.elapsed().as_nanos() >= self.next_tick && self.loops < 10 {
                        if (state_ref.settings.speed - 1.0).abs() < 0.01 {
                            self.next_tick += state_ref.timing_mode.get_delta() as u128;
                        } else {
                            self.next_tick +=
                                (state_ref.timing_mode.get_delta() as f64 / state_ref.settings.speed) as u128;
                        }
                        self.loops += 1;
                    }

                    if self.loops == 10 {
                        log::warn!("Frame skip is way too high, a long system lag occurred?");
                        self.last_tick = self.start_time.elapsed().as_nanos();
                        self.next_tick = self.last_tick
                            + (state_ref.timing_mode.get_delta() as f64 / state_ref.settings.speed) as u128;
                        self.loops = 0;
                    }

                    if self.loops != 0 {
                        scene.draw_tick(state_ref)?;
                        self.last_tick = last_tick;
                    }

                    for _ in 0..self.loops {
                        scene.tick(state_ref, ctx)?;
                    }
                }
                TimingMode::FrameSynchronized => {
                    scene.tick(state_ref, ctx)?;
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let state_ref = unsafe { &mut *self.state.get() };

        if state_ref.timing_mode != TimingMode::FrameSynchronized {
            let mut elapsed = self.start_time.elapsed().as_nanos();

            // Even with the non-monotonic Instant mitigation at the start of the event loop, there's still a chance of it not working.
            // This check here should trigger if that happens and makes sure there's no panic from an underflow.
            if elapsed < self.last_tick {
                elapsed = self.last_tick;
            }

            let n1 = (elapsed - self.last_tick) as f64;
            let n2 = (self.next_tick - self.last_tick) as f64;
            state_ref.frame_time = if state_ref.settings.motion_interpolation { n1 / n2 } else { 1.0 };
        }
        unsafe {
            G_MAG = if state_ref.settings.subpixel_coords { state_ref.scale } else { 1.0 };
            I_MAG = state_ref.scale;
        }
        self.loops = 0;

        graphics::prepare_draw(ctx)?;
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        if let Some(scene) = self.scene.as_mut() {
            scene.draw(state_ref, ctx)?;
            if state_ref.settings.touch_controls {
                state_ref.touch_controls.draw(
                    state_ref.canvas_size,
                    state_ref.scale,
                    &state_ref.constants,
                    &mut state_ref.texture_set,
                    ctx,
                )?;
            }

            self.ui.draw(state_ref, ctx, scene)?;
        }

        graphics::present(ctx)?;

        Ok(())
    }
}

pub fn init() -> GameResult {
    let _ = simple_logger::init_with_level(log::Level::Info);

    #[cfg(not(target_os = "android"))]
    let resource_dir = if let Ok(data_dir) = env::var("CAVESTORY_DATA_DIR") {
        PathBuf::from(data_dir)
    } else {
        let mut resource_dir = env::current_exe()?;
        if resource_dir.file_name().is_some() {
            let _ = resource_dir.pop();
        }

        #[cfg(target_os = "macos")]
        {
            let mut bundle_dir = resource_dir.clone();
            let _ = bundle_dir.pop();
            let mut bundle_exec_dir = bundle_dir.clone();

            bundle_exec_dir.push("MacOS");
            bundle_dir.push("Resources");

            if bundle_exec_dir.is_dir() && bundle_dir.is_dir() {
                log::info!("Running in macOS bundle mode");
                resource_dir = bundle_dir
            }
        }

        resource_dir.push("data");
        resource_dir
    };

    #[cfg(not(target_os = "android"))]
    log::info!("Resource directory: {:?}", resource_dir);
    log::info!("Initializing engine...");

    let mut context = Context::new();
    mount_vfs(&mut context, Box::new(BuiltinFS::new()));

    #[cfg(not(target_os = "android"))]
    mount_vfs(&mut context, Box::new(PhysicalFS::new(&resource_dir, true)));

    #[cfg(not(target_os = "android"))]
    let project_dirs = match ProjectDirs::from("", "", "doukutsu-rs") {
        Some(dirs) => dirs,
        None => {
            return Err(GameError::FilesystemError(String::from("No valid home directory path could be retrieved.")));
        }
    };
    #[cfg(target_os = "android")]
    {
        let mut data_path =
            PathBuf::from(ndk_glue::native_activity().internal_data_path().to_string_lossy().to_string());
        let mut user_path = data_path.clone();

        data_path.push("data");
        user_path.push("saves");

        let _ = std::fs::create_dir_all(&data_path);
        let _ = std::fs::create_dir_all(&user_path);

        log::info!("Android data directories: data_path={:?} user_path={:?}", &data_path, &user_path);

        mount_vfs(&mut context, Box::new(PhysicalFS::new(&data_path, true)));
        mount_user_vfs(&mut context, Box::new(PhysicalFS::new(&user_path, false)));
    }

    #[cfg(not(target_os = "android"))]
        {
            if let Ok(_) = crate::framework::filesystem::open(&mut context, "/.drs_localstorage") {
                let mut user_dir = resource_dir.clone();
                user_dir.push("_drs_profile");

                let _ = std::fs::create_dir_all(&user_dir);
                mount_user_vfs(&mut context, Box::new(PhysicalFS::new(&user_dir, false)));
            } else {
                mount_user_vfs(&mut context, Box::new(PhysicalFS::new(project_dirs.data_local_dir(), false)));
            }
        }

    let game = UnsafeCell::new(Game::new(&mut context)?);
    let state_ref = unsafe { &mut *((&mut *game.get()).state.get()) };
    #[cfg(feature = "scripting")]
    {
        state_ref.lua.update_refs(unsafe { (&*game.get()).state.get() }, &mut context as *mut Context);
    }

    state_ref.next_scene = Some(Box::new(LoadingScene::new()));
    context.run(unsafe { &mut *game.get() })?;

    Ok(())
}
