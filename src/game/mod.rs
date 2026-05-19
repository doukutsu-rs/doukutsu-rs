use std::backtrace::Backtrace;
use std::cell::RefCell;
use std::panic::PanicHookInfo;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::clap_derive::Parser;

use log::LevelFilter as LogLevel;
use scripting::tsc::text_script::ScriptMode;

use crate::components::pacing_graph::PacingGraph;
use crate::framework::backend::{BackendCallbacks, WindowParams};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::frame_pacer::FramePacer;
use crate::framework::graphics::{self, SwapMode};
use crate::framework::keyboard::{self, ScanCode};
use crate::framework::ui::UI;
use crate::game::filesystem_container::FilesystemContainer;
use crate::game::settings::{Settings, VSyncMode};
use crate::game::shared_game_state::{Fps, SharedGameState, TimingMode, WindowMode};
use crate::graphics::texture_set::{G_MAG, I_MAG};
use crate::scene::loading_scene::LoadingScene;
use crate::scene::Scene;

pub mod aspect_ratio;
pub mod caret;
pub mod filesystem_container;
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

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct LaunchOptions {
    #[arg(long, hide = cfg!(not(feature = "netplay")))]
    /// Do not create a window and skip audio initialization.
    pub server_mode: bool,

    #[arg(long)]
    /// Window height in pixels.
    pub window_height: Option<u16>,

    #[arg(long)]
    /// Window width in pixels.
    pub window_width: Option<u16>,

    #[arg(long)]
    /// Startup in fullscreen mode.
    pub window_fullscreen: bool,

    #[arg(long)]
    /// The renderer to prefer
    pub prefer_renderer: Option<String>,

    // todo: add user_dir / data_dir?
    #[arg(long, default_value_t = Self::default().log_level)]
    /// The minimum level of records that will be written to the log file.
    ///
    /// Possible values: error, warn, info, debug, trace.
    pub log_level: LogLevel,
}

impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            server_mode: false,
            window_height: None,
            window_width: None,
            window_fullscreen: cfg!(target_os = "android"),
            prefer_renderer: None,
            log_level: if cfg!(debug_assertions) { LogLevel::Debug } else { LogLevel::Info },
        }
    }
}

impl LaunchOptions {
    pub fn apply_defaults(&mut self, ctx: &Context, settings: &Settings) {
        if let Some(geometry) = settings.window_geometry {
            self.window_width = Some(self.window_width.unwrap_or(geometry.width.min(u16::MAX as u32) as u16));
            self.window_height = Some(self.window_height.unwrap_or(geometry.height.min(u16::MAX as u32) as u16));
        } else {
            self.window_width = Some(self.window_width.unwrap_or(ctx.window.size_hint.0));
            self.window_height = Some(self.window_height.unwrap_or(ctx.window.size_hint.1));
        }

        if !self.window_fullscreen {
            self.window_fullscreen = settings.window_mode.is_fullscreen();
        }
    }

    pub fn window(&self) -> WindowParams {
        let default = WindowParams::default();

        let width = self.window_width.unwrap_or(default.size_hint.0);
        let height = self.window_height.unwrap_or(default.size_hint.1);

        WindowParams {
            size_hint: (width, height),
            mode: if self.window_fullscreen { WindowMode::Fullscreen } else { WindowMode::Windowed },
        }
    }
}

pub struct Game {
    pub(crate) scene: RefCell<Option<Box<dyn Scene>>>,
    pub(crate) state: RefCell<SharedGameState>,
    ui: UI,
    start_time: Instant,
    pacer: FramePacer,
    graph: PacingGraph,
    pub(crate) loops: u32,
    fps: Fps,
}

impl Game {
    fn new(ctx: &mut Context) -> GameResult<Game> {
        let s = Game {
            scene: RefCell::new(None),
            ui: UI::new(ctx)?,
            state: RefCell::new(SharedGameState::new(ctx)?),
            start_time: Instant::now(),
            pacer: FramePacer::new(),
            graph: PacingGraph,
            loops: 0,
            fps: Fps::new(),
        };

        Ok(s)
    }

