use crate::common::Color;
use crate::framework::context::Context;
use crate::framework::error::{GameResult, GameError};
use crate::framework::backend::BackendTexture;

pub enum FilterMode {
    Nearest,
    Linear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlendMode {
    /// When combining two fragments, add their values together, saturating
    /// at 1.0
    Add,
    /// When combining two fragments, add the value of the source times its
    /// alpha channel with the value of the destination multiplied by the inverse
    /// of the source alpha channel. Has the usual transparency effect: mixes the
    /// two colors using a fraction of each one specified by the alpha of the source.
    Alpha,
    /// When combining two fragments, multiply their values together.
    Multiply,
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

pub fn create_texture_mutable(ctx: &mut Context, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
    if let Some(renderer) = ctx.renderer.as_mut() {
        return renderer.create_texture_mutable(width, height);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
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

pub fn set_render_target(ctx: &mut Context, texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
    if let Some(renderer) = ctx.renderer.as_mut() {
        return renderer.set_render_target(texture);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn set_blend_mode(ctx: &mut Context, blend: BlendMode) -> GameResult {
    if let Some(renderer) = ctx.renderer.as_mut() {
        return renderer.set_blend_mode(blend);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn imgui_context(ctx: &Context) -> GameResult<&mut imgui::Context> {
    if let Some(renderer) = ctx.renderer.as_ref() {
        return renderer.imgui();
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn render_imgui(ctx: &mut Context, draw_data: &imgui::DrawData) -> GameResult {
    if let Some(renderer) = ctx.renderer.as_mut() {
        return renderer.render_imgui(draw_data);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn imgui_prepare_frame(ctx: &Context, ui: &imgui::Ui) -> GameResult {
    if let Some(renderer) = ctx.renderer.as_ref() {
        return renderer.prepare_frame(ui);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}
