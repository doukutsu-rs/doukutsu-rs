use std::cell::RefCell;
use std::mem;
use std::ops::DerefMut;
use std::time::Duration;

use imgui::{DrawData, Ui};
use ndk::input_queue::InputQueue;
use sokol::app::{SApp, SAppDesc, SAppEvent, SAppEventType, SAppKeycode};
use sokol::gfx::{sg_isvalid, sg_query_backend, sg_shutdown};

use crate::common::{Color, Rect};
use crate::framework::backend::{Backend, BackendEventLoop, BackendRenderer, BackendTexture, SpriteBatchCommand};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics::BlendMode;
use crate::framework::keyboard::ScanCode;
use crate::Game;

pub struct SokolBackend;

impl SokolBackend {
    pub fn new() -> GameResult<Box<dyn Backend>> {
        Ok(Box::new(SokolBackend))
    }
}

impl Backend for SokolBackend {
    fn create_event_loop(&self) -> GameResult<Box<dyn BackendEventLoop>> {
        Ok(Box::new(SokolEventLoop))
    }
}

pub struct SokolEventLoop;

#[cfg(target_os = "android")]
extern "C" {
    fn sapp_android_on_create(
        activity: *mut ndk_sys::ANativeActivity,
        window: *mut ndk_sys::ANativeWindow,
        input_queue: *mut ndk_sys::AInputQueue,
    );
}

impl BackendEventLoop for SokolEventLoop {
    fn run(&mut self, game: &mut Game, ctx: &mut Context) {
        #[cfg(target_os = "android")]
        unsafe {
            let activity = ndk_glue::native_activity().ptr().as_ptr();
            let window = match ndk_glue::native_window().as_ref() {
                None => std::ptr::null_mut(),
                Some(p) => p.ptr().as_ptr(),
            };
            let input_queue = match ndk_glue::input_queue().as_ref() {
                None => std::ptr::null_mut(),
                Some(p) => p.ptr().as_ptr(),
            };

            println!("activity = {:?} window = {:?} input_queue = {:?}", activity, window, input_queue);

            sapp_android_on_create(activity, window, input_queue);
        }

        struct Callbacks<'a, 'b> {
            ctx: &'a mut Context,
            game: &'b mut Game,
        };

        impl<'a, 'b> SApp for Callbacks<'a, 'b> {
            fn sapp_init(&mut self) {
                let state_ref = unsafe { &mut *self.game.state.get() };

                self.ctx.screen_size = (640.0, 480.0);
                state_ref.handle_resize(self.ctx).unwrap();
            }

            fn sapp_frame(&mut self) {
                let state_ref = unsafe { &mut *self.game.state.get() };

                self.game.update(self.ctx).unwrap();

                // todo: not really supported on iOS/consoles
                if state_ref.shutdown {
                    log::info!("Shutting down...");
                    std::process::exit(0);
                    return;
                }

                if state_ref.next_scene.is_some() {
                    mem::swap(&mut self.game.scene, &mut state_ref.next_scene);
                    state_ref.next_scene = None;
                    self.game.scene.as_mut().unwrap().init(state_ref, self.ctx).unwrap();
                    self.game.loops = 0;
                    state_ref.frame_time = 0.0;
                }

                self.game.draw(self.ctx).unwrap();
            }

            fn sapp_cleanup(&mut self) {
                if sg_isvalid() {
                    sg_shutdown();
                }
            }

            fn sapp_event(&mut self, event: SAppEvent) {
                let state_ref = unsafe { &mut *self.game.state.get() };
                println!("event: {:?}", event.event_type);

                match event.event_type {
                    SAppEventType::Invalid => {}
                    SAppEventType::KeyDown => {
                        if let Some(drs_scan) = conv_scancode(event.key_code) {
                            state_ref.process_debug_keys(drs_scan);
                            self.ctx.keyboard_context.set_key(drs_scan, true);
                        }
                    }
                    SAppEventType::KeyUp => {
                        if let Some(drs_scan) = conv_scancode(event.key_code) {
                            self.ctx.keyboard_context.set_key(drs_scan, false);
                        }
                    }
                    SAppEventType::Char => {}
                    SAppEventType::MouseDown => {}
                    SAppEventType::MouseUp => {}
                    SAppEventType::MouseScroll => {}
                    SAppEventType::MouseMove => {}
                    SAppEventType::MouseEnter => {}
                    SAppEventType::MouseLeave => {}
                    SAppEventType::TouchesBegan => {}
                    SAppEventType::TouchesMoved => {}
                    SAppEventType::TouchesEnded => {}
                    SAppEventType::TouchesCancelled => {}
                    SAppEventType::Resized => {}
                    SAppEventType::Iconified => {}
                    SAppEventType::Restored => {}
                    SAppEventType::Suspended => {}
                    SAppEventType::Resumed => {}
                    SAppEventType::UpdateCursor => {}
                    SAppEventType::QuitRequested => {
                        state_ref.shutdown();
                    }
                }
            }
        }

        sokol::app::sapp_run(
            Callbacks { ctx: unsafe { std::mem::transmute(ctx) }, game: unsafe { std::mem::transmute(game) } },
            SAppDesc {
                width: 640,
                height: 480,
                window_title: "doukutsu-rs".to_string(),
                ios_keyboard_resizes_canvas: false,
                ..Default::default()
            },
        );

        loop {
            std::thread::sleep(Duration::from_millis(10))
        }
    }

