use crate::common::Color;
use crate::framework::context::Context;
use crate::framework::error::{GameResult, GameError};
use crate::framework::backend::BackendTexture;

pub enum FilterMode {
    Nearest,
    Linear,
}

pub struct Canvas {}

impl Canvas {

}

pub fn clear(ctx: &mut Context, color: Color) {
    if let Some(renderer) = ctx.renderer.as_mut() {
        renderer.clear(color)
    }
}

pub fn present(ctx: &mut Context) -> GameResult {
    if let Some(renderer) = ctx.renderer.as_mut() {
        renderer.present()?;
    }

    Ok(())
}

pub fn renderer_initialized(ctx: &mut Context) -> bool {
    ctx.renderer.is_some()
}

pub fn create_texture(ctx: &mut Context, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
    if let Some(renderer) = ctx.renderer.as_mut() {
        return renderer.create_texture(width, height, data);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn screen_size(ctx: &mut Context) -> (f32, f32) {
    ctx.screen_size
}
