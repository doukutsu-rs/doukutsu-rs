use crate::bitfield;
use crate::common::Rect;
use crate::engine_constants::EngineConstants;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::graphics::texture_set::TextureSet;

bitfield! {
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct TextBuilderFlag(u8);

    pub shadow, set_shadow: 0;
    pub centered, set_centered: 1;
}

#[derive(Copy, Clone)]
pub struct Symbols<'a> {
    pub symbols: &'a [(char, Rect<u16>)],
    pub texture: &'a str,
}

#[derive(Clone, Default)]
pub struct SymbolsOwned {
    pub symbols: Vec<(char, Rect<u16>)>,
    pub texture: String,
}

impl Symbols<'_> {
    pub fn to_owned(&self) -> SymbolsOwned {
        SymbolsOwned {
            symbols: self.symbols.to_vec(),
            texture: self.texture.to_owned()
        }
    }
}

impl SymbolsOwned {
    pub fn as_ref<'a>(&'a self) -> Symbols<'a> {
        Symbols {
            symbols: self.symbols.as_slice(),
            texture: self.texture.as_str()
        }
    }
}

pub static EMPTY_SYMBOLS: Symbols = Symbols { symbols: &[], texture: "" };

pub trait Font {
    fn builder(&self) -> TextBuilder<'_, '_>
    where
        Self: Sized,
    {
        TextBuilder::new(self)
    }

    fn line_height(&self) -> f32;

    fn compute_width(&self, text: &mut dyn Iterator<Item = char>, symbols: Option<&Symbols>) -> f32;

    fn draw(
        &self,
        text: &mut dyn Iterator<Item = char>,
        x: f32,
        y: f32,
        scale: f32,
        box_width: f32,
        shadow_color: (u8, u8, u8, u8),
        color: (u8, u8, u8, u8),
        flags: TextBuilderFlag,
        constants: &EngineConstants,
        texture_set: &mut TextureSet,
        symbols: Option<Symbols>,
        ctx: &mut Context,
    ) -> GameResult;
}

pub struct TextBuilder<'a, 'b> {
    font: &'a dyn Font,
    x: f32,
    y: f32,
    scale: f32,
    shadow_color: (u8, u8, u8, u8),
    color: (u8, u8, u8, u8),
    flags: TextBuilderFlag,
    box_width: f32,
    symbols: Option<Symbols<'b>>,
}

#[allow(dead_code)]
impl<'a, 'b> TextBuilder<'a, 'b> {
    #[inline]
    pub fn new(font: &'a dyn Font) -> Self {
        Self {
            font,
            x: 0.0,
            y: 0.0,
            scale: 1.0,
            shadow_color: (0, 0, 0, 150),
            color: (255, 255, 255, 255),
            flags: TextBuilderFlag(0),
            box_width: 0.0,
            symbols: None,
        }
    }

    #[inline]
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    #[inline]
    pub fn x(mut self, x: f32) -> Self {
        self.x = x;
        self
    }

    #[inline]
    pub fn y(mut self, y: f32) -> Self {
        self.y = y;
        self
    }

    #[inline]
    pub fn get_position(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    #[inline]
    pub const fn color(mut self, color: (u8, u8, u8, u8)) -> Self {
        self.color = color;
        self
    }

    #[inline]
    pub const fn get_color(&self) -> (u8, u8, u8, u8) {
        self.color
    }

    #[inline]
    pub const fn shadow_color(mut self, color: (u8, u8, u8, u8)) -> Self {
        self.shadow_color = color;
        self
    }

    #[inline]
    pub const fn get_shadow_color(&self) -> (u8, u8, u8, u8) {
        self.shadow_color
    }

    #[inline]
    pub const fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    #[inline]
    pub const fn get_scale(&self) -> f32 {
        self.scale
    }

    #[inline]
    pub fn shadow(mut self, shadow: bool) -> Self {
        self.flags.set_shadow(shadow);
        self
    }

    #[inline]
    pub fn get_shadow(&self) -> bool {
        self.flags.shadow()
    }

    #[inline]
    pub fn with_symbols(mut self, symbols: Option<Symbols<'b>>) -> Self {
        self.symbols = symbols;
        self
    }

    #[inline]
    pub fn center(mut self, box_width: f32) -> Self {
        self.box_width = box_width;
        self.flags.set_centered(true);

        self
    }

    #[inline]
    pub fn compute_width(&self, text: &str) -> f32 {
        self.compute_width_iter(text.chars())
    }

    #[inline]
    pub fn compute_width_iter(&self, mut text: impl Iterator<Item = char>) -> f32 {
        self.font.compute_width(&mut text, self.symbols.as_ref())
    }

    #[inline]
    pub fn draw(
        self,
        text: &str,
        ctx: &mut Context,
        constants: &EngineConstants,
        texture_set: &mut TextureSet,
    ) -> GameResult {
        self.draw_iter(text.chars(), ctx, constants, texture_set)
    }

    pub fn draw_iter(
        self,
        mut text: impl Iterator<Item = char>,
        ctx: &mut Context,
        constants: &EngineConstants,
        texture_set: &mut TextureSet,
    ) -> GameResult {
        self.font.draw(
            &mut text,
            self.x,
            self.y,
            self.scale,
            self.box_width,
            self.shadow_color,
            self.color,
            self.flags,
            constants,
            texture_set,
            self.symbols,
            ctx,
        )
    }
}