    pub(crate) fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Snapshot config we need for the timing math while not holding any borrows on `self`.
        let (timing_mode, speed) = {
            let s = self.state.get_mut();
            let speed_mul = if s.textscript_vm.mode == ScriptMode::Map && s.textscript_vm.flags.cutscene_skip() {
                4.0
            } else {
                1.0
            };
            (s.settings.timing_mode, speed_mul * s.settings.speed)
        };

        match timing_mode {
            TimingMode::_50Hz | TimingMode::_60Hz => {
                let base_delta_ns = timing_mode.get_delta() as u64;
                let effective_delta_ns = if (speed - 1.0).abs() < 0.01 {
                    base_delta_ns
                } else if speed > 0.0 {
                    ((base_delta_ns as f64) / speed).max(1.0) as u64
                } else {
                    base_delta_ns
                };
                self.pacer.set_tick_delta(Duration::from_nanos(effective_delta_ns));

                // VRR: advance against the upcoming-present deadline to keep tick/present
                // clocks phase-locked (no 0/2 staircasing).
                let target = {
                    let s = self.state.get_mut();
                    match s.settings.vsync_mode {
                        VSyncMode::VRRTickSync1x
                        | VSyncMode::VRRTickSync2x
                        | VSyncMode::VRRTickSync3x
                        | VSyncMode::VRRTickSyncAuto => self.pacer.next_present_instant(),
                        _ => Instant::now(),
                    }
                };
                let loops = self.pacer.advance_ticks_until(target);
                self.loops = loops;
                if loops != 0 {
                    if let Some(scene) = self.scene.get_mut() {
                        let state_ref = self.state.get_mut();
                        scene.draw_tick(state_ref)?;
                        for _ in 0..loops {
                            scene.tick(state_ref, ctx)?;
                        }
                    }
                    self.fps.tick_count = self.fps.tick_count.saturating_add(loops);
                }
            }
            TimingMode::FrameSynchronized => {
                if let Some(scene) = self.scene.get_mut() {
                    let state_ref = self.state.get_mut();
                    scene.tick(state_ref, ctx)?;
                }
            }
        }

        let next_scene = std::mem::take(&mut self.state.get_mut().next_scene);
        if let Some(mut next_scene) = next_scene {
            {
                let state_ref = self.state.get_mut();
                next_scene.init(state_ref, ctx)?;
                state_ref.frame_time = 0.0;
            }
            *self.scene.get_mut() = Some(next_scene);

            self.loops = 0;
            self.pacer.reset();
        }

