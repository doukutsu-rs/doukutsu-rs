use std::any::Any;
use std::cell::{RefCell, UnsafeCell};
use std::ffi::c_void;
use std::io::Read;
use std::mem;
use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::Arc;
use std::vec::Vec;

use imgui::{DrawCmdParams, DrawData, DrawIdx, DrawVert};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, TouchPhase, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{self, Icon, Window, WindowAttributes, WindowId};

use crate::common::Rect;
use crate::framework::backend::{Backend, BackendEventLoop, BackendRenderer, BackendTexture, SpriteBatchCommand};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::framework::gl;
use crate::framework::keyboard::ScanCode;
use crate::framework::render_opengl::{GLContext, OpenGLRenderer};
use crate::game::GAME_SUSPENDED;
use crate::game::{stage, Game};
use crate::input::touch_controls::TouchPoint;

use super::error::GameError;
use super::ui::init_imgui;

pub struct WinitBackend;

impl WinitBackend {
    pub fn new() -> GameResult<Box<dyn Backend>> {
        Ok(Box::new(WinitBackend))
    }
}

impl Backend for WinitBackend {
    fn create_event_loop(&self, ctx: &Context) -> GameResult<Box<dyn BackendEventLoop>> {
        let event_loop =
            EventLoop::new().map_err(|e| GameError::WindowError(format!("Failed to create event loop: {}", e)))?;

        Ok(Box::new(WinitEventLoop {
            event_loop: Some(event_loop),
            window: None,
            game_ref: null_mut(),
            ctx_ref: null_mut(),
        }))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct WinitEventLoop {
    event_loop: Option<EventLoop<()>>,
    window: Option<Window>,
    game_ref: *mut Game,
    ctx_ref: *mut Context,
}

impl WinitEventLoop {
    fn get_insets(&self) -> GameResult<(f32, f32, f32, f32)> {
        #[cfg(target_os = "android")]
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

            //Game always runs with horizontal orientation so top and bottom cutouts are not needed and only waste piece of the screen
            elements[1] = 0;
            elements[3] = 0;

            return Ok((elements[0] as f32, elements[1] as f32, elements[2] as f32, elements[3] as f32));
        }

        Ok((0.0, 0.0, 0.0, 0.0))
    }

    #[inline]
    fn game<'a, 'b: 'a>(&'a self) -> &'b mut Game {
        unsafe { &mut *self.game_ref }
    }

    #[inline]
    fn ctx<'a, 'b: 'a>(&'a self) -> &'b mut Context {
        unsafe { &mut *self.ctx_ref }
    }

    fn create_initial_window(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes();
        let attrs = attrs.with_title(self.ctx().window_title.clone());

        // TODO: how to handle those errors?
        let window = event_loop.create_window(attrs).unwrap();

        {
            let size = window.inner_size();
            let ctx = self.ctx();
            let game = self.game();

            ctx.real_screen_size = (size.width, size.height);
            ctx.screen_size = get_scaled_size(size.width.max(1), size.height.max(1));
            game.state.get_mut().handle_resize(ctx).unwrap();
        }

        self.window = Some(window);
    }
}

fn get_scaled_size(width: u32, height: u32) -> (f32, f32) {
    let scaled_height = ((height / 480).max(1) * 480) as f32;
    let scaled_width = (width as f32 * (scaled_height as f32 / height as f32)).floor();

    (scaled_width, scaled_height)
}

impl ApplicationHandler for WinitEventLoop {
    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let mut mutex = GAME_SUSPENDED.lock().unwrap();
        *mutex = true;

        self.game().state.borrow_mut().sound_manager.resume();
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_initial_window(event_loop);
        let mut mutex = GAME_SUSPENDED.lock().unwrap();
        *mutex = false;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let window = if let Some(window) = &self.window {
            window
        } else {
            return;
        };

        let ctx = self.ctx();

