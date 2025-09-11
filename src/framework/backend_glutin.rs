use std::any::Any;
use std::cell::{RefCell, UnsafeCell};
use std::ffi::c_void;
use std::io::Read;
use std::mem;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::vec::Vec;

use glutin::event::{ElementState, Event, TouchPhase, VirtualKeyCode, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::{Fullscreen, WindowBuilder};
use glutin::{Api, ContextBuilder, GlProfile, GlRequest, PossiblyCurrent, WindowedContext};
use imgui::{DrawCmdParams, DrawData, DrawIdx, DrawVert};
use winit::window::Icon;

use super::backend::{
    Backend, BackendCallbacks, BackendEventLoop, BackendRenderer, BackendTexture, DeviceFormFactor, SpriteBatchCommand,
    WindowParams,
};
use super::context::Context;
use super::error::GameResult;
use super::keyboard::ScanCode;
use super::render_opengl::OpenGLRenderer;
use super::{filesystem, render_opengl};
use crate::common::Rect;
use crate::framework::error::GameError;
use crate::framework::render_opengl::GLContextType;
use crate::game::shared_game_state::WindowMode;
use crate::game::Game;
use crate::game::GAME_SUSPENDED;
use crate::input::touch_controls::TouchPoint;

trait WindowModeExt {
    fn get_glutin_fullscreen_type(&self) -> Option<Fullscreen>;
}

impl WindowModeExt for WindowMode {
    fn get_glutin_fullscreen_type(&self) -> Option<Fullscreen> {
        match self {
            WindowMode::Windowed => None,
            WindowMode::Fullscreen => Some(Fullscreen::Borderless(None)),
        }
    }
}

pub struct GlutinBackend;

impl GlutinBackend {
    pub fn new(window_params: WindowParams) -> GameResult<Box<dyn Backend>> {
        Ok(Box::new(GlutinBackend))
    }
}

impl Backend for GlutinBackend {
    fn create_event_loop(&self, _ctx: &Context) -> GameResult<Box<dyn BackendEventLoop>> {
        Ok(Box::new(GlutinEventLoop { refs: Rc::new(RefCell::new(None)) }))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

type ContextHandle = Rc<WindowedContext<PossiblyCurrent>>;
type Refs = Rc<RefCell<Option<ContextHandle>>>;

pub struct GlutinEventLoop {
    refs: Refs,
}

impl GlutinEventLoop {
    fn get_context(&mut self, ctx: &Context, event_loop: &EventLoop<()>) -> ContextHandle {
        if let Some(rc) = self.refs.borrow().as_ref() {
            return rc.clone();
        }

        #[cfg(target_os = "android")]
        loop {
            match ndk_glue::native_window().as_ref() {
                Some(_) => {
                    log::info!("NativeWindow Found: {:?}", ndk_glue::native_window());
                    break;
                }
                None => (),
            }
        }

        let mut window = WindowBuilder::new();

        let windowed_context = ContextBuilder::new();
        let windowed_context = windowed_context.with_gl(GlRequest::Latest);
        #[cfg(target_os = "android")]
        let windowed_context = windowed_context //
            .with_gl(GlRequest::Specific(Api::OpenGlEs, (2, 0)))
            .with_gl_debug_flag(false);

        let windowed_context = windowed_context //
            .with_gl_profile(GlProfile::Compatibility)
            .with_pixel_format(24, 8)
            .with_vsync(true);

        #[cfg(target_os = "windows")]
        {
            use glutin::platform::windows::WindowBuilderExtWindows;
            window = window.with_drag_and_drop(false);
        }

        window = window.with_title("doukutsu-rs");

        #[cfg(not(any(target_os = "windows", target_os = "android", target_os = "horizon")))]
        {
            let mut file = filesystem::open(&ctx, "/builtin/icon.bmp").unwrap();
            let mut buf: Vec<u8> = Vec::new();
            file.read_to_end(&mut buf);

            let mut img = match image::load_from_memory_with_format(buf.as_slice(), image::ImageFormat::Bmp) {
                Ok(image) => image.into_rgba8(),
                Err(e) => panic!("Cannot set window icon"),
            };

            let (width, height) = img.dimensions();
            let icon = Icon::from_rgba(img.into_raw(), width, height).unwrap();

            window = window.with_window_icon(Some(icon));
        }

        let windowed_context = windowed_context.build_windowed(window, event_loop);
        if let Err(e) = &windowed_context {
            log::error!("Failed to build windowed context: {}", e);
        }
        let windowed_context = windowed_context.unwrap();

        let windowed_context = unsafe { windowed_context.make_current().expect("Failed to make current") };

        #[cfg(target_os = "android")]
        if let Some(nwin) = ndk_glue::native_window().as_ref() {
            unsafe {
                windowed_context.surface_created(nwin.ptr().as_ptr() as *mut std::ffi::c_void);
            }
        }

        let rc = Rc::new(windowed_context);
        *self.refs.borrow_mut() = Some(rc.clone());

        rc
    }
}

#[cfg(target_os = "android")]
fn request_android_redraw() {
    match ndk_glue::native_window().as_ref() {
        Some(native_window) => {
            let a_native_window: *mut ndk_sys::ANativeWindow = native_window.ptr().as_ptr();
            let a_native_activity: *mut ndk_sys::ANativeActivity = ndk_glue::native_activity().ptr().as_ptr();
            unsafe {
                match (*(*a_native_activity).callbacks).onNativeWindowRedrawNeeded {
                    Some(callback) => callback(a_native_activity, a_native_window),
                    None => (),
                };
            };
        }
        None => (),
    }
}

#[cfg(target_os = "android")]
fn get_insets() -> GameResult<(f32, f32, f32, f32)> {
    unsafe {
        use jni::objects::JObject;
        use jni::JavaVM;

        let vm_ptr = ndk_glue::native_activity().vm();
        let vm = JavaVM::from_raw(vm_ptr)?;
        let vm_env = vm.attach_current_thread()?;

        let class = vm_env.new_global_ref(JObject::from_raw(ndk_glue::native_activity().activity()))?;
        let field = vm_env.get_field(class.as_obj(), "displayInsets", "[I")?.to_jni().l as jni::sys::jintArray;

        let mut elements = [0; 4];
        vm_env.get_int_array_region(field, 0, &mut elements)?;

        vm_env.delete_local_ref(JObject::from_raw(field));

        //Game always runs with horizontal orientation so top and bottom cutouts not needed and only wastes piece of the screen
        elements[1] = 0;
        elements[3] = 0;

        Ok((elements[0] as f32, elements[1] as f32, elements[2] as f32, elements[3] as f32))
    }
}

fn get_scaled_size(width: u32, height: u32) -> (f32, f32) {
    let scaled_height = ((height / 480).max(1) * 480) as f32;
    let scaled_width = (width as f32 * (scaled_height as f32 / height as f32)).floor();

    (scaled_width, scaled_height)
}

impl BackendEventLoop for GlutinEventLoop {
    fn run(&mut self, mut game: Pin<Box<Game>>, mut ctx: Pin<Box<Context>>) {
        const IS_MOBILE: bool = cfg!(any(target_os = "android", target_os = "ios"));
        ctx.flags.set_supports_windowed_fullscreen(!IS_MOBILE);
        ctx.flags.set_has_touch_screen(IS_MOBILE); // TODO: how do we not assume Android always has a touch screen?
        ctx.flags.set_form_factor(if IS_MOBILE { DeviceFormFactor::Mobile } else { DeviceFormFactor::Computer });

        fn handle_err_impl(result: GameResult, shutdown_requested: &mut bool) {
            if let Err(e) = result {
                log::error!("{}", e);
                *shutdown_requested = true;
            }
        };

        macro_rules! handle_err {
            (
                $result:expr
            ) => {
                handle_err_impl($result, &mut ctx.shutdown_requested);
            };
        }

        let event_loop = EventLoop::new();
        let window = self.get_context(&ctx, &event_loop);
        {
            let size = window.window().inner_size();
            ctx.real_screen_size = (size.width, size.height);
            ctx.screen_size = get_scaled_size(size.width.max(1), size.height.max(1));

            handle_err!(game.on_resize(&mut ctx));
        }

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, window_id }
                    if window_id == window.window().id() =>
                {
                    ctx.shutdown_requested = true;
                }
                Event::Resumed => {
                    handle_err!(game.on_focus_gained(&mut ctx));

                    #[cfg(target_os = "android")]
                    if let Some(nwin) = ndk_glue::native_window().as_ref() {
                        handle_err!(game.on_context_lost(&mut ctx));
                        unsafe {
                            window.surface_created(nwin.ptr().as_ptr() as *mut std::ffi::c_void);
                            request_android_redraw();
                        }
                    }
                }
                Event::Suspended => {
                    handle_err!(game.on_focus_lost(&mut ctx));

                    #[cfg(target_os = "android")]
                    unsafe {
                        window.surface_destroyed();
                    }
                }
                Event::WindowEvent { event: WindowEvent::Resized(size), window_id }
                    if window_id == window.window().id() =>
                {
                    if let Some(renderer) = &ctx.renderer {
                        ctx.real_screen_size = (size.width, size.height);
                        ctx.screen_size = get_scaled_size(size.width.max(1), size.height.max(1));
                        handle_err!(game.on_resize(&mut ctx));
                    }
                }
                Event::WindowEvent { event: WindowEvent::Touch(touch), window_id }
                    if window_id == window.window().id() =>
                {
                    let state_ref = game.state.get_mut();
                    let mut controls = &mut state_ref.touch_controls;
                    let scale = state_ref.scale as f64;
                    let loc_x = (touch.location.x * ctx.screen_size.0 as f64 / ctx.real_screen_size.0 as f64) / scale;
                    let loc_y = (touch.location.y * ctx.screen_size.1 as f64 / ctx.real_screen_size.1 as f64) / scale;

                    match touch.phase {
                        TouchPhase::Started | TouchPhase::Moved => {
                            if let Some(point) = controls.points.iter_mut().find(|p| p.id == touch.id) {
                                point.last_position = point.position;
                                point.position = (loc_x, loc_y);
                            } else {
                                controls.touch_id_counter = controls.touch_id_counter.wrapping_add(1);

                                let point = TouchPoint {
                                    id: touch.id,
                                    touch_id: controls.touch_id_counter,
                                    position: (loc_x, loc_y),
                                    last_position: (0.0, 0.0),
                                };
                                controls.points.push(point);

                                if touch.phase == TouchPhase::Started {
                                    controls.clicks.push(point);
                                }
                            }
                        }
                        TouchPhase::Ended | TouchPhase::Cancelled => {
                            controls.points.retain(|p| p.id != touch.id);
                            controls.clicks.retain(|p| p.id != touch.id);
                        }
                    }
                }
                Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, window_id }
                    if window_id == window.window().id() =>
                {
                    if let Some(keycode) = input.virtual_keycode {
                        if let Some(drs_scan) = conv_keycode(keycode) {
                            let key_state = match input.state {
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            };

                            ctx.keyboard_context.set_key(drs_scan, key_state);
                        }
                    }
                }
                Event::RedrawRequested(id) if id == window.window().id() => {
                    {
                        let mutex = GAME_SUSPENDED.lock().unwrap();
                        if *mutex {
                            return;
                        }
                    }

                    #[cfg(not(target_os = "android"))]
                    {
                        if let Err(err) = game.draw(&mut ctx) {
                            log::error!("Failed to draw frame: {}", err);
                        }

                        window.window().request_redraw();
                    }

                    #[cfg(target_os = "android")]
                    request_android_redraw();
                }
                Event::MainEventsCleared => {
                    if ctx.shutdown_requested {
                        log::info!("Shutting down...");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    {
                        let mutex = GAME_SUSPENDED.lock().unwrap();
                        if *mutex {
                            return;
                        }
                    }

                    #[cfg(not(any(target_os = "android", target_os = "horizon")))]
                    {
                        if ctx.window.mode.get_glutin_fullscreen_type() != window.window().fullscreen() {
                            let fullscreen_type = ctx.window.mode.get_glutin_fullscreen_type();
                            let cursor_visible = ctx.window.mode.should_display_mouse_cursor();

                            window.window().set_fullscreen(fullscreen_type);
                            window.window().set_cursor_visible(cursor_visible);
                        }
                    }

                    if let Err(err) = game.update(&mut ctx) {
                        log::error!("Update loop returned an error: {}", err);
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    #[cfg(target_os = "android")]
                    {
                        match get_insets() {
                            Ok(insets) => {
                                ctx.screen_insets = insets;
                            }
                            Err(e) => {
                                log::error!("Failed to update insets: {}", e);
                            }
                        }

                        if let Err(err) = game.draw(ctx) {
                            log::error!("Failed to draw frame: {}", err);
                        }
                    }
                }
                _ => (),
            }
        });
    }

    fn new_renderer(&self, ctx: &mut Context) -> GameResult<Box<dyn BackendRenderer>> {
        struct GlutinGLPlatform(Refs);

        impl render_opengl::GLPlatformFunctions for GlutinGLPlatform {
            fn get_proc_address(&self, name: &str) -> *const c_void {
                let window = self.0.borrow();
                if let Some(window) = window.as_ref() {
                    window.get_proc_address(name)
                } else {
                    std::ptr::null()
                }
            }

            fn swap_buffers(&self) {
                let window = self.0.borrow();
                if let Some(window) = window.as_ref() {
                    let _ = window.swap_buffers();
                }
            }

            fn set_swap_mode(&self, mode: super::graphics::SwapMode) {
                // Not supported by glutin rn
                let _ = mode;
            }

            fn get_context_type(&self) -> GLContextType {
                let window = self.0.borrow();
                let window = window.as_ref().expect("get_context_type called before context is available");
                match window.get_api() {
                    Api::OpenGl => GLContextType::DesktopGL2,
                    Api::OpenGlEs => GLContextType::GLES2,
                    Api::WebGl => GLContextType::GLES2, // TODO
                }
            }
        }

        let platform = Box::new(GlutinGLPlatform(self.refs.clone()));

        Ok(Box::new(OpenGLRenderer::new(platform)))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn conv_keycode(code: VirtualKeyCode) -> Option<ScanCode> {
    match code {
        VirtualKeyCode::Key1 => Some(ScanCode::Key1),
        VirtualKeyCode::Key2 => Some(ScanCode::Key2),
        VirtualKeyCode::Key3 => Some(ScanCode::Key3),
        VirtualKeyCode::Key4 => Some(ScanCode::Key4),
        VirtualKeyCode::Key5 => Some(ScanCode::Key5),
        VirtualKeyCode::Key6 => Some(ScanCode::Key6),
        VirtualKeyCode::Key7 => Some(ScanCode::Key7),
        VirtualKeyCode::Key8 => Some(ScanCode::Key8),
        VirtualKeyCode::Key9 => Some(ScanCode::Key9),
        VirtualKeyCode::Key0 => Some(ScanCode::Key0),
        VirtualKeyCode::A => Some(ScanCode::A),
        VirtualKeyCode::B => Some(ScanCode::B),
        VirtualKeyCode::C => Some(ScanCode::C),
        VirtualKeyCode::D => Some(ScanCode::D),
        VirtualKeyCode::E => Some(ScanCode::E),
        VirtualKeyCode::F => Some(ScanCode::F),
        VirtualKeyCode::G => Some(ScanCode::G),
        VirtualKeyCode::H => Some(ScanCode::H),
        VirtualKeyCode::I => Some(ScanCode::I),
        VirtualKeyCode::J => Some(ScanCode::J),
        VirtualKeyCode::K => Some(ScanCode::K),
        VirtualKeyCode::L => Some(ScanCode::L),
        VirtualKeyCode::M => Some(ScanCode::M),
        VirtualKeyCode::N => Some(ScanCode::N),
        VirtualKeyCode::O => Some(ScanCode::O),
        VirtualKeyCode::P => Some(ScanCode::P),
        VirtualKeyCode::Q => Some(ScanCode::Q),
        VirtualKeyCode::R => Some(ScanCode::R),
        VirtualKeyCode::S => Some(ScanCode::S),
        VirtualKeyCode::T => Some(ScanCode::T),
        VirtualKeyCode::U => Some(ScanCode::U),
        VirtualKeyCode::V => Some(ScanCode::V),
        VirtualKeyCode::W => Some(ScanCode::W),
        VirtualKeyCode::X => Some(ScanCode::X),
        VirtualKeyCode::Y => Some(ScanCode::Y),
        VirtualKeyCode::Z => Some(ScanCode::Z),
        VirtualKeyCode::Escape => Some(ScanCode::Escape),
        VirtualKeyCode::F1 => Some(ScanCode::F1),
        VirtualKeyCode::F2 => Some(ScanCode::F2),
        VirtualKeyCode::F3 => Some(ScanCode::F3),
        VirtualKeyCode::F4 => Some(ScanCode::F4),
        VirtualKeyCode::F5 => Some(ScanCode::F5),
        VirtualKeyCode::F6 => Some(ScanCode::F6),
        VirtualKeyCode::F7 => Some(ScanCode::F7),
        VirtualKeyCode::F8 => Some(ScanCode::F8),
        VirtualKeyCode::F9 => Some(ScanCode::F9),
        VirtualKeyCode::F10 => Some(ScanCode::F10),
        VirtualKeyCode::F11 => Some(ScanCode::F11),
        VirtualKeyCode::F12 => Some(ScanCode::F12),
        VirtualKeyCode::F13 => Some(ScanCode::F13),
        VirtualKeyCode::F14 => Some(ScanCode::F14),
        VirtualKeyCode::F15 => Some(ScanCode::F15),
        VirtualKeyCode::F16 => Some(ScanCode::F16),
        VirtualKeyCode::F17 => Some(ScanCode::F17),
        VirtualKeyCode::F18 => Some(ScanCode::F18),
        VirtualKeyCode::F19 => Some(ScanCode::F19),
        VirtualKeyCode::F20 => Some(ScanCode::F20),
        VirtualKeyCode::F21 => Some(ScanCode::F21),
        VirtualKeyCode::F22 => Some(ScanCode::F22),
        VirtualKeyCode::F23 => Some(ScanCode::F23),
        VirtualKeyCode::F24 => Some(ScanCode::F24),
        VirtualKeyCode::Snapshot => Some(ScanCode::Snapshot),
        VirtualKeyCode::Scroll => Some(ScanCode::Scroll),
        VirtualKeyCode::Pause => Some(ScanCode::Pause),
        VirtualKeyCode::Insert => Some(ScanCode::Insert),
        VirtualKeyCode::Home => Some(ScanCode::Home),
        VirtualKeyCode::Delete => Some(ScanCode::Delete),
        VirtualKeyCode::End => Some(ScanCode::End),
        VirtualKeyCode::PageDown => Some(ScanCode::PageDown),
        VirtualKeyCode::PageUp => Some(ScanCode::PageUp),
        VirtualKeyCode::Left => Some(ScanCode::Left),
        VirtualKeyCode::Up => Some(ScanCode::Up),
        VirtualKeyCode::Right => Some(ScanCode::Right),
        VirtualKeyCode::Down => Some(ScanCode::Down),
        VirtualKeyCode::Back => Some(ScanCode::Back),
        VirtualKeyCode::Return => Some(ScanCode::Return),
        VirtualKeyCode::Space => Some(ScanCode::Space),
        VirtualKeyCode::Compose => Some(ScanCode::Compose),
        VirtualKeyCode::Caret => Some(ScanCode::Caret),
        VirtualKeyCode::Numlock => Some(ScanCode::Numlock),
        VirtualKeyCode::Numpad0 => Some(ScanCode::Numpad0),
        VirtualKeyCode::Numpad1 => Some(ScanCode::Numpad1),
        VirtualKeyCode::Numpad2 => Some(ScanCode::Numpad2),
        VirtualKeyCode::Numpad3 => Some(ScanCode::Numpad3),
        VirtualKeyCode::Numpad4 => Some(ScanCode::Numpad4),
        VirtualKeyCode::Numpad5 => Some(ScanCode::Numpad5),
        VirtualKeyCode::Numpad6 => Some(ScanCode::Numpad6),
        VirtualKeyCode::Numpad7 => Some(ScanCode::Numpad7),
        VirtualKeyCode::Numpad8 => Some(ScanCode::Numpad8),
        VirtualKeyCode::Numpad9 => Some(ScanCode::Numpad9),
        VirtualKeyCode::NumpadAdd => Some(ScanCode::NumpadAdd),
        VirtualKeyCode::NumpadDivide => Some(ScanCode::NumpadDivide),
        VirtualKeyCode::NumpadDecimal => Some(ScanCode::NumpadDecimal),
        VirtualKeyCode::NumpadComma => Some(ScanCode::NumpadComma),
        VirtualKeyCode::NumpadEnter => Some(ScanCode::NumpadEnter),
        VirtualKeyCode::NumpadEquals => Some(ScanCode::NumpadEquals),
        VirtualKeyCode::NumpadMultiply => Some(ScanCode::NumpadMultiply),
        VirtualKeyCode::NumpadSubtract => Some(ScanCode::NumpadSubtract),
        VirtualKeyCode::AbntC1 => Some(ScanCode::AbntC1),
        VirtualKeyCode::AbntC2 => Some(ScanCode::AbntC2),
        VirtualKeyCode::Apostrophe => Some(ScanCode::Apostrophe),
        VirtualKeyCode::Apps => Some(ScanCode::Apps),
        VirtualKeyCode::Asterisk => Some(ScanCode::Asterisk),
        VirtualKeyCode::At => Some(ScanCode::At),
        VirtualKeyCode::Ax => Some(ScanCode::Ax),
        VirtualKeyCode::Backslash => Some(ScanCode::Backslash),
        VirtualKeyCode::Calculator => Some(ScanCode::Calculator),
        VirtualKeyCode::Capital => Some(ScanCode::Capital),
        VirtualKeyCode::Colon => Some(ScanCode::Colon),
        VirtualKeyCode::Comma => Some(ScanCode::Comma),
        VirtualKeyCode::Convert => Some(ScanCode::Convert),
        VirtualKeyCode::Equals => Some(ScanCode::Equals),
        VirtualKeyCode::Grave => Some(ScanCode::Grave),
        VirtualKeyCode::Kana => Some(ScanCode::Kana),
        VirtualKeyCode::Kanji => Some(ScanCode::Kanji),
        VirtualKeyCode::LAlt => Some(ScanCode::LAlt),
        VirtualKeyCode::LBracket => Some(ScanCode::LBracket),
        VirtualKeyCode::LControl => Some(ScanCode::LControl),
        VirtualKeyCode::LShift => Some(ScanCode::LShift),
        VirtualKeyCode::LWin => Some(ScanCode::LWin),
        VirtualKeyCode::Mail => Some(ScanCode::Mail),
        VirtualKeyCode::MediaSelect => Some(ScanCode::MediaSelect),
        VirtualKeyCode::MediaStop => Some(ScanCode::MediaStop),
        VirtualKeyCode::Minus => Some(ScanCode::Minus),
        VirtualKeyCode::Mute => Some(ScanCode::Mute),
        VirtualKeyCode::MyComputer => Some(ScanCode::MyComputer),
        VirtualKeyCode::NavigateForward => Some(ScanCode::NavigateForward),
        VirtualKeyCode::NavigateBackward => Some(ScanCode::NavigateBackward),
        VirtualKeyCode::NextTrack => Some(ScanCode::NextTrack),
        VirtualKeyCode::NoConvert => Some(ScanCode::NoConvert),
        VirtualKeyCode::OEM102 => Some(ScanCode::OEM102),
        VirtualKeyCode::Period => Some(ScanCode::Period),
        VirtualKeyCode::PlayPause => Some(ScanCode::PlayPause),
        VirtualKeyCode::Plus => Some(ScanCode::Plus),
        VirtualKeyCode::Power => Some(ScanCode::Power),
        VirtualKeyCode::PrevTrack => Some(ScanCode::PrevTrack),
        VirtualKeyCode::RAlt => Some(ScanCode::RAlt),
        VirtualKeyCode::RBracket => Some(ScanCode::RBracket),
        VirtualKeyCode::RControl => Some(ScanCode::RControl),
        VirtualKeyCode::RShift => Some(ScanCode::RShift),
        VirtualKeyCode::RWin => Some(ScanCode::RWin),
        VirtualKeyCode::Semicolon => Some(ScanCode::Semicolon),
        VirtualKeyCode::Slash => Some(ScanCode::Slash),
        VirtualKeyCode::Sleep => Some(ScanCode::Sleep),
        VirtualKeyCode::Stop => Some(ScanCode::Stop),
        VirtualKeyCode::Sysrq => Some(ScanCode::Sysrq),
        VirtualKeyCode::Tab => Some(ScanCode::Tab),
        VirtualKeyCode::Underline => Some(ScanCode::Underline),
        VirtualKeyCode::Unlabeled => Some(ScanCode::Unlabeled),
        VirtualKeyCode::VolumeDown => Some(ScanCode::VolumeDown),
        VirtualKeyCode::VolumeUp => Some(ScanCode::VolumeUp),
        VirtualKeyCode::Wake => Some(ScanCode::Wake),
        VirtualKeyCode::WebBack => Some(ScanCode::WebBack),
        VirtualKeyCode::WebFavorites => Some(ScanCode::WebFavorites),
        VirtualKeyCode::WebForward => Some(ScanCode::WebForward),
        VirtualKeyCode::WebHome => Some(ScanCode::WebHome),
        VirtualKeyCode::WebRefresh => Some(ScanCode::WebRefresh),
        VirtualKeyCode::WebSearch => Some(ScanCode::WebSearch),
        VirtualKeyCode::WebStop => Some(ScanCode::WebStop),
        VirtualKeyCode::Yen => Some(ScanCode::Yen),
        VirtualKeyCode::Copy => Some(ScanCode::Copy),
        VirtualKeyCode::Paste => Some(ScanCode::Paste),
        VirtualKeyCode::Cut => Some(ScanCode::Cut),
    }
}