        Ok(())
    }

    pub(crate) fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let vsync_mode = self.state.get_mut().settings.vsync_mode;

        match vsync_mode {
            VSyncMode::Uncapped => {
                graphics::set_swap_mode(ctx, SwapMode::Immediate)?;
            }
            VSyncMode::VSync => {
                graphics::set_swap_mode(ctx, SwapMode::VSync)?;
            }
            _ => {
                graphics::set_swap_mode(ctx, SwapMode::Adaptive)?;

                let base_delta_ns = self.state.get_mut().settings.timing_mode.get_delta() as u64;

                let divisor: u32 = match vsync_mode {
                    VSyncMode::VRRTickSync1x => 1,
                    VSyncMode::VRRTickSync2x => 2,
                    VSyncMode::VRRTickSync3x => 3,
                    VSyncMode::VRRTickSyncAuto if base_delta_ns > 0 => {
                        let tick_hz = 1.0e9 / base_delta_ns as f64;
                        let mon_hz = ctx
                            .viewport
                            .refresh_rate_mhz
                            .map(|m| m as f64 / 1000.0)
                            .or_else(|| {
                                let p = self.pacer.measured_period();
                                if p.is_zero() {
                                    None
                                } else {
                                    Some(1.0 / p.as_secs_f64())
                                }
                            })
                            .unwrap_or(tick_hz);
                        ((mon_hz / tick_hz).floor() as u32).max(1)
                    }
                    _ => 1,
                };

                if base_delta_ns > 0 {
                    let period = Duration::from_nanos((base_delta_ns / divisor as u64).max(1));
                    self.pacer.set_present_period(period);
                    self.pacer.wait_for_present();
                }
            }
        }

        if ctx.headless {
            self.loops = 0;
            self.state.get_mut().frame_time = 1.0;
            return Ok(());
        }

        if self.state.get_mut().settings.timing_mode != TimingMode::FrameSynchronized {
            let alpha = self.pacer.interpolation_alpha();
            self.state.get_mut().frame_time =
                if self.state.get_mut().settings.motion_interpolation { alpha } else { 1.0 };
        }
        unsafe {
            G_MAG = if self.state.get_mut().settings.subpixel_coords { self.state.get_mut().scale } else { 1.0 };
            I_MAG = self.state.get_mut().scale;
        }
        let frame_loops = self.loops;
        self.loops = 0;

        graphics::prepare_draw(ctx)?;
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        if let Some(scene) = self.scene.get_mut() {
            let state_ref = self.state.get_mut();

            scene.draw(state_ref, ctx)?;
            if state_ref.settings.touch_controls && state_ref.settings.display_touch_controls {
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

            self.graph.draw(&self.pacer, state_ref, ctx)?;

            // Blit the canvas to the window framebuffer; overlay renders on top.
            graphics::present(ctx)?;

            self.ui.draw(state_ref, ctx, scene)?;
        }

        let pre_finalize = Instant::now();
        graphics::finalize_frame(ctx)?;
        if !matches!(vsync_mode, VSyncMode::VSync) {
            self.pacer.record_swap_latency(Instant::now().saturating_duration_since(pre_finalize));
        }
        self.pacer.record_present(frame_loops);

        Ok(())
    }
}

impl BackendCallbacks for Game {
    fn on_resize(&mut self, ctx: &mut Context) -> GameResult {
        self.state.get_mut().handle_resize(ctx)?;
        Ok(())
    }

    fn on_focus_gained(&mut self, ctx: &mut Context) -> GameResult {
        let state_ref = self.state.get_mut();
        if state_ref.settings.pause_on_focus_loss {
            ctx.suspended = false;
            state_ref.sound_manager.resume();
            self.loops = 0;
            self.pacer.reset();
        }
        Ok(())
    }

    fn on_focus_lost(&mut self, ctx: &mut Context) -> GameResult {
        let state_ref = self.state.get_mut();
        if state_ref.settings.pause_on_focus_loss {
            ctx.suspended = true;
            state_ref.sound_manager.pause();
        }
        Ok(())
    }

    fn on_key_down(&mut self, ctx: &mut Context, key: keyboard::ScanCode) -> GameResult {
        if ctx.input.want_capture_keyboard {
            return Ok(());
        }
        if let Some(scene) = self.scene.get_mut() {
            if key == ScanCode::F11 {
                let state = self.state.get_mut();
                if ctx.keyboard_context.active_mods().shift() {
                    state.settings.pacing_debug = !state.settings.pacing_debug;
                } else {
                    state.settings.fps_counter = !state.settings.fps_counter;
                }
            }

            let _ = scene.process_debug_keys(self.state.get_mut(), ctx, key);
        }
        Ok(())
    }

    fn on_key_up(&mut self, ctx: &mut Context, _key: keyboard::ScanCode) -> GameResult {
        if ctx.input.want_capture_keyboard {
            return Ok(());
        }
        Ok(())
    }

    fn on_fullscreen_state_changed(&mut self, _ctx: &mut Context, new_mode: WindowMode) -> GameResult {
        let state_ref = self.state.get_mut();
        state_ref.settings.window_mode = new_mode;
        Ok(())
    }

    fn on_context_lost(&mut self, _ctx: &mut Context) -> GameResult {
        // TODO: get rid of this on texture manager rework
        let state_ref = self.state.get_mut();
        state_ref.texture_set.unload_all();
        Ok(())
    }
}

