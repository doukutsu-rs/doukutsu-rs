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
use crate::framework::render::null_impl::NullRenderer;
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