        match event {
            WindowEvent::CloseRequested if window_id == window.id() => {
                ctx.shutdown_requested = true;
            }
            WindowEvent::Resized(size) if window_id == window.id() => {
                if let Some(renderer) = &ctx.renderer {
                    if let Ok(imgui) = renderer.imgui() {
                        imgui.io_mut().display_size = [size.width as f32, size.height as f32];
                    }

                    ctx.real_screen_size = (size.width, size.height);
                    ctx.screen_size = get_scaled_size(size.width.max(1), size.height.max(1));
                    self.game().state.get_mut().handle_resize(ctx).unwrap();
                }
            }
            WindowEvent::Touch(touch) if window_id == window.id() => {
                let state_ref = self.game().state.get_mut();
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
            WindowEvent::KeyboardInput { event, .. } if window_id == window.id() => {
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    if let Some(drs_scan) = conv_keycode(key_code) {
                        let key_state = match event.state {
                            ElementState::Pressed => true,
                            ElementState::Released => false,
                        };

                        ctx.keyboard_context.set_key(drs_scan, key_state);
                    }
                }
            }
            WindowEvent::RedrawRequested if window_id == window.id() => {
                {
                    let mutex = GAME_SUSPENDED.lock().unwrap();
                    if *mutex {
                        return;
                    }
                }

                {
                    if let Err(err) = self.game().draw(ctx) {
                        log::error!("Failed to draw frame: {}", err);
                    }

                    window.request_redraw();
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let ctx = self.ctx();
        let game = self.game();

        if ctx.shutdown_requested {
            log::info!("Shutting down...");
            event_loop.exit();
            return;
        }

        {
            let mutex = GAME_SUSPENDED.lock().unwrap();
            if *mutex {
                return;
            }
        }

        let window = if let Some(window) = &self.window {
            window
        } else {
            return;
        };

        // #[cfg(not(any(target_os = "android", target_os = "ios")))]
        // {
        //     if state_ref.settings.window_mode.get_glutin_fullscreen_type() != window.fullscreen() {
        //         let fullscreen_type = state_ref.settings.window_mode.get_glutin_fullscreen_type();
        //         let cursor_visible = state_ref.settings.window_mode.should_display_mouse_cursor();

        //         window.set_fullscreen(fullscreen_type);
        //         window.set_cursor_visible(cursor_visible);
        //     }
        // }

        game.update(ctx).unwrap();

        match self.get_insets() {
            Ok(insets) => {
                ctx.screen_insets = insets;
            }
            Err(e) => {
                log::error!("Failed to update insets: {}", e);
            }
        }

        window.request_redraw();
    }
}

impl BackendEventLoop for WinitEventLoop {
    fn run(&mut self, game: &mut Game, ctx: &mut Context) {
        self.game_ref = game as *mut _;
        self.ctx_ref = ctx as *mut _;

        let event_loop = std::mem::take(&mut self.event_loop).unwrap();
        event_loop.run_app(self);
    }

    fn new_renderer(&self, ctx: *mut Context) -> GameResult<Box<dyn BackendRenderer>> {
        let mut imgui = init_imgui()?;
        imgui.io_mut().display_size = [640.0, 480.0];

        // let refs = self.refs.clone();
        // let user_data = Rc::into_raw(refs) as *mut c_void;

        // unsafe fn get_proc_address(user_data: &mut *mut c_void, name: &str) -> *const c_void {
        //     let refs = Rc::from_raw(*user_data as *mut UnsafeCell<Option<WindowedContext<PossiblyCurrent>>>);

        //     let result = {
        //         let refs = &mut *refs.get();

        //         if let Some(refs) = refs {
        //             refs.get_proc_address(name)
        //         } else {
        //             std::ptr::null()
        //         }
        //     };

        //     *user_data = Rc::into_raw(refs) as *mut c_void;

        //     result
        // }

        // unsafe fn swap_buffers(user_data: &mut *mut c_void) {
        //     let refs = Rc::from_raw(*user_data as *mut UnsafeCell<Option<WindowedContext<PossiblyCurrent>>>);

        //     {
        //         let refs = &mut *refs.get();

        //         if let Some(refs) = refs {
        //             refs.swap_buffers();
        //         }
        //     }

        //     *user_data = Rc::into_raw(refs) as *mut c_void;
        // }

        // let gl_context = GLContext { gles2_mode: true, is_sdl: false, get_proc_address, swap_buffers, user_data, ctx };

        // Ok(Box::new(OpenGLRenderer::new(gl_context, imgui)))
        Ok(Box::new(super::backend_null::NullRenderer::new(imgui)))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn conv_keycode(code: KeyCode) -> Option<ScanCode> {
    match code {
        KeyCode::KeyA => Some(ScanCode::A),
        KeyCode::KeyB => Some(ScanCode::B),
        KeyCode::KeyC => Some(ScanCode::C),
        KeyCode::KeyD => Some(ScanCode::D),
        KeyCode::KeyE => Some(ScanCode::E),
        KeyCode::KeyF => Some(ScanCode::F),
        KeyCode::KeyG => Some(ScanCode::G),
        KeyCode::KeyH => Some(ScanCode::H),
        KeyCode::KeyI => Some(ScanCode::I),
        KeyCode::KeyJ => Some(ScanCode::J),
        KeyCode::KeyK => Some(ScanCode::K),
        KeyCode::KeyL => Some(ScanCode::L),
        KeyCode::KeyM => Some(ScanCode::M),
        KeyCode::KeyN => Some(ScanCode::N),
        KeyCode::KeyO => Some(ScanCode::O),
        KeyCode::KeyP => Some(ScanCode::P),
        KeyCode::KeyQ => Some(ScanCode::Q),
        KeyCode::KeyR => Some(ScanCode::R),
        KeyCode::KeyS => Some(ScanCode::S),
        KeyCode::KeyT => Some(ScanCode::T),
        KeyCode::KeyU => Some(ScanCode::U),
        KeyCode::KeyV => Some(ScanCode::V),
        KeyCode::KeyW => Some(ScanCode::W),
        KeyCode::KeyX => Some(ScanCode::X),
        KeyCode::KeyY => Some(ScanCode::Y),
        KeyCode::KeyZ => Some(ScanCode::Z),
        KeyCode::Digit1 => Some(ScanCode::Key1),
        KeyCode::Digit2 => Some(ScanCode::Key2),
        KeyCode::Digit3 => Some(ScanCode::Key3),
        KeyCode::Digit4 => Some(ScanCode::Key4),
        KeyCode::Digit5 => Some(ScanCode::Key5),
        KeyCode::Digit6 => Some(ScanCode::Key6),
        KeyCode::Digit7 => Some(ScanCode::Key7),
        KeyCode::Digit8 => Some(ScanCode::Key8),
        KeyCode::Digit9 => Some(ScanCode::Key9),
        KeyCode::Digit0 => Some(ScanCode::Key0),
        KeyCode::Enter => Some(ScanCode::Return),
        KeyCode::Escape => Some(ScanCode::Escape),
        KeyCode::Backspace => Some(ScanCode::Backspace),
        KeyCode::Tab => Some(ScanCode::Tab),
        KeyCode::Space => Some(ScanCode::Space),
        KeyCode::Minus => Some(ScanCode::Minus),
        KeyCode::Equal => Some(ScanCode::Equals),
        KeyCode::BracketLeft => Some(ScanCode::LBracket),
        KeyCode::BracketRight => Some(ScanCode::RBracket),
        KeyCode::Backslash => Some(ScanCode::Backslash),
        KeyCode::Semicolon => Some(ScanCode::Semicolon),
        KeyCode::Quote => Some(ScanCode::Apostrophe),
        KeyCode::Backquote => Some(ScanCode::Grave),
        KeyCode::Comma => Some(ScanCode::Comma),
        KeyCode::Period => Some(ScanCode::Period),
        KeyCode::Slash => Some(ScanCode::Slash),
        KeyCode::CapsLock => Some(ScanCode::Capslock),
        KeyCode::F1 => Some(ScanCode::F1),
        KeyCode::F2 => Some(ScanCode::F2),
        KeyCode::F3 => Some(ScanCode::F3),
        KeyCode::F4 => Some(ScanCode::F4),
        KeyCode::F5 => Some(ScanCode::F5),
        KeyCode::F6 => Some(ScanCode::F6),
        KeyCode::F7 => Some(ScanCode::F7),
        KeyCode::F8 => Some(ScanCode::F8),
        KeyCode::F9 => Some(ScanCode::F9),
        KeyCode::F10 => Some(ScanCode::F10),
        KeyCode::F11 => Some(ScanCode::F11),
        KeyCode::F12 => Some(ScanCode::F12),
        KeyCode::PrintScreen => Some(ScanCode::Sysrq),
        KeyCode::ScrollLock => Some(ScanCode::Scrolllock),
        KeyCode::Pause => Some(ScanCode::Pause),
        KeyCode::Insert => Some(ScanCode::Insert),
        KeyCode::Home => Some(ScanCode::Home),
        KeyCode::PageUp => Some(ScanCode::PageUp),
        KeyCode::Delete => Some(ScanCode::Delete),
        KeyCode::End => Some(ScanCode::End),
        KeyCode::PageDown => Some(ScanCode::PageDown),
        KeyCode::ArrowRight => Some(ScanCode::Right),
        KeyCode::ArrowLeft => Some(ScanCode::Left),
        KeyCode::ArrowDown => Some(ScanCode::Down),
        KeyCode::ArrowUp => Some(ScanCode::Up),
        KeyCode::NumLock => Some(ScanCode::Numlock),
        KeyCode::NumpadDivide => Some(ScanCode::NumpadDivide),
        KeyCode::NumpadMultiply => Some(ScanCode::NumpadMultiply),
        KeyCode::NumpadSubtract => Some(ScanCode::NumpadSubtract),
        KeyCode::NumpadAdd => Some(ScanCode::NumpadAdd),
        KeyCode::NumpadEnter => Some(ScanCode::NumpadEnter),
        KeyCode::Numpad1 => Some(ScanCode::Numpad1),
        KeyCode::Numpad2 => Some(ScanCode::Numpad2),
        KeyCode::Numpad3 => Some(ScanCode::Numpad3),
        KeyCode::Numpad4 => Some(ScanCode::Numpad4),
        KeyCode::Numpad5 => Some(ScanCode::Numpad5),
        KeyCode::Numpad6 => Some(ScanCode::Numpad6),
        KeyCode::Numpad7 => Some(ScanCode::Numpad7),
        KeyCode::Numpad8 => Some(ScanCode::Numpad8),
        KeyCode::Numpad9 => Some(ScanCode::Numpad9),
        KeyCode::Numpad0 => Some(ScanCode::Numpad0),
        KeyCode::ContextMenu => Some(ScanCode::Apps),
        KeyCode::Power => Some(ScanCode::Power),
        KeyCode::NumpadEqual => Some(ScanCode::NumpadEquals),
        KeyCode::F13 => Some(ScanCode::F13),
        KeyCode::F14 => Some(ScanCode::F14),
        KeyCode::F15 => Some(ScanCode::F15),
        KeyCode::F16 => Some(ScanCode::F16),
        KeyCode::F17 => Some(ScanCode::F17),
        KeyCode::F18 => Some(ScanCode::F18),
        KeyCode::F19 => Some(ScanCode::F19),
        KeyCode::F20 => Some(ScanCode::F20),
        KeyCode::F21 => Some(ScanCode::F21),
        KeyCode::F22 => Some(ScanCode::F22),
        KeyCode::F23 => Some(ScanCode::F23),
        KeyCode::F24 => Some(ScanCode::F24),
        KeyCode::MediaStop => Some(ScanCode::Stop),
        KeyCode::Cut => Some(ScanCode::Cut),
        KeyCode::Copy => Some(ScanCode::Copy),
        KeyCode::Paste => Some(ScanCode::Paste),
        KeyCode::AudioVolumeMute => Some(ScanCode::Mute),
        KeyCode::AudioVolumeUp => Some(ScanCode::VolumeUp),
        KeyCode::AudioVolumeDown => Some(ScanCode::VolumeDown),
        KeyCode::NumpadComma => Some(ScanCode::NumpadComma),
        KeyCode::ControlLeft => Some(ScanCode::LControl),
        KeyCode::ShiftLeft => Some(ScanCode::LShift),
        KeyCode::AltLeft => Some(ScanCode::LAlt),
        KeyCode::SuperLeft => Some(ScanCode::LWin),
        KeyCode::ControlRight => Some(ScanCode::RControl),
        KeyCode::ShiftRight => Some(ScanCode::RShift),
        KeyCode::AltRight => Some(ScanCode::RAlt),
        KeyCode::SuperRight => Some(ScanCode::RWin),
        KeyCode::MediaTrackNext => Some(ScanCode::NextTrack),
        KeyCode::MediaTrackPrevious => Some(ScanCode::PrevTrack),
        KeyCode::MediaStop => Some(ScanCode::MediaStop),
        KeyCode::MediaPlayPause => Some(ScanCode::PlayPause),
        KeyCode::MediaSelect => Some(ScanCode::MediaSelect),
        KeyCode::LaunchMail => Some(ScanCode::Mail),
        KeyCode::LaunchApp2 => Some(ScanCode::Calculator),
        KeyCode::Sleep => Some(ScanCode::Sleep),
        _ => None,
    }
}
