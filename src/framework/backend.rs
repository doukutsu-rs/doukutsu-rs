use std::any::Any;
use imgui::DrawData;

use crate::common::{Color, Rect};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics::BlendMode;
use crate::Game;
use crate::graphics::VSyncMode;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct VertexData {
    pub position: (f32, f32),
    pub color: (u8, u8, u8, u8),
    pub uv: (f32, f32),
}

#[derive(Copy, Clone, PartialEq)]
pub enum BackendShader {
    /// (scale, t, (frame_x, frame_y))
    WaterFill(f32, f32, (f32, f32)),
    Fill,
    Texture,
}

pub trait Backend {
    fn create_event_loop(&self) -> GameResult<Box<dyn BackendEventLoop>>;
}

pub trait BackendEventLoop {
    fn run(&mut self, game: &mut Game, ctx: &mut Context);

    fn new_renderer(&self, ctx: *mut Context) -> GameResult<Box<dyn BackendRenderer>>;
}

pub trait BackendRenderer {
    fn renderer_name(&self) -> String;

    fn clear(&mut self, color: Color);

    fn present(&mut self) -> GameResult;

    fn set_vsync_mode(&mut self, _mode: VSyncMode) -> GameResult { Ok(()) }

    fn prepare_draw(&mut self, _width: f32, _height: f32) -> GameResult {
        Ok(())
    }

    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>>;

    fn create_texture(&mut self, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>>;

    fn set_blend_mode(&mut self, blend: BlendMode) -> GameResult;

    fn set_render_target(&mut self, texture: Option<&Box<dyn BackendTexture>>) -> GameResult;

    fn draw_rect(&mut self, rect: Rect, color: Color) -> GameResult;

    fn draw_outline_rect(&mut self, rect: Rect, line_width: usize, color: Color) -> GameResult;

    fn set_clip_rect(&mut self, rect: Option<Rect>) -> GameResult;

    fn imgui(&self) -> GameResult<&mut imgui::Context>;

    fn imgui_texture_id(&self, texture: &Box<dyn BackendTexture>) -> GameResult<imgui::TextureId>;

    fn prepare_imgui(&mut self, ui: &imgui::Ui) -> GameResult;

    fn render_imgui(&mut self, draw_data: &DrawData) -> GameResult;

    fn supports_vertex_draw(&self) -> bool {
        false
    }

    fn draw_triangle_list(
        &mut self,
        vertices: &[VertexData],
        texture: Option<&Box<dyn BackendTexture>>,
        shader: BackendShader,
    ) -> GameResult;
}

pub trait BackendTexture {
    fn dimensions(&self) -> (u16, u16);

    fn add(&mut self, command: SpriteBatchCommand);

    fn clear(&mut self);

    fn draw(&mut self) -> GameResult;

    fn as_any(&self) -> &dyn Any;
}

#[allow(unreachable_code)]
pub fn init_backend(headless: bool, size_hint: (u16, u16)) -> GameResult<Box<dyn Backend>> {
    if headless {
        return crate::framework::backend_null::NullBackend::new();
    }

    #[cfg(all(feature = "backend-glutin"))]
    {
        return crate::framework::backend_glutin::GlutinBackend::new();
    }

    #[cfg(feature = "backend-sdl")]
    {
        return crate::framework::backend_sdl2::SDL2Backend::new(size_hint);
    }

    log::warn!("No backend compiled in, using null backend instead.");
    crate::framework::backend_null::NullBackend::new()
}

pub enum SpriteBatchCommand {
    DrawRect(Rect<f32>, Rect<f32>),
    DrawRectFlip(Rect<f32>, Rect<f32>, bool, bool),
    DrawRectTinted(Rect<f32>, Rect<f32>, Color),
}
