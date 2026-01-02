use core::mem;
use std::any::Any;
use std::cell::{RefCell, UnsafeCell};
use std::ffi::c_void;
use std::io::Read;
use std::ops::Deref;
use std::pin::Pin;
use std::ptr::{null, null_mut};
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::vec::Vec;

use imgui::internal::RawWrapper;
use imgui::sys::{ImGuiKey_Backspace, ImGuiKey_Delete, ImGuiKey_Enter};
use imgui::{ConfigFlags, DrawCmd, DrawData, DrawIdx, DrawVert, Key, MouseCursor, TextureId, Ui};
use sdl2::controller::GameController;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Scancode;
use sdl2::mouse::{Cursor, SystemCursor};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, TextureCreator, TextureQuery, WindowCanvas};
use sdl2::rwops::RWops;
use sdl2::surface::Surface;
use sdl2::video::{FullscreenType, GLProfile, Window, WindowContext};
use sdl2::{controller, keyboard, pixels, EventPump, GameControllerSubsystem, Sdl, VideoSubsystem};

use super::backend::{
    Backend, BackendCallbacks, BackendEventLoop, BackendGamepad, BackendRenderer, BackendShader, BackendTexture,
    DeviceFormFactor, SpriteBatchCommand, VertexData, WindowParams,
};
use super::context::Context;
use super::error::{GameError, GameResult};
use super::filesystem;
use super::gamepad::{Axis, Button, GamepadType};
use super::graphics::{BlendMode, SwapMode};
use super::keyboard::ScanCode;
use super::ui::init_imgui;
use crate::common::{Color, Rect};
use crate::framework::graphics::IndexData;
use crate::game::shared_game_state::WindowMode;
use crate::game::Game;

fn handle_err_impl(result: GameResult, shutdown_requested: &mut bool) {
    if let Err(e) = result {
        log::error!("{}", e);
        *shutdown_requested = true;
    }
}

trait WindowModeExt {
    fn get_sdl2_fullscreen_type(&self) -> sdl2::video::FullscreenType;
}

impl WindowModeExt for WindowMode {
    fn get_sdl2_fullscreen_type(&self) -> sdl2::video::FullscreenType {
        match self {
            WindowMode::Windowed => sdl2::video::FullscreenType::Off,
            WindowMode::Fullscreen => sdl2::video::FullscreenType::Desktop,
        }
    }
}

pub struct SDL2Backend {
    context: Sdl,
}

impl SDL2Backend {
    pub fn new(window_params: WindowParams) -> GameResult<Box<dyn Backend>> {
        sdl2::hint::set("SDL_JOYSTICK_THREAD", "1");

        let context = sdl2::init().map_err(GameError::WindowError)?;

        let backend = SDL2Backend { context };

        Ok(Box::new(backend))
    }
}

