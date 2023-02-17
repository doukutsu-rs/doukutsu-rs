use std::cell::UnsafeCell;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use lazy_static::lazy_static;

use scripting::tsc::text_script::ScriptMode;

use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics;
use crate::framework::graphics::VSyncMode;
use crate::framework::ui::UI;
use crate::game::filesystem_container::FilesystemContainer;
use crate::game::shared_game_state::{Fps, SharedGameState, TimingMode};
use crate::graphics::texture_set::{G_MAG, I_MAG};
use crate::scene::loading_scene::LoadingScene;
use crate::scene::Scene;

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

    let mut context = Box::pin(Context::new());

    let mut fs_container = FilesystemContainer::new();
    fs_container.mount_fs(&mut context)?;

    if options.server_mode {
        log::info!("Running in server mode...");
        context.headless = true;
    }

    let mut game = Box::pin(Game::new(&mut context)?);
    #[cfg(feature = "scripting-lua")]
    {
        game.state.get().lua.update_refs(unsafe { &mut *game.state.get() }, &mut context as *mut Context);
    }

    game.state.get_mut().fs_container = Some(fs_container);

    game.state.get_mut().next_scene = Some(Box::new(LoadingScene::new()));
    log::info!("Starting main loop...");
    context.run(game.as_mut().get_mut())?;

    Ok(())
}
