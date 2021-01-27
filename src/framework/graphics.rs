use crate::common::Color;
use crate::framework::context::Context;
use crate::framework::error::GameResult;

pub enum FilterMode {
    Nearest,
    Linear,
}

pub struct Canvas {}

impl Canvas {

}

pub fn clear(ctx: &mut Context, color: Color) {}

pub fn present(ctx: &mut Context) -> GameResult<()> {
    Ok(())
}

pub fn drawable_size(ctx: &mut Context) -> (f32, f32) {
    (320.0, 240.0)
}