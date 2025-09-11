use std::cell::RefMut;

use super::backend::{BackendShader, BackendTexture, VertexData};
use super::context::Context;
use super::error::{GameError, GameResult};
use crate::common::{Color, Rect};

#[derive(Copy, Clone)]
pub enum IndexData<'a> {
    UByte(&'a [u8]),
    UShort(&'a [u16]),
    UInt(&'a [u32]),
}

impl<'a> IndexData<'a> {
    #[inline]
    pub const fn is_empty(&self) -> bool {
        match self {
            IndexData::UByte(items) => items.is_empty(),
            IndexData::UShort(items) => items.is_empty(),
            IndexData::UInt(items) => items.is_empty(),
        }
    }

    #[inline]
    pub const fn element_size(&self) -> usize {
        match self {
            IndexData::UByte(_) => 1,
            IndexData::UShort(_) => 2,
            IndexData::UInt(_) => 4,
        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        match self {
            IndexData::UByte(buf) => buf.len(),
            IndexData::UShort(buf) => buf.len(),
            IndexData::UInt(buf) => buf.len(),
        }
    }

    #[inline]
    pub const fn bytes_len(&self) -> usize {
        self.len() * self.element_size()
    }

    #[inline]
    pub const fn as_bytes_slice(&'a self) -> &'a [u8] {
        let ptr = match self {
            IndexData::UByte(buf) => buf.as_ptr(),
            IndexData::UShort(buf) => buf.as_ptr() as *const u8,
            IndexData::UInt(buf) => buf.as_ptr() as *const u8,
        };

        unsafe { std::slice::from_raw_parts(ptr, self.bytes_len()) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

pub fn clear(ctx: &mut Context, color: Color) {
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

pub fn draw_rect(ctx: &mut Context, rect: Rect, color: Color) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.draw_rect(rect, color);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

#[allow(unused)]
pub fn draw_outline_rect(ctx: &mut Context, rect: Rect, line_width: usize, color: Color) -> GameResult {
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

pub fn prepare_draw(ctx: &mut Context) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.prepare_draw(ctx.screen_size.0, ctx.screen_size.1);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}

pub fn draw_triangles(
    ctx: &mut Context,
    vertices: &[VertexData],
    texture: Option<&Box<dyn BackendTexture>>,
    shader: BackendShader,
) -> GameResult {
    if let Some(renderer) = &mut ctx.renderer {
        return renderer.draw_triangles(vertices, texture, shader);
    }

    Err(GameError::RenderError("Rendering backend hasn't been initialized yet.".to_string()))
}