impl Backend for SDL2Backend {
    fn create_event_loop(&self, ctx: &Context) -> GameResult<Box<dyn BackendEventLoop>> {
        SDL2EventLoop::new(&self.context, ctx)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub(crate) enum WindowOrCanvas {
    None,
    Win(Window),
    Canvas(WindowCanvas, TextureCreator<WindowContext>),
}

impl Default for WindowOrCanvas {
    fn default() -> Self {
        WindowOrCanvas::None
    }
}

impl WindowOrCanvas {
    #[inline]
    pub fn window(&self) -> &Window {
        match self {
            WindowOrCanvas::Win(ref window) => window,
            WindowOrCanvas::Canvas(ref canvas, _) => canvas.window(),
            _ => unsafe {
                std::hint::unreachable_unchecked();
            },
        }
    }

    #[inline]
    pub fn window_mut(&mut self) -> &mut Window {
        match self {
            WindowOrCanvas::Win(ref mut window) => window,
            WindowOrCanvas::Canvas(ref mut canvas, _) => canvas.window_mut(),
            _ => unsafe {
                std::hint::unreachable_unchecked();
            },
        }
    }

    #[inline]
    pub fn canvas(&mut self) -> &mut WindowCanvas {
        match self {
            WindowOrCanvas::Canvas(ref mut canvas, _) => canvas,
            _ => unsafe {
                std::hint::unreachable_unchecked();
            },
        }
    }

    #[inline]
    pub fn texture_creator(&mut self) -> &mut TextureCreator<WindowContext> {
        match self {
            WindowOrCanvas::Canvas(_, ref mut texture_creator) => texture_creator,
            _ => unsafe {
                std::hint::unreachable_unchecked();
            },
        }
    }

    pub fn make_canvas(self) -> GameResult<WindowOrCanvas> {
        if let WindowOrCanvas::Win(window) = self {
            let canvas = window
                .into_canvas()
                .accelerated()
                .present_vsync()
                .build()
                .map_err(|e| GameError::RenderError(e.to_string()))?;

            let texture_creator = canvas.texture_creator();

            Ok(WindowOrCanvas::Canvas(canvas, texture_creator))
        } else {
            Ok(self)
        }
    }
}

struct SDL2EventLoop {
    event_pump: EventPump,
    refs: Rc<RefCell<SDL2Context>>,
    opengl_available: RefCell<bool>,
}

pub(crate) struct SDL2Context {
    pub(crate) video: VideoSubsystem,
    pub(crate) window: WindowOrCanvas,
    pub(crate) gl_context: Option<sdl2::video::GLContext>,
    pub(crate) blend_mode: sdl2::render::BlendMode,
    pub(crate) fullscreen_type: sdl2::video::FullscreenType,
    pub(crate) game_controller: GameControllerSubsystem,
}

impl SDL2EventLoop {
    pub fn new(sdl: &Sdl, ctx: &Context) -> GameResult<Box<dyn BackendEventLoop>> {
        let event_pump = sdl.event_pump().map_err(GameError::WindowError)?;
        let video = sdl.video().map_err(GameError::WindowError)?;

        let game_controller = sdl.game_controller().map_err(GameError::GamepadError)?;
        let mut controller_mappings = filesystem::open(ctx, "/builtin/gamecontrollerdb.txt")?;
        game_controller.load_mappings_from_read(&mut controller_mappings).unwrap();

        let gl_attr = video.gl_attr();

        gl_attr.set_context_profile(GLProfile::Compatibility);
        gl_attr.set_context_version(2, 1);

        let mut win_builder =
            video.window("Cave Story (doukutsu-rs)", ctx.window.size_hint.0 as _, ctx.window.size_hint.1 as _);
        win_builder.position_centered();
        win_builder.resizable();

        if ctx.window.mode.is_fullscreen() {
            win_builder.fullscreen();
        }

        #[cfg(feature = "render-opengl")]
        win_builder.opengl();

        let mut window = win_builder.build().map_err(|e| GameError::WindowError(e.to_string()))?;
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "android", target_os = "horizon")))]
        {
            let mut file = filesystem::open(&ctx, "/builtin/icon.bmp").unwrap();
            let mut buf: Vec<u8> = Vec::new();
            file.read_to_end(&mut buf)?;

            let mut rwops = RWops::from_bytes(buf.as_slice()).unwrap();
            let icon = Surface::load_bmp_rw(&mut rwops).unwrap();

            window.set_icon(icon);
        }

        let opengl_available = if let Ok(v) = std::env::var("CAVESTORY_NO_OPENGL") { v != "1" } else { true };

        let event_loop = SDL2EventLoop {
            event_pump,
            refs: Rc::new(RefCell::new(SDL2Context {
                video,
                window: WindowOrCanvas::Win(window),
                gl_context: None,
                blend_mode: sdl2::render::BlendMode::Blend,
                fullscreen_type: sdl2::video::FullscreenType::Off,
                game_controller,
            })),
            opengl_available: RefCell::new(opengl_available),
        };

        Ok(Box::new(event_loop))
    }

    fn set_seamless_titlebar(&mut self, enabled: bool) {
        // #[cfg(target_os = "macos")]
        #[cfg(any())] // buggy, disable for now.
        #[allow(non_upper_case_globals)]
        unsafe {
            use objc2::ffi::*;
            use objc2::*;

            const NSWindowTitleVisible: i32 = 0;
            const NSWindowTitleHidden: i32 = 1;
            const NSWindowStyleMaskFullSizeContentView: u32 = 1 << 15;

            // safety: fields are initialized by SDL_GetWindowWMInfo
            let mut winfo: sdl2_sys::SDL_SysWMinfo = mem::MaybeUninit::zeroed().assume_init();
            winfo.version.major = sdl2_sys::SDL_MAJOR_VERSION as _;
            winfo.version.minor = sdl2_sys::SDL_MINOR_VERSION as _;
            winfo.version.patch = sdl2_sys::SDL_PATCHLEVEL as _;

            let mut whandle = self.refs.deref().borrow().window.window().raw();

            if sdl2_sys::SDL_GetWindowWMInfo(whandle, &mut winfo as *mut _) != sdl2_sys::SDL_bool::SDL_FALSE {
                let window = winfo.info.x11.display as *mut objc2::runtime::AnyObject;

                if enabled {
                    let _: () = msg_send![window, setTitlebarAppearsTransparent:YES];
                    let _: () = msg_send![window, setTitleVisibility:NSWindowTitleHidden];
                } else {
                    let _: () = msg_send![window, setTitlebarAppearsTransparent:NO];
                    let _: () = msg_send![window, setTitleVisibility:NSWindowTitleVisible];
                }

                let mut style_mask: u32 = msg_send![window, styleMask];

                if enabled {
                    style_mask |= NSWindowStyleMaskFullSizeContentView;
                } else {
                    style_mask &= !NSWindowStyleMaskFullSizeContentView;
                }

                let _: () = msg_send![window, setStyleMask: style_mask];
            }
        }

        let _ = enabled;
    }

    fn fullscreen_update(&self, ctx: &mut Context, game: &mut Game) {
        let mut refs = self.refs.borrow_mut();
        let window = refs.window.window_mut();
        let requested_fullscreen_type = ctx.window.mode.get_sdl2_fullscreen_type();
        let current_fullscreen_type = window.fullscreen_state();

        if requested_fullscreen_type != current_fullscreen_type {
            let show_cursor = ctx.window.mode.should_display_mouse_cursor();

            #[cfg(target_os = "macos")]
            let show_cursor = true; // always display it on macOS

            window.set_fullscreen(requested_fullscreen_type);
            window.subsystem().sdl().mouse().show_cursor(show_cursor);

            game.on_fullscreen_state_changed(ctx, ctx.window.mode);
        }
    }

    fn handle_event(event: Event, ctx: &mut Context, game: &mut Game, refs: &Rc<RefCell<SDL2Context>>) {
        macro_rules! handle_err {
            (
                $result:expr
            ) => {
                handle_err_impl($result, &mut ctx.shutdown_requested);
            };
        }

        match event {
            Event::Quit { .. } => {
                ctx.shutdown_requested = true;
            }
            Event::Window { win_event, .. } => match win_event {
                WindowEvent::FocusGained | WindowEvent::Shown => {
                    game.on_focus_gained(ctx);
                }
                WindowEvent::FocusLost | WindowEvent::Hidden => {
                    game.on_focus_lost(ctx);
                }
                WindowEvent::SizeChanged(width, height) => {
                    ctx.screen_size = (width.max(1) as f32, height.max(1) as f32);

                    game.on_resize(ctx);
                }
                _ => {}
            },
            Event::KeyDown { scancode: Some(scancode), repeat, keymod, .. } => {
                if let Some(drs_scan) = conv_scancode(scancode) {
                    if !repeat {
                        game.on_key_down(ctx, drs_scan);
                        if keymod.intersects(keyboard::Mod::RALTMOD | keyboard::Mod::LALTMOD)
                            && drs_scan == ScanCode::Return
                        {
                            let new_mode = match ctx.window.mode {
                                WindowMode::Windowed => WindowMode::Fullscreen,
                                WindowMode::Fullscreen => WindowMode::Windowed,
                            };
                            ctx.window.mode = new_mode;
                        }
                    }
                    ctx.keyboard_context.set_key(drs_scan, true);
                }
            }
            Event::KeyUp { scancode: Some(scancode), .. } => {
                if let Some(drs_scan) = conv_scancode(scancode) {
                    game.on_key_up(ctx, drs_scan);
                    ctx.keyboard_context.set_key(drs_scan, false);
                }
            }
            Event::JoyDeviceAdded { which, .. } => {
                let game_controller = &refs.borrow().game_controller;

                if game_controller.is_game_controller(which) {
                    let controller = game_controller.open(which).unwrap();
                    let id = controller.instance_id();

                    log::info!("Connected gamepad: {} (ID: {})", controller.name(), id);

                    let axis_sensitivity = game.state.get_mut().settings.get_gamepad_axis_sensitivity(which);
                    ctx.gamepad_context.add_gamepad(SDL2Gamepad::new(controller), axis_sensitivity);

                    unsafe {
                        let controller_type =
                            get_game_controller_type(sdl2_sys::SDL_GameControllerTypeForIndex(id as _));
                        ctx.gamepad_context.set_gamepad_type(id, controller_type);
                    }
                }
            }
            Event::ControllerDeviceRemoved { which, .. } => {
                let game_controller = &refs.borrow().game_controller;
                log::info!("Disconnected gamepad with ID {}", which);
                ctx.gamepad_context.remove_gamepad(which);
            }
            Event::ControllerAxisMotion { which, axis, value, .. } => {
                if let Some(drs_axis) = conv_gamepad_axis(axis) {
                    let new_value = (value as f64) / i16::MAX as f64;
                    ctx.gamepad_context.set_axis_value(which, drs_axis, new_value);
                    ctx.gamepad_context.update_axes(which);
                }
            }
            Event::ControllerButtonDown { which, button, .. } => {
                if let Some(drs_button) = conv_gamepad_button(button) {
                    ctx.gamepad_context.set_button(which, drs_button, true);
                }
            }
            Event::ControllerButtonUp { which, button, .. } => {
                if let Some(drs_button) = conv_gamepad_button(button) {
                    ctx.gamepad_context.set_button(which, drs_button, false);
                }
            }
            _ => {}
        }
    }
}

