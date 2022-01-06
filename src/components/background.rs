use crate::{Context, GameResult, graphics, SharedGameState};
use crate::common::{Color, Rect};
use crate::frame::Frame;
use crate::stage::{BackgroundType, Stage, StageTexturePaths};

pub struct Background {
    pub tick: usize,
    pub prev_tick: usize,
}

impl Background {
    pub fn new() -> Self {
        Background {
            tick: 0,
            prev_tick: 0,
        }
    }

    pub fn tick(&mut self) -> GameResult<()> {
        self.tick = self.tick.wrapping_add(1);

        Ok(())
    }

    pub fn draw_tick(&mut self) -> GameResult<()> {
        self.prev_tick = self.tick;

        Ok(())
    }

    pub fn draw(
        &self,
        state: &mut SharedGameState,
        ctx: &mut Context,
        frame: &Frame,
        textures: &StageTexturePaths,
        stage: &Stage,
    ) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(
            ctx,
            &state.constants,
            &textures.background,
        )?;
        let scale = state.scale;
        let (frame_x, frame_y) = frame.xy_interpolated(state.frame_time);

        match stage.data.background_type {
            BackgroundType::TiledStatic => {
                graphics::clear(ctx, stage.data.background_color);

                let (bg_width, bg_height) = (batch.width() as i32, batch.height() as i32);
                let count_x = state.canvas_size.0 as i32 / bg_width + 1;
                let count_y = state.canvas_size.1 as i32 / bg_height + 1;

                for y in -1..count_y {
                    for x in -1..count_x {
                        batch.add((x * bg_width) as f32, (y * bg_height) as f32);
                    }
                }
            }
            BackgroundType::TiledParallax | BackgroundType::Tiled | BackgroundType::Waterway => {
                graphics::clear(ctx, stage.data.background_color);

                let (off_x, off_y) = if stage.data.background_type == BackgroundType::Tiled {
                    (frame_x % (batch.width() as f32), frame_y % (batch.height() as f32))
                } else {
                    (
                        ((frame_x / 2.0 * scale).floor() / scale) % (batch.width() as f32),
                        ((frame_y / 2.0 * scale).floor() / scale) % (batch.height() as f32),
                    )
                };

                let (bg_width, bg_height) = (batch.width() as i32, batch.height() as i32);
                let count_x = state.canvas_size.0 as i32 / bg_width + 2;
                let count_y = state.canvas_size.1 as i32 / bg_height + 2;

                for y in -1..count_y {
                    for x in -1..count_x {
                        batch.add((x * bg_width) as f32 - off_x, (y * bg_height) as f32 - off_y);
                    }
                }
            }
            BackgroundType::Water => {
                graphics::clear(ctx, stage.data.background_color);
            }
            BackgroundType::Black => {
                graphics::clear(ctx, stage.data.background_color);
            }
            BackgroundType::Scrolling => {
                graphics::clear(ctx, stage.data.background_color);
            }
            BackgroundType::OutsideWind | BackgroundType::Outside | BackgroundType::OutsideUnknown => {
                graphics::clear(ctx, Color::from_rgb(0, 0, 0));

                let offset_x = (self.tick % 640) as i32;
                let offset_y = ((state.canvas_size.1 - 240.0) / 2.0).floor();

                for x in (0..(state.canvas_size.0 as i32)).step_by(100) {
                    batch.add_rect(x as f32, offset_y, &Rect::new_size(128, 0, 100, 88));
                }

                // top / bottom edges
                if offset_y > 0.0 {
                    let scale = offset_y;

                    for x in (0..(state.canvas_size.0 as i32)).step_by(100) {
                        batch.add_rect_scaled(x as f32,0.0, 1.0, scale,  &Rect::new_size(128, 0, 100, 1));
                    }

                    batch.add_rect_scaled((state.canvas_size.0 - 320.0) / 2.0, 0.0, 1.0, scale, &Rect::new_size(0, 0, 320, 1));

                    for x in ((-offset_x * 4)..(state.canvas_size.0 as i32)).step_by(320) {
                        batch.add_rect_scaled(x as f32, offset_y + 240.0, 1.0, scale + 4.0, &Rect::new_size(0, 239, 320, 1));
                    }
                }

                batch.add_rect((state.canvas_size.0 - 320.0) / 2.0, offset_y, &Rect::new_size(0, 0, 320, 88));

                for x in ((-offset_x / 2)..(state.canvas_size.0 as i32)).step_by(320) {
                    batch.add_rect(x as f32, offset_y + 88.0, &Rect::new_size(0, 88, 320, 35));
                }

                for x in ((-offset_x % 320)..(state.canvas_size.0 as i32)).step_by(320) {
                    batch.add_rect(x as f32, offset_y + 123.0, &Rect::new_size(0, 123, 320, 23));
                }

                for x in ((-offset_x * 2)..(state.canvas_size.0 as i32)).step_by(320) {
                    batch.add_rect(x as f32, offset_y + 146.0, &Rect::new_size(0, 146, 320, 30));
                }

                for x in ((-offset_x * 4)..(state.canvas_size.0 as i32)).step_by(320) {
                    batch.add_rect(x as f32, offset_y + 176.0, &Rect::new_size(0, 176, 320, 64));
                }
            }
        }

        batch.draw(ctx)?;

        Ok(())
    }
}
