use std::collections::HashSet;
use std::path::PathBuf;

use crate::bmfont::BMFont;
use crate::common::{FILE_TYPES, Rect};
use crate::engine_constants::EngineConstants;
use crate::ggez::{Context, filesystem, GameResult};
use crate::ggez::GameError::ResourceLoadError;
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
        let desc_stem = full_path.file_stem()
            .ok_or_else(|| ResourceLoadError(str!("Cannot extract the file stem.")))?;
        let stem = full_path.parent().unwrap_or(full_path).join(desc_stem);

        let font = BMFont::load_from(filesystem::open(ctx, &full_path)?)?;
        let mut pages = Vec::new();

        println!("stem: {:?}", stem);
        let (zeros, ext, format) = FILE_TYPES
            .iter()
            .map(|ext| (1, ext, format!("{}_0{}", stem.to_string_lossy(), ext)))
            .find(|(_, _, path)| filesystem::exists(ctx, &path))
            .or_else(|| FILE_TYPES
                .iter()
                .map(|ext| (2, ext, format!("{}_00{}", stem.to_string_lossy(), ext)))
                .find(|(_, _, path)| filesystem::exists(ctx, &path)))
            .ok_or_else(|| ResourceLoadError(format!("Cannot find glyph atlas 0 for font: {:?}", desc_path)))?;

        for i in 0..font.pages {
            let page_path = format!("{}_{:02$}", stem.to_string_lossy(), i, zeros);
            println!("x: {}", &page_path);

            pages.push(page_path);
        }

        Ok(Self {
            font,
            pages,
        })
    }

    pub fn draw_text<I: Iterator<Item=char>>(&self, iter: I, x: f32, y: f32, constants: &EngineConstants, texture_set: &mut TextureSet, ctx: &mut Context) -> GameResult {
        if self.pages.len() == 1 {
            let batch = texture_set.get_or_load_batch(ctx, constants, self.pages.get(0).unwrap())?;
            let mut offset_x = x;

            for chr in iter {
                if let Some(glyph) = self.font.chars.get(&chr) {
                    batch.add_rect_scaled(offset_x, y + (glyph.yoffset as f32 * constants.font_scale).floor(),
                                          constants.font_scale, constants.font_scale,
                                          &Rect::<usize>::new_size(
                                              glyph.x as usize, glyph.y as usize,
                                              glyph.width as usize, glyph.height as usize,
                                          ));

                    offset_x += ((glyph.width as f32 + glyph.xoffset as f32) * constants.font_scale).floor() + if chr != ' ' { 1.0 } else { constants.font_space_offset };
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
                        batch.add_rect_scaled(offset_x, y + (glyph.yoffset as f32 * constants.font_scale).floor(),
                                              constants.font_scale, constants.font_scale,
                                              &Rect::<usize>::new_size(
                                                  glyph.x as usize, glyph.y as usize,
                                                  glyph.width as usize, glyph.height as usize,
                                              ));
                    }

                    offset_x += ((glyph.width as f32 + glyph.xoffset as f32) * constants.font_scale).floor() + if *chr != ' ' { 1.0 } else { constants.font_space_offset };
                }

                batch.draw(ctx)?;
            }
        }

        Ok(())
    }
}