    fn new_renderer(&self) -> GameResult<Box<dyn BackendRenderer>> {
        let mut imgui = imgui::Context::create();
        imgui.io_mut().display_size = [640.0, 480.0];
        imgui.fonts().build_alpha8_texture();

        log::info!("Using Sokol backend: {:?}", sg_query_backend());

        Ok(Box::new(SokolRenderer(RefCell::new(imgui))))
    }
}

pub struct NullTexture(u16, u16);

impl BackendTexture for NullTexture {
    fn dimensions(&self) -> (u16, u16) {
        (self.0, self.1)
    }

    fn add(&mut self, command: SpriteBatchCommand) {}

    fn clear(&mut self) {}

    fn draw(&mut self) -> GameResult<()> {
        Ok(())
    }
}

pub struct SokolRenderer(RefCell<imgui::Context>);

impl BackendRenderer for SokolRenderer {
    fn clear(&mut self, color: Color) {}

    fn present(&mut self) -> GameResult {
        Ok(())
    }

    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
        Ok(Box::new(NullTexture(width, height)))
    }

    fn create_texture(&mut self, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
        Ok(Box::new(NullTexture(width, height)))
    }

    fn set_blend_mode(&mut self, blend: BlendMode) -> GameResult {
        Ok(())
    }

    fn set_render_target(&mut self, texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
        Ok(())
    }

    fn draw_rect(&mut self, rect: Rect<isize>, color: Color) -> GameResult {
        Ok(())
    }

    fn draw_outline_rect(&mut self, rect: Rect<isize>, line_width: usize, color: Color) -> GameResult {
        Ok(())
    }

    fn imgui(&self) -> GameResult<&mut imgui::Context> {
        unsafe { Ok(&mut *self.0.as_ptr()) }
    }

    fn render_imgui(&mut self, draw_data: &DrawData) -> GameResult {
        Ok(())
    }

    fn prepare_frame<'ui>(&self, ui: &Ui<'ui>) -> GameResult {
        Ok(())
    }
}

