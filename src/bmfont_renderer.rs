use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::bmfont::BMFont;
use crate::common::{Rect, FILE_TYPES};
use crate::engine_constants::EngineConstants;
use crate::framework::context::Context;
use crate::framework::error::GameError::ResourceLoadError;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::texture_set::TextureSet;

pub struct BMFontRenderer {
    font: BMFont,
    pages: Vec<String>,
}

impl BMFontRenderer {
    pub fn load(roots: &Vec<String>, desc_path: &str, ctx: &mut Context) -> GameResult<BMFontRenderer> {
        let full_path = PathBuf::from(desc_path);
        let desc_stem =
            full_path.file_stem().ok_or_else(|| ResourceLoadError("Cannot extract the file stem.".to_owned()))?;
        let stem = full_path.parent().unwrap_or(&full_path).join(desc_stem);

        let font = BMFont::load_from(filesystem::open_find(ctx, roots, &full_path)?)?;
        let mut pages = Vec::new();

        let (zeros, _, _) = FILE_TYPES
            .iter()
            .map(|ext| (1, ext, format!("{}_0{}", stem.to_string_lossy(), ext)))
            .find(|(_, _, path)| filesystem::exists_find(ctx, roots, &path))
            .or_else(|| {
                FILE_TYPES
                    .iter()
                    .map(|ext| (2, ext, format!("{}_00{}", stem.to_string_lossy(), ext)))
                    .find(|(_, _, path)| filesystem::exists_find(ctx, roots, &path))
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
                offset_x += glyph.xadvance as f32 * constants.font_scale;
            }
        }

        offset_x
    }