impl BackendEventLoop for SDL2EventLoop {
    fn run(&mut self, mut game: Pin<Box<Game>>, mut ctx: Pin<Box<Context>>) {
        macro_rules! handle_err {
            (
                $result:expr
            ) => {
                handle_err_impl($result, &mut ctx.shutdown_requested);
            };
        }

        const IS_MOBILE: bool = cfg!(any(target_os = "android", target_os = "ios"));
        const IS_CONSOLE: bool = cfg!(any(target_os = "horizon"));

        // SDL has no API for this so we need to guess :)
        ctx.flags.set_supports_windowed_fullscreen(!IS_MOBILE && !IS_CONSOLE);
        ctx.flags.set_has_touch_screen(false); // TODO: implement touch support in SDL backend
        ctx.flags.set_form_factor(match (IS_MOBILE, IS_CONSOLE) {
            (true, _) => DeviceFormFactor::Mobile,
            (_, true) => DeviceFormFactor::Console,
            (false, false) => DeviceFormFactor::Computer,
        });

        {
            let (width, height) = self.refs.deref().borrow().window.window().size();
            ctx.real_screen_size = (width, height);
            ctx.screen_size = (width.max(1) as f32, height.max(1) as f32);

            handle_err!(game.on_resize(&mut ctx));
        }

        loop {
            for event in self.event_pump.poll_iter() {
                Self::handle_event(event, &mut ctx, &mut game, &self.refs);
            }

            self.fullscreen_update(&mut ctx, &mut game);

            if ctx.shutdown_requested {
                log::info!("Shutting down...");
                break;
            }

            {
                if ctx.suspended {
                    let event = self.event_pump.wait_event_timeout(1);
                    if let Some(event) = event {
                        Self::handle_event(event, &mut ctx, &mut game, &self.refs);
                    }
                    continue;
                }
            }

            handle_err!(game.update(&mut ctx));
            handle_err!(game.draw(&mut ctx));
        }
    }

