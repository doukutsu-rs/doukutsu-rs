#![allow(unused)]

use crate::common::{Color, Rect};
use crate::framework::backend::BackendTexture;
use crate::framework::error::GameResult;
use crate::framework::graphics::{BlendMode, FilterMode};

pub trait SpriteRenderer {
    /// Set blending mode used for subsequent draws.
    fn set_blend_mode(&mut self, mode: BlendMode);

    /// Set texture filtering used for subsequent textured draws.
    fn set_filter_mode(&mut self, mode: FilterMode);

    /// Set scissor/clip rectangle in destination pixels; None disables clipping.
    fn set_clip_rect(&mut self, rect: Option<Rect>);

    /// Save the current state (blend/filter/clip) and return an RAII guard that
    /// restores it when dropped.
    fn save<'a>(&'a mut self) -> StateGuard<'a>;

    /// Restore the most recently saved state. Intended for RAII guard use.
    fn restore_state(&mut self, guard: &StateGuard);

    /// Draw a filled rectangle in destination pixels with the current state.
    fn fill_rect(&mut self, rect: Rect, color: Color) -> GameResult;

    /// Draw a rectangle outline with given line width in pixels.
    fn draw_rect(&mut self, rect: Rect, line_width: usize, color: Color) -> GameResult;

    fn copy(&mut self, texture: &Box<dyn BackendTexture>, src: Option<Rect<u16>>, dst: Rect<f32>) -> GameResult;

    fn copy_tinted(
        &mut self,
        texture: &Box<dyn BackendTexture>,
        src: Option<Rect<u16>>,
        dst: Rect<f32>,
        color: Color,
    ) -> GameResult;

    fn copy_ex(
        &mut self,
        texture: &Box<dyn BackendTexture>,
        src: Option<Rect<u16>>,
        dst: Rect<f32>,
        angle_deg: f32,
        center: Option<(f32, f32)>,
        flip_x: bool,
        flip_y: bool,
        color: Color,
    ) -> GameResult;

    /// Ensure any buffered draws are submitted to the underlying backend.
    fn flush(&mut self) -> GameResult;
}

/// RAII guard that restores renderer state on drop.
pub struct StateGuard<'a> {
    renderer: Option<&'a mut dyn SpriteRenderer>,
    state: SavedState,
}

impl<'a> StateGuard<'a> {
    pub(crate) fn with_state(renderer: &'a mut dyn SpriteRenderer, state: SavedState) -> Self {
        Self { renderer: Some(renderer), state }
    }
}

impl<'a> Drop for StateGuard<'a> {
    fn drop(&mut self) {
        if let Some(renderer) = self.renderer.take() {
            renderer.restore_state(self);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SavedState {
    blend: BlendMode,
    filter: FilterMode,
    clip: Option<Rect>,
}

pub mod null_impl;

#[cfg(feature = "render-opengl")]
pub mod opengl_impl;

#[cfg(feature = "backend-sdl")]
pub mod sdl2_impl;
