use std::collections::HashMap;
use std::io::{BufReader, Read, Seek, SeekFrom};

use image::RgbaImage;
use itertools::Itertools;
use log::info;

use crate::common;
use crate::common::{Rect, FILE_TYPES};
use crate::engine_constants::EngineConstants;
use crate::framework::backend::{BackendTexture, SpriteBatchCommand};
use crate::framework::context::Context;
use crate::framework::error::{GameError, GameResult};
use crate::framework::filesystem;
use crate::framework::graphics::{create_texture, FilterMode};
use crate::settings::Settings;
use crate::shared_game_state::Season;

pub static mut I_MAG: f32 = 1.0;
pub static mut G_MAG: f32 = 1.0;

pub trait SpriteBatch {
    fn width(&self) -> usize;

    fn height(&self) -> usize;

    fn dimensions(&self) -> (usize, usize);

    fn real_dimensions(&self) -> (usize, usize);

    fn scale(&self) -> (f32, f32);

    fn has_glow_layer(&self) -> bool;

    fn has_normal_layer(&self) -> bool;

    fn glow(&mut self) -> Option<&mut dyn SpriteBatch> {
        None
    }

    fn normal(&mut self) -> Option<&mut dyn SpriteBatch> {
        None
    }

    fn to_rect(&self) -> common::Rect<usize>;

    fn clear(&mut self);

    fn add(&mut self, x: f32, y: f32);

    fn add_rect(&mut self, x: f32, y: f32, rect: &common::Rect<u16>);

    fn add_rect_flip(&mut self, x: f32, y: f32, flip_x: bool, flip_y: bool, rect: &common::Rect<u16>);

    fn add_rect_tinted(&mut self, x: f32, y: f32, color: (u8, u8, u8, u8), rect: &common::Rect<u16>);

    fn add_rect_scaled(&mut self, x: f32, y: f32, scale_x: f32, scale_y: f32, rect: &common::Rect<u16>);

    fn add_rect_scaled_tinted(
        &mut self,
        x: f32,
        y: f32,
        color: (u8, u8, u8, u8),
        scale_x: f32,
        scale_y: f32,
        rect: &common::Rect<u16>,
    );

    fn draw(&mut self, ctx: &mut Context) -> GameResult;

    fn draw_filtered(&mut self, _filter: FilterMode, _ctx: &mut Context) -> GameResult;

    fn get_texture(&self) -> Option<&Box<dyn BackendTexture>>;
}

pub struct DummyBatch;

impl SpriteBatch for DummyBatch {
    fn width(&self) -> usize {
        1
    }

    fn height(&self) -> usize {
        1
    }

    fn dimensions(&self) -> (usize, usize) {
        (1, 1)
    }

    fn real_dimensions(&self) -> (usize, usize) {
        (1, 1)
    }

    fn scale(&self) -> (f32, f32) {
        (1.0, 1.0)
    }

    fn has_glow_layer(&self) -> bool {
        false
    }

    fn has_normal_layer(&self) -> bool {
        false
    }

    fn to_rect(&self) -> Rect<usize> {
        Rect::new(0, 0, 1, 1)
    }

    fn clear(&mut self) {}

    fn add(&mut self, _x: f32, _y: f32) {}

    fn add_rect(&mut self, _x: f32, _y: f32, _rect: &Rect<u16>) {}

    fn add_rect_flip(&mut self, _x: f32, _y: f32, _flip_x: bool, _flip_y: bool, _rect: &Rect<u16>) {}

    fn add_rect_tinted(&mut self, _x: f32, _y: f32, _color: (u8, u8, u8, u8), _rect: &Rect<u16>) {}

    fn add_rect_scaled(&mut self, _x: f32, _y: f32, _scale_x: f32, _scale_y: f32, _rect: &Rect<u16>) {}

