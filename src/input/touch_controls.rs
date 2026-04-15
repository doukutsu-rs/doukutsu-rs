use crate::common::Rect;
use crate::engine_constants::EngineConstants;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics::screen_insets_scaled;
use crate::graphics::texture_set::TextureSet;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TouchControlType {
    None,
    Dialog,
    Controls,
}

#[derive(Copy, Clone, Debug)]
pub struct TouchPoint {
    pub id: u64,
    pub touch_id: u64,
    pub position: (f64, f64),
    pub first_position: (f64, f64),
    pub last_position: (f64, f64),
}

impl TouchPoint {
    pub fn is_in(&self, bounds: Rect) -> bool {
        (self.position.0 as isize) > bounds.left
            && (self.position.0 as isize) < bounds.right
            && (self.position.1 as isize) > bounds.top
            && (self.position.1 as isize) < bounds.bottom
    }
}

pub struct TouchControls {
    pub control_type: TouchControlType,
    pub points: Vec<TouchPoint>,
    pub interact_icon: bool,
    pub touch_id_counter: u64,
    pub clicks: Vec<TouchPoint>,
}

impl TouchControls {
    pub fn new() -> TouchControls {
        TouchControls {
            control_type: TouchControlType::None,
            points: Vec::with_capacity(8),
            interact_icon: false,
            touch_id_counter: 0,
            clicks: Vec::with_capacity(8),
        }
    }

    pub fn find_point_in(&self, bounds: Rect) -> Option<TouchPoint> {
        self.points
            .iter()
            .find(|p| p.is_in(bounds))
            .map(|p| *p)
    }

    pub fn point_in(&self, bounds: Rect) -> Option<u64> {
        self.find_point_in(bounds).map(|p| p.touch_id)
    }

    pub fn consume_click_in(&mut self, bounds: Rect) -> bool {
        self.clicks.retain(|p| p.touch_id != 0);

        if let Some(point) = self.clicks.iter_mut().find(|p| p.is_in(bounds)) {
            point.touch_id = 0;

            return true;
        }

        false
    }

    pub fn draw(
        &self,
        canvas_size: (f32, f32),
        scale: f32,
        constants: &EngineConstants,
        texture_set: &mut TextureSet,
        ctx: &mut Context,
    ) -> GameResult {
        let color = (255, 255, 255, 160);

        let (left, top, right, bottom) = screen_insets_scaled(ctx, scale);

        match self.control_type {
            TouchControlType::None => {}
            TouchControlType::Dialog => {
                let batch = texture_set.get_or_load_batch(ctx, constants, "builtin/touch")?;
                // Fast-Forward
                batch.add_rect_tinted(
                    canvas_size.0 - (4.0 + 48.0) + 8.0 - right,
                    4.0 + 8.0 + top,
                    color,
                    &Rect::new_size(2 * 32, 3 * 32, 32, 32),
                );

                batch.draw(ctx)?;
            }
            TouchControlType::Controls => {
                let batch = texture_set.get_or_load_batch(ctx, constants, "builtin/touch")?;
                // Movement
                for x in 0..3 {
                    for y in 0..3 {
                        let mut icon_x = x;
                        let icon_y = y;

                        if self.interact_icon && x == 1 && y == 2 {
                            icon_x = 3;
                        }

                        batch.add_rect_tinted(
                            4.0 + 48.0 * x as f32 + 8.0 + left,
                            (canvas_size.1 - 4.0 - 48.0 * 3.0) + 48.0 * y as f32 + 8.0 - bottom,
                            color,
                            &Rect::new_size(icon_x * 32, icon_y * 32, 32, 32),
                        );
                    }
                }

                // Jump
                batch.add_rect_tinted(
                    canvas_size.0 - (4.0 + 48.0) + 8.0 - right,
                    canvas_size.1 - (4.0 + 48.0) + 8.0 - bottom,
                    color,
                    &Rect::new_size(3 * 32, 32, 32, 32),
                );

                // Shoot
                batch.add_rect_tinted(
                    canvas_size.0 - (4.0 + 48.0) + 8.0 - right,
                    canvas_size.1 - (4.0 + 48.0) * 2.0 + 8.0 - bottom,
                    color,
                    &Rect::new_size(3 * 32, 0, 32, 32),
                );

                // Inventory
                batch.add_rect_tinted(
                    canvas_size.0 - (4.0 + 48.0) + 8.0 - right,
                    4.0 + 8.0 + top,
                    color,
                    &Rect::new_size(0, 3 * 32, 32, 32),
                );

                // Pause
                batch.add_rect_tinted(4.0, 4.0, color, &Rect::new_size(32, 3 * 32, 32, 32));

                batch.draw(ctx)?;
            }
        }

        Ok(())
    }
}