    fn new_renderer(&self, ctx: &mut Context) -> GameResult<Box<dyn BackendRenderer>> {
        #[cfg(feature = "render-opengl")]
        {
            let mut refs = self.refs.borrow_mut();
            match refs.window.window().gl_create_context() {
                Ok(gl_ctx) => {
                    refs.window.window().gl_make_current(&gl_ctx).map_err(|e| GameError::RenderError(e.to_string()))?;
                    refs.gl_context = Some(gl_ctx);
                }
                Err(err) => {
                    *self.opengl_available.borrow_mut() = false;
                    log::error!("Failed to initialize OpenGL context, falling back to SDL2 renderer: {}", err);
                }
            }
        }

        #[cfg(feature = "render-opengl")]
        if *self.opengl_available.borrow() {
            use crate::framework::render::opengl_impl::{GLContextType, GLPlatformFunctions, OpenGLRenderer};

            struct SDL2GLPlatform(Rc<RefCell<SDL2Context>>);

            impl GLPlatformFunctions for SDL2GLPlatform {
                fn get_proc_address(&self, name: &str) -> *const c_void {
                    let refs = self.0.borrow();
                    refs.video.gl_get_proc_address(name) as *const _
                }

                fn swap_buffers(&self) {
                    let mut refs = self.0.borrow();
                    refs.window.window().gl_swap_window();
                }

                fn set_swap_mode(&self, mode: super::graphics::SwapMode) {
                    match mode {
                        SwapMode::Immediate => unsafe {
                            sdl2_sys::SDL_GL_SetSwapInterval(0);
                        },
                        SwapMode::VSync => unsafe {
                            sdl2_sys::SDL_GL_SetSwapInterval(1);
                        },
                        SwapMode::Adaptive => unsafe {
                            if sdl2_sys::SDL_GL_SetSwapInterval(-1) == -1 {
                                log::warn!("Failed to enable variable refresh rate, falling back to non-V-Sync.");
                                sdl2_sys::SDL_GL_SetSwapInterval(0);
                            }
                        },
                    }
                }

                fn get_context_type(&self) -> GLContextType {
                    use sdl2_sys::{SDL_GLattr, SDL_GLprofile};

                    let mut refs = self.0.borrow_mut();
                    let mut attributes = 0;
                    let ok = unsafe {
                        sdl2_sys::SDL_GL_GetAttribute(SDL_GLattr::SDL_GL_CONTEXT_PROFILE_MASK, &mut attributes)
                    };

                    if ok == 0 {
                        if ((attributes as u32) & (SDL_GLprofile::SDL_GL_CONTEXT_PROFILE_ES as u32)) != 0 {
                            return GLContextType::GLES2;
                        } else {
                            return GLContextType::DesktopGL2;
                        }
                    } else {
                        GLContextType::Unknown
                    }
                }
            }

            let platform = Box::new(SDL2GLPlatform(self.refs.clone()));
            return Ok(Box::new(OpenGLRenderer::new(platform)));
        }

        {
            use crate::framework::render::sdl2_impl::SDL2Renderer;

            let mut refs = self.refs.borrow_mut();
            let window = std::mem::take(&mut refs.window);
            refs.window = window.make_canvas()?;
            return Ok(Box::new(SDL2Renderer::new(self.refs.clone())));
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn get_game_controller_type(ctype: sdl2_sys::SDL_GameControllerType) -> GamepadType {
    match ctype as i32 {
        1 => GamepadType::Xbox360,
        2 => GamepadType::XboxOne,
        3 => GamepadType::PS3,
        4 => GamepadType::PS4,
        5 => GamepadType::NintendoSwitchPro,
        6 => GamepadType::Virtual,
        7 => GamepadType::PS5,
        8 => GamepadType::AmazonLuma,
        9 => GamepadType::GoogleStadia,
        10 => GamepadType::NVIDIAShield,
        11 => GamepadType::NintendoSwitchJoyConLeft,
        12 => GamepadType::NintendoSwitchJoyConRight,
        13 => GamepadType::NintendoSwitchJoyConPair,
        _ => GamepadType::Unknown,
    }
}

struct SDL2Gamepad {
    inner: GameController,
}

impl SDL2Gamepad {
    pub fn new(inner: GameController) -> Box<dyn BackendGamepad> {
        Box::new(SDL2Gamepad { inner })
    }
}

impl BackendGamepad for SDL2Gamepad {
    fn set_rumble(&mut self, low_freq: u16, high_freq: u16, duration_ms: u32) -> GameResult {
        let _ = self.inner.set_rumble(low_freq, high_freq, duration_ms);
        Ok(())
    }

    fn instance_id(&self) -> u32 {
        self.inner.instance_id()
    }
}

fn conv_scancode(code: keyboard::Scancode) -> Option<ScanCode> {
    match code {
        Scancode::A => Some(ScanCode::A),
        Scancode::B => Some(ScanCode::B),
        Scancode::C => Some(ScanCode::C),
        Scancode::D => Some(ScanCode::D),
        Scancode::E => Some(ScanCode::E),
        Scancode::F => Some(ScanCode::F),
        Scancode::G => Some(ScanCode::G),
        Scancode::H => Some(ScanCode::H),
        Scancode::I => Some(ScanCode::I),
        Scancode::J => Some(ScanCode::J),
        Scancode::K => Some(ScanCode::K),
        Scancode::L => Some(ScanCode::L),
        Scancode::M => Some(ScanCode::M),
        Scancode::N => Some(ScanCode::N),
        Scancode::O => Some(ScanCode::O),
        Scancode::P => Some(ScanCode::P),
        Scancode::Q => Some(ScanCode::Q),
        Scancode::R => Some(ScanCode::R),
        Scancode::S => Some(ScanCode::S),
        Scancode::T => Some(ScanCode::T),
        Scancode::U => Some(ScanCode::U),
        Scancode::V => Some(ScanCode::V),
        Scancode::W => Some(ScanCode::W),
        Scancode::X => Some(ScanCode::X),
        Scancode::Y => Some(ScanCode::Y),
        Scancode::Z => Some(ScanCode::Z),
        Scancode::Num1 => Some(ScanCode::Key1),
        Scancode::Num2 => Some(ScanCode::Key2),
        Scancode::Num3 => Some(ScanCode::Key3),
        Scancode::Num4 => Some(ScanCode::Key4),
        Scancode::Num5 => Some(ScanCode::Key5),
        Scancode::Num6 => Some(ScanCode::Key6),
        Scancode::Num7 => Some(ScanCode::Key7),
        Scancode::Num8 => Some(ScanCode::Key8),
        Scancode::Num9 => Some(ScanCode::Key9),
        Scancode::Num0 => Some(ScanCode::Key0),
        Scancode::Return => Some(ScanCode::Return),
        Scancode::Escape => Some(ScanCode::Escape),
        Scancode::Backspace => Some(ScanCode::Backspace),
        Scancode::Tab => Some(ScanCode::Tab),
        Scancode::Space => Some(ScanCode::Space),
        Scancode::Minus => Some(ScanCode::Minus),
        Scancode::Equals => Some(ScanCode::Equals),
        Scancode::LeftBracket => Some(ScanCode::LBracket),
        Scancode::RightBracket => Some(ScanCode::RBracket),
        Scancode::Backslash => Some(ScanCode::Backslash),
        Scancode::NonUsHash => Some(ScanCode::NonUsHash),
        Scancode::Semicolon => Some(ScanCode::Semicolon),
        Scancode::Apostrophe => Some(ScanCode::Apostrophe),
        Scancode::Grave => Some(ScanCode::Grave),
        Scancode::Comma => Some(ScanCode::Comma),
        Scancode::Period => Some(ScanCode::Period),
        Scancode::Slash => Some(ScanCode::Slash),
        Scancode::CapsLock => Some(ScanCode::Capslock),
        Scancode::F1 => Some(ScanCode::F1),
        Scancode::F2 => Some(ScanCode::F2),
        Scancode::F3 => Some(ScanCode::F3),
        Scancode::F4 => Some(ScanCode::F4),
        Scancode::F5 => Some(ScanCode::F5),
        Scancode::F6 => Some(ScanCode::F6),
        Scancode::F7 => Some(ScanCode::F7),
        Scancode::F8 => Some(ScanCode::F8),
        Scancode::F9 => Some(ScanCode::F9),
        Scancode::F10 => Some(ScanCode::F10),
        Scancode::F11 => Some(ScanCode::F11),
        Scancode::F12 => Some(ScanCode::F12),
        Scancode::PrintScreen => Some(ScanCode::Sysrq),
        Scancode::ScrollLock => Some(ScanCode::Scrolllock),
        Scancode::Pause => Some(ScanCode::Pause),
        Scancode::Insert => Some(ScanCode::Insert),
        Scancode::Home => Some(ScanCode::Home),
        Scancode::PageUp => Some(ScanCode::PageUp),
        Scancode::Delete => Some(ScanCode::Delete),
        Scancode::End => Some(ScanCode::End),
        Scancode::PageDown => Some(ScanCode::PageDown),
        Scancode::Right => Some(ScanCode::Right),
        Scancode::Left => Some(ScanCode::Left),
        Scancode::Down => Some(ScanCode::Down),
        Scancode::Up => Some(ScanCode::Up),
        Scancode::NumLockClear => Some(ScanCode::Numlock),
        Scancode::KpDivide => Some(ScanCode::NumpadDivide),
        Scancode::KpMultiply => Some(ScanCode::NumpadMultiply),
        Scancode::KpMinus => Some(ScanCode::NumpadSubtract),
        Scancode::KpPlus => Some(ScanCode::NumpadAdd),
        Scancode::KpEnter => Some(ScanCode::NumpadEnter),
        Scancode::Kp1 => Some(ScanCode::Numpad1),
        Scancode::Kp2 => Some(ScanCode::Numpad2),
        Scancode::Kp3 => Some(ScanCode::Numpad3),
        Scancode::Kp4 => Some(ScanCode::Numpad4),
        Scancode::Kp5 => Some(ScanCode::Numpad5),
        Scancode::Kp6 => Some(ScanCode::Numpad6),
        Scancode::Kp7 => Some(ScanCode::Numpad7),
        Scancode::Kp8 => Some(ScanCode::Numpad8),
        Scancode::Kp9 => Some(ScanCode::Numpad9),
        Scancode::Kp0 => Some(ScanCode::Numpad0),
        Scancode::NonUsBackslash => Some(ScanCode::NonUsBackslash),
        Scancode::Application => Some(ScanCode::Apps),
        Scancode::Power => Some(ScanCode::Power),
        Scancode::KpEquals => Some(ScanCode::NumpadEquals),
        Scancode::F13 => Some(ScanCode::F13),
        Scancode::F14 => Some(ScanCode::F14),
        Scancode::F15 => Some(ScanCode::F15),
        Scancode::F16 => Some(ScanCode::F16),
        Scancode::F17 => Some(ScanCode::F17),
        Scancode::F18 => Some(ScanCode::F18),
        Scancode::F19 => Some(ScanCode::F19),
        Scancode::F20 => Some(ScanCode::F20),
        Scancode::F21 => Some(ScanCode::F21),
        Scancode::F22 => Some(ScanCode::F22),
        Scancode::F23 => Some(ScanCode::F23),
        Scancode::F24 => Some(ScanCode::F24),
        Scancode::Stop => Some(ScanCode::Stop),
        Scancode::Cut => Some(ScanCode::Cut),
        Scancode::Copy => Some(ScanCode::Copy),
        Scancode::Paste => Some(ScanCode::Paste),
        Scancode::Mute => Some(ScanCode::Mute),
        Scancode::VolumeUp => Some(ScanCode::VolumeUp),
        Scancode::VolumeDown => Some(ScanCode::VolumeDown),
        Scancode::KpComma => Some(ScanCode::NumpadComma),
        Scancode::SysReq => Some(ScanCode::Sysrq),
        Scancode::Return2 => Some(ScanCode::NumpadEnter),
        Scancode::LCtrl => Some(ScanCode::LControl),
        Scancode::LShift => Some(ScanCode::LShift),
        Scancode::LAlt => Some(ScanCode::LAlt),
        Scancode::LGui => Some(ScanCode::LWin),
        Scancode::RCtrl => Some(ScanCode::RControl),
        Scancode::RShift => Some(ScanCode::RShift),
        Scancode::RAlt => Some(ScanCode::RAlt),
        Scancode::RGui => Some(ScanCode::RWin),
        Scancode::AudioNext => Some(ScanCode::NextTrack),
        Scancode::AudioPrev => Some(ScanCode::PrevTrack),
        Scancode::AudioStop => Some(ScanCode::MediaStop),
        Scancode::AudioPlay => Some(ScanCode::PlayPause),
        Scancode::AudioMute => Some(ScanCode::Mute),
        Scancode::MediaSelect => Some(ScanCode::MediaSelect),
        Scancode::Mail => Some(ScanCode::Mail),
        Scancode::Calculator => Some(ScanCode::Calculator),
        Scancode::Sleep => Some(ScanCode::Sleep),
        _ => None,
    }
}

fn conv_gamepad_button(code: controller::Button) -> Option<Button> {
    match code {
        controller::Button::A => Some(Button::South),
        controller::Button::B => Some(Button::East),
        controller::Button::X => Some(Button::West),
        controller::Button::Y => Some(Button::North),
        controller::Button::Back => Some(Button::Back),
        controller::Button::Guide => Some(Button::Guide),
        controller::Button::Start => Some(Button::Start),
        controller::Button::LeftStick => Some(Button::LeftStick),
        controller::Button::RightStick => Some(Button::RightStick),
        controller::Button::LeftShoulder => Some(Button::LeftShoulder),
        controller::Button::RightShoulder => Some(Button::RightShoulder),
        controller::Button::DPadUp => Some(Button::DPadUp),
        controller::Button::DPadDown => Some(Button::DPadDown),
        controller::Button::DPadLeft => Some(Button::DPadLeft),
        controller::Button::DPadRight => Some(Button::DPadRight),
        _ => None,
    }
}

fn conv_gamepad_axis(code: controller::Axis) -> Option<Axis> {
    match code {
        controller::Axis::LeftX => Some(Axis::LeftX),
        controller::Axis::LeftY => Some(Axis::LeftY),
        controller::Axis::RightX => Some(Axis::RightX),
        controller::Axis::RightY => Some(Axis::RightY),
        controller::Axis::TriggerLeft => Some(Axis::TriggerLeft),
        controller::Axis::TriggerRight => Some(Axis::TriggerRight),
        _ => None,
    }
}

// based on imgui-sdl2 crate
pub struct ImguiSdl2 {
    mouse_press: [bool; 5],
    ignore_mouse: bool,
    ignore_keyboard: bool,
    cursor: Option<MouseCursor>,
    sdl_cursor: Option<Cursor>,
}

struct Sdl2ClipboardBackend(sdl2::clipboard::ClipboardUtil);

impl imgui::ClipboardBackend for Sdl2ClipboardBackend {
    fn get(&mut self) -> Option<String> {
        if !self.0.has_clipboard_text() {
            return None;
        }

        self.0.clipboard_text().ok()
    }

    fn set(&mut self, value: &str) {
        let _ = self.0.set_clipboard_text(value);
    }
}

impl ImguiSdl2 {
    pub fn new(imgui: &mut imgui::Context, window: &sdl2::video::Window) -> Self {
        let clipboard_util = window.subsystem().clipboard();
        imgui.set_clipboard_backend(Sdl2ClipboardBackend(clipboard_util));

        imgui.io_mut().key_map[Key::Tab as usize] = Scancode::Tab as u32;
        imgui.io_mut().key_map[Key::LeftArrow as usize] = Scancode::Left as u32;
        imgui.io_mut().key_map[Key::RightArrow as usize] = Scancode::Right as u32;
        imgui.io_mut().key_map[Key::UpArrow as usize] = Scancode::Up as u32;
        imgui.io_mut().key_map[Key::DownArrow as usize] = Scancode::Down as u32;
        imgui.io_mut().key_map[Key::PageUp as usize] = Scancode::PageUp as u32;
        imgui.io_mut().key_map[Key::PageDown as usize] = Scancode::PageDown as u32;
        imgui.io_mut().key_map[Key::Home as usize] = Scancode::Home as u32;
        imgui.io_mut().key_map[Key::End as usize] = Scancode::End as u32;
        imgui.io_mut().key_map[Key::Delete as usize] = Scancode::Delete as u32;
        imgui.io_mut().key_map[Key::Backspace as usize] = Scancode::Backspace as u32;
        imgui.io_mut().key_map[Key::Enter as usize] = Scancode::Return as u32;
        imgui.io_mut().key_map[Key::Escape as usize] = Scancode::Escape as u32;
        imgui.io_mut().key_map[Key::Space as usize] = Scancode::Space as u32;
        imgui.io_mut().key_map[Key::A as usize] = Scancode::A as u32;
        imgui.io_mut().key_map[Key::C as usize] = Scancode::C as u32;
        imgui.io_mut().key_map[Key::V as usize] = Scancode::V as u32;
        imgui.io_mut().key_map[Key::X as usize] = Scancode::X as u32;
        imgui.io_mut().key_map[Key::Y as usize] = Scancode::Y as u32;
        imgui.io_mut().key_map[Key::Z as usize] = Scancode::Z as u32;

        Self { mouse_press: [false; 5], ignore_keyboard: false, ignore_mouse: false, cursor: None, sdl_cursor: None }
    }

    pub fn handle_event(&mut self, imgui: &mut imgui::Context, event: &Event) {
        use sdl2::mouse::MouseButton;

        fn set_mod(imgui: &mut imgui::Context, keymod: keyboard::Mod) {
            let ctrl = keymod.intersects(keyboard::Mod::RCTRLMOD | keyboard::Mod::LCTRLMOD);
            let alt = keymod.intersects(keyboard::Mod::RALTMOD | keyboard::Mod::LALTMOD);
            let shift = keymod.intersects(keyboard::Mod::RSHIFTMOD | keyboard::Mod::LSHIFTMOD);
            let super_ = keymod.intersects(keyboard::Mod::RGUIMOD | keyboard::Mod::LGUIMOD);

            imgui.io_mut().key_ctrl = ctrl;
            imgui.io_mut().key_alt = alt;
            imgui.io_mut().key_shift = shift;
            imgui.io_mut().key_super = super_;
        }

        match *event {
            Event::MouseWheel { y, .. } => {
                imgui.io_mut().mouse_wheel = y as f32;
            }
            Event::MouseButtonDown { mouse_btn, .. } => {
                if mouse_btn != MouseButton::Unknown {
                    let index = match mouse_btn {
                        MouseButton::Left => 0,
                        MouseButton::Right => 1,
                        MouseButton::Middle => 2,
                        MouseButton::X1 => 3,
                        MouseButton::X2 => 4,
                        MouseButton::Unknown => unreachable!(),
                    };
                    self.mouse_press[index] = true;
                }
            }
            Event::TextInput { ref text, .. } => {
                for chr in text.chars() {
                    imgui.io_mut().add_input_character(chr);
                }
            }
            Event::KeyDown { scancode, keymod, .. } => {
                set_mod(imgui, keymod);
                if let Some(scancode) = scancode {
                    imgui.io_mut().keys_down[scancode as usize] = true;
                }
            }
            Event::KeyUp { scancode, keymod, .. } => {
                set_mod(imgui, keymod);
                if let Some(scancode) = scancode {
                    imgui.io_mut().keys_down[scancode as usize] = false;
                }
            }
            _ => {}
        }
    }

    pub fn prepare_frame(
        &mut self,
        io: &mut imgui::Io,
        window: &sdl2::video::Window,
        mouse_state: &sdl2::mouse::MouseState,
    ) {
        let mouse_util = window.subsystem().sdl().mouse();

        let (win_w, win_h) = window.size();
        let (draw_w, draw_h) = window.drawable_size();

        io.display_size = [win_w as f32, win_h as f32];
        io.display_framebuffer_scale = [(draw_w as f32) / (win_w as f32), (draw_h as f32) / (win_h as f32)];

        // Merging the mousedown events we received into the current state prevents us from missing
        // clicks that happen faster than a frame
        io.mouse_down = [
            self.mouse_press[0] || mouse_state.left(),
            self.mouse_press[1] || mouse_state.right(),
            self.mouse_press[2] || mouse_state.middle(),
            self.mouse_press[3] || mouse_state.x1(),
            self.mouse_press[4] || mouse_state.x2(),
        ];
        self.mouse_press = [false; 5];

        let any_mouse_down = io.mouse_down.iter().any(|&b| b);
        mouse_util.capture(any_mouse_down);

        io.mouse_pos = [mouse_state.x() as f32, mouse_state.y() as f32];

        self.ignore_keyboard = io.want_capture_keyboard;
        self.ignore_mouse = io.want_capture_mouse;
    }

    pub fn prepare_render(&mut self, ui: &imgui::Ui, window: &sdl2::video::Window) {
        let io = ui.io();
        if !io.config_flags.contains(ConfigFlags::NO_MOUSE_CURSOR_CHANGE) {
            let mouse_util = window.subsystem().sdl().mouse();

            match ui.mouse_cursor() {
                Some(mouse_cursor) if !io.mouse_draw_cursor => {
                    mouse_util.show_cursor(true);

                    let sdl_cursor = match mouse_cursor {
                        MouseCursor::Arrow => SystemCursor::Arrow,
                        MouseCursor::TextInput => SystemCursor::IBeam,
                        MouseCursor::ResizeAll => SystemCursor::SizeAll,
                        MouseCursor::ResizeNS => SystemCursor::SizeNS,
                        MouseCursor::ResizeEW => SystemCursor::SizeWE,
                        MouseCursor::ResizeNESW => SystemCursor::SizeNESW,
                        MouseCursor::ResizeNWSE => SystemCursor::SizeNWSE,
                        MouseCursor::Hand => SystemCursor::Hand,
                        MouseCursor::NotAllowed => SystemCursor::No,
                    };

                    if self.cursor != Some(mouse_cursor) {
                        let sdl_cursor = Cursor::from_system(sdl_cursor).unwrap();
                        sdl_cursor.set();
                        self.cursor = Some(mouse_cursor);
                        self.sdl_cursor = Some(sdl_cursor);
                    }
                }
                _ => {
                    self.cursor = None;
                    self.sdl_cursor = None;
                    mouse_util.show_cursor(false);
                }
            }
        }
    }
}