    fn add_rect_scaled_tinted(
        &mut self,
        _x: f32,
        _y: f32,
        _color: (u8, u8, u8, u8),
        _scale_x: f32,
        _scale_y: f32,
        _rect: &Rect<u16>,
    ) {
    }

    fn draw(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw_filtered(&mut self, _filter: FilterMode, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn get_texture(&self) -> Option<&Box<dyn BackendTexture>> {
        None
    }
}

pub struct SubBatch {
    batch: Box<dyn BackendTexture>,
    width: u16,
    height: u16,
    real_width: u16,
    real_height: u16,
    scale_x: f32,
    scale_y: f32,
}

pub struct CombinedBatch {
    main_batch: SubBatch,
    glow_batch: Option<SubBatch>,
}

impl SpriteBatch for SubBatch {
    #[inline(always)]
    fn width(&self) -> usize {
        self.width as _
    }

    #[inline(always)]
    fn height(&self) -> usize {
        self.height as _
    }

    #[inline(always)]
    fn dimensions(&self) -> (usize, usize) {
        (self.width as _, self.height as _)
    }

    #[inline(always)]
    fn real_dimensions(&self) -> (usize, usize) {
        (self.real_width as _, self.real_height as _)
    }

    #[inline(always)]
    fn scale(&self) -> (f32, f32) {
        (self.scale_x, self.scale_y)
    }

    #[inline(always)]
    fn has_glow_layer(&self) -> bool {
        false
    }

    #[inline(always)]
    fn has_normal_layer(&self) -> bool {
        false
    }

    #[inline(always)]
    fn to_rect(&self) -> common::Rect<usize> {
        common::Rect::<usize>::new(0, 0, self.width as _, self.height as _)
    }

    #[inline(always)]
    fn clear(&mut self) {
        self.batch.clear();
    }

    fn add(&mut self, x: f32, y: f32) {
        let mag = unsafe { I_MAG };

        self.batch.add(SpriteBatchCommand::DrawRect(
            Rect { left: 0 as f32, top: 0 as f32, right: self.real_width as f32, bottom: self.real_height as f32 },
            Rect {
                left: x * mag,
                top: y * mag,
                right: (x + self.width() as f32) * mag,
                bottom: (y + self.height() as f32) * mag,
            },
        ));
    }

    #[inline(always)]
    fn add_rect(&mut self, x: f32, y: f32, rect: &common::Rect<u16>) {
        self.add_rect_scaled(x, y, 1.0, 1.0, rect)
    }

    fn add_rect_flip(&mut self, x: f32, y: f32, flip_x: bool, flip_y: bool, rect: &common::Rect<u16>) {
        if (rect.right - rect.left) == 0 || (rect.bottom - rect.top) == 0 {
            return;
        }

        let mag = unsafe { I_MAG };

        self.batch.add(SpriteBatchCommand::DrawRectFlip(
            Rect {
                left: rect.left as f32 / self.scale_x,
                top: rect.top as f32 / self.scale_y,
                right: rect.right as f32 / self.scale_x,
                bottom: rect.bottom as f32 / self.scale_y,
            },
            Rect {
                left: x * mag,
                top: y * mag,
                right: (x + rect.width() as f32) * mag,
                bottom: (y + rect.height() as f32) * mag,
            },
            flip_x,
            flip_y,
        ));
    }

    #[inline(always)]
    fn add_rect_tinted(&mut self, x: f32, y: f32, color: (u8, u8, u8, u8), rect: &common::Rect<u16>) {
        self.add_rect_scaled_tinted(x, y, color, 1.0, 1.0, rect)
    }

    fn add_rect_scaled(&mut self, x: f32, y: f32, scale_x: f32, scale_y: f32, rect: &common::Rect<u16>) {
        if (rect.right.saturating_sub(rect.left)) == 0 || (rect.bottom.saturating_sub(rect.top)) == 0 {
            return;
        }

        let mag = unsafe { I_MAG };

        self.batch.add(SpriteBatchCommand::DrawRect(
            Rect {
                left: rect.left as f32 / self.scale_x,
                top: rect.top as f32 / self.scale_y,
                right: rect.right as f32 / self.scale_x,
                bottom: rect.bottom as f32 / self.scale_y,
            },
            Rect {
                left: x * mag,
                top: y * mag,
                right: (x + rect.width() as f32 * scale_x) * mag,
                bottom: (y + rect.height() as f32 * scale_y) * mag,
            },
        ));
    }

    fn add_rect_scaled_tinted(
        &mut self,
        x: f32,
        y: f32,
        color: (u8, u8, u8, u8),
        scale_x: f32,
        scale_y: f32,
        rect: &common::Rect<u16>,
    ) {
        if (rect.right - rect.left) == 0 || (rect.bottom - rect.top) == 0 {
            return;
        }

        let mag = unsafe { I_MAG };

        self.batch.add(SpriteBatchCommand::DrawRectTinted(
            Rect { left: rect.left as f32, top: rect.top as f32, right: rect.right as f32, bottom: rect.bottom as f32 },
            Rect {
                left: x * mag,
                top: y * mag,
                right: (x + rect.width() as f32 * scale_x) * mag,
                bottom: (y + rect.height() as f32 * scale_y) * mag,
            },
            color.into(),
        ));
    }

    #[inline(always)]
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.draw_filtered(FilterMode::Nearest, ctx)
    }

    fn draw_filtered(&mut self, _filter: FilterMode, _ctx: &mut Context) -> GameResult {
        //self.batch.set_filter(filter);
        self.batch.draw()?;
        self.batch.clear();
        Ok(())
    }

    fn get_texture(&self) -> Option<&Box<dyn BackendTexture>> {
        Some(&self.batch)
    }
}

impl SpriteBatch for CombinedBatch {
    fn width(&self) -> usize {
        self.main_batch.width as _
    }