fn conv_scancode(code: SAppKeycode) -> Option<ScanCode> {
    match code {
        SAppKeycode::KeySpace => Some(ScanCode::Space),
        SAppKeycode::KeyApostrophe => Some(ScanCode::Apostrophe),
        SAppKeycode::KeyComma => Some(ScanCode::Comma),
        SAppKeycode::KeyMinus => Some(ScanCode::Minus),
        SAppKeycode::KeyPeriod => Some(ScanCode::Period),
        SAppKeycode::KeySlash => Some(ScanCode::Slash),
        SAppKeycode::Key0 => Some(ScanCode::Key0),
        SAppKeycode::Key1 => Some(ScanCode::Key1),
        SAppKeycode::Key2 => Some(ScanCode::Key2),
        SAppKeycode::Key3 => Some(ScanCode::Key3),
        SAppKeycode::Key4 => Some(ScanCode::Key4),
        SAppKeycode::Key5 => Some(ScanCode::Key5),
        SAppKeycode::Key6 => Some(ScanCode::Key6),
        SAppKeycode::Key7 => Some(ScanCode::Key7),
        SAppKeycode::Key8 => Some(ScanCode::Key8),
        SAppKeycode::Key9 => Some(ScanCode::Key9),
        SAppKeycode::KeySemicolon => Some(ScanCode::Semicolon),
        SAppKeycode::KeyEqual => Some(ScanCode::Equals),
        SAppKeycode::KeyA => Some(ScanCode::A),
        SAppKeycode::KeyB => Some(ScanCode::B),
        SAppKeycode::KeyC => Some(ScanCode::C),
        SAppKeycode::KeyD => Some(ScanCode::D),
        SAppKeycode::KeyE => Some(ScanCode::E),
        SAppKeycode::KeyF => Some(ScanCode::F),
        SAppKeycode::KeyG => Some(ScanCode::G),
        SAppKeycode::KeyH => Some(ScanCode::H),
        SAppKeycode::KeyI => Some(ScanCode::I),
        SAppKeycode::KeyJ => Some(ScanCode::J),
        SAppKeycode::KeyK => Some(ScanCode::K),
        SAppKeycode::KeyL => Some(ScanCode::L),
        SAppKeycode::KeyM => Some(ScanCode::M),
        SAppKeycode::KeyN => Some(ScanCode::N),
        SAppKeycode::KeyO => Some(ScanCode::O),
        SAppKeycode::KeyP => Some(ScanCode::P),
        SAppKeycode::KeyQ => Some(ScanCode::Q),
        SAppKeycode::KeyR => Some(ScanCode::R),
        SAppKeycode::KeyS => Some(ScanCode::S),
        SAppKeycode::KeyT => Some(ScanCode::T),
        SAppKeycode::KeyU => Some(ScanCode::U),
        SAppKeycode::KeyV => Some(ScanCode::V),
        SAppKeycode::KeyW => Some(ScanCode::W),
        SAppKeycode::KeyX => Some(ScanCode::X),
        SAppKeycode::KeyY => Some(ScanCode::Y),
        SAppKeycode::KeyZ => Some(ScanCode::Z),
        SAppKeycode::KeyLeftBracket => Some(ScanCode::LBracket),
        SAppKeycode::KeyBackslash => Some(ScanCode::Backslash),
        SAppKeycode::KeyRightBracket => Some(ScanCode::RBracket),
        SAppKeycode::KeyGraveAccent => Some(ScanCode::Grave),
        SAppKeycode::KeyWorld1 => Some(ScanCode::AbntC1),
        SAppKeycode::KeyWorld2 => Some(ScanCode::AbntC2),
        SAppKeycode::KeyEscape => Some(ScanCode::Escape),
        SAppKeycode::KeyEnter => Some(ScanCode::Return),
        SAppKeycode::KeyTab => Some(ScanCode::Tab),
        SAppKeycode::KeyBackspace => Some(ScanCode::Backspace),
        SAppKeycode::KeyInsert => Some(ScanCode::Insert),
        SAppKeycode::KeyDelete => Some(ScanCode::Delete),
        SAppKeycode::KeyRight => Some(ScanCode::Right),
        SAppKeycode::KeyLeft => Some(ScanCode::Left),
        SAppKeycode::KeyDown => Some(ScanCode::Down),
        SAppKeycode::KeyUp => Some(ScanCode::Up),
        SAppKeycode::KeyPageUp => Some(ScanCode::PageUp),
        SAppKeycode::KeyPageDown => Some(ScanCode::PageDown),
        SAppKeycode::KeyHome => Some(ScanCode::Home),
        SAppKeycode::KeyEnd => Some(ScanCode::End),
        SAppKeycode::KeyCapsLock => Some(ScanCode::Capslock),
        SAppKeycode::KeyScrollLock => Some(ScanCode::Scrolllock),
        SAppKeycode::KeyNumLock => Some(ScanCode::Numlock),
        SAppKeycode::KeyPrintScreen => Some(ScanCode::Sysrq),
        SAppKeycode::KeyPause => Some(ScanCode::Pause),
        SAppKeycode::KeyF1 => Some(ScanCode::F1),
        SAppKeycode::KeyF2 => Some(ScanCode::F2),
        SAppKeycode::KeyF3 => Some(ScanCode::F3),
        SAppKeycode::KeyF4 => Some(ScanCode::F4),
        SAppKeycode::KeyF5 => Some(ScanCode::F5),
        SAppKeycode::KeyF6 => Some(ScanCode::F6),
        SAppKeycode::KeyF7 => Some(ScanCode::F7),
        SAppKeycode::KeyF8 => Some(ScanCode::F8),
        SAppKeycode::KeyF9 => Some(ScanCode::F9),
        SAppKeycode::KeyF10 => Some(ScanCode::F10),
        SAppKeycode::KeyF11 => Some(ScanCode::F11),
        SAppKeycode::KeyF12 => Some(ScanCode::F12),
        SAppKeycode::KeyF13 => Some(ScanCode::F13),
        SAppKeycode::KeyF14 => Some(ScanCode::F14),
        SAppKeycode::KeyF15 => Some(ScanCode::F15),
        SAppKeycode::KeyF16 => Some(ScanCode::F16),
        SAppKeycode::KeyF17 => Some(ScanCode::F17),
        SAppKeycode::KeyF18 => Some(ScanCode::F18),
        SAppKeycode::KeyF19 => Some(ScanCode::F19),
        SAppKeycode::KeyF20 => Some(ScanCode::F20),
        SAppKeycode::KeyF21 => Some(ScanCode::F21),
        SAppKeycode::KeyF22 => Some(ScanCode::F22),
        SAppKeycode::KeyF23 => Some(ScanCode::F23),
        SAppKeycode::KeyF24 => Some(ScanCode::F24),
        SAppKeycode::KeyKP0 => Some(ScanCode::Numpad0),
        SAppKeycode::KeyKP1 => Some(ScanCode::Numpad1),
        SAppKeycode::KeyKP2 => Some(ScanCode::Numpad2),
        SAppKeycode::KeyKP3 => Some(ScanCode::Numpad3),
        SAppKeycode::KeyKP4 => Some(ScanCode::Numpad4),
        SAppKeycode::KeyKP5 => Some(ScanCode::Numpad5),
        SAppKeycode::KeyKP6 => Some(ScanCode::Numpad6),
        SAppKeycode::KeyKP7 => Some(ScanCode::Numpad7),
        SAppKeycode::KeyKP8 => Some(ScanCode::Numpad8),
        SAppKeycode::KeyKP9 => Some(ScanCode::Numpad9),
        SAppKeycode::KeyKPDecimal => Some(ScanCode::NumpadDecimal),
        SAppKeycode::KeyKPDivide => Some(ScanCode::NumpadDivide),
        SAppKeycode::KeyKPMultiply => Some(ScanCode::NumpadMultiply),
        SAppKeycode::KeyKPSubtract => Some(ScanCode::NumpadSubtract),
        SAppKeycode::KeyKPAdd => Some(ScanCode::NumpadAdd),
        SAppKeycode::KeyKPEnter => Some(ScanCode::NumpadEnter),
        SAppKeycode::KeyKPEqual => Some(ScanCode::NumpadEquals),
        SAppKeycode::KeyLeftShift => Some(ScanCode::LShift),
        SAppKeycode::KeyLeftControl => Some(ScanCode::LControl),
        SAppKeycode::KeyLeftAlt => Some(ScanCode::LAlt),
        SAppKeycode::KeyLeftSuper => Some(ScanCode::LWin),
        SAppKeycode::KeyRightShift => Some(ScanCode::RShift),
        SAppKeycode::KeyRightControl => Some(ScanCode::RControl),
        SAppKeycode::KeyRightAlt => Some(ScanCode::RAlt),
        SAppKeycode::KeyRightSuper => Some(ScanCode::RWin),
        SAppKeycode::KeyMenu => Some(ScanCode::Menu),
        _ => None,
    }
}
