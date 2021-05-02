use std::collections::HashSet;
use std::path::PathBuf;

use crate::bmfont::BMFont;
use crate::common::{FILE_TYPES, Rect};
use crate::engine_constants::EngineConstants;
use crate::framework::context::Context;
use crate::framework::error::GameError::ResourceLoadError;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::str;
use crate::texture_set::TextureSet;

pub struct BMFontRenderer {
    font: BMFont,
    pages: Vec<String>,
}

impl BMFontRenderer {
    pub fn load(root: &str, desc_path: &str, ctx: &mut Context) -> GameResult<BMFontRenderer> {
        let root = PathBuf::from(root);
        let full_path = &root.join(PathBuf::from(desc_path));
        let desc_stem =
            full_path.file_stem().ok_or_else(|| ResourceLoadError(str!("Cannot extract the file stem.")))?;
        let stem = full_path.parent().unwrap_or(full_path).join(desc_stem);

        let font = BMFont::load_from(filesystem::open(ctx, &full_path)?)?;
        let mut pages = Vec::new();

        let (zeros, _, _) = FILE_TYPES
            .iter()
            .map(|ext| (1, ext, format!("{}_0{}", stem.to_string_lossy(), ext)))
            .find(|(_, _, path)| filesystem::exists(ctx, &path))
            .or_else(|| {
                FILE_TYPES
                    .iter()
                    .map(|ext| (2, ext, format!("{}_00{}", stem.to_string_lossy(), ext)))
                    .find(|(_, _, path)| filesystem::exists(ctx, &path))
            })
            .ok_or_else(|| ResourceLoadError(format!("Cannot find glyph atlas 0 for font: {:?}", desc_path)))?;

        for i in 0..font.pages {
            let page_path = format!("{}_{:02$}", stem.to_string_lossy(), i, zeros);

            pages.push(page_path);
        }

        Ok(Self { font, pages })
    }

    pub fn line_height(&self, constants: &EngineConstants) -> f32 {
        self.font.line_height as f32 * constants.font_scale
    }

    pub fn text_width<I: Iterator<Item = char>>(&self, iter: I, constants: &EngineConstants) -> f32 {
        let mut offset_x = 0.0;

        for chr in iter {
            if let Some(glyph) = self.font.chars.get(&chr) {
                offset_x += ((glyph.width as f32 + glyph.xoffset as f32) * constants.font_scale).floor()
                    + if chr != ' ' { 1.0 } else { constants.font_space_offset };
            }
        }

        offset_x
    }

    pub fn draw_text<I: Iterator<Item = char>>(
        &self,
        iter: I,
        x: f32,
        y: f32,
        constants: &EngineConstants,
        texture_set: &mut TextureSet,
        ctx: &mut Context,
    ) -> GameResult {
        self.draw_colored_text(iter, x, y, (255, 255, 255, 255), constants, texture_set, ctx)
    }

    pub fn draw_text_with_shadow<I: Iterator<Item = char> + Clone>(
        &self,
        iter: I,
        x: f32,
        y: f32,
        constants: &EngineConstants,
        texture_set: &mut TextureSet,
        ctx: &mut Context,
    ) -> GameResult {
        self.draw_colored_text(iter.clone(), x + 1.0, y + 1.0, (0, 0, 0, 150), constants, texture_set, ctx)?;
        self.draw_colored_text(iter, x, y, (255, 255, 255, 255), constants, texture_set, ctx)
    }

    pub fn draw_colored_text_scaled<I: Iterator<Item = char>>(
        &self,
        iter: I,
        x: f32,
        y: f32,
        scale: f32,
        color: (u8, u8, u8, u8),
        constants: &EngineConstants,
        texture_set: &mut TextureSet,
        ctx: &mut Context,
    ) -> GameResult {
        if self.pages.len() == 1 {
            let batch = texture_set.get_or_load_batch(ctx, constants, self.pages.get(0).unwrap())?;
            let mut offset_x = x;

            for chr in iter {
                if let Some(glyph) = self.font.chars.get(&chr) {
                    batch.add_rect_scaled_tinted(
                        offset_x,
                        y + (glyph.yoffset as f32 * constants.font_scale).floor(),
                        color,
                        constants.font_scale,
                        constants.font_scale,
                        &Rect::new_size(glyph.x as u16, glyph.y as u16, glyph.width as u16, glyph.height as u16),
                    );

                    offset_x += ((glyph.width as f32 + glyph.xoffset as f32) * constants.font_scale).floor()
                        + if chr != ' ' { 1.0 } else { constants.font_space_offset };
                }
            }

            batch.draw(ctx)?;
        } else {
            let mut pages = HashSet::new();
            let mut chars = Vec::new();

            for chr in iter {
                if let Some(glyph) = self.font.chars.get(&chr) {
                    pages.insert(glyph.page);
                    chars.push((chr, glyph));
                }
            }

            for page in pages {
                let page_tex = if let Some(p) = self.pages.get(page as usize) {
                    p
                } else {
                    continue;
                };

                let batch = texture_set.get_or_load_batch(ctx, constants, page_tex)?;
                let mut offset_x = x;

                for (chr, glyph) in chars.iter() {
                    if glyph.page == page {
                        batch.add_rect_scaled_tinted(
                            offset_x,
                            y + (glyph.yoffset as f32 * constants.font_scale).floor(),
                            color,
                            constants.font_scale * scale,
                            constants.font_scale * scale,
                            &Rect::new_size(glyph.x as u16, glyph.y as u16, glyph.width as u16, glyph.height as u16),
                        );
                    }

                    offset_x += scale
                        * (((glyph.width as f32 + glyph.xoffset as f32) * constants.font_scale).floor()
                            + if *chr != ' ' { 1.0 } else { constants.font_space_offset });
                }

                batch.draw(ctx)?;
            }
        }

        Ok(())
    }

    pub fn draw_colored_text<I: Iterator<Item = char>>(
        &self,
        iter: I,
        x: f32,
        y: f32,
        color: (u8, u8, u8, u8),
        constants: &EngineConstants,
        texture_set: &mut TextureSet,
        ctx: &mut Context,
    ) -> GameResult {
        self.draw_colored_text_scaled(iter, x, y, 1.0, color, constants, texture_set, ctx)
    }
}
