use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use imgui::DrawData;

use super::context::Context;
use super::error::GameResult;
use super::graphics::BlendMode;
use super::graphics::SwapMode;

use crate::common::{Colorf, Rect};
use crate::game::Game;

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
    fn create_event_loop(&self, ctx: &Context) -> GameResult<Box<dyn BackendEventLoop>>;

    fn as_any(&self) -> &dyn Any;
}

pub trait BackendEventLoop {
    fn run(&mut self, game: &mut Game, ctx: &mut Context);

    fn new_renderer(&self) -> GameResult<Box<dyn BackendRenderer>>;

    fn as_any(&self) -> &dyn Any;
}

pub trait BackendRenderer {
    /// Human-readable name for the renderer. May return different values based on current platform or settings.
    fn renderer_name(&self) -> String;

    /// Clear the current render target with the specified color.
    fn clear(&mut self, color: Colorf);

    /// Present the current frame to the screen.
    fn present(&mut self) -> GameResult;

    /// Sets the preferred frame swap mode.
    fn set_swap_mode(&mut self, _mode: SwapMode) -> GameResult {
        Ok(())
    }

    // Prepare the renderer for drawing.
    fn prepare_draw(&mut self, _width: f32, _height: f32) -> GameResult {
        Ok(())
    }

    /// Create a new mutable texture with the specified dimensions.
    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>>;

    /// Create a new texture with the specified dimensions and data.
    fn create_texture(&mut self, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>>;

    /// Set the current blend mode.
    fn set_blend_mode(&mut self, blend: BlendMode) -> GameResult;

    /// Set the current render target.
    fn set_render_target(&mut self, texture: Option<&Box<dyn BackendTexture>>) -> GameResult;

    /// Draw a filled rectangle with the specified color.
    fn draw_rect(&mut self, rect: Rect, color: Colorf) -> GameResult;

    /// Draw an outlined rectangle with the specified line width and color.
    fn draw_outline_rect(&mut self, rect: Rect, line_width: usize, color: Colorf) -> GameResult;

    /// Set the current clipping rectangle.
    fn set_clip_rect(&mut self, rect: Option<Rect>) -> GameResult;

    /// Get a reference to the imgui context.
    fn imgui(&self) -> GameResult<Rc<RefCell<imgui::Context>>>;

    /// Get an imgui texture id for the specified texture.
    fn imgui_texture_id(&self, texture: &Box<dyn BackendTexture>) -> GameResult<imgui::TextureId>;

    /// Prepare the imgui context for rendering.
    fn prepare_imgui(&mut self, ui: &imgui::Ui) -> GameResult;

    /// Render the imgui draw data.
    fn render_imgui(&mut self, draw_data: &DrawData) -> GameResult;

    /// Draw a list of triangles, in mode similar to GL_TRIANGLES.
    fn draw_triangle_list(
        &mut self,
        vertices: &[VertexData],
        texture: Option<&Box<dyn BackendTexture>>,
        shader: BackendShader,
    ) -> GameResult;

    fn as_any(&self) -> &dyn Any;
}

pub trait BackendTexture {
    /// Get the dimensions of the texture.
    fn dimensions(&self) -> (u16, u16);

    /// Adds a new drawing command to the texture batch.
    fn add(&mut self, command: SpriteBatchCommand);

    /// Clear the texture batch.
    fn clear(&mut self);

    /// Draw the texture batch to the screen.
    fn draw(&mut self) -> GameResult;

    fn as_any(&self) -> &dyn Any;
}

pub trait BackendGamepad {
    /// Run a gamepad rumble effect using the specified parameters.
    fn set_rumble(&mut self, low_freq: u16, high_freq: u16, duration_ms: u32) -> GameResult;

    fn instance_id(&self) -> u32;
}

#[allow(unreachable_code)]
pub fn init_backend(headless: bool, size_hint: (u16, u16)) -> GameResult<Box<dyn Backend>> {
    if headless {
        return super::backend_null::NullBackend::new();
    }

    #[cfg(all(feature = "backend-horizon"))]
    {
        return super::backend_horizon::HorizonBackend::new();
    }

    #[cfg(all(feature = "backend-winit"))]
    {
        return super::backend_winit::WinitBackend::new();
    }

    #[cfg(feature = "backend-sdl")]
    {
        return super::backend_sdl2::SDL2Backend::new(size_hint);
    }

    log::warn!("No backend compiled in, using null backend instead.");
    super::backend_null::NullBackend::new()
}

pub enum SpriteBatchCommand {
    DrawRect(Rect<f32>, Rect<f32>),
    DrawRectFlip(Rect<f32>, Rect<f32>, bool, bool),
    DrawRectTinted(Rect<f32>, Rect<f32>, Colorf),
    DrawRectFlipTinted(Rect<f32>, Rect<f32>, bool, bool, Colorf),
}
