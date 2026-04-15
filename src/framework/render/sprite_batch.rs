use std::mem;

use crate::common::{Color, Rect};
use crate::framework::backend::{
    BackendTexture, BackendVertexBuffer, BufferUsage, PrimitiveType, VertexData,
};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::render::effect::BackendEffect;
use crate::framework::render::vertex::{HasVertexDeclaration, VertexDeclaration};

/// Commands that can be batched into a SpriteBatch for deferred rendering.
pub enum SpriteBatchCommand {
    DrawRect(Rect<f32>, Rect<f32>),
    DrawRectFlip(Rect<f32>, Rect<f32>, bool, bool),
    DrawRectTinted(Rect<f32>, Rect<f32>, Color),
    DrawRectFlipTinted(Rect<f32>, Rect<f32>, bool, bool, Color),
}

const INITIAL_CAPACITY: usize = 256;

/// Framework-level SpriteBatch that accumulates sprite draw commands and renders
/// them in a single batch using a VertexBuffer + Effect.
pub struct SpriteBatch {
    texture: Box<dyn BackendTexture>,
    vertices: Vec<VertexData>,
    vb: Box<dyn BackendVertexBuffer>,
    vb_capacity: usize,
    decl: VertexDeclaration,
}

impl SpriteBatch {
    pub fn new(ctx: &mut Context, texture: Box<dyn BackendTexture>) -> GameResult<Self> {
        let decl = VertexData::vertex_declaration();
        let renderer = ctx.renderer.as_mut().ok_or_else(|| {
            crate::framework::error::GameError::RenderError(
                "Rendering backend hasn't been initialized yet.".to_string(),
            )
        })?;

        let vb = renderer.create_vertex_buffer(decl.clone(), INITIAL_CAPACITY, BufferUsage::Stream)?;

        Ok(SpriteBatch {
            texture,
            vertices: Vec::with_capacity(INITIAL_CAPACITY),
            vb,
            vb_capacity: INITIAL_CAPACITY,
            decl,
        })
    }

    pub fn texture(&self) -> &dyn BackendTexture {
        &*self.texture
    }

    pub fn texture_ref(&self) -> &Box<dyn BackendTexture> {
        &self.texture
    }

    pub fn dimensions(&self) -> (u16, u16) {
        self.texture.dimensions()
    }

    pub fn add(&mut self, command: SpriteBatchCommand) {
        let (width, height) = self.texture.dimensions();
        let tex_scale_x = 1.0 / width as f32;
        let tex_scale_y = 1.0 / height as f32;

        match command {
            SpriteBatchCommand::DrawRect(src, dest) => {
                self.push_quad(src, dest, tex_scale_x, tex_scale_y, (255, 255, 255, 255));
            }
            SpriteBatchCommand::DrawRectFlip(mut src, dest, flip_x, flip_y) => {
                if flip_x {
                    std::mem::swap(&mut src.left, &mut src.right);
                }
                if flip_y {
                    std::mem::swap(&mut src.top, &mut src.bottom);
                }
                self.push_quad(src, dest, tex_scale_x, tex_scale_y, (255, 255, 255, 255));
            }
            SpriteBatchCommand::DrawRectTinted(src, dest, color) => {
                self.push_quad(src, dest, tex_scale_x, tex_scale_y, color.to_rgba());
            }
            SpriteBatchCommand::DrawRectFlipTinted(mut src, dest, flip_x, flip_y, color) => {
                if flip_x {
                    std::mem::swap(&mut src.left, &mut src.right);
                }
                if flip_y {
                    std::mem::swap(&mut src.top, &mut src.bottom);
                }
                self.push_quad(src, dest, tex_scale_x, tex_scale_y, color.to_rgba());
            }
        }
    }

    fn push_quad(
        &mut self,
        src: Rect<f32>,
        dest: Rect<f32>,
        tex_scale_x: f32,
        tex_scale_y: f32,
        color: (u8, u8, u8, u8),
    ) {
        let vertices = [
            VertexData {
                position: (dest.left, dest.bottom),
                uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                color,
            },
            VertexData {
                position: (dest.left, dest.top),
                uv: (src.left * tex_scale_x, src.top * tex_scale_y),
                color,
            },
            VertexData {
                position: (dest.right, dest.top),
                uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                color,
            },
            VertexData {
                position: (dest.left, dest.bottom),
                uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                color,
            },
            VertexData {
                position: (dest.right, dest.top),
                uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                color,
            },
            VertexData {
                position: (dest.right, dest.bottom),
                uv: (src.right * tex_scale_x, src.bottom * tex_scale_y),
                color,
            },
        ];
        self.vertices.extend_from_slice(&vertices);
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult {
        if self.vertices.is_empty() {
            return Ok(());
        }

        let renderer = ctx.renderer.as_mut().ok_or_else(|| {
            crate::framework::error::GameError::RenderError(
                "Rendering backend hasn't been initialized yet.".to_string(),
            )
        })?;

        // Grow VB if needed
        if self.vertices.len() > self.vb_capacity {
            self.vb_capacity = self.vertices.len().next_power_of_two();
            self.vb = renderer.create_vertex_buffer(self.decl.clone(), self.vb_capacity, BufferUsage::Stream)?;
        }

        // Upload vertex data
        let data = unsafe {
            std::slice::from_raw_parts(
                self.vertices.as_ptr() as *const u8,
                self.vertices.len() * mem::size_of::<VertexData>(),
            )
        };
        self.vb.set_data_raw(data, 0)?;

        // Draw using the old draw_triangles path for now (compatibility)
        // TODO: switch to draw_primitives with BasicEffect once ready
        renderer.draw_triangles(
            &self.vertices,
            Some(&self.texture),
            crate::framework::backend::BackendShader::Texture,
        )?;

        Ok(())
    }
}
