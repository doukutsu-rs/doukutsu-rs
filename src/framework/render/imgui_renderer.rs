use imgui::{DrawCmd, DrawData, DrawIdx, FontConfig, FontSource, TextureId};

use crate::common::Rect;
use crate::framework::backend::{BackendShader, BackendTexture, VertexData};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics::{self, BlendMode, IndexData};
use crate::framework::viewport::{DebugOverlayMode, Viewport};

const FONT_ATLAS_TEX_ID: usize = 1;
const FIRST_USER_TEX_ID: usize = 16;
/// Nominal imgui font size in imgui coord units. Atlas glyphs are rasterized
/// at `BASE_FONT_SIZE * framebuffer_scale` pixels so upscaling doesn't blur.
const BASE_FONT_SIZE: f32 = 13.0;

pub struct ImguiRenderer {
    font_texture: Option<Box<dyn BackendTexture>>,
    user_textures: Vec<Option<Box<dyn BackendTexture>>>,
    vertex_scratch: Vec<VertexData>,
    /// Framebuffer scale the current font atlas was built for. `None` = never
    /// built. Rebuild whenever this disagrees with `viewport.overlay_framebuffer_scale()`.
    last_font_scale: Option<f32>,
}

impl ImguiRenderer {
    pub fn new() -> Self {
        Self { font_texture: None, user_textures: Vec::new(), vertex_scratch: Vec::new(), last_font_scale: None }
    }

    /// Build (or rebuild) the font atlas at the current overlay framebuffer
    /// scale. Must be called before `imgui::Context::new_frame()`.
    pub fn ensure_font_texture(&mut self, ctx: &mut Context, imgui: &mut imgui::Context) -> GameResult {
        let (sx, _sy) = ctx.viewport.overlay_framebuffer_scale();
        let scale = sx.max(1.0);
        if self.font_texture.is_some() && self.last_font_scale == Some(scale) {
            return Ok(());
        }

        // Rebuild atlas at `BASE_FONT_SIZE * scale` so rasterized glyphs land
        // on physical pixels when the overlay is rendered.
        let fonts = imgui.fonts();
        fonts.clear();
        fonts.add_font(&[FontSource::DefaultFontData {
            config: Some(FontConfig { size_pixels: BASE_FONT_SIZE * scale, ..FontConfig::default() }),
        }]);

        let (width, height, data) = {
            let tex = imgui.fonts().build_rgba32_texture();
            (tex.width as u16, tex.height as u16, tex.data.to_vec())
        };

        // Drop any previous backend texture before creating a new one (avoids
        // hanging on to an atlas we no longer use).
        self.font_texture = None;
        let texture = graphics::create_texture(ctx, width, height, &data)?;
        self.font_texture = Some(texture);
        imgui.fonts().tex_id = TextureId::new(FONT_ATLAS_TEX_ID);
        self.last_font_scale = Some(scale);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn register_user_texture(&mut self, texture: Box<dyn BackendTexture>) -> TextureId {
        let idx = self.user_textures.len();
        self.user_textures.push(Some(texture));
        TextureId::new(FIRST_USER_TEX_ID + idx)
    }

    fn lookup_texture(&self, id: TextureId) -> Option<&Box<dyn BackendTexture>> {
        let raw = id.id();
        if raw == FONT_ATLAS_TEX_ID {
            return self.font_texture.as_ref();
        }
        if raw >= FIRST_USER_TEX_ID {
            return self.user_textures.get(raw - FIRST_USER_TEX_ID).and_then(|slot| slot.as_ref());
        }
        None
    }

    pub fn render(&mut self, ctx: &mut Context, draw_data: &DrawData) -> GameResult {
        if draw_data.total_vtx_count == 0 {
            return Ok(());
        }

        let clip_xf = OverlayClipTransform::from_viewport(&ctx.viewport);

        graphics::set_blend_mode(ctx, BlendMode::Alpha)?;

        for draw_list in draw_data.draw_lists() {
            let vtx_buffer = draw_list.vtx_buffer();
            let idx_buffer: &[DrawIdx] = draw_list.idx_buffer();

            self.vertex_scratch.clear();
            self.vertex_scratch.reserve(vtx_buffer.len());
            for v in vtx_buffer {
                self.vertex_scratch.push(VertexData {
                    position: (v.pos[0], v.pos[1]),
                    color: (v.col[0], v.col[1], v.col[2], v.col[3]),
                    uv: (v.uv[0], v.uv[1]),
                });
            }

            for cmd in draw_list.commands() {
                match cmd {
                    DrawCmd::Elements { count, cmd_params } => {
                        if count == 0 {
                            continue;
                        }

                        let [x0, y0, x1, y1] = cmd_params.clip_rect;
                        if let Some(phys) = clip_xf.apply(x0, y0, x1, y1) {
                            graphics::set_clip_rect(ctx, Some(phys))?;
                        } else {
                            continue;
                        }

                        let idx_start = cmd_params.idx_offset;
                        let indices = &idx_buffer[idx_start..idx_start + count];

                        let texture = self.lookup_texture(cmd_params.texture_id);

                        if let Some(renderer) = ctx.renderer.as_mut() {
                            renderer.draw_triangles_indexed(
                                &self.vertex_scratch,
                                IndexData::UShort(indices),
                                texture,
                                BackendShader::Texture,
                            )?;
                        }
                    }
                    DrawCmd::ResetRenderState => {}
                    DrawCmd::RawCallback { .. } => {}
                }
            }
        }

        graphics::set_clip_rect(ctx, None)?;
        Ok(())
    }
}

/// Maps imgui display-space clip rects to physical window pixel rects.
struct OverlayClipTransform {
    origin_x: f32,
    origin_y: f32,
    scale_x: f32,
    scale_y: f32,
}

impl OverlayClipTransform {
    fn from_viewport(viewport: &Viewport) -> Self {
        match viewport.debug_overlay_mode {
            DebugOverlayMode::Viewported => {
                let r = viewport.viewport_rect;
                let rw = r.right.saturating_sub(r.left).max(1) as f32;
                let rh = r.bottom.saturating_sub(r.top).max(1) as f32;
                Self {
                    origin_x: r.left as f32,
                    origin_y: r.top as f32,
                    scale_x: rw / viewport.logical_size.0.max(1.0),
                    scale_y: rh / viewport.logical_size.1.max(1.0),
                }
            }
            DebugOverlayMode::Unconstrained => Self { origin_x: 0.0, origin_y: 0.0, scale_x: 1.0, scale_y: 1.0 },
        }
    }

    fn apply(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> Option<Rect<isize>> {
        let px0 = self.origin_x + x0 * self.scale_x;
        let py0 = self.origin_y + y0 * self.scale_y;
        let px1 = self.origin_x + x1 * self.scale_x;
        let py1 = self.origin_y + y1 * self.scale_y;
        let left = px0.max(0.0) as isize;
        let top = py0.max(0.0) as isize;
        let right = px1.max(0.0) as isize;
        let bottom = py1.max(0.0) as isize;
        if right <= left || bottom <= top {
            return None;
        }
        Some(Rect { left, top, right, bottom })
    }
}
