use std::collections::HashMap;

use ggez::{Context, GameError, GameResult};
use ggez::filesystem;
use ggez::graphics::{Drawable, DrawParam, FilterMode, Image, Rect};
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::nalgebra::{Point2, Vector2};
use log::info;

use crate::common;
use crate::str;

pub struct SizedBatch {
    pub batch: SpriteBatch,
    width: usize,
    height: usize,
    real_width: usize,
    real_height: usize,
    scale_x: f32,
    scale_y: f32,
}

impl SizedBatch {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn real_dimensions(&self) -> (usize, usize) {
        (self.real_width, self.real_height)
    }

    pub fn to_rect(&self) -> common::Rect<usize> {
        common::Rect::<usize>::new(0, 0, self.width, self.height)
    }

    pub fn clear(&mut self) {
        self.batch.clear();
    }

    pub fn add(&mut self, x: f32, y: f32) {
        let param = DrawParam::new()
            .dest(Point2::new(x, y))
            .scale(Vector2::new(self.scale_x, self.scale_y));

        self.batch.add(param);
    }

    pub fn add_rect(&mut self, x: f32, y: f32, rect: &common::Rect<usize>) {
        let param = DrawParam::new()
            .src(Rect::new(rect.left as f32 / self.width as f32,
                           rect.top as f32 / self.height as f32,
                           (rect.right - rect.left) as f32 / self.width as f32,
                           (rect.bottom - rect.top) as f32 / self.height as f32))
            .dest(Point2::new(x, y))
            .scale(Vector2::new(self.scale_x, self.scale_y));

        self.batch.add(param);
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.batch.set_filter(FilterMode::Nearest);
        self.batch.draw(ctx, DrawParam::new())?;
        self.batch.clear();
        Ok(())
    }
}

pub struct TextureSet {
    pub tex_map: HashMap<String, SizedBatch>,
    base_path: String,
}

static FILE_TYPES: [&str; 3] = [".png", ".bmp", ".pbm"];

impl TextureSet {
    pub fn new(base_path: &str) -> TextureSet {
        TextureSet {
            tex_map: HashMap::new(),
            base_path: str!(base_path),
        }
    }

    pub fn load_texture(&self, ctx: &mut Context, path: &str) -> GameResult<SizedBatch> {
        let path = FILE_TYPES
            .iter()
            .map(|ext| [&self.base_path, path, ext].join(""))
            .find(|path| filesystem::exists(ctx, path))
            .ok_or_else(|| GameError::ResourceLoadError(format!("Texture {:?} does not exist.", path)))?;

        info!("Loading texture: {}", path);

        let image = Image::new(ctx, path)?;
        let size = image.dimensions();

        assert_ne!(size.w, 0.0, "size.w == 0");
        assert_ne!(size.h, 0.0, "size.h == 0");

        let scale_x = 1.0;
        let scale_y = 1.0;
        let width = (size.w / scale_x) as usize;
        let height = (size.h / scale_y) as usize;

        Ok(SizedBatch {
            batch: SpriteBatch::new(image),
            width,
            height,
            scale_x,
            scale_y,
            real_width: size.w as usize,
            real_height: size.h as usize,
        })
    }

    pub fn ensure_texture_loaded(&mut self, ctx: &mut Context, name: &str) -> GameResult {
        if self.tex_map.contains_key(name) {
            return Ok(());
        }

        let batch = self.load_texture(ctx, name)?;
        self.tex_map.insert(str!(name), batch);

        Ok(())
    }
}
