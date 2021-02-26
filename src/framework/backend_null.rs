use crate::framework::backend::{Backend, BackendEventLoop, BackendRenderer, BackendTexture, SpriteBatchCommand};
use crate::framework::error::GameResult;
use crate::framework::context::Context;
use crate::Game;
use crate::common::{Rect, Color};
use imgui::{DrawData};
use crate::framework::graphics::BlendMode;
use std::cell::RefCell;
use std::mem;

pub struct NullBackend;

impl NullBackend {
    pub fn new() -> GameResult<Box<dyn Backend>> {
        Ok(Box::new(NullBackend))
    }
}

impl Backend for NullBackend {
    fn create_event_loop(&self) -> GameResult<Box<dyn BackendEventLoop>> {
        Ok(Box::new(NullEventLoop))
    }
}

pub struct NullEventLoop;

impl BackendEventLoop for NullEventLoop {
    fn run(&mut self, game: &mut Game, ctx: &mut Context) {
        let state_ref = unsafe { &mut *game.state.get() };

        ctx.screen_size = (640.0, 480.0);
        state_ref.handle_resize(ctx).unwrap();

        loop {
            game.update(ctx).unwrap();
            if state_ref.shutdown {
                log::info!("Shutting down...");
                break;
            }
            if state_ref.next_scene.is_some() {
                mem::swap(&mut game.scene, &mut state_ref.next_scene);
                state_ref.next_scene = None;
                game.scene.as_mut().unwrap().init(state_ref, ctx).unwrap();
                game.loops = 0;
                state_ref.frame_time = 0.0;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            game.draw(ctx).unwrap();
        }
    }

    fn new_renderer(&self) -> GameResult<Box<dyn BackendRenderer>> {
        let mut imgui = imgui::Context::create();
        imgui.io_mut().display_size = [640.0, 480.0];
        imgui.fonts().build_alpha8_texture();

        Ok(Box::new(NullRenderer(RefCell::new(imgui))))
    }
}

pub struct NullTexture(u16, u16);

impl BackendTexture for NullTexture {
    fn dimensions(&self) -> (u16, u16) {
        (self.0, self.1)
    }

    fn add(&mut self, command: SpriteBatchCommand) {

    }

    fn clear(&mut self) {

    }

    fn draw(&mut self) -> GameResult<()> {
        Ok(())
    }
}

pub struct NullRenderer(RefCell<imgui::Context>);

impl BackendRenderer for NullRenderer {
    fn clear(&mut self, color: Color) {

    }

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
}