    fn height(&self) -> usize {
        self.main_batch.height as _
    }

    fn dimensions(&self) -> (usize, usize) {
        self.main_batch.dimensions()
    }

    fn real_dimensions(&self) -> (usize, usize) {
        self.main_batch.real_dimensions()
    }

    fn scale(&self) -> (f32, f32) {
        self.main_batch.scale()
    }

    fn has_glow_layer(&self) -> bool {
        self.glow_batch.is_some()
    }

    fn has_normal_layer(&self) -> bool {
        false
    }

    fn glow(&mut self) -> Option<&mut dyn SpriteBatch> {
        self.glow_batch.as_mut().map(|batch| batch as &mut dyn SpriteBatch)
    }

    fn to_rect(&self) -> Rect<usize> {
        self.main_batch.to_rect()
    }

    fn clear(&mut self) {
        self.main_batch.clear()
    }

    fn add(&mut self, x: f32, y: f32) {
        self.main_batch.add(x, y)
    }

    fn add_rect(&mut self, x: f32, y: f32, rect: &Rect<u16>) {
        self.main_batch.add_rect(x, y, rect)
    }

    fn add_rect_flip(&mut self, x: f32, y: f32, flip_x: bool, flip_y: bool, rect: &Rect<u16>) {
        self.main_batch.add_rect_flip(x, y, flip_x, flip_y, rect)
    }

    fn add_rect_tinted(&mut self, x: f32, y: f32, color: (u8, u8, u8, u8), rect: &Rect<u16>) {
        self.main_batch.add_rect_tinted(x, y, color, rect)
    }

    fn add_rect_scaled(&mut self, x: f32, y: f32, scale_x: f32, scale_y: f32, rect: &Rect<u16>) {
        self.main_batch.add_rect_scaled(x, y, scale_x, scale_y, rect)
    }

    fn add_rect_scaled_tinted(
        &mut self,
        x: f32,
        y: f32,
        color: (u8, u8, u8, u8),
        scale_x: f32,
        scale_y: f32,
        rect: &Rect<u16>,
    ) {
        self.main_batch.add_rect_scaled_tinted(x, y, color, scale_x, scale_y, rect)
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.main_batch.draw(ctx)
    }

