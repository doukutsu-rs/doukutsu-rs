use std::collections::HashMap;
use std::io::{BufReader, Read, Seek, SeekFrom};

use image::RgbaImage;
use itertools::Itertools;
use log::info;

use crate::common;
use crate::common::{FILE_TYPES, Point, Rect};
use crate::engine_constants::EngineConstants;
use crate::framework::backend::{BackendTexture, SpriteBatchCommand};
use crate::framework::context::Context;
use crate::framework::error::{GameError, GameResult};
use crate::framework::filesystem;
use crate::framework::graphics::{create_texture, FilterMode};
use crate::settings::Settings;
use crate::shared_game_state::Season;
use crate::str;

pub static mut I_MAG: f32 = 1.0;
pub static mut G_MAG: f32 = 1.0;

pub struct SizedBatch {
    batch: Box<dyn BackendTexture>,
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

    pub fn add(&mut self, mut x: f32, mut y: f32) {
        unsafe {
            x = (x * G_MAG).round() / G_MAG;
            y = (y * G_MAG).round() / G_MAG;
        }
        let mag = unsafe { I_MAG };

        self.batch.add(SpriteBatchCommand::DrawRect(
            Rect {
                left: 0 as f32,
                top: 0 as f32,
                right: self.real_width as f32,
                bottom: self.real_height as f32,
            },
            Rect {
                left: x * mag,
                top: y * mag,
                right: (x + self.width() as f32) * mag,
                bottom: (y + self.height() as f32) * mag,
            },
        ));
    }

    #[inline(always)]
    pub fn add_rect(&mut self, x: f32, y: f32, rect: &common::Rect<u16>) {
        self.add_rect_scaled(x, y, self.scale_x, self.scale_y, rect)
    }

    #[inline(always)]
    pub fn add_rect_tinted(&mut self, x: f32, y: f32, color: (u8, u8, u8, u8), rect: &common::Rect<u16>) {
        self.add_rect_scaled_tinted(x, y, color, self.scale_x, self.scale_y, rect)
    }

    pub fn add_rect_scaled(&mut self, mut x: f32, mut y: f32, mut scale_x: f32, mut scale_y: f32, rect: &common::Rect<u16>) {
        if (rect.right - rect.left) == 0 || (rect.bottom - rect.top) == 0 {
            return;
        }

        unsafe {
            x = (x * G_MAG).round() / G_MAG;
            y = (y * G_MAG).round() / G_MAG;
        }
        let mag = unsafe { I_MAG };

        self.batch.add(SpriteBatchCommand::DrawRect(
            Rect {
                left: rect.left as f32,
                top: rect.top as f32,
                right: rect.right as f32,
                bottom: rect.bottom as f32,
            },
            Rect {
                left: x * mag * scale_x,
                top: y * mag * scale_y,
                right: (x + rect.width() as f32) * mag * scale_x,
                bottom: (y + rect.height() as f32) * mag * scale_y,
            },
        ));
    }

    pub fn add_rect_scaled_tinted(&mut self, mut x: f32, mut y: f32, color: (u8, u8, u8, u8), mut scale_x: f32, mut scale_y: f32, rect: &common::Rect<u16>) {
        if (rect.right - rect.left) == 0 || (rect.bottom - rect.top) == 0 {
            return;
        }

        unsafe {
            x = (x * G_MAG).floor() / G_MAG;
            y = (y * G_MAG).floor() / G_MAG;
        }
        let mag = unsafe { I_MAG };

        self.batch.add(SpriteBatchCommand::DrawRectTinted(
            Rect {
                left: rect.left as f32,
                top: rect.top as f32,
                right: rect.right as f32,
                bottom: rect.bottom as f32,
            },
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
    pub fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.draw_filtered(FilterMode::Nearest, ctx)
    }

    pub fn draw_filtered(&mut self, filter: FilterMode, ctx: &mut Context) -> GameResult {
        ///self.batch.set_filter(filter);
        self.batch.draw()?;
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

    fn load_image(&self, ctx: &mut Context, path: &str) -> GameResult<Box<dyn BackendTexture>> {
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

        create_texture(ctx, width as u16, height as u16, &img)
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

        let batch = self.load_image(ctx, &path)?;
        let size = batch.dimensions();

        assert_ne!(size.0 as isize, 0, "size.width == 0");
        assert_ne!(size.1 as isize, 0, "size.height == 0");

        let orig_dimensions = constants.tex_sizes.get(name).unwrap_or_else(|| &size);
        let scale = orig_dimensions.0 as f32 / size.0 as f32;
        let width = (size.0 as f32 * scale) as usize;
        let height = (size.1 as f32 * scale) as usize;
        println!("{} {} {} {}", size.0, size.1, width, height);

        Ok(SizedBatch {
            batch,
            width,
            height,
            scale_x: scale,
            scale_y: scale,
            real_width: size.0 as usize,
            real_height: size.1 as usize,
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
        Ok(())
    }

    pub fn draw_outline_rect(&self, rect: common::Rect, width: f32, color: [f32; 4], ctx: &mut Context) -> GameResult {
        Ok(())
    }
}
