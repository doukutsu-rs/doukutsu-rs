use std::cell::RefCell;
use std::rc::Rc;

use crate::common::{Colorf, Rect};
use crate::framework::backend::{BackendShader, BackendTexture, VertexData};
use crate::framework::context::Context;
use crate::framework::error::{GameError, GameResult};

pub enum FilterMode {
    Nearest,
    Linear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlendMode {
    None,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SwapMode {
    Immediate = 0,
    VSync = 1,
    Adaptive = -1,
}

pub fn clear(ctx: &mut Context, color: Colorf) {
    if let Some(renderer) = &mut ctx.renderer {
        renderer.clear(color)
    }
}

pub fn present(ctx: &mut Context) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        renderer.present()?;
    }

    Ok(())
}

pub fn set_swap_mode(ctx: &mut Context, mode: SwapMode) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        if ctx.swap_mode != mode {
            ctx.swap_mode = mode;
            renderer.set_swap_mode(mode);
        }
    }

    Ok(())
}

#[allow(unused)]
pub fn renderer_initialized(ctx: &mut Context) -> bool {
    ctx.renderer.is_some()
}

pub fn create_texture_mutable(ctx: &mut Context, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.create_texture_mutable(width, height);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn create_texture(ctx: &mut Context, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.create_texture(width, height, data);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn screen_size(ctx: &mut Context) -> (f32, f32) {
    ctx.screen_size
}

#[allow(unused)]
pub fn screen_insets(ctx: &mut Context) -> (f32, f32, f32, f32) {
    ctx.screen_insets
}

pub fn screen_insets_scaled(ctx: &mut Context, scale: f32) -> (f32, f32, f32, f32) {
    (ctx.screen_insets.0 / scale, ctx.screen_insets.1 / scale, ctx.screen_insets.2 / scale, ctx.screen_insets.3 / scale)
}

pub fn set_render_target(ctx: &mut Context, texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.set_render_target(texture);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn set_blend_mode(ctx: &mut Context, blend: BlendMode) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.set_blend_mode(blend);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn draw_rect(ctx: &mut Context, rect: Rect, color: Colorf) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.draw_rect(rect, color);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

#[allow(unused)]
pub fn draw_outline_rect(ctx: &mut Context, rect: Rect, line_width: usize, color: Colorf) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.draw_outline_rect(rect, line_width, color);
    }

    Ok(())
}

pub fn set_clip_rect(ctx: &mut Context, rect: Option<Rect>) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.set_clip_rect(rect);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn imgui_context(ctx: &Context) -> GameResult<Rc<RefCell<imgui::Context>>> {
    if let Some(renderer) = ctx.renderer.as_ref() {
        return renderer.imgui();
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn imgui_texture_id(ctx: &Context, texture: &Box<dyn BackendTexture>) -> GameResult<imgui::TextureId> {
    if let Some(renderer) = ctx.renderer.as_ref() {
        return renderer.imgui_texture_id(texture);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn prepare_imgui(ctx: &mut Context, ui: &imgui::Ui) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.prepare_imgui(ui);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn render_imgui(ctx: &mut Context, draw_data: &imgui::DrawData) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.render_imgui(draw_data);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn prepare_draw(ctx: &mut Context) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.prepare_draw(ctx.screen_size.0, ctx.screen_size.1);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn draw_triangle_list(
    ctx: &mut Context,
    vertices: &[VertexData],
    texture: Option<&Box<dyn BackendTexture>>,
    shader: BackendShader,
) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.draw_triangle_list(vertices, texture, shader);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}
