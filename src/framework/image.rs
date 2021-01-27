use crate::framework::context::Context;
use crate::framework::error::GameResult;

pub struct Image {}

impl Image {
    pub fn from_rgba8(
        context: &mut Context,
        width: u16,
        height: u16,
        rgba: &[u8],
    ) -> GameResult<Self> {
        Ok(Image {})
    }
}