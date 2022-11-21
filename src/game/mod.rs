use std::cell::UnsafeCell;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use lazy_static::lazy_static;

use scripting::tsc::text_script::ScriptMode;

use crate::data::builtin_fs::BuiltinFS;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem::{mount_user_vfs, mount_vfs};
use crate::framework::graphics;
use crate::framework::graphics::VSyncMode;
use crate::framework::ui::UI;
use crate::framework::vfs::PhysicalFS;
use crate::game::shared_game_state::{Fps, SharedGameState, TimingMode};
use crate::graphics::texture_set::{G_MAG, I_MAG};
use crate::scene::loading_scene::LoadingScene;
use crate::scene::Scene;

pub mod caret;
pub mod frame;
pub mod inventory;
pub mod map;
pub mod npc;
pub mod physics;
pub mod player;
pub mod profile;
pub mod scripting;
pub mod settings;
pub mod shared_game_state;
pub mod stage;
pub mod weapon;

pub struct LaunchOptions {
    pub server_mode: bool,
    pub editor: bool,
}

lazy_static! {
    pub static ref GAME_SUSPENDED: Mutex<bool> = Mutex::new(false);
}

pub struct Game {
    pub(crate) scene: Option<Box<dyn Scene>>,
    pub(crate) state: UnsafeCell<SharedGameState>,
    ui: UI,
    start_time: Instant,
    last_tick: u128,
    next_tick: u128,
    pub(crate) loops: u32,
    next_tick_draw: u128,
    present: bool,
    fps: Fps,
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
            next_tick_draw: 0,
            present: true,
            fps: Fps::new(),
        };

        Ok(s)
    }

    pub(crate) fn update(&mut self, ctx: &mut Context) -> GameResult {
        if let Some(scene) = &mut self.scene {
            let state_ref = unsafe { &mut *self.state.get() };

            let speed =
                if state_ref.textscript_vm.mode == ScriptMode::Map && state_ref.textscript_vm.flags.cutscene_skip() {
                    4.0 * state_ref.settings.speed
                } else {
                    1.0 * state_ref.settings.speed
                };

            match state_ref.settings.timing_mode {
                TimingMode::_50Hz | TimingMode::_60Hz => {
                    let last_tick = self.next_tick;

                    while self.start_time.elapsed().as_nanos() >= self.next_tick && self.loops < 10 {
                        if (speed - 1.0).abs() < 0.01 {
                            self.next_tick += state_ref.settings.timing_mode.get_delta() as u128;
                        } else {
                            self.next_tick += (state_ref.settings.timing_mode.get_delta() as f64 / speed) as u128;
                        }
                        self.loops += 1;
                    }

                    if self.loops == 10 {
                        log::warn!("Frame skip is way too high, a long system lag occurred?");
                        self.last_tick = self.start_time.elapsed().as_nanos();
                        self.next_tick =
                            self.last_tick + (state_ref.settings.timing_mode.get_delta() as f64 / speed) as u128;
                        self.loops = 0;
                    }

                    if self.loops != 0 {
                        scene.draw_tick(state_ref)?;
                        self.last_tick = last_tick;
                    }

                    for _ in 0..self.loops {
                        scene.tick(state_ref, ctx)?;
                    }
                    self.fps.tick_count = self.fps.tick_count.saturating_add(self.loops as u32);
                }
                TimingMode::FrameSynchronized => {
                    scene.tick(state_ref, ctx)?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let state_ref = unsafe { &mut *self.state.get() };

        match ctx.vsync_mode {
            VSyncMode::Uncapped | VSyncMode::VSync => {
                self.present = true;
            }
            _ => unsafe {
                self.present = false;

                let divisor = match ctx.vsync_mode {
                    VSyncMode::VRRTickSync1x => 1,
                    VSyncMode::VRRTickSync2x => 2,
                    VSyncMode::VRRTickSync3x => 3,
                    _ => std::hint::unreachable_unchecked(),
                };

                let delta = (state_ref.settings.timing_mode.get_delta() / divisor) as u64;

                let now = self.start_time.elapsed().as_nanos();
                if now > self.next_tick_draw + delta as u128 * 4 {
                    self.next_tick_draw = now;
                }

                while self.start_time.elapsed().as_nanos() >= self.next_tick_draw {
                    self.next_tick_draw += delta as u128;
                    self.present = true;
                }
            },
        }

        if !self.present {
            std::thread::sleep(Duration::from_millis(2));
            self.loops = 0;
            return Ok(());
        }

        if ctx.headless {
            self.loops = 0;
            state_ref.frame_time = 1.0;
            return Ok(());
        }

        if state_ref.settings.timing_mode != TimingMode::FrameSynchronized {
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

        if let Some(scene) = &mut self.scene {
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

            if state_ref.settings.fps_counter {
                self.fps.act(state_ref, ctx, self.start_time.elapsed().as_nanos())?;
            }

            self.ui.draw(state_ref, ctx, scene)?;
        }

        graphics::present(ctx)?;

        Ok(())
    }
}

pub fn init(options: LaunchOptions) -> GameResult {
    let _ = simple_logger::SimpleLogger::new()
        .without_timestamps()
        .with_colors(true)
        .with_level(log::Level::Info.to_level_filter())
        .init();

    #[cfg(not(any(target_os = "android", target_os = "horizon")))]
    let resource_dir = if let Ok(data_dir) = std::env::var("CAVESTORY_DATA_DIR") {
        PathBuf::from(data_dir)
    } else {
        let mut resource_dir = std::env::current_exe()?;
        if resource_dir.file_name().is_some() {
            let _ = resource_dir.pop();
        }

        #[cfg(target_os = "macos")]
        {
            let mut bundle_dir = resource_dir.clone();
            let _ = bundle_dir.pop();
            let mut bundle_exec_dir = bundle_dir.clone();
            let mut csplus_data_dir = bundle_dir.clone();
            let _ = csplus_data_dir.pop();
            let _ = csplus_data_dir.pop();
            let mut csplus_data_base_dir = csplus_data_dir.clone();
            csplus_data_base_dir.push("data");
            csplus_data_base_dir.push("base");

            bundle_exec_dir.push("MacOS");
            bundle_dir.push("Resources");

            if bundle_exec_dir.is_dir() && bundle_dir.is_dir() {
                log::info!("Running in macOS bundle mode");

                if csplus_data_base_dir.is_dir() {
                    log::info!("Cave Story+ Steam detected");
                    resource_dir = csplus_data_dir;
                } else {
                    resource_dir = bundle_dir;
                }
            }
        }

        resource_dir.push("data");
        resource_dir
    };

    #[cfg(not(any(target_os = "android", target_os = "horizon")))]
    log::info!("Resource directory: {:?}", resource_dir);
    log::info!("Initializing engine...");

    let mut context = Context::new();
    #[cfg(not(any(target_os = "android", target_os = "horizon")))]
    mount_vfs(&mut context, Box::new(PhysicalFS::new(&resource_dir, true)));

    #[cfg(not(any(target_os = "android", target_os = "horizon")))]
    let project_dirs = match directories::ProjectDirs::from("", "", "doukutsu-rs") {
        Some(dirs) => dirs,
        None => {
            use crate::framework::error::GameError;
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

    #[cfg(not(any(target_os = "android", target_os = "horizon")))]
    {
        if crate::framework::filesystem::open(&context, "/.drs_localstorage").is_ok() {
            let mut user_dir = resource_dir.clone();
            user_dir.push("_drs_profile");

            let _ = std::fs::create_dir_all(&user_dir);
            mount_user_vfs(&mut context, Box::new(PhysicalFS::new(&user_dir, false)));
        } else {
            mount_user_vfs(&mut context, Box::new(PhysicalFS::new(project_dirs.data_local_dir(), false)));
        }
    }

    mount_vfs(&mut context, Box::new(BuiltinFS::new()));

    if options.server_mode {
        log::info!("Running in server mode...");
        context.headless = true;
    }

    let game = UnsafeCell::new(Game::new(&mut context)?);
    let state_ref = unsafe { &mut *((&mut *game.get()).state.get()) };
    #[cfg(feature = "scripting-lua")]
    {
        state_ref.lua.update_refs(unsafe { (&*game.get()).state.get() }, &mut context as *mut Context);
    }

    state_ref.next_scene = Some(Box::new(LoadingScene::new()));
    context.run(unsafe { &mut *game.get() })?;

    Ok(())
}
