#[macro_use]
extern crate log;
#[cfg_attr(feature = "scripting", macro_use)]
#[cfg(feature = "scripting")]
extern crate lua_ffi;
extern crate strum;
#[macro_use]
extern crate strum_macros;

use std::{env, mem};
use std::cell::UnsafeCell;
use std::path;
use std::time::Instant;

use ggez::{Context, ContextBuilder, GameError, GameResult};
use ggez::conf::{Backend, WindowMode, WindowSetup};
use ggez::event::{KeyCode, KeyMods};
use ggez::filesystem::mount_vfs;
use ggez::graphics;
use ggez::graphics::{Canvas, DrawParam, window};
use ggez::graphics::glutin_ext::WindowUpdateExt;
use ggez::input::keyboard;
use ggez::mint::ColumnMatrix4;
use ggez::nalgebra::Vector2;
use log::*;
use pretty_env_logger::env_logger::Env;
use winit::event::{ElementState, Event, KeyboardInput, WindowEvent};
use winit::event_loop::ControlFlow;

use crate::builtin_fs::BuiltinFS;
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
mod components;
mod difficulty_modifier;
mod encoding;
mod engine_constants;
mod entity;
mod frame;
mod inventory;
mod input;
mod live_debugger;
mod macros;
mod map;
mod menu;
mod npc;
mod physics;
mod player;
mod profile;
mod rng;
mod scene;
#[cfg(feature = "scripting")]
mod scripting;
mod settings;
mod shaders;
mod shared_game_state;
mod stage;
mod sound;
mod text_script;
mod texture_set;
mod ui;
mod weapon;

struct Game {
    scene: Option<Box<dyn Scene>>,
    state: UnsafeCell<SharedGameState>,
    ui: UI,
    def_matrix: ColumnMatrix4<f32>,
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
            def_matrix: DrawParam::new().to_matrix(),
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
                            self.next_tick += (state_ref.timing_mode.get_delta() as f64 / state_ref.settings.speed) as u128;
                        }
                        self.loops += 1;
                    }

                    if self.loops == 10 {
                        log::warn!("Frame skip is way too high, a long system lag occurred?");
                        self.last_tick = self.start_time.elapsed().as_nanos();
                        self.next_tick = self.last_tick + (state_ref.timing_mode.get_delta() as f64 / state_ref.settings.speed) as u128;
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
            // Even with the non-monotonic Instant mitigation at the start of the event loop,
            // there's still a theoretical chance of it not working.
            // This check here should trigger if that happens and makes sure there's no panic from an underflow.
            let mut elapsed = self.start_time.elapsed().as_nanos();
            if elapsed < self.last_tick {
                warn!("Elapsed time was less than last tick! elapsed: {}, last tick: {}", elapsed, self.last_tick);
                elapsed = self.last_tick;
            }
            let n1 = (elapsed - self.last_tick) as f64;
            let n2 = (self.next_tick - self.last_tick) as f64;
            state_ref.frame_time = n1 / n2;
        }
        self.loops = 0;

        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());
        graphics::set_transform(ctx, DrawParam::new()
            .scale(Vector2::new(state_ref.scale, state_ref.scale))
            .to_matrix());
        graphics::apply_transformations(ctx)?;

        if let Some(scene) = self.scene.as_mut() {
            scene.draw(state_ref, ctx)?;
            if state_ref.settings.touch_controls {
                state_ref.touch_controls.draw(state_ref.canvas_size, &state_ref.constants, &mut state_ref.texture_set, ctx)?;
            }

            graphics::set_transform(ctx, self.def_matrix);
            graphics::apply_transformations(ctx)?;
            self.ui.draw(state_ref, ctx, scene)?;
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, key_code: KeyCode, _key_mod: KeyMods, repeat: bool) {
        if repeat { return; }

        let state = unsafe { &mut *self.state.get() };
        match key_code {
            KeyCode::F7 => { state.set_speed(1.0) }
            KeyCode::F8 => {
                if state.settings.speed > 0.2 {
                    state.set_speed(state.settings.speed - 0.1);
                }
            }
            KeyCode::F9 => {
                if state.settings.speed < 3.0 {
                    state.set_speed(state.settings.speed + 0.1);
                }
            }
            KeyCode::F10 => { state.settings.debug_outlines = !state.settings.debug_outlines }
            KeyCode::F11 => { state.settings.god_mode = !state.settings.god_mode }
            KeyCode::F12 => { state.settings.infinite_booster = !state.settings.infinite_booster }
            _ => {}
        }
    }

    fn key_up_event(&mut self, _key_code: KeyCode, _key_mod: KeyMods) {
        //
    }
}

