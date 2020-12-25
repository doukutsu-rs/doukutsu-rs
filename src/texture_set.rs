use std::collections::HashMap;
use std::io::{BufReader, Read, Seek, SeekFrom};

use ggez;
use ggez::{Context, GameError, GameResult, graphics};
use ggez::filesystem;
use ggez::graphics::{Drawable, DrawMode, DrawParam, FilterMode, Image, Mesh, mint, Rect};
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::nalgebra::{Point2, Vector2};
use image::RgbaImage;
use itertools::Itertools;
use log::info;

use crate::common;
use crate::common::FILE_TYPES;
use crate::engine_constants::EngineConstants;
use crate::settings::Settings;
use crate::shared_game_state::Season;
use crate::str;

pub static mut G_MAG: f32 = 1.0;

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
    #[inline(always)]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline(always)]
    pub fn height(&self) -> usize {
        self.height
    }

    #[inline(always)]
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    #[inline(always)]
    pub fn real_dimensions(&self) -> (usize, usize) {
        (self.real_width, self.real_height)
    }

    #[inline(always)]
    pub fn scale(&self) -> (f32, f32) {
        (self.scale_x, self.scale_y)
    }

    #[inline(always)]
    pub fn to_rect(&self) -> common::Rect<usize> {
        common::Rect::<usize>::new(0, 0, self.width, self.height)
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.batch.clear();
    }

    pub fn add(&mut self, x: f32, y: f32) {
        let param = DrawParam::new()
            .dest(Point2::new(x, y))
            .scale(Vector2::new(self.scale_x, self.scale_y));

        self.batch.add(param);
    }

    #[inline(always)]
    pub fn add_rect(&mut self, x: f32, y: f32, rect: &common::Rect<u16>) {
        self.add_rect_scaled(x, y, self.scale_x, self.scale_y, rect)
    }

    #[inline(always)]
    pub fn add_rect_tinted(&mut self, x: f32, y: f32, color: (u8, u8, u8, u8), rect: &common::Rect<u16>) {
        self.add_rect_scaled_tinted(x, y, color, self.scale_x, self.scale_y, rect)
    }

    pub fn add_rect_scaled(&mut self, mut x: f32, mut y: f32, scale_x: f32, scale_y: f32, rect: &common::Rect<u16>) {
        if (rect.right - rect.left) == 0 || (rect.bottom - rect.top) == 0 {
            return;
        }

        unsafe {
            x = (x * G_MAG).floor() / G_MAG;
            y = (y * G_MAG).floor() / G_MAG;
        }

        let param = DrawParam::new()
            .src(Rect::new(rect.left as f32 / self.width as f32,
                           rect.top as f32 / self.height as f32,
                           (rect.right - rect.left) as f32 / self.width as f32,
                           (rect.bottom - rect.top) as f32 / self.height as f32))
            .dest(mint::Point2 { x, y })
            .scale(Vector2::new(scale_x, scale_y));

        self.batch.add(param);
    }

    pub fn add_rect_scaled_tinted(&mut self, x: f32, y: f32, color: (u8, u8, u8, u8), scale_x: f32, scale_y: f32, rect: &common::Rect<u16>) {
        if (rect.right - rect.left) == 0 || (rect.bottom - rect.top) == 0 {
            return;
        }

        let param = DrawParam::new()
            .color(color.into())
            .src(Rect::new(rect.left as f32 / self.width as f32,
                           rect.top as f32 / self.height as f32,
                           (rect.right - rect.left) as f32 / self.width as f32,
                           (rect.bottom - rect.top) as f32 / self.height as f32))
            .dest(mint::Point2 { x, y })
            .scale(Vector2::new(scale_x, scale_y));

        self.batch.add(param);
    }

    #[inline(always)]
    pub fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.draw_filtered(FilterMode::Nearest, ctx)
    }

    pub fn draw_filtered(&mut self, filter: FilterMode, ctx: &mut Context) -> GameResult {
        self.batch.set_filter(filter);
        self.batch.draw(ctx, DrawParam::new())?;
        self.batch.clear();
        Ok(())
    }
}