    fn draw_filtered(&mut self, filter: FilterMode, ctx: &mut Context) -> GameResult {
        self.main_batch.draw_filtered(filter, ctx)
    }

    fn get_texture(&self) -> Option<&Box<dyn BackendTexture>> {
        self.main_batch.get_texture()
    }
}

pub struct TextureSet {
    pub tex_map: HashMap<String, Box<dyn SpriteBatch>>,
    pub paths: Vec<String>,
    dummy_batch: Box<dyn SpriteBatch>,
}

impl TextureSet {
    pub fn new(base_path: &str) -> TextureSet {
        TextureSet {
            tex_map: HashMap::new(),
            paths: vec![base_path.to_string(), "".to_string()],
            dummy_batch: Box::new(DummyBatch),
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

    fn load_image(&self, ctx: &mut Context, path: &str) -> GameResult<Box<dyn BackendTexture>> {
        let img = {
            let mut buf = [0u8; 8];
            let mut reader = filesystem::open(ctx, path)?;
            reader.read_exact(&mut buf)?;
            reader.seek(SeekFrom::Start(0))?;

            let image = image::load(BufReader::new(reader), image::guess_format(&buf)?)?;
            let mut rgba = image.to_rgba8();
            if image.color().channel_count() != 4 {
                TextureSet::make_transparent(&mut rgba);
            }
            rgba
        };
        let (width, height) = img.dimensions();

        create_texture(ctx, width as u16, height as u16, &img)
    }

    pub fn find_texture(&self, ctx: &mut Context, name: &str) -> Option<String> {
        self.paths.iter().find_map(|s| {
            FILE_TYPES.iter().map(|ext| [s, name, ext].join("")).find(|path| filesystem::exists(ctx, path))
        })
    }

    pub fn load_texture(
        &self,
        ctx: &mut Context,
        constants: &EngineConstants,
        name: &str,
    ) -> GameResult<Box<dyn SpriteBatch>> {
        let path = self
            .find_texture(ctx, name)
            .ok_or_else(|| GameError::ResourceLoadError(format!("Texture {} does not exist.", name)))?;

        let glow_path = self.find_texture(ctx, &[name, ".glow"].join(""));

        info!("Loading texture: {} -> {}", name, path);

        fn make_batch(name: &str, constants: &EngineConstants, batch: Box<dyn BackendTexture>) -> SubBatch {
            let size = batch.dimensions();

            let orig_dimensions = constants.tex_sizes.get(name).unwrap_or(&size);
            let scale = orig_dimensions.0 as f32 / size.0 as f32;
            let width = (size.0 as f32 * scale) as _;
            let height = (size.1 as f32 * scale) as _;

            SubBatch {
                batch,
                width,
                height,
                scale_x: scale,
                scale_y: scale,
                real_width: size.0 as _,
                real_height: size.1 as _,
            }
        }

        let main_batch = make_batch(name, constants, self.load_image(ctx, &path)?);
        let glow_batch = if let Some(glow_path) = glow_path {
            self.load_image(ctx, &glow_path).ok().map(|b| make_batch(name, constants, b))
        } else {
            None
        };

        Ok(Box::new(CombinedBatch { main_batch, glow_batch }))
    }

    pub fn get_or_load_batch(
        &mut self,
        ctx: &mut Context,
        constants: &EngineConstants,
        name: &str,
    ) -> GameResult<&mut Box<dyn SpriteBatch>> {
        if ctx.headless {
            return Ok(&mut self.dummy_batch);
        }

        if !self.tex_map.contains_key(name) {
            let batch = self.load_texture(ctx, constants, name)?;
            self.tex_map.insert(name.to_owned(), batch);
        }

        Ok(self.tex_map.get_mut(name).unwrap())
    }
}
