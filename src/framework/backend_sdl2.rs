use core::mem;
use std::cell::RefCell;
use std::rc::Rc;

use sdl2::{EventPump, keyboard, pixels, Sdl};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Scancode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{BlendMode, Texture, TextureCreator, WindowCanvas};
use sdl2::video::WindowContext;

use crate::common::Color;
use crate::framework::backend::{Backend, BackendEventLoop, BackendRenderer, BackendTexture, SpriteBatchCommand};
use crate::framework::context::Context;
use crate::framework::error::{GameError, GameResult};
use crate::framework::keyboard::ScanCode;
use crate::Game;

pub struct SDL2Backend {
    context: Sdl,
}

impl SDL2Backend {
    pub fn new() -> GameResult<Box<dyn Backend>> {
        let context = sdl2::init().map_err(|e| GameError::WindowError(e))?;

        let backend = SDL2Backend {
            context,
        };

        Ok(Box::new(backend))
    }
}

impl Backend for SDL2Backend {
    fn create_event_loop(&self) -> GameResult<Box<dyn BackendEventLoop>> {
        SDL2EventLoop::new(&self.context)
    }
}

struct SDL2EventLoop {
    event_pump: EventPump,
    refs: Rc<RefCell<SDL2Context>>,
}

struct SDL2Context {
    canvas: WindowCanvas,
    texture_creator: TextureCreator<WindowContext>,
}

impl SDL2EventLoop {
    pub fn new(sdl: &Sdl) -> GameResult<Box<dyn BackendEventLoop>> {
        let event_pump = sdl.event_pump().map_err(|e| GameError::WindowError(e))?;
        let video = sdl.video().map_err(|e| GameError::WindowError(e))?;
        let window = video.window("Cave Story (doukutsu-rs)", 640, 480)
            .position_centered()
            .resizable()
            .build()
            .map_err(|e| GameError::WindowError(e.to_string()))?;

        let canvas = window.into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .map_err(|e| GameError::RenderError(e.to_string()))?;

        let texture_creator = canvas.texture_creator();

        let event_loop = SDL2EventLoop {
            event_pump,
            refs: Rc::new(RefCell::new(SDL2Context {
                canvas,
                texture_creator,
            })),
        };

        Ok(Box::new(event_loop))
    }
}

impl BackendEventLoop for SDL2EventLoop {
    fn run(&mut self, game: &mut Game, ctx: &mut Context) {
        let state = unsafe { &mut *game.state.get() };

        loop {
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => {
                        state.shutdown();
                    }
                    Event::Window { win_event, .. } => {
                        match win_event {
                            WindowEvent::Shown => {}
                            WindowEvent::Hidden => {}
                            WindowEvent::SizeChanged(width, height) => {
                                ctx.screen_size = (width.max(1) as f32, height.max(1) as f32);
                                state.handle_resize(ctx);
                            }
                            _ => {}
                        }
                    }
                    Event::KeyDown { scancode, repeat, .. } => {
                        if let Some(scancode) = scancode {
                            if let Some(drs_scan) = conv_scancode(scancode) {
                                game.key_down_event(drs_scan, repeat);
                                ctx.keyboard_context.set_key(drs_scan, true);
                            }
                        }
                    }
                    Event::KeyUp { scancode, .. } => {
                        if let Some(scancode) = scancode {
                            if let Some(drs_scan) = conv_scancode(scancode) {
                                ctx.keyboard_context.set_key(drs_scan, false);
                            }
                        }
                    }
                    _ => {}
                }
            }

            game.update(ctx).unwrap();

            if state.shutdown {
                log::info!("Shutting down...");
                break;
            }

            if state.next_scene.is_some() {
                mem::swap(&mut game.scene, &mut state.next_scene);
                state.next_scene = None;

                game.scene.as_mut().unwrap().init(state, ctx).unwrap();
                game.loops = 0;
                state.frame_time = 0.0;
            }

            game.draw(ctx).unwrap();
        }
    }

    fn new_renderer(&self) -> GameResult<Box<dyn BackendRenderer>> {
        SDL2Renderer::new(self.refs.clone())
    }
}