#[cfg(target_os = "android")]
fn request_perms() -> GameResult {
    use jni::objects::JValue;
    use jni::objects::JObject;

    let native_activity = ndk_glue::native_activity();
    let vm_ptr = native_activity.vm();
    let vm = unsafe { jni::JavaVM::from_raw(vm_ptr) }?;
    let vm_env = vm.attach_current_thread()?;

    fn perm_name<'a, 'b, 'c>(vm_env: &'b jni::AttachGuard<'a>, name: &'c str) -> GameResult<jni::objects::JValue<'a>> {
        let class = vm_env.find_class("android/Manifest$permission")?;
        Ok(vm_env.get_static_field(class, name.to_owned(), "Ljava/lang/String;")?)
    }

    fn has_permission(vm_env: &jni::AttachGuard, activity: &jni::sys::jobject, name: &str) -> GameResult<bool> {
        let perm_granted = {
            let class = vm_env.find_class("android/content/pm/PackageManager")?;
            vm_env.get_static_field(class, "PERMISSION_GRANTED", "I")?.i()?
        };

        let perm = perm_name(vm_env, name)?;
        let activity_obj = JObject::from(*activity);
        let result = vm_env.call_method(activity_obj, "checkSelfPermission", "(Ljava/lang/String;)I", &[perm])?.i()?;
        Ok(result == perm_granted)
    }

    let str_class = vm_env.find_class("java/lang/String")?;
    let array = vm_env.new_object_array(2, str_class, JObject::null())?;
    vm_env.set_object_array_element(array, 0, perm_name(&vm_env, "READ_EXTERNAL_STORAGE")?.l()?)?;
    vm_env.set_object_array_element(array, 1, perm_name(&vm_env, "WRITE_EXTERNAL_STORAGE")?.l()?)?;
    let activity_obj = JObject::from(native_activity.activity());

    loop {
        if has_permission(&vm_env, &native_activity.activity(), "READ_EXTERNAL_STORAGE")?
            && has_permission(&vm_env, &native_activity.activity(), "WRITE_EXTERNAL_STORAGE")? {
            break;
        }

        vm_env.call_method(activity_obj, "requestPermissions", "([Ljava/lang/String;I)V", &[JValue::from(array), JValue::from(0)])?;
    }

    Ok(())
}

#[cfg(target_os = "android")]
#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn android_main() {
    println!("main invoked.");

    request_perms().expect("Failed to attach to the JVM and request storage permissions.");

    env::set_var("CAVESTORY_DATA_DIR", "/storage/emulated/0/doukutsu");
    init().unwrap();
}

#[cfg(target_os = "android")]
static BACKENDS: [Backend; 2] = [
    Backend::OpenGLES { major: 3, minor: 0 },
    Backend::OpenGLES { major: 2, minor: 0 }
];

#[cfg(not(target_os = "android"))]
static BACKENDS: [Backend; 4] = [
    Backend::OpenGL { major: 3, minor: 2 },
    Backend::OpenGLES { major: 3, minor: 2 },
    Backend::OpenGLES { major: 3, minor: 0 },
    Backend::OpenGLES { major: 2, minor: 0 }
];

fn init_ctx<P: Into<path::PathBuf> + Clone>(event_loop: &winit::event_loop::EventLoopWindowTarget<()>, resource_dir: P) -> GameResult<Context> {
    for backend in BACKENDS.iter() {
        let ctx = ContextBuilder::new("doukutsu-rs")
            .window_setup(WindowSetup::default().title("Cave Story ~ Doukutsu Monogatari (doukutsu-rs)"))
            .window_mode(WindowMode::default()
                .resizable(true)
                .min_dimensions(320.0, 240.0)
                .dimensions(854.0, 480.0))
            .add_resource_path(resource_dir.clone())
            .backend(*backend)
            .build(event_loop);

        match ctx {
            Ok(mut ctx) => {
                mount_vfs(&mut ctx, Box::new(BuiltinFS::new()));

                return Ok(ctx);
            }
            Err(err) => {
                log::warn!("Failed to create backend using config {:?}: {}", backend, err);
            }
        }
    }

    Err(GameError::EventLoopError("Failed to initialize OpenGL backend. Perhaps the driver is outdated?".to_string()))
}