    pub fn text_width_with_rects<I: Iterator<Item = char> + Clone>(
        &self,
        iter: I,
        rect_map: &HashMap<char, Rect<u16>>,
        constants: &EngineConstants,
    ) -> f32 {
        let mut width = self.text_width(iter.clone(), constants);

        for chr in iter {
            if let Some(rect) = rect_map.get(&chr) {
                if let Some(glyph) = self.font.chars.get(&chr) {
                    width += rect.width() as f32;
                    width -= glyph.xadvance as f32 * constants.font_scale;
                }
            }
        }

        width
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

    pub fn draw_text_with_rects<I: Iterator<Item = char>>(
        &self,
        iter: I,
        x: f32,
        y: f32,
        constants: &EngineConstants,
        texture_set: &mut TextureSet,
        rect_map: &HashMap<char, Rect<u16>>,
        sprite_batch_name: Option<&str>,
        ctx: &mut Context,
    ) -> GameResult {
        self.draw_colored_text_with_rects(
            iter,
            x,
            y,
            (255, 255, 255, 255),
            constants,
            texture_set,
            rect_map,
            sprite_batch_name,
            ctx,
        )
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

    pub fn draw_text_with_shadow_and_rects<I: Iterator<Item = char> + Clone>(
        &self,
        iter: I,
        x: f32,
        y: f32,
        constants: &EngineConstants,
        texture_set: &mut TextureSet,
        rect_map: &HashMap<char, Rect<u16>>,
        sprite_batch_name: Option<&str>,
        ctx: &mut Context,
    ) -> GameResult {
        self.draw_colored_text_with_rects(
            iter.clone(),
            x + 1.0,
            y + 1.0,
            (0, 0, 0, 150),
            constants,
            texture_set,
            rect_map,
            None,
            ctx,
        )?;
        self.draw_colored_text_with_rects(
            iter,
            x,
            y,
            (255, 255, 255, 255),
            constants,
            texture_set,
            rect_map,
            sprite_batch_name,
            ctx,
        )
    }

    pub fn draw_colored_text_with_shadow_scaled<I: Iterator<Item = char> + Clone>(
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
        self.draw_colored_text_scaled(
            iter.clone(),
            x + scale,
            y + scale,
            scale,
            (0, 0, 0, 150),
            constants,
            texture_set,
            ctx,
        )?;
        self.draw_colored_text_scaled(iter, x, y, scale, color, constants, texture_set, ctx)
    }

    pub fn draw_colored_text_with_shadow_and_rects_scaled<I: Iterator<Item = char> + Clone>(
        &self,
        iter: I,
        x: f32,
        y: f32,
        scale: f32,
        color: (u8, u8, u8, u8),
        constants: &EngineConstants,
        texture_set: &mut TextureSet,
        rect_map: &HashMap<char, Rect<u16>>,
        sprite_batch_name: Option<&str>,
        ctx: &mut Context,
    ) -> GameResult {
        self.draw_colored_text_with_rects_scaled(
            iter.clone(),
            x + scale,
            y + scale,
            scale,
            (0, 0, 0, 150),
            constants,
            texture_set,
            rect_map,
            None,
            ctx,
        )?;
        self.draw_colored_text_with_rects_scaled(
            iter,
            x,
            y,
            scale,
            color,
            constants,
            texture_set,
            rect_map,
            sprite_batch_name,
            ctx,
        )
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
        let rect_map: HashMap<char, Rect<u16>> = HashMap::new();

        self.draw_colored_text_with_rects_scaled(iter, x, y, scale, color, constants, texture_set, &rect_map, None, ctx)
    }

    pub fn draw_colored_text_with_rects_scaled<I: Iterator<Item = char>>(
        &self,
        iter: I,
        x: f32,
        y: f32,
        scale: f32,
        color: (u8, u8, u8, u8),
        constants: &EngineConstants,
        texture_set: &mut TextureSet,
        rect_map: &HashMap<char, Rect<u16>>,
        sprite_batch_name: Option<&str>,
        ctx: &mut Context,
    ) -> GameResult {
        let mut sprite_rects: Vec<(f32, f32, &Rect<u16>)> = Vec::new();

        if self.pages.len() == 1 {
            let batch = texture_set.get_or_load_batch(ctx, constants, self.pages.get(0).unwrap())?;
            let mut offset_x = x;

            for chr in iter {
                if let Some(glyph) = self.font.chars.get(&chr) {
                    if let Some(rect) = rect_map.get(&chr) {
                        sprite_rects.push((
                            offset_x,
                            y + self.line_height(constants) / 2.0 - rect.height() as f32 / 2.0,
                            rect,
                        ));
                        offset_x += rect.width() as f32;
                    } else {
                        batch.add_rect_scaled_tinted(
                            offset_x + (glyph.xoffset as f32 * constants.font_scale),
                            y + (glyph.yoffset as f32 * constants.font_scale),
                            color,
                            constants.font_scale * scale,
                            constants.font_scale * scale,
                            &Rect::new_size(glyph.x as u16, glyph.y as u16, glyph.width as u16, glyph.height as u16),
                        );

                        offset_x += glyph.xadvance as f32 * constants.font_scale * scale;
                    }
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
                    if let Some(rect) = rect_map.get(&chr) {
                        sprite_rects.push((
                            offset_x,
                            y + self.line_height(constants) / 2.0 - rect.height() as f32 / 2.0,
                            rect,
                        ));
                        offset_x += rect.width() as f32;
                    } else {
                        if glyph.page == page {
                            batch.add_rect_scaled_tinted(
                                offset_x + (glyph.xoffset as f32 * constants.font_scale),
                                y + (glyph.yoffset as f32 * constants.font_scale),
                                color,
                                constants.font_scale * scale,
                                constants.font_scale * scale,
                                &Rect::new_size(
                                    glyph.x as u16,
                                    glyph.y as u16,
                                    glyph.width as u16,
                                    glyph.height as u16,
                                ),
                            );
                        }

                        offset_x += scale * (glyph.xadvance as f32 * constants.font_scale);
                    }
                }

                batch.draw(ctx)?;
            }
        }

        if let Some(sprite_batch_name) = sprite_batch_name {
            if !sprite_rects.is_empty() {
                let sprite_batch = texture_set.get_or_load_batch(ctx, constants, sprite_batch_name)?;

                for (x, y, rect) in sprite_rects {
                    sprite_batch.add_rect_scaled(x, y, scale, scale, rect);
                }

                sprite_batch.draw(ctx)?;
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

    pub fn draw_colored_text_with_rects<I: Iterator<Item = char>>(
        &self,
        iter: I,
        x: f32,
        y: f32,
        color: (u8, u8, u8, u8),
        constants: &EngineConstants,
        texture_set: &mut TextureSet,
        rect_map: &HashMap<char, Rect<u16>>,
        sprite_batch_name: Option<&str>,
        ctx: &mut Context,
    ) -> GameResult {
        self.draw_colored_text_with_rects_scaled(
            iter,
            x,
            y,
            1.0,
            color,
            constants,
            texture_set,
            rect_map,
            sprite_batch_name,
            ctx,
        )
    }
}