// For the most part this is just a copy-paste of the code from FilesystemContainer because it logs
// some messages during init, but the default logger cannot be replaced with another
// one or deinited(so we can't create the console-only logger and replace it by the
// console&file logger after FilesystemContainer has been initialized)
fn get_logs_dir() -> GameResult<PathBuf> {
    let mut logs_dir: PathBuf;

    #[cfg(target_os = "android")]
    {
        logs_dir =
            PathBuf::from(sdl2::filesystem::pref_path(crate::common::ORG_NAME, crate::common::APP_NAME).unwrap());
    }

    #[cfg(target_os = "horizon")]
    {
        logs_dir = PathBuf::from("sdmc:/switch/doukutsu-rs");
    }

    #[cfg(not(any(target_os = "android", target_os = "horizon")))]
    {
        let project_dirs = match directories::ProjectDirs::from("", "", "doukutsu-rs") {
            Some(dirs) => dirs,
            None => {
                use crate::framework::error::GameError;
                return Err(GameError::FilesystemError(String::from(
                    "No valid home directory path could be retrieved.",
                )));
            }
        };

        logs_dir = project_dirs.data_local_dir().to_path_buf();
    }

    logs_dir.push("logs");

    Ok(logs_dir)
}

fn init_logger(options: &LaunchOptions) -> GameResult {
    let logs_dir = get_logs_dir()?;
    let _ = std::fs::create_dir_all(&logs_dir);

    // On Android, the jni-rs library generates many trace records, making it difficult to analyze logs in real time
    let stdout_log_level = if cfg!(target_os = "android") && options.log_level != LogLevel::Trace {
        LogLevel::Debug
    } else {
        LogLevel::Trace
    };

    let mut dispatcher = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!("{} [{}] {}", record.level(), record.module_path().unwrap().to_owned(), message))
        })
        .chain(fern::Dispatch::new().level(stdout_log_level).chain(std::io::stderr()));

    let date = chrono::Utc::now();
    let mut file = logs_dir.clone();
    file.push(format!("log_{}", date.format("%Y-%m-%d")));
    file.set_extension("txt");

    dispatcher = dispatcher.chain(fern::Dispatch::new().level(options.log_level).chain(fern::log_file(file).unwrap()));
    dispatcher.apply()?;

    Ok(())
}

fn panic_hook(info: &PanicHookInfo<'_>) {
    let backtrace = Backtrace::force_capture();
    let msg = info.payload().downcast_ref::<&str>().unwrap_or(&"(no message)");
    let location = info.location();

    if location.is_some() {
        log::error!("Panic occurred in {} with message: '{msg}'\n {backtrace:#}", location.unwrap().to_string());
    } else {
        log::error!("Panic occurred with message: '{msg}'\n {backtrace:#}");
    }
}

pub fn init(mut options: LaunchOptions) -> GameResult {
    let _ = init_logger(&options);
    std::panic::set_hook(Box::new(panic_hook));

    let mut context = Box::pin(Context::new());

    let mut fs_container = FilesystemContainer::new();
    fs_container.mount_fs(&mut context)?;

    if options.server_mode {
        log::info!("Running in server mode...");
        context.headless = true;
    }

    let mut game = Box::pin(Game::new(&mut context)?);
    game.state.get_mut().fs_container = Some(fs_container);

    options.apply_defaults(&context, &game.state.get_mut().settings);

    #[cfg(feature = "discord-rpc")]
    if game.state.get_mut().settings.discord_rpc {
        game.state.get_mut().discord_rpc.enabled = true;
        game.state.get_mut().discord_rpc.start()?;
    }

    context.preferred_renderer = std::mem::take(&mut options.prefer_renderer);
    context.window = options.window();

    game.state.get_mut().next_scene = Some(Box::new(LoadingScene::new()));
    log::info!("Starting main loop...");
    context.run(game)?;

    Ok(())
}