pub fn init() -> GameResult {
    pretty_env_logger::env_logger::from_env(Env::default().default_filter_or("info"))
        .filter(Some("gfx_device_gl::factory"), LevelFilter::Warn)
        .init();

    let resource_dir = if let Ok(data_dir) = env::var("CAVESTORY_DATA_DIR") {
        path::PathBuf::from(data_dir)
    } else if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("data");
        path
    } else {
        path::PathBuf::from("data")
    };

    info!("Resource directory: {:?}", resource_dir);
    info!("Initializing engine...");

    let event_loop = winit::event_loop::EventLoop::new();
    let mut context: Option<Context>;
    let mut game: Option<Game> = None;

    #[cfg(target_os = "android")]
        {
            loop {
                match ndk_glue::native_window().as_ref() {
                    Some(_) => {
                        println!("NativeScreen Found:{:?}", ndk_glue::native_window());
                        break;
                    }
                    None => ()
                }
            }
        }

    context = Some(init_ctx(&event_loop, resource_dir.clone())?);

    event_loop.run(move |event, target, flow| {
        #[cfg(target_os = "windows")]
        {
            // Windows' system clock implementation isn't monotonic when the process gets switched to another core.
            // Rust has mitigations for this, but apparently aren't very effective unless Instant is called very often.
            let _ = Instant::now();
        }
        if let Some(ctx) = &mut context {
            ctx.process_event(&event);

            if let Some(game) = &mut game {
                game.ui.handle_events(ctx, &event);
            } else {
                let mut new_game = Game::new(ctx).unwrap();
                let state_ref = unsafe { &mut *new_game.state.get() };
                state_ref.next_scene = Some(Box::new(LoadingScene::new()));
                game = Some(new_game);

                #[cfg(feature = "scripting")]
                    {
                        unsafe {
                            let game_ref = game.as_mut().unwrap();
                            let state_ref = game_ref.state.get();

                            (&mut *state_ref).lua.update_refs(game_ref.state.get(), ctx as *mut Context);
                        }
                    }
            }
        }

        match event {
            Event::Resumed => {
                #[cfg(target_os = "android")]
                if context.is_none() {
                    context = Some(init_ctx(target, resource_dir.clone()).unwrap());
                }
                let _ = target;

                if let Some(game) = &mut game {
                    game.loops = 0;
                }
            }
            Event::Suspended => {
                #[cfg(target_os = "android")]
                    {
                        context = None;
                    }
                if let Some(game) = &mut game {
                    game.loops = 0;
                }
            }
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        if let Some(game) = &mut game {
                            let state_ref = unsafe { &mut *game.state.get() };
                            state_ref.shutdown();
                        }
                        *flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(_) => {
                        if let (Some(ctx), Some(game)) = (&mut context, &mut game) {
                            let state_ref = unsafe { &mut *game.state.get() };

                            state_ref.tmp_canvas = Canvas::with_window_size(ctx).unwrap();
                            state_ref.game_canvas = Canvas::with_window_size(ctx).unwrap();
                            state_ref.lightmap_canvas = Canvas::with_window_size(ctx).unwrap();
                            state_ref.handle_resize(ctx).unwrap();
                            graphics::window(ctx).update_gfx(&mut game.ui.main_color, &mut game.ui.main_depth);
                        }
                    }
                    WindowEvent::Touch(touch) => {
                        if let Some(game) = &mut game {
                            let state_ref = unsafe { &mut *game.state.get() };
                            state_ref.touch_controls.process_winit_event(state_ref.scale, touch);
                        }
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
                        if let (Some(ctx), Some(game)) = (&mut context, &mut game) {
                            match el_state {
                                ElementState::Pressed => {
                                    let repeat = keyboard::is_key_repeated(ctx);
                                    game.key_down_event(keycode, modifiers.into(), repeat);
                                }
                                ElementState::Released => {
                                    game.key_up_event(keycode, modifiers.into());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(win) => {
                if let (Some(ctx), Some(game)) = (&mut context, &mut game) {
                    if win == window(ctx).window().id() {
                        ctx.timer_context.tick();
                        game.draw(ctx).unwrap();
                    }
                }
            }
            Event::MainEventsCleared => {
                if let (Some(ctx), Some(game)) = (&mut context, &mut game) {
                    game.update(ctx).unwrap();

                    #[cfg(target_os = "android")]
                        {
                            ctx.timer_context.tick();
                            game.draw(ctx).unwrap(); // redraw request is unimplemented on shitdroid
                        }
                    window(ctx).window().request_redraw();

                    let state_ref = unsafe { &mut *game.state.get() };

                    if state_ref.shutdown {
                        log::info!("Shutting down...");
                        *flow = ControlFlow::Exit;
                        return;
                    }

                    if state_ref.next_scene.is_some() {
                        mem::swap(&mut game.scene, &mut state_ref.next_scene);
                        state_ref.next_scene = None;

                        game.scene.as_mut().unwrap().init(state_ref, ctx).unwrap();
                        game.loops = 0;
                        state_ref.frame_time = 0.0;
                    }
                }
            }
            _ => {}
        }
    })
}
