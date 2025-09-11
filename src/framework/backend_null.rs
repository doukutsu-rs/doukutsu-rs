use std::any::Any;
use std::cell::{RefCell, RefMut};
use std::mem;
use std::pin::Pin;

use imgui::{DrawData, TextureId, Ui};

use crate::common::{Color, Rect};
use crate::framework::backend::{
    Backend, BackendCallbacks, BackendEventLoop, BackendRenderer, BackendShader, BackendTexture, SpriteBatchCommand,
    VertexData,
};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics::BlendMode;
use crate::game::Game;

pub struct NullBackend;

impl NullBackend {
    pub fn new() -> GameResult<Box<dyn Backend>> {
        Ok(Box::new(NullBackend))
    }
}

impl Backend for NullBackend {
    fn create_event_loop(&self, _ctx: &Context) -> GameResult<Box<dyn BackendEventLoop>> {
        Ok(Box::new(NullEventLoop))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct NullEventLoop;

impl BackendEventLoop for NullEventLoop {
    fn run(&mut self, mut game: Pin<Box<Game>>, mut ctx: Pin<Box<Context>>) {
        ctx.screen_size = (640.0, 480.0);
        game.on_resize(&mut ctx);

        loop {
            game.update(&mut ctx).unwrap();

            if ctx.shutdown_requested {
                log::info!("Shutting down...");
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
            game.draw(&mut ctx).unwrap();
        }
    }

    fn new_renderer(&self, ctx: &mut Context) -> GameResult<Box<dyn BackendRenderer>> {
        ctx.imgui.borrow_mut().fonts().build_alpha8_texture();

        Ok(Box::new(NullRenderer))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct NullTexture(u16, u16);

impl BackendTexture for NullTexture {
    fn dimensions(&self) -> (u16, u16) {
        (self.0, self.1)
    }

    fn add(&mut self, _command: SpriteBatchCommand) {}

    fn clear(&mut self) {}

    fn draw(&mut self) -> GameResult<()> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct NullRenderer;

impl BackendRenderer for NullRenderer {
    fn renderer_name(&self) -> String {
        "Null".to_owned()
    }

    fn clear(&mut self, _color: Color) {}

    fn present(&mut self) -> GameResult {
        Ok(())
    }

    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
        Ok(Box::new(NullTexture(width, height)))
    }

    fn create_texture(&mut self, width: u16, height: u16, _data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
        Ok(Box::new(NullTexture(width, height)))
    }

    fn set_blend_mode(&mut self, _blend: BlendMode) -> GameResult {
        Ok(())
    }

    fn set_render_target(&mut self, _texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
        Ok(())
    }

    fn draw_rect(&mut self, _rect: Rect<isize>, _color: Color) -> GameResult {
        Ok(())
    }

    fn draw_outline_rect(&mut self, _rect: Rect<isize>, _line_width: usize, _color: Color) -> GameResult {
        Ok(())
    }

    fn set_clip_rect(&mut self, _rect: Option<Rect>) -> GameResult {
        Ok(())
    }

    fn draw_triangles(
        &mut self,
        _vertices: &[VertexData],
        _texture: Option<&Box<dyn BackendTexture>>,
        _shader: BackendShader,
    ) -> GameResult<()> {
        Ok(())
    }

    fn draw_triangles_indexed(
        &mut self,
        vertices: &[VertexData],
        indices: super::graphics::IndexData,
        texture: Option<&Box<dyn BackendTexture>>,
        shader: BackendShader,
    ) -> GameResult {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