struct SDL2Renderer {
    refs: Rc<RefCell<SDL2Context>>,
}

impl SDL2Renderer {
    pub fn new(refs: Rc<RefCell<SDL2Context>>) -> GameResult<Box<dyn BackendRenderer>> {
        Ok(Box::new(SDL2Renderer {
            refs,
        }))
    }
}

fn to_sdl(color: Color) -> pixels::Color {
    let (r, g, b, a) = color.to_rgba();
    pixels::Color::RGBA(r, g, b, a)
}

impl BackendRenderer for SDL2Renderer {
    fn clear(&mut self, color: Color) {
        let mut refs = self.refs.borrow_mut();

        refs.canvas.set_draw_color(to_sdl(color));
        refs.canvas.clear();
    }

    fn present(&mut self) -> GameResult {
        let mut refs = self.refs.borrow_mut();

        refs.canvas.present();

        Ok(())
    }

    fn create_texture(&mut self, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
        let mut refs = self.refs.borrow_mut();

        let mut texture = refs.texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA32, width as u32, height as u32)
            .map_err(|e| GameError::RenderError(e.to_string()))?;

        texture.set_blend_mode(BlendMode::Blend);
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..(height as usize) {
                for x in 0..(width as usize) {
                    let offset = y * pitch + x * 4;
                    let data_offset = (y * width as usize + x) * 4;

                    buffer[offset] = data[data_offset];
                    buffer[offset + 1] = data[data_offset + 1];
                    buffer[offset + 2] = data[data_offset + 2];
                    buffer[offset + 3] = data[data_offset + 3];
                }
            }
        }).map_err(|e| GameError::RenderError(e.to_string()))?;

        return Ok(Box::new(SDL2Texture {
            refs: self.refs.clone(),
            texture: Some(texture),
            width,
            height,
            commands: vec![],
        }));
    }
}

struct SDL2Texture {
    refs: Rc<RefCell<SDL2Context>>,
    texture: Option<Texture>,
    width: u16,
    height: u16,
    commands: Vec<SpriteBatchCommand>,
}

impl BackendTexture for SDL2Texture {
    fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn add(&mut self, command: SpriteBatchCommand) {
        self.commands.push(command);
    }

    fn clear(&mut self) {
        self.commands.clear();
    }

    fn draw(&mut self) -> GameResult {
        match self.texture.as_mut() {
            None => Ok(()),
            Some(texture) => {
                let mut refs = self.refs.borrow_mut();
                for command in self.commands.iter() {
                    match command {
                        SpriteBatchCommand::DrawRect(src, dest) => {
                            texture.set_color_mod(255, 255, 255);
                            texture.set_alpha_mod(255);

                            refs.canvas.copy(texture,
                                             Some(sdl2::rect::Rect::new(src.left as i32, src.top as i32, src.width() as u32, src.height() as u32)),
                                             Some(sdl2::rect::Rect::new(dest.left as i32, dest.top as i32, dest.width() as u32, dest.height() as u32)))
                                .map_err(|e| GameError::RenderError(e.to_string()))?;
                        }
                        SpriteBatchCommand::DrawRectTinted(src, dest, color) => {
                            let (r, g, b, a) = color.to_rgba();
                            texture.set_color_mod(r, g, b);
                            texture.set_alpha_mod(a);

                            refs.canvas.copy(texture,
                                             Some(sdl2::rect::Rect::new(src.left as i32, src.top as i32, src.width() as u32, src.height() as u32)),
                                             Some(sdl2::rect::Rect::new(dest.left as i32, dest.top as i32, dest.width() as u32, dest.height() as u32)))
                                .map_err(|e| GameError::RenderError(e.to_string()))?;
                        }
                    }
                }

                Ok(())
            }
        }
    }
}

impl Drop for SDL2Texture {
    fn drop(&mut self) {
        let mut texture_opt = None;
        mem::swap(&mut self.texture, &mut texture_opt);

        if let Some(texture) = texture_opt {
            unsafe { texture.destroy(); }
        }
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