pub struct TextureSet {
    pub tex_map: HashMap<String, SizedBatch>,
    pub paths: Vec<String>,
}

impl TextureSet {
    pub fn new(base_path: &str) -> TextureSet {
        TextureSet {
            tex_map: HashMap::new(),
            paths: vec![base_path.to_string(), "".to_string()],
        }
    }

    pub fn apply_seasonal_content(&mut self, season: Season, settings: &Settings) {
        if settings.original_textures {
            self.paths.insert(0, "/base/ogph/".to_string())
        } else if settings.seasonal_textures {
            match season {
                Season::Halloween => self.paths.insert(0, "/Halloween/season/".to_string()),
                Season::Christmas => self.paths.insert(0, "/Christmas/season/".to_string()),
                _ => {}
            }
        }
    }

    fn make_transparent(rgba: &mut RgbaImage) {
        for (r, g, b, a) in rgba.iter_mut().tuples() {
            if *r == 0 && *g == 0 && *b == 0 {
                *a = 0;
            }
        }
    }

    fn load_image(&self, ctx: &mut Context, path: &str) -> GameResult<Image> {
        let img = {
            let mut buf = [0u8; 8];
            let mut reader = filesystem::open(ctx, path)?;
            reader.read_exact(&mut buf)?;
            reader.seek(SeekFrom::Start(0))?;

            let image = image::load(BufReader::new(reader), image::guess_format(&buf)?)?;
            let mut rgba = image.to_rgba();
            if image.color().channel_count() != 4 {
                TextureSet::make_transparent(&mut rgba);
            }
            rgba
        };
        let (width, height) = img.dimensions();

        Image::from_rgba8(ctx, width as u16, height as u16, img.as_ref())
    }

    pub fn load_texture(&self, ctx: &mut Context, constants: &EngineConstants, name: &str) -> GameResult<SizedBatch> {
        let path = self.paths.iter().find_map(|s| FILE_TYPES
            .iter()
            .map(|ext| [s, name, ext].join(""))
            .find(|path| {
                println!("{}", path);
                filesystem::exists(ctx, path)
            })
        ).ok_or_else(|| GameError::ResourceLoadError(format!("Texture {} does not exist.", name)))?;

        info!("Loading texture: {}", path);

        let image = self.load_image(ctx, &path)?;
        let size = image.dimensions();

        assert_ne!(size.w as isize, 0, "size.w == 0");
        assert_ne!(size.h as isize, 0, "size.h == 0");

        let dim = (size.w as usize, size.h as usize);
        let orig_dimensions = constants.tex_sizes.get(name).unwrap_or_else(|| &dim);
        let scale_x = orig_dimensions.0 as f32 / size.w;
        let scale_y = orig_dimensions.0 as f32 / size.w;
        let width = (size.w * scale_x) as usize;
        let height = (size.h * scale_y) as usize;

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

    pub fn get_or_load_batch(&mut self, ctx: &mut Context, constants: &EngineConstants, name: &str) -> GameResult<&mut SizedBatch> {
        if !self.tex_map.contains_key(name) {
            let batch = self.load_texture(ctx, constants, name)?;
            self.tex_map.insert(str!(name), batch);
        }

        Ok(self.tex_map.get_mut(name).unwrap())
    }

    pub fn draw_rect(&self, rect: common::Rect, color: [f32; 4], ctx: &mut Context) -> GameResult {
        let rect = Mesh::new_rectangle(ctx, DrawMode::fill(), rect.into(), color.into())?;
        graphics::draw(ctx, &rect, DrawParam::new())?;
        Ok(())
    }

    pub fn draw_outline_rect(&self, rect: common::Rect, width: f32, color: [f32; 4], ctx: &mut Context) -> GameResult {
        let rect = Mesh::new_rectangle(ctx, DrawMode::stroke(width), rect.into(), color.into())?;
        graphics::draw(ctx, &rect, DrawParam::new())?;
        Ok(())
    }
}
