use crate::common::{Color, Rect, Point};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::Game;

pub trait Backend {
    fn create_event_loop(&self) -> GameResult<Box<dyn BackendEventLoop>>;
}

pub trait BackendEventLoop {
    fn run(&mut self, game: &mut Game, ctx: &mut Context);

    fn new_renderer(&self) -> GameResult<Box<dyn BackendRenderer>>;
}

pub trait BackendRenderer {
    fn clear(&mut self, color: Color);

    fn present(&mut self) -> GameResult;

    fn create_texture(&mut self, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>>;
}

pub trait BackendTexture {
    fn dimensions(&self) -> (u16, u16);
    fn add(&mut self, command: SpriteBatchCommand);
    fn clear(&mut self);
    fn draw(&mut self) -> GameResult;
}

pub fn init_backend() -> GameResult<Box<dyn Backend>> {
    crate::framework::backend_sdl2::SDL2Backend::new()
}

pub enum SpriteBatchCommand {
    DrawRect(Rect<f32>, Rect<f32>),
    DrawRectTinted(Rect<f32>, Rect<f32>, Color),
}
